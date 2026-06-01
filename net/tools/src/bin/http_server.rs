use std::fs;
use std::io::{self, Read};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use tiny_http::{Header, Method, Response, Server, StatusCode};

fn main() {
    let addr = "0.0.0.0:8080";
    let server = Server::http(addr).expect("Failed to bind to address");
    println!("Listening on http://{}", addr);

    for request in server.incoming_requests() {
        let method = request.method();
        let url = request.url();
        println!("{} {}", method, url);

        let mut path = PathBuf::from(".");
        if url != "/" {
            // Remove leading slash and append to path
            let path_str = &url[1..];
            path.push(path_str);
        }

        // If path is a directory, try to serve index.html
        if path.is_dir() {
            path.push("index.html");
        }

        if !path.exists() || !path.is_file() {
            let mut response = Response::empty(StatusCode(404));
            let _ = request.respond(response);
            continue;
        }

        let mut file = fs::File::open(&path).expect("Failed to open file");
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).expect("Failed to read file");

        let mut response = Response::from_data(contents);
        let content_type = get_content_type(&path);
        response.add_header(Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap());
        let _ = request.respond(response);
    }
}

fn get_content_type(path: &Path) -> &'static str {
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    match extension.as_str() {
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "txt" => "text/plain",
        _ => "application/octet-stream",
    }
}