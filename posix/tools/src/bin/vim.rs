use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::env;
use std::fs;
use std::io::{stdout, BufRead, BufReader, Result, Write};

/// 編輯器的三種基本模式
#[derive(PartialEq)]
enum Mode {
    Normal,
    Insert,
    Command,
}

struct Editor {
    cx: usize,
    cy: usize,
    mode: Mode,
    lines: Vec<Vec<char>>,
    should_quit: bool,
    command_buffer: String,
    filename: Option<String>,
    modified: bool,
}

impl Editor {
    fn new(filename: Option<String>) -> Self {
        let lines = if let Some(ref fname) = filename {
            match fs::File::open(fname) {
                Ok(file) => {
                    let reader = BufReader::new(file);
                    reader.lines().map(|l| l.unwrap_or_default().chars().collect()).collect()
                }
                Err(_) => vec![Vec::new()],
            }
        } else {
            vec![Vec::new()]
        };
        Self {
            cx: 0,
            cy: 0,
            mode: Mode::Normal,
            lines,
            should_quit: false,
            command_buffer: String::new(),
            filename,
            modified: false,
        }
    }

    fn run(&mut self) -> Result<()> {
        while !self.should_quit {
            self.draw_screen()?;
            self.process_keypress()?;
        }
        Ok(())
    }

    fn draw_screen(&self) -> Result<()> {
        let mut stdout = stdout();
        let (_cols, rows) = size()?;
        
        queue!(stdout, Hide, MoveTo(0, 0))?;

        // 繪製文字內容
        for y in 0..(rows - 1) {
            queue!(stdout, Clear(ClearType::CurrentLine))?;
            // 修正 2：將 y as usize 用括號包起來，避免編譯器誤認為是泛型括號 < >
            if (y as usize) < self.lines.len() {
                let line: String = self.lines[y as usize].iter().collect();
                queue!(stdout, Print(line))?;
            } else {
                queue!(stdout, Print("~"))?;
            }
            queue!(stdout, Print("\r\n"))?;
        }

        // 繪製狀態列 / 命令列
        queue!(stdout, Clear(ClearType::CurrentLine))?;
        queue!(
            stdout,
            SetBackgroundColor(Color::White),
            SetForegroundColor(Color::Black)
        )?;
        
        let fname = self.filename.as_deref().unwrap_or("[No Name]");
        let modified = if self.modified { " [+]"} else { "" };
        match self.mode {
            Mode::Normal => queue!(stdout, Print(format!(" {}{}  NORMAL ", fname, modified)))?,
            Mode::Insert => queue!(stdout, Print(format!(" {}{}  INSERT ", fname, modified)))?,
            Mode::Command => queue!(stdout, Print(format!(" :{} ", self.command_buffer)))?,
        }
        
        queue!(stdout, ResetColor)?;

        // 放置游標
        if self.mode == Mode::Command {
            queue!(stdout, MoveTo((self.command_buffer.len() + 2) as u16, rows - 1))?;
        } else {
            queue!(stdout, MoveTo(self.cx as u16, self.cy as u16))?;
        }

        queue!(stdout, Show)?;
        stdout.flush()?;
        Ok(())
    }

    fn process_keypress(&mut self) -> Result<()> {
        if let Event::Key(KeyEvent { code, modifiers, .. }) = read()? {
            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                self.should_quit = true;
                return Ok(());
            }

            match self.mode {
                Mode::Normal => self.process_normal(code),
                Mode::Insert => self.process_insert(code),
                Mode::Command => self.process_command(code),
            }
        }
        Ok(())
    }

    fn process_normal(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('h') | KeyCode::Left => self.cx = self.cx.saturating_sub(1),
            KeyCode::Char('j') | KeyCode::Down => {
                if self.cy < self.lines.len() - 1 {
                    self.cy += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => self.cy = self.cy.saturating_sub(1),
            KeyCode::Char('l') | KeyCode::Right => self.cx += 1,
            
            KeyCode::Char('i') => self.mode = Mode::Insert,
            KeyCode::Char(':') => {
                self.mode = Mode::Command;
                self.command_buffer.clear();
            }
            _ => {}
        }
        self.fix_cursor();
    }

    fn process_insert(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.cx = self.cx.saturating_sub(1);
            }
            KeyCode::Left => self.cx = self.cx.saturating_sub(1),
            KeyCode::Right => self.cx += 1,
            KeyCode::Up => self.cy = self.cy.saturating_sub(1),
            KeyCode::Down => {
                if self.cy < self.lines.len() - 1 {
                    self.cy += 1;
                }
            }
            KeyCode::Char(c) => {
                self.lines[self.cy].insert(self.cx, c);
                self.cx += 1;
                self.modified = true;
            }
            KeyCode::Enter => {
                let rest: Vec<char> = self.lines[self.cy].drain(self.cx..).collect();
                self.lines.insert(self.cy + 1, rest);
                self.cy += 1;
                self.cx = 0;
                self.modified = true;
            }
            KeyCode::Backspace => {
                if self.cx > 0 {
                    self.cx -= 1;
                    self.lines[self.cy].remove(self.cx);
                } else if self.cy > 0 {
                    let current_line = self.lines.remove(self.cy);
                    self.cy -= 1;
                    self.cx = self.lines[self.cy].len();
                    self.lines[self.cy].extend(current_line);
                }
                self.modified = true;
            }
            _ => {}
        }
        self.fix_cursor();
    }

    fn save(&mut self) {
        let fname = match self.filename.clone() {
            Some(n) => n,
            None => return,
        };
        let content: String = self.lines.iter().map(|l| l.iter().collect::<String>() + "\n").collect();
        let _ = fs::write(&fname, content);
        self.modified = false;
    }

    fn process_command(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Enter => {
                let cmd = self.command_buffer.trim().to_string();
                if cmd == "q" || cmd == "q!" {
                    self.should_quit = true;
                } else if cmd == "w" {
                    self.save();
                } else if cmd == "wq" || cmd == "x" {
                    self.save();
                    self.should_quit = true;
                }
                self.mode = Mode::Normal;
            }
            KeyCode::Char(c) => self.command_buffer.push(c),
            KeyCode::Backspace => {
                self.command_buffer.pop();
            }
            _ => {}
        }
    }

    fn fix_cursor(&mut self) {
        if self.cy >= self.lines.len() {
            self.cy = self.lines.len().saturating_sub(1);
        }
        let line_len = self.lines[self.cy].len();
        
        if self.cx > line_len {
            self.cx = line_len;
        }
        
        if self.mode == Mode::Normal && self.cx == line_len && line_len > 0 {
            self.cx = line_len - 1;
        }
    }
}

struct Cleanup;
impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
    }
}

fn main() -> Result<()> {
    let filename = env::args().nth(1);

    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    
    let _cleanup = Cleanup;

    let mut editor = Editor::new(filename);
    editor.run()?;

    Ok(())
}