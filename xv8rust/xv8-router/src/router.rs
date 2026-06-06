use alloc::string::String;
use alloc::vec::Vec;
use xv8_http::{Method, Request, Response, StatusCode};

use crate::handler::HandlerFn;

#[derive(Clone)]
struct Route {
    method: Method,
    segments: Vec<PathSegment>,
    handler: HandlerFn,
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum PathSegment {
    Literal(String),
    Param,
}

#[derive(Clone)]
pub struct Router {
    routes: Vec<Route>,
    fallback: HandlerFn,
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: Vec::new(),
            fallback: crate::handler::handler_fn(|| async {
                Response::new(StatusCode::NOT_FOUND)
                    .header("content-type", b"text/plain")
                    .body("404 Not Found".into())
            }),
        }
    }

    pub fn get(self, path: &str, handler: HandlerFn) -> Self {
        self.route(Method::Get, path, handler)
    }

    pub fn post(self, path: &str, handler: HandlerFn) -> Self {
        self.route(Method::Post, path, handler)
    }

    pub fn route(mut self, method: Method, path: &str, handler: HandlerFn) -> Self {
        let segments = parse_path(path);
        self.routes.push(Route {
            method,
            segments,
            handler,
        });
        self
    }

    pub fn fallback(mut self, handler: HandlerFn) -> Self {
        self.fallback = handler;
        self
    }

    pub fn find(&self, req: &Request) -> &HandlerFn {
        for route in &self.routes {
            if route.method != req.method {
                continue;
            }
            if match_path(&route.segments, &req.uri.path) {
                return &route.handler;
            }
        }
        &self.fallback
    }
}

fn parse_path(path: &str) -> Vec<PathSegment> {
    path.split('/')
        .filter(|s| !s.is_empty())
        .map(|seg| {
            if seg.starts_with(':') {
                PathSegment::Param
            } else {
                PathSegment::Literal(seg.into())
            }
        })
        .collect()
}

fn match_path(segments: &[PathSegment], path: &str) -> bool {
    let path_segs: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if segments.len() != path_segs.len() {
        return false;
    }
    for (seg, actual) in segments.iter().zip(path_segs.iter()) {
        match seg {
            PathSegment::Literal(expected) => {
                if expected != actual {
                    return false;
                }
            }
            PathSegment::Param => {}
        }
    }
    true
}
