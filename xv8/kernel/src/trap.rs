use core::mem;

use crate::kernelvec::kernelvec;
use crate::memlayout::{E1000_IRQ, TRAMPOLINE, UART0_IRQ, VIRTIO0_IRQ};
use crate::param::NKSTACK_PAGES;
use crate::proc::{self, Channel, TrapFrame};
use crate::riscv::{
    PGSIZE, interrupts,
    registers::{satp, scause, sepc, sstatus, stimecmp, stval, stvec, time, tp},
};
use crate::signal;
use crate::spinlock::SpinLock;
use crate::syscall::syscall;
use crate::trampoline::{trampoline, userret, uservec};
use crate::uart;
use crate::virtio_disk;
use crate::vm::VA;
use crate::{e1000, plic};

pub static TICKS: SpinLock<usize> = SpinLock::new(0, "time");

pub fn current_ticks() -> u64 {
    *TICKS.lock() as u64
}

/// Handles an interrupt, exception, or system call from user space.
///
/// # Safety
/// Called from `trampoline.rs`
#[unsafe(no_mangle)]
pub unsafe fn usertrap() {
    unsafe {
        // make sure interrupt came from user space
        assert!(
            (sstatus::read() & sstatus::SPP) == 0,
            "usertrap: not from user mode"
        );

        // send subsequent interrupts and exceptions to kerneltrap, since we are in kernel mode now
        stvec::write(kernelvec as *const () as usize);

        let (proc, data) = proc::current_proc_and_data_mut();
        let (pagetable, trapframe) = data.pagetable_and_trapframe_mut();

        // save user program counter in case, this handler yields to another core, and the new core
        // switches to user space, overwriting sepc.
        trapframe.epc = sepc::read();

        let scause = scause::Scause::from(scause::read());
        let mut which_dev = None;

        match scause.cause() {
            // System call
            scause::Trap::Exception(scause::Exception::EnvironmentCall) => {
                if proc.inner.lock().killed {
                    proc::exit(-1);
                }

                // sepc points to the ecall instruction, but we want to return to the next instruction.
                trapframe.epc += 4;

                // an interrupt will change sepc, scause, and sstatus, so enable only now that we're
                // done with those registers.
                interrupts::enable();

                syscall(trapframe);
            }

            // page fault on lazily-allocated page
            scause::Trap::Exception(scause::Exception::StorePageFault)
            | scause::Trap::Exception(scause::Exception::LoadPageFault) => {
                // vmfault handles the page fault
                // if err, either out-of-memory or out-of-bound, kill the process
                if log!(pagetable.vmfault(VA::from(stval::read()))).is_err() {
                    #[cfg(debug_assertions)]
                    {
                        let pid = proc.inner.lock().pid;
                        println!(
                            "! unhandled page fault scause=0x{:X} pid={} sepc=0x{:X} stval=0x{:X}",
                            scause.bits(),
                            *pid,
                            sepc::read(),
                            stval::read(),
                        );
                    }
                    proc.inner.lock().killed = true;
                }
            }

            // Illegal instruction — emulate CSR reads for atomic support
            scause::Trap::Exception(scause::Exception::IllegalInstruction) => {
                let inst = stval::read();
                // CSR instructions have opcode 0x73 (1110011)
                if inst & 0x7f == 0x73 {
                    let rd = (inst >> 7) & 0x1f;
                    if rd != 0 {
                        write_xreg(trapframe, rd, 0);
                    }
                    trapframe.epc += 4;
                } else {
                    let pid = proc.inner.lock().pid;
                    println!(
                        "! illegal instruction scause=0x{:X} pid={} sepc=0x{:X} stval=0x{:X}",
                        scause.bits(),
                        *pid,
                        sepc::read(),
                        inst,
                    );
                    proc.inner.lock().killed = true;
                }
            }

            // device interrupt
            scause::Trap::Interrupt(intr)
                if {
                    which_dev = device_interrupt(intr);
                    which_dev.is_some()
                } =>
            {
                // dev_intr handles the interrupt if it is a device interrupt
                // nothing to do
            }

            // something else
            _ => {
                let pid = proc.inner.lock().pid;
                println!(
                    "! unexpected interrupt scause=0x{:X} pid={} sepc=0x{:X} stval=0x{:X}",
                    scause.bits(),
                    *pid,
                    sepc::read(),
                    stval::read(),
                );
                proc.inner.lock().killed = true;
            }
        }

        if proc.inner.lock().killed {
            proc::exit(-1);
        }

        if Some(InterruptType::Timer) == which_dev {
            proc::r#yield();
        }

        usertrapret();
    }
}

