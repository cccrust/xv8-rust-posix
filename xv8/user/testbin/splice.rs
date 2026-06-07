#![no_std]
#![no_main]

use user::*;

fn test_splice_pipe_to_pipe() {
    let (r_fd_in, w_fd_in) = pipe2(0).expect("pipe_in");
    let (r_fd_out, w_fd_out) = pipe2(0).expect("pipe_out");

    let data = b"hello splice";
    write(w_fd_in, data).expect("write to in pipe");

    let n = splice(r_fd_in, core::ptr::null(), w_fd_out, core::ptr::null(), 64, 0)
        .expect("splice pipe to pipe");
    assert_eq!(n, data.len(), "splice returned {n}");

    close(w_fd_in).expect("close w_fd_in");
    close(w_fd_out).expect("close w_fd_out");

    let mut buf = [0u8; 64];
    let n = read(r_fd_out, &mut buf).expect("read from out pipe");
    assert_eq!(&buf[..n], data, "data mismatch");

    close(r_fd_in).expect("close r_fd_in");
    close(r_fd_out).expect("close r_fd_out");
    println!("ok splice_pipe_to_pipe");
}

fn test_tee_pipe_to_pipe() {
    let (r_fd_in, w_fd_in) = pipe2(0).expect("pipe_in");
    let (r_fd_out, w_fd_out) = pipe2(0).expect("pipe_out");

    let data = b"hello tee";
    write(w_fd_in, data).expect("write to in pipe");

    let n = tee(r_fd_in, w_fd_out, 64, 0).expect("tee pipe to pipe");
    assert_eq!(n, data.len(), "tee returned {n}");

    // tee also consumes from source in our implementation
    close(w_fd_in).expect("close w_fd_in");
    close(w_fd_out).expect("close w_fd_out");

    let mut buf = [0u8; 64];
    let n = read(r_fd_out, &mut buf).expect("read from out pipe");
    assert_eq!(&buf[..n], data, "data mismatch");

    close(r_fd_in).expect("close r_fd_in");
    close(r_fd_out).expect("close r_fd_out");
    println!("ok tee_pipe_to_pipe");
}

#[unsafe(no_mangle)]
fn main(_args: Args) {
    test_splice_pipe_to_pipe();
    test_tee_pipe_to_pipe();
}
