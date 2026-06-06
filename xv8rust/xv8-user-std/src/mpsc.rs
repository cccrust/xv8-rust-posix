use alloc::collections::VecDeque;
use alloc::sync::Arc;

use crate::sync::{Condvar, Mutex, LockResult};

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new(Inner {
        queue: Mutex::new(VecDeque::new()),
        condvar: Condvar::new(),
    });
    (Sender { inner: inner.clone() }, Receiver { inner })
}

struct Inner<T> {
    queue: Mutex<VecDeque<T>>,
    condvar: Condvar,
}

pub struct Sender<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), T> {
        let mut queue = lock_guard(&self.inner.queue);
        queue.push_back(value);
        self.inner.condvar.notify_one();
        Ok(())
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Sender { inner: self.inner.clone() }
    }
}

pub struct Receiver<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> LockResult<T> {
        let mut queue = lock_guard(&self.inner.queue);
        loop {
            if let Some(value) = queue.pop_front() {
                return Ok(value);
            }
            queue = self.inner.condvar.wait(queue).unwrap_or_else(|e| e.into_inner());
        }
    }

    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        let mut queue = lock_guard(&self.inner.queue);
        queue.pop_front().ok_or(TryRecvError::Empty)
    }
}

fn lock_guard<'a, T>(mutex: &'a Mutex<T>) -> crate::sync::MutexGuard<'a, T> {
    match mutex.lock() {
        Ok(g) => g,
        Err(e) => e.into_inner(),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TryRecvError {
    Empty,
}