/// Returns to user space.
///
/// # Safety
/// Called from `usertrap()`
#[unsafe(no_mangle)]
fn is_fatal_signal(sig: usize) -> bool {
    matches!(
        sig,
        signal::SIGKILL
            | signal::SIGSEGV
            | signal::SIGABRT
            | signal::SIGQUIT
            | signal::SIGTERM
            | signal::SIGINT
            | signal::SIGHUP
            | signal::SIGPIPE
            | signal::SIGALRM
            | signal::SIGUSR1
            | signal::SIGUSR2
            | signal::SIGVTALRM
            | signal::SIGPROF
            | signal::SIGIO
    )
}

/// Deliver pending signals before returning to user space.
unsafe fn deliver_pending_signals(data: &mut crate::proc::ProcData) {
    // Check if any signalfd matches pending signals
    // If so, the signal is consumed by signalfd and not delivered to handler
    let my_pid = *crate::proc::current_proc().inner.lock().pid;

    loop {
        let pending = data.signals.get_pending();
        let blocked = data.signals.get_blocked();
        let unblocked = pending & !blocked;
        if unblocked == 0 {
            break;
        }
        let sig = unblocked.trailing_zeros() as usize + 1;

        // If a signalfd monitors this signal, consume it via signalfd
        if crate::signalfd::signalfd_notify(my_pid, sig) {
            data.signals.clear_signal(sig);
            continue;
        }

        let idx = sig - 1;
        let act = data.sigactions.as_ref().unwrap().lock()[idx];

        data.signals.clear_signal(sig);

        match act.handler {
            0 => {
                if sig == signal::SIGSTOP || sig == signal::SIGCONT {
                    continue;
                }
                if is_fatal_signal(sig) {
                    crate::proc::exit(-(sig as isize));
                }
            }
            1 => {}
            _ => {
                // Read trapframe registers before any mutable access
                let (tf_epc, tf_ra, tf_sp, tf_gp, tf_tp, tf_t0, tf_t1, tf_t2, tf_s0, tf_s1,
                     tf_a0, tf_a1, tf_a2, tf_a3, tf_a4, tf_a5, tf_a6, tf_a7,
                     tf_s2, tf_s3, tf_s4, tf_s5, tf_s6, tf_s7, tf_s8, tf_s9, tf_s10, tf_s11,
                     tf_t3, tf_t4, tf_t5, tf_t6) = {
                    let tf = data.trapframe();
                    (tf.epc, tf.ra, tf.sp, tf.gp, tf.tp, tf.t0, tf.t1, tf.t2, tf.s0, tf.s1,
                     tf.a0, tf.a1, tf.a2, tf.a3, tf.a4, tf.a5, tf.a6, tf.a7,
                     tf.s2, tf.s3, tf.s4, tf.s5, tf.s6, tf.s7, tf.s8, tf.s9, tf.s10, tf.s11,
                     tf.t3, tf.t4, tf.t5, tf.t6)
                };

                let frame_size = core::mem::size_of::<signal::SigFrame>();
                let frame_va = VA::new(tf_sp - frame_size);

                let oldmask = data.signals.get_blocked();
                let newmask = oldmask | (act.mask as usize);
                let extra = if act.flags & signal::SA_NODEFER == 0 {
                    1 << (sig - 1)
                } else {
                    0
                };
                data.signals
                    .blocked
                    .store(newmask | extra, core::sync::atomic::Ordering::Relaxed);

                let frame = signal::SigFrame {
                    signo: sig as i32,
                    pad: 0,
                    epc: tf_epc as u64,
                    ra: tf_ra as u64,
                    sp: tf_sp as u64,
                    gp: tf_gp as u64,
                    tp: tf_tp as u64,
                    t0: tf_t0 as u64,
                    t1: tf_t1 as u64,
                    t2: tf_t2 as u64,
                    s0: tf_s0 as u64,
                    s1: tf_s1 as u64,
                    a0: tf_a0 as u64,
                    a1: tf_a1 as u64,
                    a2: tf_a2 as u64,
                    a3: tf_a3 as u64,
                    a4: tf_a4 as u64,
                    a5: tf_a5 as u64,
                    a6: tf_a6 as u64,
                    a7: tf_a7 as u64,
                    s2: tf_s2 as u64,
                    s3: tf_s3 as u64,
                    s4: tf_s4 as u64,
                    s5: tf_s5 as u64,
                    s6: tf_s6 as u64,
                    s7: tf_s7 as u64,
                    s8: tf_s8 as u64,
                    s9: tf_s9 as u64,
                    s10: tf_s10 as u64,
                    s11: tf_s11 as u64,
                    t3: tf_t3 as u64,
                    t4: tf_t4 as u64,
                    t5: tf_t5 as u64,
                    t6: tf_t6 as u64,
                    oldmask: oldmask as u64,
                };

                let frame_bytes = unsafe {
                    core::slice::from_raw_parts(
                        &frame as *const signal::SigFrame as *const u8,
                        core::mem::size_of::<signal::SigFrame>(),
                    )
                };
                data.pagetable_mut()
                    .copy_to(frame_bytes, frame_va)
                    .unwrap_or(());

                let tf = data.trapframe_mut();
                tf.epc = act.handler;
                tf.sp = frame_va.as_usize();
                tf.ra = 0;
                tf.a0 = sig;
                tf.a1 = 0;
                tf.a2 = frame_va.as_usize();
                data.signals
                    .in_handler
                    .store(true, core::sync::atomic::Ordering::Relaxed);
                break;
            }
        }
    }
}

