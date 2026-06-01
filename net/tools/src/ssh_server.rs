use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;

use ssh2::{KeyType, Listener, Session};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut port = 2222;
    if args.len() > 1 {
        port = args[1].parse().unwrap_or(2222);
    }

    // Generate a temporary RSA key pair for host key
    let mut rng = rand::thread_rng();
    let private_key = ssh2::key::generate_rsa_key(Some(2048), &mut rng).unwrap();
    let public_key = private_key.public_key();

    // Save the keys to temporary files (in a real server, you'd load from disk)
    let private_key_path = "/tmp/ssh_host_rsa_key";
    let public_key_path = "/tmp/ssh_host_rsa_key.pub";
    fs::write(private_key_path, private_key.pem().as_bytes()).unwrap();
    fs::write(public_key_path, format!("{} {}", public_key.base64(), "host-key")).unwrap();

    // Set up the listener
    let tcp_listener = TcpListener::bind(("0.0.0.0", port)).unwrap();
    println!("SSH server listening on 0.0.0.0:{}", port);
    println!("Host key files: {} and {}", private_key_path, public_key_path);
    println!("Try connecting with: ssh -p {} testuser@localhost", port);
    println!("Password: testpass");

    let listener = Listener::new(tcp_listener).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                // Handle each connection in a new thread
                thread::spawn(move || {
                    handle_client(stream, &private_key);
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}

fn handle_client(stream: std::net::TcpStream, private_key: &ssh2::key::KeyPair) {
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(stream);
    if let Err(e) = sess.handshake() {
        eprintln!("SSH handshake failed: {}", e);
        return;
    }

    // Host key authentication (server sends its public key)
    sess.hostkeys()
        .add(private_key, KeyType::RSA)
        .unwrap();

    // We don't verify the client's host key (it's a client, so it doesn't have one to verify in the same way)
    // In a real server, you would have a known_hosts file for the client, but we skip for simplicity.

    // Authenticate the client
    // We'll use a simple username/password authentication
    let username = "testuser";
    let password = "testpass";

    if let Err(e) = sess.userauth_password(username, password) {
        eprintln!("Authentication failed for {}: {}", username, e);
        let _ = sess.disconnect(None, "Authentication failed", "");
        return;
    }

    println!("Authenticated user: {}", username);

    // Open a channel and execute a command
    let mut channel = match sess.channel_session() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to open channel: {}", e);
            let _ = sess.disconnect(None, "Unable to open channel", "");
            return;
        }
    };

    // We'll run a simple command
    let command = "echo 'SSH server is working'; uname -a";
    if let Err(e) = channel.exec(command) {
        eprintln!("Failed to execute command: {}", e);
        let _ = sess.disconnect(None, "Command execution failed", "");
        return;
    }

    // Read the output and send it to the client
    let mut output = String::new();
    if let Err(e) = channel.read_to_string(&mut output) {
        eprintln!("Failed to read command output: {}", e);
    } else {
        // Send the output back to the client
        let _ = channel.write(output.as_bytes());
        let _ = channel.flush();
    }

    // Close the channel
    let _ = channel.close();
    let _ = channel.wait_close();

    // Disconnect the session
    let _ = sess.disconnect(None, "Bye", "");
}