use std::fs;
use std::io::{self, Read, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = if args.len() > 1 { &args[1] } else { "_tmp_vi" };

    // Read file
    let mut buffer: Vec<String> = if args.len() > 1 {
        fs::read_to_string(path).unwrap_or_default().lines().map(|s| s.to_string()).collect()
    } else {
        vec![String::new()]
    };
    if buffer.is_empty() { buffer.push(String::new()); }

    // Terminal setup
    let mut tio: libc::termios = unsafe { std::mem::zeroed() };
    unsafe { libc::tcgetattr(libc::STDIN_FILENO, &mut tio); }
    let orig = tio;
    set_raw_mode(&mut tio, true);
    let (rows, cols) = term_size();
    let mut cx = 0;
    let mut cy = 0;
    let mut top = 0; // scroll offset
    let mut mode = 'n'; // n=normal, i=insert, :=command
    let mut cmd = String::new();
    let mut modified = false;

    loop {
        // Render
        let screen_rows = (rows as usize).saturating_sub(1);
        let mut out = String::new();
        // Clear screen and move home
        out.push_str("\x1b[2J\x1b[H");
        for y in 0..screen_rows {
            let idx = top + y;
            if idx < buffer.len() {
                let line = &buffer[idx];
                let display = if line.len() > cols as usize { &line[..cols as usize] } else { line.as_str() };
                out.push_str(display);
                // Clear to end of line
                out.push_str("\x1b[K");
            } else {
                out.push_str("~\x1b[K");
            }
            if y + 1 < screen_rows { out.push('\n'); }
        }
        // Status line
        let fname = if path == "_tmp_vi" { "[No Name]" } else { path };
        let mod_ind = if modified { "[+]" } else { "" };
        let mode_name = match mode { 'i' => "INSERT", ':' => "COMMAND", _ => "NORMAL" };
        let status = format!("{} {} {}  line {}/{} col {}", fname, mod_ind, mode_name, cy + 1, buffer.len(), cx + 1);
        out.push_str(&status);
        // Clear rest of status line
        out.push_str("\x1b[K");
        // Move cursor
        let screen_cx = cx.min(cols as usize - 1);
        let screen_cy = (cy - top).min(screen_rows - 1);
        out.push_str(&format!("\x1b[{};{}H", screen_cy + 1, screen_cx + 1));
        io::stdout().write_all(out.as_bytes()).unwrap();
        io::stdout().flush().unwrap();

        // Handle input
        let mut buf = [0u8; 8];
        let n = io::stdin().read(&mut buf).unwrap_or(0);
        if n == 0 { break; }

        match mode {
            'n' => handle_normal(&buf[..n], &mut buffer, &mut cx, &mut cy, &mut top, &mut mode, &mut cmd, &mut modified, screen_rows, cols as usize),
            'i' => handle_insert(&buf[..n], &mut buffer, &mut cx, &mut cy, &mut mode, &mut modified),
            ':' => handle_command(&buf[..n], &mut cmd, &mut mode, &mut buffer, &mut modified, path, &mut cx, &mut cy, &mut top),
            _ => {}
        }
        if mode == 'q' { break; }
    }

    set_raw_mode(&mut tio, false);
    unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &orig); }
}

fn handle_normal(buf: &[u8], buffer: &mut Vec<String>, cx: &mut usize, cy: &mut usize, top: &mut usize,
    mode: &mut char, cmd: &mut String, modified: &mut bool, screen_rows: usize, _cols: usize) {
    let b = buf[0];
    match b {
        b'h' | 0x08 => { if *cx > 0 { *cx -= 1; } }
        b'l' => { *cx = (*cx + 1).min(buffer[*cy].len()); }
        b'j' => {
            if *cy + 1 < buffer.len() {
                *cy += 1;
                *cx = (*cx).min(buffer[*cy].len());
                if *cy >= *top + screen_rows { *top = *cy - screen_rows + 1; }
            }
        }
        b'k' => {
            if *cy > 0 {
                *cy -= 1;
                *cx = (*cx).min(buffer[*cy].len());
                if *cy < *top { *top = *cy; }
            }
        }
        b'i' => { *mode = 'i'; }
        b'a' => { if *cx < buffer[*cy].len() { *cx += 1; } *mode = 'i'; }
        b'o' => {
            buffer.insert(*cy + 1, String::new());
            *cy += 1;
            *cx = 0;
            if *cy >= *top + screen_rows { *top = *cy - screen_rows + 1; }
            *modified = true;
            *mode = 'i';
        }
        b'x' => {
            if *cx < buffer[*cy].len() {
                buffer[*cy].remove(*cx);
                *modified = true;
            }
        }
        b'd' => {
            // dd delete line
            if buf.len() > 1 && buf[1] == b'd' {
                if buffer.len() > 1 {
                    buffer.remove(*cy);
                    if *cy >= buffer.len() { *cy = buffer.len() - 1; }
                    *cx = (*cx).min(buffer[*cy].len());
                    *modified = true;
                }
            }
        }
        b'u' => { /* undo — no-op for simple version */ }
        b':' => {
            *mode = ':';
            cmd.clear();
        }
        b'0' => { *cx = 0; }
        b'$' => { *cx = buffer[*cy].len(); }
        b'w' => { move_forward_word(buffer, cx, cy); }
        b'b' => { move_back_word(buffer, cx, cy); }
        b'G' => {
            *cy = buffer.len() - 1;
            if *cy >= *top + screen_rows { *top = *cy - screen_rows + 1; }
        }
        b'g' => {
            *cy = 0;
            *top = 0;
            *cx = 0;
        }
        _ => {}
    }
}

