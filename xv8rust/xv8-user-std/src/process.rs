use alloc::string::String;
use alloc::vec::Vec;

use crate::ffi::CString;
use crate::fs::File;
use crate::io::{self, Read, Write};

pub fn exit(code: i32) -> ! {
    xv8_libc::exit(code as usize)
}

pub fn id() -> u32 {
    xv8_libc::getpid() as u32
}

pub struct Command {
    prog: String,
    args: Vec<String>,
    stdin: Stdio,
    stdout: Stdio,
    stderr: Stdio,
}

impl Command {
    pub fn new(prog: &str) -> Self {
        Command {
            prog: prog.into(),
            args: Vec::new(),
            stdin: Stdio::Inherit,
            stdout: Stdio::Inherit,
            stderr: Stdio::Inherit,
        }
    }

    pub fn arg<S: AsRef<str>>(&mut self, arg: S) -> &mut Self {
        self.args.push(arg.as_ref().into());
        self
    }

    pub fn args<I: IntoIterator<Item = S>, S: AsRef<str>>(&mut self, args: I) -> &mut Self {
        for arg in args {
            self.args.push(arg.as_ref().into());
        }
        self
    }

    pub fn env_clear(&mut self) -> &mut Self { self }
    pub fn env<K: AsRef<str>, V: AsRef<str>>(&mut self, _key: K, _val: V) -> &mut Self { self }
    pub fn env_remove<K: AsRef<str>>(&mut self, _key: K) -> &mut Self { self }
    pub fn envs<I: IntoIterator<Item = (K, V)>, K: AsRef<str>, V: AsRef<str>>(&mut self, _envs: I) -> &mut Self { self }

    pub fn stdin<T: Into<Stdio>>(&mut self, config: T) -> &mut Self {
        self.stdin = config.into();
        self
    }

    pub fn stdout<T: Into<Stdio>>(&mut self, config: T) -> &mut Self {
        self.stdout = config.into();
        self
    }

    pub fn stderr<T: Into<Stdio>>(&mut self, config: T) -> &mut Self {
        self.stderr = config.into();
        self
    }

    pub fn status(&mut self) -> io::Result<ExitStatus> {
        let mut child = self.spawn()?;
        child.wait()
    }

    pub fn spawn(&mut self) -> io::Result<Child> {
        let resolved_prog = resolve_program(&self.prog).ok_or(io::ErrorKind::NotFound)?;
        let program = CString::new(resolved_prog.as_str()).map_err(|_| io::ErrorKind::InvalidInput)?;
        let argv_storage = build_argv(&self.prog, &self.args)?;

        let mut stdin_pipe = None;
        let mut stdout_pipe = None;
        let mut stderr_pipe = None;
        let mut null_stdin_pipe = None;

        if matches!(&self.stdin, Stdio::Piped) {
            let (read_fd, write_fd) = make_pipe()?;
            stdin_pipe = Some((read_fd, write_fd));
        } else if matches!(&self.stdin, Stdio::Null) {
            let (read_fd, write_fd) = make_pipe()?;
            null_stdin_pipe = Some((read_fd, write_fd));
        }

        if matches!(&self.stdout, Stdio::Piped) {
            let (read_fd, write_fd) = make_pipe()?;
            stdout_pipe = Some((read_fd, write_fd));
        }

        if matches!(&self.stderr, Stdio::Piped) {
            let (read_fd, write_fd) = make_pipe()?;
            stderr_pipe = Some((read_fd, write_fd));
        }

        let pid = xv8_libc::fork();
        if pid < 0 {
            close_pipe(stdin_pipe);
            close_pipe(stdout_pipe);
            close_pipe(stderr_pipe);
            close_pipe(null_stdin_pipe);
            return Err(io::ErrorKind::Other.into());
        }

        if pid == 0 {
            if let Some((read_fd, write_fd)) = stdin_pipe {
                let _ = xv8_libc::dup2(read_fd, 0);
                let _ = xv8_libc::close(read_fd);
                let _ = xv8_libc::close(write_fd);
            } else if let Some((read_fd, write_fd)) = null_stdin_pipe {
                let _ = xv8_libc::dup2(read_fd, 0);
                let _ = xv8_libc::close(read_fd);
                let _ = xv8_libc::close(write_fd);
            }

            if let Some((read_fd, write_fd)) = stdout_pipe {
                let _ = xv8_libc::dup2(write_fd, 1);
                let _ = xv8_libc::close(read_fd);
                let _ = xv8_libc::close(write_fd);
            }

            if let Some((read_fd, write_fd)) = stderr_pipe {
                let _ = xv8_libc::dup2(write_fd, 2);
                let _ = xv8_libc::close(read_fd);
                let _ = xv8_libc::close(write_fd);
            }

            let mut argv_ptrs = argv_storage.iter().map(|arg| arg.as_ptr() as *const u8).collect::<Vec<_>>();
            argv_ptrs.push(core::ptr::null());
            let _ = xv8_libc::exec(program.as_ptr() as *const u8, argv_ptrs.as_ptr());
            xv8_libc::exit(127);
        }

        if let Some((read_fd, write_fd)) = stdin_pipe {
            let _ = xv8_libc::close(read_fd);
            return Ok(Child {
                pid: pid as usize,
                exit_status: None,
                stdin: Some(ChildStdin(File::from_raw_fd(write_fd))),
                stdout: stdout_pipe.map(|(read_fd, write_fd)| {
                    let _ = xv8_libc::close(write_fd);
                    ChildStdout(File::from_raw_fd(read_fd))
                }),
                stderr: stderr_pipe.map(|(read_fd, write_fd)| {
                    let _ = xv8_libc::close(write_fd);
                    ChildStderr(File::from_raw_fd(read_fd))
                }),
            });
        }

        if let Some((read_fd, write_fd)) = null_stdin_pipe {
            let _ = xv8_libc::close(read_fd);
            let _ = xv8_libc::close(write_fd);
        }

        Ok(Child {
            pid: pid as usize,
            exit_status: None,
            stdin: None,
            stdout: stdout_pipe.map(|(read_fd, write_fd)| {
                let _ = xv8_libc::close(write_fd);
                ChildStdout(File::from_raw_fd(read_fd))
            }),
            stderr: stderr_pipe.map(|(read_fd, write_fd)| {
                let _ = xv8_libc::close(write_fd);
                ChildStderr(File::from_raw_fd(read_fd))
            }),
        })
    }

