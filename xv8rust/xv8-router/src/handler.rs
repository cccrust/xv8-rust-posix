use alloc::boxed::Box;
use alloc::sync::Arc;
use core::future::Future;
use core::pin::Pin;
use xv8_http::{Request, Response};

pub type BoxedFuture = Pin<Box<dyn Future<Output = Response> + Send>>;
pub type HandlerFn = Arc<dyn Fn(Request) -> BoxedFuture + Send + Sync>;

pub fn handler_fn<F, Fut>(f: F) -> HandlerFn
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response> + Send + 'static,
{
    Arc::new(move |_req: Request| Box::pin(f()))
}

pub fn handler_fn_with_req<F, Fut>(f: F) -> HandlerFn
where
    F: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Response> + Send + 'static,
{
    Arc::new(move |req: Request| Box::pin(f(req)))
}
