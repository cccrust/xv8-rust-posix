pub mod oneshot {
    use alloc::sync::Arc;
    use core::future::Future;
    use core::pin::Pin;
    use core::task::{Context, Poll};
    use xv8_user_std::sync::Mutex;

    struct Inner<T> {
        value: Option<T>,
        waker: Option<core::task::Waker>,
    }

    pub struct Sender<T> {
        inner: Arc<Mutex<Inner<T>>>,
    }

    pub struct Receiver<T> {
        inner: Arc<Mutex<Inner<T>>>,
    }

    pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
        let inner = Arc::new(Mutex::new(Inner {
            value: None,
            waker: None,
        }));
        (Sender { inner: inner.clone() }, Receiver { inner })
    }

    impl<T> Sender<T> {
        pub fn send(self, value: T) -> Result<(), T> {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            inner.value = Some(value);
            if let Some(waker) = inner.waker.take() {
                waker.wake();
            }
            Ok(())
        }
    }

    impl<T> Future for Receiver<T> {
        type Output = Result<T, ()>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(value) = inner.value.take() {
                Poll::Ready(Ok(value))
            } else {
                inner.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}
