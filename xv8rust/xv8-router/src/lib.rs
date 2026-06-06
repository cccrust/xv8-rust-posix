#![no_std]

extern crate alloc;

pub mod router;
pub mod handler;
pub mod into_response;

pub use router::Router;
pub use handler::{handler_fn, handler_fn_with_req, HandlerFn, BoxedFuture};
pub use into_response::IntoResponse;
