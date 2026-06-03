use std::io;

pub(crate) fn cursor_position(column: u16, row: u16) -> String {
    format!("\x1B[{};{}H", row + 1, column + 1)
}

pub(crate) fn move_up(cells: u16) -> String {
    format!("\x1B[{}A", cells)
}

pub(crate) fn move_down(cells: u16) -> String {
    format!("\x1B[{}B", cells)
}

pub(crate) fn move_left(cells: u16) -> String {
    format!("\x1B[{}D", cells)
}

pub(crate) fn move_right(cells: u16) -> String {
    format!("\x1B[{}C", cells)
}

pub(crate) fn move_to_next_line(cells: u16) -> String {
    format!("\x1B[{}E", cells)
}

pub(crate) fn move_to_previous_line(cells: u16) -> String {
    format!("\x1B[{}F", cells)
}

pub(crate) fn move_to_column(column: u16) -> String {
    format!("\x1B[{}G", column + 1)
}

pub(crate) fn save_position() -> String {
    "\x1B7".to_string()
}

pub(crate) fn restore_position() -> String {
    "\x1B8".to_string()
}

pub(crate) fn hide_cursor() -> String {
    "\x1B[?25l".to_string()
}

pub(crate) fn show_cursor() -> String {
    "\x1B[?25h".to_string()
}

pub(crate) fn enable_blinking_cursor() -> String {
    "\x1B[?12h".to_string()
}

pub(crate) fn disable_blinking_cursor() -> String {
    "\x1B[?12l".to_string()
}

pub(crate) fn set_cursor_shape(shape: crate::cursor::CursorShape) -> String {
    match shape {
        crate::cursor::CursorShape::Default => "\x1B[0 q".to_string(),
        crate::cursor::CursorShape::Block => "\x1B[2 q".to_string(),
        crate::cursor::CursorShape::UnderScore => "\x1B[4 q".to_string(),
        crate::cursor::CursorShape::Line => "\x1B[6 q".to_string(),
    }
}

pub(crate) fn reset_cursor_shape() -> String {
    "\x1B[0 q".to_string()
}

pub(crate) fn cursor_up() -> String {
    move_up(1)
}

pub(crate) fn cursor_down() -> String {
    move_down(1)
}

pub(crate) fn cursor_left() -> String {
    move_left(1)
}

pub(crate) fn cursor_right() -> String {
    move_right(1)
}

/// This function is only used on Windows, but we need to have it available on all platforms.
/// For more info see `Command::write_to` in `src/cursor.rs`.
pub(crate) fn cursor_position_winapi(_column: u16, _row: u16) -> Result<String, io::Error> {
    Ok(cursor_position(_column, _row))
}