    pub fn output(&mut self) -> io::Result<Output> {
        self.stdin(Stdio::Null);
        self.stdout(Stdio::Piped);
        self.stderr(Stdio::Piped);
        let child = self.spawn()?;
        child.wait_with_output()
    }
}

fn build_argv(prog: &str, args: &[String]) -> io::Result<Vec<CString>> {
    let mut argv = Vec::with_capacity(args.len() + 1);
    argv.push(CString::new(prog).map_err(|_| io::ErrorKind::InvalidInput)?);
    for arg in args {
        argv.push(CString::new(arg.as_str()).map_err(|_| io::ErrorKind::InvalidInput)?);
    }
    Ok(argv)
}

fn resolve_program(prog: &str) -> Option<String> {
    if prog.contains('/') {
        return Some(String::from(prog));
    }

    let search_path = crate::env::var("PATH").unwrap_or_else(|_| String::from("/:/bin:/usr/bin:/usr/local/bin"));
    for directory in search_path.split(':').chain(["/", "/bin", "/usr/bin", "/usr/local/bin"].into_iter()) {
        let candidate = if directory.is_empty() {
            String::from(prog)
        } else if directory.ends_with('/') {
            format!("{}{}", directory, prog)
        } else {
            format!("{}/{}", directory, prog)
        };
        if crate::path::Path::new(candidate.as_bytes()).exists() {
            return Some(candidate);
        }
    }

    None
}

fn make_pipe() -> io::Result<(usize, usize)> {
    let mut fds = [0usize; 2];
    let ret = xv8_libc::pipe(fds.as_mut_ptr());
    if ret < 0 {
        Err(io::ErrorKind::Other.into())
    } else {
        Ok((fds[0], fds[1]))
    }
}

fn close_pipe(pipe: Option<(usize, usize)>) {
    if let Some((read_fd, write_fd)) = pipe {
        let _ = xv8_libc::close(read_fd);
        let _ = xv8_libc::close(write_fd);
    }
}

pub struct Child {
    pid: usize,
    exit_status: Option<ExitStatus>,
    pub stdin: Option<ChildStdin>,
    pub stdout: Option<ChildStdout>,
    pub stderr: Option<ChildStderr>,
}

impl Child {
    pub fn id(&self) -> usize { self.pid }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        if let Some(status) = self.exit_status {
            return Ok(status);
        }
        let _ = self.stdin.take();

