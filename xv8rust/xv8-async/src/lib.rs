#![no_std]

extern crate alloc;

use alloc::collections::VecDeque;
use alloc::sync::{Arc, Weak};
use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use core::sync::atomic::{AtomicBool, Ordering};

use xv8_user_std::sync::{Mutex, MutexGuard};
use xv8_user_std::time::{Duration, Instant};

pub mod reactor;
pub mod io_async;

struct ExecutorInner {
    queue: Mutex<VecDeque<Arc<Task>>>,
    timers: Mutex<alloc::vec::Vec<TimerEntry>>,
}

fn lock<'a, T>(mutex: &'a Mutex<T>) -> MutexGuard<'a, T> {
    mutex.lock().unwrap_or_else(|error| error.into_inner())
}

impl ExecutorInner {
    fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            timers: Mutex::new(alloc::vec::Vec::new()),
        }
    }

    fn enqueue(&self, task: Arc<Task>) {
        if task.enqueued.swap(true, Ordering::AcqRel) {
            return;
        }
        lock(&self.queue).push_back(task);
    }

    fn pop_ready(&self) -> Option<Arc<Task>> {
        lock(&self.queue).pop_front()
    }

    fn register_timer(&self, deadline: Instant, waker: Waker) {
        lock(&self.timers).push(TimerEntry { deadline, waker });
    }

    fn next_deadline(&self) -> Option<Instant> {
        lock(&self.timers).iter().map(|entry| entry.deadline).min()
    }

    fn wake_expired(&self, now: Instant) {
        let mut ready = alloc::vec::Vec::new();
        {
            let mut timers = lock(&self.timers);
            let mut index = 0;
            while index < timers.len() {
                if timers[index].deadline <= now {
                    ready.push(timers.swap_remove(index));
                } else {
                    index += 1;
                }
            }
        }

        for entry in ready {
            entry.waker.wake();
        }
    }
}

struct TimerEntry {
    deadline: Instant,
    waker: Waker,
}

struct Task {
    future: Mutex<Option<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>>,
    executor: Weak<ExecutorInner>,
    enqueued: AtomicBool,
}

impl Task {
    fn new(
        future: Pin<Box<dyn Future<Output = ()> + Send + 'static>>,
        executor: Weak<ExecutorInner>,
    ) -> Self {
        Self {
            future: Mutex::new(Some(future)),
            executor,
            enqueued: AtomicBool::new(false),
        }
    }

    fn schedule(self: &Arc<Self>) {
        if let Some(executor) = self.executor.upgrade() {
            executor.enqueue(self.clone());
        }
    }

    fn poll(self: Arc<Self>) {
        let waker = self.waker();
        let mut cx = Context::from_waker(&waker);
        let mut future_slot = lock(&self.future);
        if let Some(mut future) = future_slot.take() {
            self.enqueued.store(false, Ordering::Release);
            match future.as_mut().poll(&mut cx) {
                Poll::Ready(()) => {}
                Poll::Pending => {
                    *future_slot = Some(future);
                }
            }
        }
    }

    fn waker(self: &Arc<Self>) -> Waker {
        unsafe { Waker::from_raw(raw_waker(self.clone())) }
    }
}

fn raw_waker(task: Arc<Task>) -> RawWaker {
    RawWaker::new(Arc::into_raw(task) as *const (), &VTABLE)
}

unsafe fn clone_waker(data: *const ()) -> RawWaker {
    let arc = Arc::<Task>::from_raw(data as *const Task);
    let cloned = arc.clone();
    core::mem::forget(arc);
    raw_waker(cloned)
}

unsafe fn wake_waker(data: *const ()) {
    let arc = Arc::<Task>::from_raw(data as *const Task);
    arc.schedule();
}

unsafe fn wake_by_ref_waker(data: *const ()) {
    let arc = Arc::<Task>::from_raw(data as *const Task);
    arc.schedule();
    core::mem::forget(arc);
}

unsafe fn drop_waker(data: *const ()) {
    let _ = Arc::<Task>::from_raw(data as *const Task);
}

static VTABLE: RawWakerVTable = RawWakerVTable::new(
    clone_waker,
    wake_waker,
    wake_by_ref_waker,
    drop_waker,
);

static CURRENT_STACK: Mutex<alloc::vec::Vec<Arc<ExecutorInner>>> =
    Mutex::new(alloc::vec::Vec::new());

