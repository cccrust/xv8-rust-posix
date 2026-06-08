use alloc::vec::Vec;

use crate::proc;
use crate::syscall::SysError;
use crate::vm::VA;

// Seccomp operations
pub const SECCOMP_SET_MODE_FILTER: usize = 2;
pub const _SECCOMP_FILTER_FLAG_TSYNC: usize = 1;

// Seccomp return values
pub const SECCOMP_RET_KILL_PROCESS: u32 = 0x80000000;
pub const _SECCOMP_RET_KILL_THREAD: u32 = 0x00000000;
pub const _SECCOMP_RET_TRAP: u32 = 0x00030000;
pub const SECCOMP_RET_ERRNO: u32 = 0x00050000;
pub const SECCOMP_RET_ALLOW: u32 = 0x7fff0000;

// BPF instruction
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SockFilter {
    pub code: u16,
    pub jt: u8,
    pub jf: u8,
    pub k: u32,
}

// BPF program
#[derive(Debug, Clone)]
pub struct SeccompFilter {
    pub filter: Vec<SockFilter>,
}

// BPF instruction classes
const BPF_LD: u16 = 0x00;
const BPF_JMP: u16 = 0x05;
const BPF_RET: u16 = 0x06;
const BPF_ALU: u16 = 0x04;
const BPF_MISC: u16 = 0x07;

// BPF sizes
const _BPF_W: u16 = 0x00;
const _BPF_H: u16 = 0x08;
const _BPF_B: u16 = 0x10;

// BPF modes
const BPF_IMM: u16 = 0x00;
const BPF_ABS: u16 = 0x20;
const BPF_IND: u16 = 0x40;
const BPF_MEM: u16 = 0x60;
const _BPF_LEN: u16 = 0x80;

// BPF JMP conditions
const BPF_JEQ: u16 = 0x10;
const _BPF_JGT: u16 = 0x20;
const _BPF_JGE: u16 = 0x30;
const _BPF_JSET: u16 = 0x40;

// BPF ALU operations
const BPF_ADD: u16 = 0x00;
const _BPF_SUB: u16 = 0x10;
const _BPF_MUL: u16 = 0x20;
const _BPF_DIV: u16 = 0x30;
const BPF_OR: u16 = 0x40;
const BPF_AND: u16 = 0x50;
const _BPF_LSH: u16 = 0x60;
const _BPF_RSH: u16 = 0x70;
const _BPF_NEG: u16 = 0x80;
const _BPF_MOD: u16 = 0x90;
const BPF_XOR: u16 = 0xa0;

// BPF misc
const BPF_TAX: u16 = 0x00;
const BPF_TXA: u16 = 0x80;

// Seccomp data offsets
const SECCOMP_DATA_NR: usize = 0;

pub struct SeccompData {
    pub nr: i32,
    pub _arch: u32,
}

#[derive(Debug, Clone)]
pub enum SeccompState {
    Disabled,
    Filter(SeccompFilter),
}

