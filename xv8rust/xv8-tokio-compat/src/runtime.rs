use core::future::Future;

pub use xv8_async::JoinHandle;

/// Wrapper around xv8_async::Runtime providing a tokio-compatible API.
pub struct Runtime {
    inner: xv8_async::Runtime,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            inner: xv8_async::Runtime::new(),
        }
    }

    pub fn spawn<F, T>(&self, future: F) -> JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        self.inner.spawn(future)
    }

    pub fn block_on<F, T>(&self, future: F) -> T
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        self.inner.block_on(future)
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

/// Spawns a new task on the current thread-local runtime.
/// Panics if there is no active xv8_async runtime.
pub fn spawn<F, T>(future: F) -> JoinHandle<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    xv8_async::spawn(future)
}