pub unsafe fn usertrapret() {
    let (_proc, data) = proc::current_proc_and_data_mut();

    // Deliver pending signals before returning to user space
    unsafe { deliver_pending_signals(data); }

    // we're about to switch the destination of traps from `kerneltrap()` to `usertrap()`, so turn
    // off interrupts until we're back in user space, where `usertrap()` is correct.
    interrupts::disable();

    // send syscalls, interrupts, and exceptions to `uservec` in `trampoline.S`
    let trampoline_uservec =
        TRAMPOLINE + (uservec as *const () as usize - trampoline as *const () as usize);
    unsafe { stvec::write(trampoline_uservec) };

    // set up trapframe values that uservec will need when the process next traps into the kernel.
    let kstack = data.kstack;
    let trapframe = data.trapframe_mut();
    trapframe.kernel_satp = unsafe { satp::read() }; // kernel page table
    trapframe.kernel_sp = (kstack + NKSTACK_PAGES * PGSIZE).as_usize(); // process's kernel stack
    trapframe.kernel_trap = usertrap as *const () as usize;
    trapframe.kernel_hartid = unsafe { tp::read() }; // hartid for `current_id()`

    // set up the registers that trampoline.S's sret will use to get to user space.

    // set Supervisor Previous Privilege mode to User.
    let mut x = unsafe { sstatus::read() };
    x &= !sstatus::SPP; // clear SPP to 0 for user mode
    x |= sstatus::SPIE; // enable interrupts in user mode
    unsafe { sstatus::write(x) };

    // set S Exception Program Counter to the saved user pc.
    unsafe { sepc::write(trapframe.epc) };

    // tell trampoline.S the user page table to switch to.
    let user_satp = satp::make(data.pagetable().inner.as_pa().as_usize());

    // jump to userret in trampoline.S at the top of memory, which switches to the user page table,
    // restores user registers, and switches to user mode with sret.
    unsafe {
        // calculate the virtual address of userret since we have to use the trampoline base address.
        // directly using `userret` would be an address in the kernel page table.
        let trampoline_userret: usize =
            TRAMPOLINE + (userret as *const () as usize - trampoline as *const () as usize);
        let trampoline_userret: fn(usize) -> ! = mem::transmute(trampoline_userret);
        trampoline_userret(user_satp);
    }
}