impl SeccompFilter {
    pub fn evaluate(&self, data: &SeccompData) -> u32 {
        let mut A: u32 = 0;
        let mut X: u32 = 0;

        let seccomp_mem: [u32; 8] = [
            data.nr as u32,
            data._arch,
            0, 0, 0, 0, 0, 0,
        ];

        let mut pc = 0;
        while pc < self.filter.len() {
            let inst = &self.filter[pc];
            let code = inst.code & 0xe0;
            let cls = inst.code & 0x1f;

            match cls {
                BPF_LD => {
                    match code {
                        BPF_IMM => { A = inst.k; }
                        BPF_ABS => {
                            let addr = inst.k as usize;
                            if addr < seccomp_mem.len() * 4 {
                                A = seccomp_mem[addr / 4];
                            } else {
                                A = 0;
                            }
                        }
                        BPF_IND => {
                            let addr = (inst.k.wrapping_add(X)) as usize;
                            if addr < seccomp_mem.len() * 4 {
                                A = seccomp_mem[addr / 4];
                            } else {
                                A = 0;
                            }
                        }
                        BPF_MEM => {
                            // mem is internal, skip for seccomp simplicity
                        }
                        _ => {}
                    }
                    pc += 1;
                }
                BPF_JMP => {
                    let ja = inst.code & 0x0f == 0;
                    let condition = if ja {
                        true
                    } else {
                        let val_b = if inst.code & 0x08 != 0 { X } else { inst.k };
                        match inst.code & 0x0f {
                            BPF_JEQ => A == val_b,
                            0x20 => A > val_b,  // JGT
                            0x30 => A >= val_b, // JGE
                            0x40 => (A & val_b) != 0, // JSET
                            _ => false,
                        }
                    };

                    if ja {
                        pc += inst.k as usize + 1;
                    } else if condition {
                        pc += inst.jt as usize + 1;
                    } else {
                        pc += inst.jf as usize + 1;
                    }
                }
                BPF_RET => {
                    return inst.k;
                }
                BPF_ALU => {
                    let op = inst.code & 0xf0;
                    let src = if inst.code & 0x08 != 0 { X } else { inst.k };
                    match op {
                        BPF_ADD => { A = A.wrapping_add(src); }
                        BPF_OR => { A |= src; }
                        BPF_AND => { A &= src; }
                        BPF_XOR => { A ^= src; }
                        _ => {}
                    }
                    pc += 1;
                }
                BPF_MISC => {
                    if inst.code & 0xf0 == BPF_TAX {
                        X = A;
                    } else if inst.code & 0xf0 == BPF_TXA {
                        A = X;
                    }
                    pc += 1;
                }
                _ => {
                    pc += 1;
                }
            }
        }

        SECCOMP_RET_ALLOW
    }
}

#[derive(Debug, Clone)]
pub enum SeccompAction {
    Allow,
    Kill,
}

impl SeccompState {
    pub fn should_allow(&self, syscall_num: usize) -> SeccompAction {
        match self {
            SeccompState::Disabled => SeccompAction::Allow,
            SeccompState::Filter(filter) => {
                let data = SeccompData {
                    nr: syscall_num as i32,
                    _arch: 0,
                };
                let result = filter.evaluate(&data);
                match result & 0xffff0000 {
                    SECCOMP_RET_ALLOW => SeccompAction::Allow,
                    _ => SeccompAction::Kill,
                }
            }
        }
    }
}

pub fn sys_seccomp(op: usize, _flags: usize, args_addr: VA) -> Result<usize, SysError> {
    match op {
        SECCOMP_SET_MODE_FILTER => {
            let mut fprog_buf = [0u8; 16];
            if proc::copy_from_user(args_addr, &mut fprog_buf).is_err() {
                return Err(SysError::BadAddress);
            }
            let len = u16::from_ne_bytes(fprog_buf[0..2].try_into().unwrap()) as usize;
            let filter_ptr = usize::from_ne_bytes(
                fprog_buf[core::mem::size_of::<usize>()..2 * core::mem::size_of::<usize>()]
                    .try_into()
                    .unwrap(),
            );

            let mut filter = Vec::with_capacity(len);
            for i in 0..len {
                let mut inst_buf = [0u8; 8];
                let addr = VA::new(filter_ptr + i * 8);
                if proc::copy_from_user(addr, &mut inst_buf).is_err() {
                    return Err(SysError::BadAddress);
                }
                filter.push(SockFilter {
                    code: u16::from_ne_bytes(inst_buf[0..2].try_into().unwrap()),
                    jt: inst_buf[2],
                    jf: inst_buf[3],
                    k: u32::from_ne_bytes(inst_buf[4..8].try_into().unwrap()),
                });
            }

            let (_proc, mut data) = proc::current_proc_and_data_mut();
            data.seccomp = SeccompState::Filter(SeccompFilter { filter });
            Ok(0)
        }
        _ => Err(SysError::InvalidArgument),
    }
}

pub fn seccomp_check(syscall_num: usize) -> SeccompAction {
    let proc = proc::current_proc();
    let data = proc.data();
    data.seccomp.should_allow(syscall_num)
}