struct CurrentExecutorGuard;

impl CurrentExecutorGuard {
    fn push(executor: Arc<ExecutorInner>) -> Self {
        lock(&CURRENT_STACK).push(executor);
        Self
    }
}

impl Drop for CurrentExecutorGuard {
    fn drop(&mut self) {
        let _ = lock(&CURRENT_STACK).pop();
    }
}

fn current_executor() -> Option<Arc<ExecutorInner>> {
    lock(&CURRENT_STACK).last().cloned()
}

#[derive(Clone)]
pub struct Runtime {
    inner: Arc<ExecutorInner>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(ExecutorInner::new()),
        }
    }

    fn enter(&self) -> CurrentExecutorGuard {
        CurrentExecutorGuard::push(self.inner.clone())
    }

    pub fn spawn<F, T>(&self, future: F) -> JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        self.spawn_inner(future)
    }

    pub fn block_on<F, T>(&self, future: F) -> T
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        reactor::ensure_init();
        let _guard = self.enter();
        let handle = self.spawn_inner(future);
        self.run_until_complete(handle)
    }

    fn spawn_inner<F, T>(&self, future: F) -> JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let state = Arc::new(Mutex::new(JoinState { result: None, waker: None }));
        let state_for_task = state.clone();
        let task_future = async move {
            let output = future.await;
            let mut state = lock(&state_for_task);
            state.result = Some(output);
            if let Some(waker) = state.waker.take() {
                waker.wake();
            }
        };

        let task = Arc::new(Task::new(Box::pin(task_future), Arc::downgrade(&self.inner)));
        self.inner.enqueue(task);
        JoinHandle { state }
    }

    fn run_until_complete<T>(&self, mut handle: JoinHandle<T>) -> T
    where
        T: Send + 'static,
    {
        loop {
            self.run_ready_tasks();

            if let Some(value) = handle.take_ready() {
                return value;
            }

            self.reactor_tick();
        }
    }

    fn run_ready_tasks(&self) {
        while let Some(task) = self.inner.pop_ready() {
            task.poll();
        }
    }

    fn reactor_tick(&self) {
        let deadline = self.inner.next_deadline();
        let timeout = deadline.map(|d| {
            let now = Instant::now();
            if d > now {
                let millis = (d - now).as_millis();
                if millis > isize::MAX as u64 { -1 } else { millis as isize }
            } else { 0 }
        }).unwrap_or(-1);

        reactor::poll_events(timeout);
        self.inner.wake_expired(Instant::now());
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

struct JoinState<T> {
    result: Option<T>,
    waker: Option<Waker>,
}

pub struct JoinHandle<T> {
    state: Arc<Mutex<JoinState<T>>>,
}

impl<T> JoinHandle<T> {
    fn take_ready(&mut self) -> Option<T> {
        lock(&self.state).result.take()
    }
}

impl<T> Future for JoinHandle<T>
where
    T: Send + 'static,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let mut state = lock(&self.state);
        if let Some(result) = state.result.take() {
            Poll::Ready(result)
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

pub struct Sleep {
    deadline: Instant,
    armed: bool,
}

impl Sleep {
    pub fn new(duration: Duration) -> Self {
        Self {
            deadline: Instant::now() + duration,
            armed: false,
        }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if Instant::now() >= self.deadline {
            return Poll::Ready(());
        }

        if !self.armed {
            let deadline = self.deadline;
            let executor = current_executor().expect("xv8_async::sleep requires an active runtime");
            executor.register_timer(deadline, cx.waker().clone());
            self.armed = true;
        }

        Poll::Pending
    }
}

pub struct YieldNow {
    yielded: bool,
}

impl YieldNow {
    pub fn new() -> Self {
        Self { yielded: false }
    }
}

impl Future for YieldNow {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.yielded {
            Poll::Ready(())
        } else {
            self.yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

pub fn runtime() -> Runtime {
    Runtime::new()
}

pub fn block_on<F, T>(future: F) -> T
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    Runtime::new().block_on(future)
}

pub fn spawn<F, T>(future: F) -> JoinHandle<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let executor = current_executor().expect("xv8_async::spawn requires an active runtime");
    let runtime = Runtime { inner: executor };
    runtime.spawn_inner(future)
}

pub async fn sleep(duration: Duration) {
    Sleep::new(duration).await
}

pub async fn yield_now() {
    YieldNow::new().await
}
