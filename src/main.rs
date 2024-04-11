use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::Path,
};
use crossterm::{
    cursor::{self, MoveTo, Hide},
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

enum Mode {
    Normal,
    Command,
}

struct Editor {
    buffer: Vec<String>,
    cursor_position: (u16, u16),
    file_name: Option<String>,
    mode: Mode,
}

impl Editor {
    fn new() -> Self {
        Self {
            buffer: Vec::new(),
            cursor_position: (0, 0),
            file_name: None,
            mode: Mode::Normal,
        }
    }

    fn load_file(&mut self, file_path: &str) -> io::Result<()> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        self.buffer.clear();
        for line in reader.lines() {
            self.buffer.push(line?);
        }
        self.file_name = Some(file_path.to_string());

        Ok(())
    }

    fn save_file(&self, file_path: &str) -> io::Result<()> {
        let mut file = File::create(file_path)?;
        for line in &self.buffer {
            writeln!(file, "{}", line)?;
        }
        Ok(())
    }

    fn print_buffer(&self) -> io::Result<()> {
        let mut stdout = io::stdout();
        terminal::enable_raw_mode()?;
        execute!(stdout, cursor::Hide)?;

        // Clear screen
        execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

        // Print buffer with syntax highlighting
        for (y, line) in self.buffer.iter().enumerate() {
            if y == self.cursor_position.0 as usize {
                // Print cursor line
                execute!(stdout, cursor::MoveTo(0, y as u16))?;
                self.print_syntax_highlighted_line(line)?;
                execute!(
                    stdout,
                    cursor::MoveTo(self.cursor_position.1, self.cursor_position.0)
                )?;
            } else {
                // Print normal line
                writeln!(stdout, "{}", line)?;
            }
        }

        stdout.flush()?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    fn print_syntax_highlighted_line(&self, line: &str) -> io::Result<()> {
        let mut stdout = io::stdout();
        let keywords = ["fn", "let", "mut", "match", "if", "else", "for", "loop", "while", "return"];

        for word in line.split_whitespace() {
            if keywords.contains(&word) {
                execute!(stdout, SetForegroundColor(Color::Yellow))?;
                write!(stdout, "{} ", word)?;
                execute!(stdout, ResetColor)?;
            } else {
                write!(stdout, "{} ", word)?;
            }
        }
        writeln!(stdout)?;

        Ok(())
    }

    fn handle_input(&mut self, key_event: KeyEvent) -> io::Result<()> {
        match self.mode {
            Mode::Normal => self.handle_normal_mode_input(key_event),
            Mode::Command => self.handle_command_mode_input(key_event),
        }
    }

    fn handle_normal_mode_input(&mut self, key_event: KeyEvent) -> io::Result<()> {
        match key_event.code {
            KeyCode::Char(c) => {
                self.buffer[self.cursor_position.0 as usize].insert(self.cursor_position.1 as usize, c);
                self.cursor_position.1 += 1;
            }
            KeyCode::Backspace => {
                if self.cursor_position.1 > 0 {
                    self.buffer[self.cursor_position.0 as usize].remove(self.cursor_position.1 as usize - 1);
                    self.cursor_position.1 -= 1;
                }
            }
            KeyCode::Enter => {
                let current_line = self.buffer.remove(self.cursor_position.0 as usize);
                let (first_half, second_half) = current_line.split_at(self.cursor_position.1 as usize);
                self.buffer.insert(self.cursor_position.0 as usize, first_half.to_string());
                self.buffer.insert(self.cursor_position.0 as usize + 1, second_half.to_string());
                self.cursor_position.0 += 1;
                self.cursor_position.1 = 0;
            }
            KeyCode::Left => {
                if self.cursor_position.1 > 0 {
                    self.cursor_position.1 -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_position.1 < self.buffer[self.cursor_position.0 as usize].len() as u16 {
                    self.cursor_position.1 += 1;
                }
            }
            KeyCode::Up => {
                if self.cursor_position.0 > 0 {
                    self.cursor_position.0 -= 1;
                }
            }
            KeyCode::Down => {
                if self.cursor_position.0 < self.buffer.len() as u16 - 1 {
                    self.cursor_position.0 += 1;
                }
            }
            KeyCode::Char(':') => self.mode = Mode::Command,
            _ => {}
        }
        Ok(())
    }

    fn handle_command_mode_input(&mut self, key_event: KeyEvent) -> io::Result<()> {
        match key_event.code {
            KeyCode::Char(c) => {
                // Handle command input
                if c == 'w' {
                    if let Some(ref file_name) = self.file_name {
                        self.save_file(file_name)?;
                    } else {
                        println!("No file name specified.");
                    }
                } else if c == 'q' {
                    // Quit command mode
                    self.mode = Mode::Normal;
                }
            }
            KeyCode::Esc => self.mode = Mode::Normal,
            _ => {}
        }
        Ok(())
    }
}


fn main() -> io::Result<()> {
    let mut editor = Editor::new();
    let file_path = "example.rs"; // Change this to the path of an existing file you want to edit

    editor.load_file(file_path)?;

    execute!(io::stdout(), EnterAlternateScreen, Hide)?;

    loop {
        editor.print_buffer()?;

        if let Event::Key(key_event) = event::read()? {
            match key_event {
                KeyEvent { code: KeyCode::Char('c'), modifiers: event::KeyModifiers::CONTROL, .. } => break,
                KeyEvent { code: KeyCode::Char('s'), modifiers: event::KeyModifiers::CONTROL, .. } => {
                    if let Some(ref file_name) = editor.file_name {
                        editor.save_file(file_name)?;
                    } else {
                        println!("No file name specified.");
                    }
                }
                key_event => editor.handle_input(key_event)?,
            }
        }
    }

    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}