/// Interrupts and exceptions from the kernel code go here via `kernelvec`, on whatever the current
/// kernel stack is.
///
/// # Safety
/// Called from `kernelvec.rs`.
#[unsafe(no_mangle)]
pub unsafe fn kerneltrap() {
    unsafe {
        let sepc = sepc::read();
        let sstatus = sstatus::read();
        let scause = scause::Scause::from(scause::read());

        assert!(
            sstatus & sstatus::SPP != 0,
            "kerneltrap: not from supervisor mode"
        );

        assert!(!interrupts::get(), "kerneltrap: interrupts enabled");

        let which_dev;

        // If we got exceptions in supervisor mode, or we got an interrupt from an unknown source,
        // it is fatal
        match scause.cause() {
            scause::Trap::Interrupt(intr)
                if {
                    which_dev = device_interrupt(intr);
                    which_dev.is_some()
                } => {}

            _ => {
                println!(
                    "scause=0x{:X} sepc=0x{:X} stval=0x{:X}",
                    scause.bits(),
                    sepc::read(),
                    stval::read()
                );
                panic!("kerneltrap");
            }
        }

        // If we got a timer interrupt, give up the cpu for another process
        if Some(InterruptType::Timer) == which_dev && proc::current_proc_opt().is_some() {
            proc::r#yield();
        }

        // The yield() may have caused some traps to occur, so restore trap registers for use by
        // kernelvec.S's sepc instruction.
        sepc::write(sepc);
        sstatus::write(sstatus);
    }
}

/// Handles clock interrupts.
pub fn clock_intr() {
    let _lock = proc::lock_current_cpu();
    let hart = unsafe { proc::current_id() };

    if hart == 0 {
        let mut ticks = TICKS.lock();
        *ticks += 1;
        proc::wakeup(Channel::Ticks);
        drop(ticks);
        crate::timerfd::tick();
    }

    unsafe { stimecmp::write(time::read() + 1_000_000) };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InterruptType {
    Device,
    Timer,
}

/// Checks if interrupt is from an external device or software timer.
fn device_interrupt(intr: scause::Interrupt) -> Option<InterruptType> {
    match intr {
        // Supervisor external interrupt via PLIC
        scause::Interrupt::SupervisorExternal => {
            let irq = plic::claim();

            match irq as usize {
                0 => {} // spurious interrupt from PLIC, ignore
                UART0_IRQ => uart::handle_interrupt(),
                VIRTIO0_IRQ => virtio_disk::handle_interrupt(),
                E1000_IRQ => e1000::handle_interrupt(),
                _ => println!("unexpected interrupt irq = {}", irq),
            }

            if irq != 0 {
                plic::complete(irq);
            }

            Some(InterruptType::Device)
        }

        // Timer interrupt
        scause::Interrupt::SupervisorTimer => {
            clock_intr();
            Some(InterruptType::Timer)
        }

        // some other interrupt, we don't recognize
        _ => None,
    }
}

fn write_xreg(tf: &mut TrapFrame, reg: usize, val: usize) {
    match reg {
        1 => tf.ra = val,
        2 => tf.sp = val,
        3 => tf.gp = val,
        4 => tf.tp = val,
        5 => tf.t0 = val,
        6 => tf.t1 = val,
        7 => tf.t2 = val,
        8 => tf.s0 = val,
        9 => tf.s1 = val,
        10 => tf.a0 = val,
        11 => tf.a1 = val,
        12 => tf.a2 = val,
        13 => tf.a3 = val,
        14 => tf.a4 = val,
        15 => tf.a5 = val,
        16 => tf.a6 = val,
        17 => tf.a7 = val,
        18 => tf.s2 = val,
        19 => tf.s3 = val,
        20 => tf.s4 = val,
        21 => tf.s5 = val,
        22 => tf.s6 = val,
        23 => tf.s7 = val,
        24 => tf.s8 = val,
        25 => tf.s9 = val,
        26 => tf.s10 = val,
        27 => tf.s11 = val,
        28 => tf.t3 = val,
        29 => tf.t4 = val,
        30 => tf.t5 = val,
        31 => tf.t6 = val,
        _ => {}
    }
}

/// Initializes the trap handling code.
pub fn init() {
    // No work since lock is already initialized
    println!("trap init");
}

/// Sets up to take exceptions and traps while in the kernel.
///
/// # Safety
/// This function must be called only once per hart during system initialization.
pub unsafe fn init_hart() {
    unsafe { stvec::write(kernelvec as *const () as usize) };
}
