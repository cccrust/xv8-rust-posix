use crate::terminal::WindowSize;
use spin::Mutex;
use std::io;

static TERMINAL_MODE_PRIOR_RAW: Mutex<bool> = Mutex::new(false);

#[cfg(feature = "events")]
pub fn supports_keyboard_enhancement() -> io::Result<bool> {
    Ok(false)
}

pub(crate) fn is_raw_mode_enabled() -> bool {
    *TERMINAL_MODE_PRIOR_RAW.lock()
}

pub(crate) fn enable_raw_mode() -> io::Result<()> {
    let mut raw = TERMINAL_MODE_PRIOR_RAW.lock();
    if *raw {
        return Ok(());
    }
    let fd = 0;
    let ret = xv8_libc::ioctl(fd, xv8_libc::IoctlCmd::CONSOLE_SET_RAW, 1);
    if ret < 0 {
        return Err(io::Error::last_os_error());
    }
    *raw = true;
    Ok(())
}

pub(crate) fn disable_raw_mode() -> io::Result<()> {
    let mut raw = TERMINAL_MODE_PRIOR_RAW.lock();
    if let true = *raw {
        let fd = 0;
        let ret = xv8_libc::ioctl(fd, xv8_libc::IoctlCmd::CONSOLE_SET_RAW, 0);
        if ret < 0 {
            return Err(io::Error::last_os_error());
        }
        *raw = false;
    }
    Ok(())
}

pub(crate) fn window_size() -> io::Result<WindowSize> {
    Ok(WindowSize {
        columns: 80,
        rows: 24,
        width: 0,
        height: 0,
    })
}

pub(crate) fn size() -> io::Result<(u16, u16)> {
    let ws = window_size()?;
    Ok((ws.columns, ws.rows))
}