        loop {
            let mut status = 0usize;
            let waited = xv8_libc::wait(&mut status as *mut usize);
            if waited < 0 {
                return Err(io::ErrorKind::Other.into());
            }
            if waited as usize == self.pid {
                let es = ExitStatus { code: status as i32 };
                self.exit_status = Some(es);
                return Ok(es);
            }
        }
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        if let Some(status) = self.exit_status {
            return Ok(Some(status));
        }
        Ok(None)
    }

    pub fn wait_with_output(mut self) -> io::Result<Output> {
        let _ = self.stdin.take();

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        match (self.stdout.take(), self.stderr.take()) {
            (None, None) => {}
            (Some(mut out), None) => {
                let _ = out.read_to_end(&mut stdout);
            }
            (None, Some(mut err)) => {
                let _ = err.read_to_end(&mut stderr);
            }
            (Some(out), Some(err)) => {
                let _ = read_output(out, err, &mut stdout, &mut stderr);
            }
        }

        let status = self.wait()?;
        Ok(Output { status, stdout, stderr })
    }
}

fn read_output(out: ChildStdout, err: ChildStderr, stdout: &mut Vec<u8>, stderr: &mut Vec<u8>) -> io::Result<()> {
    use xv8_libc::{PollFd, POLLIN, POLLHUP};

    let out_fd = out.0.as_raw_fd();
    let err_fd = err.0.as_raw_fd();

    let mut fds = [
        PollFd { fd: out_fd as i32, events: POLLIN, revents: 0 },
        PollFd { fd: err_fd as i32, events: POLLIN, revents: 0 },
    ];

    let mut buf = [0u8; 4096];
    let mut out_done = false;
    let mut err_done = false;

    while !out_done || !err_done {
        if out_done { fds[0].events = 0; } else { fds[0].events = POLLIN; }
        if err_done { fds[1].events = 0; } else { fds[1].events = POLLIN; }
        fds[0].revents = 0;
        fds[1].revents = 0;

        let ret = xv8_libc::poll(fds.as_mut_ptr(), 2, -1);
        if ret < 0 {
            return Err(io::ErrorKind::Other.into());
        }

        if (fds[0].revents & POLLIN) != 0 && !out_done {
            let n = xv8_libc::read(out_fd, buf.as_mut_ptr(), buf.len());
            if n > 0 {
                stdout.extend_from_slice(&buf[..n as usize]);
            } else {
                out_done = true;
            }
        }
        if (fds[0].revents & POLLHUP) != 0 {
            out_done = true;
        }

        if (fds[1].revents & POLLIN) != 0 && !err_done {
            let n = xv8_libc::read(err_fd, buf.as_mut_ptr(), buf.len());
            if n > 0 {
                stderr.extend_from_slice(&buf[..n as usize]);
            } else {
                err_done = true;
            }
        }
        if (fds[1].revents & POLLHUP) != 0 {
            err_done = true;
        }
    }

    Ok(())
}

pub struct ChildStdin(File);

impl Write for ChildStdin {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

pub struct ChildStdout(File);

impl Read for ChildStdout {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

pub struct ChildStderr(File);

impl Read for ChildStderr {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

pub struct Output {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[derive(Clone, Copy)]
pub struct ExitStatus {
    code: i32,
}

impl ExitStatus {
    pub fn code(&self) -> Option<i32> {
        Some(self.code)
    }

    pub fn success(&self) -> bool {
        self.code == 0
    }
}

pub enum Stdio {
    Inherit,
    Null,
    Piped,
    File(File),
}

impl Stdio {
    pub fn piped() -> Self { Stdio::Piped }
    pub fn null() -> Self { Stdio::Null }
    pub fn inherit() -> Self { Stdio::Inherit }
}

impl Default for Stdio {
    fn default() -> Self { Stdio::Inherit }
}

impl From<File> for Stdio {
    fn from(file: File) -> Self { Stdio::File(file) }
}

impl From<ChildStdout> for Stdio {
    fn from(stdout: ChildStdout) -> Self { Stdio::File(stdout.0) }
}

impl From<super::io::Stdin> for Stdio {
    fn from(_stdin: super::io::Stdin) -> Self { Stdio::Inherit }
}

impl From<super::io::Stdout> for Stdio {
    fn from(_stdout: super::io::Stdout) -> Self { Stdio::Inherit }
}
