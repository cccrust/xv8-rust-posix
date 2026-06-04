#![no_std]
#![no_main]
#![allow(dead_code, unused)]

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use user::*;

// ——— Dummy waker (does nothing, as we poll-loop anyway) ———
unsafe fn waker_clone(d: *const ()) -> RawWaker { RawWaker::new(d, &VTABLE) }
unsafe fn waker_wake(_: *const ()) {}
unsafe fn waker_wake_by_ref(_: *const ()) {}
unsafe fn waker_drop(_: *const ()) {}

static VTABLE: RawWakerVTable =
    RawWakerVTable::new(waker_clone, waker_wake, waker_wake_by_ref, waker_drop);

fn dummy_waker() -> Waker { unsafe { Waker::from_raw(RawWaker::new(core::ptr::null(), &VTABLE)) } }

// ——— Sleep: returns Pending until deadline ———
struct Sleep { deadline_ticks: u64 }

impl Future for Sleep {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        if uptime() as u64 >= self.deadline_ticks { Poll::Ready(()) } else { Poll::Pending }
    }
}

fn ms_to_ticks(ms: u64) -> u64 { ms * 100 / 1000 }

// ——— block_on: spin-loop poll (safe for single-threaded xv8) ———
fn block_on<F: Future<Output = T>, T>(future: F) -> T {
    let mut f = future;
    let mut pinned = unsafe { Pin::new_unchecked(&mut f) };
    let waker = dummy_waker();
    let mut cx = Context::from_waker(&waker);
    loop {
        match pinned.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => { let _ = nanosleep(0, 1_000_000); }
        }
    }
}

// ——— Tests ———
fn check(test: &str, ok: bool) {
    if ok { println!("_async: {} ... ok", test); }
    else { println!("_async: {} ... FAILED", test); exit(1); }
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    println!("_async: test async runtime...");

    check("block_on value", block_on(async { 42 }) == 42);
    check("block_on expr", block_on(async { 7 + 7 }) == 14);

    let r = block_on(async {
        Sleep { deadline_ticks: uptime() as u64 + ms_to_ticks(10) }.await;
        100
    });
    check("sleep 10ms", r == 100);

    let r = block_on(async {
        Sleep { deadline_ticks: uptime() as u64 + ms_to_ticks(5) }.await;
        Sleep { deadline_ticks: uptime() as u64 + ms_to_ticks(10) }.await;
        7
    });
    check("two sleeps", r == 7);

    let r = block_on(async {
        let mut sum = 0;
        for i in 0..5 {
            Sleep { deadline_ticks: uptime() as u64 + ms_to_ticks(2) }.await;
            sum += i;
        }
        sum
    });
    check("loop+sleep", r == 10);

    let r = block_on(async {
        let a = block_on(async {
            Sleep { deadline_ticks: uptime() as u64 + ms_to_ticks(5) }.await;
            10
        });
        let b = block_on(async {
            Sleep { deadline_ticks: uptime() as u64 + ms_to_ticks(3) }.await;
            20
        });
        a + b
    });
    check("nested block_on", r == 30);

    println!("_async: PASS");
    exit(0);
}