fn handle_insert(buf: &[u8], buffer: &mut Vec<String>, cx: &mut usize, cy: &mut usize, mode: &mut char, modified: &mut bool) {
    let b = buf[0];
    match b {
        0x1b => { // Esc
            *mode = 'n';
            if *cx > 0 { *cx -= 1; }
        }
        0x7f | 0x08 => { // Backspace
            if *cx > 0 {
                buffer[*cy].remove(*cx - 1);
                *cx -= 1;
                *modified = true;
            } else if *cy > 0 {
                let prev_len = buffer[*cy - 1].len();
                let line = buffer.remove(*cy);
                *cy -= 1;
                buffer[*cy].push_str(&line);
                *cx = prev_len;
                *modified = true;
            }
        }
        0x0a | 0x0d => { // Enter
            let rest = buffer[*cy].split_off(*cx);
            buffer.insert(*cy + 1, rest);
            *cy += 1;
            *cx = 0;
            *modified = true;
        }
        b if b >= 0x20 && b <= 0x7e => { // Printable
            buffer[*cy].insert(*cx, b as char);
            *cx += 1;
            *modified = true;
        }
        _ => {}
    }
}

fn handle_command(buf: &[u8], cmd: &mut String, mode: &mut char, buffer: &mut Vec<String>, modified: &mut bool, path: &str, cx: &mut usize, cy: &mut usize, top: &mut usize) {
    let b = buf[0];
    match b {
        0x1b => { *mode = 'n'; cmd.clear(); }
        0x0a | 0x0d => {
            match cmd.as_str() {
                "w" | "wq" => {
                    let content = buffer.join("\n") + "\n";
                    let _ = fs::write(path, &content);
                    *modified = false;
                    if cmd.as_str() == "wq" {
                        *mode = 'q';
                        return;
                    }
                }
                "q" => {
                    if !*modified { *mode = 'q'; return; }
                }
                "q!" => { *mode = 'q'; return; }
                "wq!" => {
                    let content = buffer.join("\n") + "\n";
                    let _ = fs::write(path, &content);
                    *mode = 'q'; return;
                }
                _ => {
                    if let Some(n) = cmd.parse::<usize>().ok() {
                        if n > 0 && n <= buffer.len() {
                            *cy = n - 1;
                            *cx = (*cx).min(buffer[*cy].len());
                            if *cy < *top { *top = *cy; }
                            if *cy >= *top + 20 { *top = (*cy).saturating_sub(19); }
                        }
                    }
                }
            }
            *mode = 'n';
            cmd.clear();
        }
        0x7f | 0x08 => { cmd.pop(); }
        b if b >= 0x20 && b <= 0x7e => { cmd.push(b as char); }
        _ => {}
    }
}

fn move_forward_word(buffer: &[String], cx: &mut usize, cy: &mut usize) {
    let line = &buffer[*cy];
    let mut pos = *cx + 1;
    while pos < line.len() && line.as_bytes()[pos] == b' ' { pos += 1; }
    while pos < line.len() && line.as_bytes()[pos] != b' ' { pos += 1; }
    *cx = pos.min(line.len());
}

fn move_back_word(buffer: &[String], cx: &mut usize, cy: &mut usize) {
    let line = &buffer[*cy];
    let mut pos = (*cx).saturating_sub(2);
    while pos > 0 && line.as_bytes()[pos] == b' ' { pos -= 1; }
    while pos > 0 && line.as_bytes()[pos] != b' ' { pos -= 1; }
    if pos > 0 || line.as_bytes().get(pos) != Some(&b' ') { *cx = pos + 1; }
    else { *cx = pos; }
}

fn set_raw_mode(tio: &mut libc::termios, enable: bool) {
    if enable {
        tio.c_iflag &= !(libc::BRKINT | libc::ICRNL | libc::IXON | libc::INPCK | libc::ISTRIP);
        tio.c_oflag &= !libc::OPOST;
        tio.c_cflag |= libc::CS8;
        tio.c_lflag &= !(libc::ECHO | libc::ICANON | libc::ISIG | libc::IEXTEN);
        tio.c_cc[libc::VMIN] = 1;
        tio.c_cc[libc::VTIME] = 0;
        unsafe {
            libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, tio);
        }
    } else {
        unsafe {
            libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, tio);
        }
    }
}

fn term_size() -> (u16, u16) {
    let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
    unsafe {
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) == 0 {
            return (ws.ws_row, ws.ws_col);
        }
    }
    (24, 80)
}
