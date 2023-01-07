extern crate termion;

use std::io::{stdout, Read, Stdout, Write};
use termion::raw::RawTerminal;
use termion::{event::Key, input::TermRead, raw::IntoRawMode, terminal_size};

const X_PADDING: usize = 3;
const Y_PADDING: usize = 1;

struct Poem {
    buffer: String,
    cursor: usize,
    target_line_pos: usize,
    name: Option<String>,
}

fn make_input(input: &String) -> (Vec<String>, usize, usize) {
    // lines, x size, y size
    if input.len() == 0 {
        return (vec![String::from(" ")], 1, 1);
    }

    let lines: Vec<_> = input
        .lines()
        .into_iter()
        .map(|e| format!("{} ", e))
        .collect();
    let longest = lines
        .iter()
        .clone()
        .reduce(|acc, e| if e.len() > acc.len() { e } else { acc })
        .unwrap();

    (lines.clone(), longest.len(), lines.len())
}

fn get_name(maybe_name: Option<String>) -> String {
    maybe_name.unwrap_or(String::from("<no name>"))
}

fn frame_buffer(
    input: &Vec<String>,
    x_size: usize,
    name: Option<String>,
) -> (Vec<String>, usize, usize) {
    // framed lines, x size, y size

    let name_line = get_name(name);

    let x_size = x_size.max(name_line.len());

    let top_line = format!("┌{}┐", "─".repeat(x_size + X_PADDING * 2));
    let pad_line = format!("│{}│", " ".repeat(x_size + X_PADDING * 2));
    let bottom_line = format!("└{}┘", "─".repeat(x_size + X_PADDING * 2));

    let mut lines: Vec<String> = input
        .iter()
        .map(|e| {
            format!(
                "│{}{}{}{}│",
                " ".repeat(X_PADDING),
                e,
                " ".repeat(x_size - e.len()),
                " ".repeat(X_PADDING),
            )
        })
        .collect();

    let mut res = vec![top_line];
    for _ in 0..Y_PADDING {
        res.push(pad_line.clone());
    }
    res.append(&mut lines);
    for _ in 0..Y_PADDING {
        res.push(pad_line.clone());
    }
    res.push(bottom_line);

    (res.clone(), x_size + 2 + X_PADDING * 2, res.len())
}

enum EditOperation {
    Insert(char),
    DeleteRight,
    DeleteLeft,
    Newline,
}

impl Poem {
    fn from_str(text: &str) -> Self {
        Poem {
            buffer: String::from(text),
            cursor: 0,
            target_line_pos: 0,
            name: None,
        }
    }

    fn with_name(self, name: String) -> Self {
        Poem {
            name: Some(name),
            ..self
        }
    }

    fn modify(&mut self, edit: EditOperation) {
        use EditOperation::*;
        match edit {
            Insert(c) => {
                self.buffer.insert(self.cursor, c);
                self.cursor += 1;
            }
            DeleteRight => {
                if self.cursor == self.buffer.len() {
                    return;
                }
                self.buffer.remove(self.cursor);
            }
            DeleteLeft => {
                if self.cursor == 0 {
                    return;
                }
                self.cursor -= 1;
                if self.cursor + 1 == self.buffer.len() {
                    // String.remove doesn't work if we want
                    // to truncate the very last character, so
                    // we have to use a different method

                    self.buffer.pop();
                } else {
                    self.buffer.remove(self.cursor);
                }
            }
            Newline => {
                self.buffer.insert(self.cursor, '\n');
            }
        }
    }

    fn get_cursor_offset(&self) -> (u16, u16) {
        let mut counter = self.cursor;
        let mut res: (u16, u16) = (0, 0);

        for (idx, length) in self.buffer.split('\n').map(|e| e.len() + 1).enumerate() {
            if counter >= length {
                counter -= length;
                continue;
            }
            res = (counter as u16, idx as u16);
            break;
        }
        res
    }

    fn cursor_end_line(&mut self) {
        let current_line = self.get_cursor_offset().1;
        self.cursor += self.buffer.lines().nth(current_line.into()).unwrap().len()
            - self.get_cursor_offset().0 as usize;
    }

    fn cursor_start_line(&mut self) {
        while self.get_cursor_offset().0 != 0 {
            self.cursor -= 1;
        }
    }
}

fn draw_screen(stdout: &mut RawTerminal<Stdout>, poem: &Poem) {
    write!(
        stdout,
        "{}{}{}",
        termion::clear::All,
        termion::cursor::Show,
        termion::cursor::BlinkingBar
    )
    .unwrap();

    let size = terminal_size().unwrap();
    let (buf, xs, _) = make_input(&poem.buffer);
    let (frame, xs, ys) = frame_buffer(&buf, xs, None);

    let frame_corner = (size.0 / 2 - (xs / 2) as u16, size.1 / 2 - (ys / 2) as u16);

    write!(
        stdout,
        "{}{}{}",
        termion::cursor::Goto(frame_corner.0, frame_corner.1 - 1),
        get_name(poem.name.clone()),
        termion::cursor::Goto(frame_corner.0, frame_corner.1)
    );

    for line in frame {
        write!(
            stdout,
            "{}{}{}",
            line,
            termion::cursor::Left(xs as u16),
            termion::cursor::Down(1)
        )
        .unwrap();
    }

    let cursor_offset = poem.get_cursor_offset();
    /*
    println!(
        "cursor at {} -> {:?}; buffer is {} chars long",
        poem.cursor,
        poem.get_cursor_offset(),
        poem.buffer.len(),
    );
    */
    write!(
        stdout,
        "{}",
        termion::cursor::Goto(
            frame_corner.0 + cursor_offset.0 + X_PADDING as u16 + 1,
            frame_corner.1 + cursor_offset.1 + Y_PADDING as u16 + 1
        )
    );
    stdout.flush().unwrap();
}

fn main() {
    let mut args = std::env::args();

    let mut poem = match std::env::args().count() {
        1 => Poem::from_str(""),
        2 => {
            let path = args.nth(1).unwrap();
            let mut file = std::fs::File::open(path.clone()).expect("ERROR: Could not open file.");
            let mut content = String::new();
            file.read_to_string(&mut content)
                .expect("ERROR: Could not read file.");
            Poem::from_str(&content).with_name(path)
        }
        _ => {
            eprintln!("USAGE: poed <path/to/file>");
            std::process::exit(-1);
        }
    };

    let mut stdout = stdout().into_raw_mode().unwrap();

    let stdin = termion::async_stdin();
    let mut it = stdin.keys();

    draw_screen(&mut stdout, &poem);

    'run: loop {
        let maybe_ev = it.next();
        match maybe_ev {
            None => std::thread::sleep(std::time::Duration::from_millis(50)),
            Some(ev) => match ev {
                Ok(key) => {
                    match key {
                        Key::Esc => break 'run,
                        Key::Ctrl('s') => {
                            let result = {
                                if let Some(path) = poem.name.as_ref() {
                                    std::fs::write(path.clone(), poem.buffer.clone())
                                } else {
                                    poem.name =
                                        Some(poem.buffer.lines().next().unwrap().to_string());
                                    poem.buffer =
                                        poem.buffer.split_once('\n').unwrap().1.to_string();
                                    std::fs::write(poem.name.as_ref().unwrap(), poem.buffer.clone())
                                }
                            };

                            if result.is_ok() {
                                println!("saved.")
                            }
                        }
                        Key::Char(c) => poem.modify(EditOperation::Insert(c)),
                        Key::Backspace => poem.modify(EditOperation::DeleteLeft),
                        Key::Delete => poem.modify(EditOperation::DeleteRight),
                        Key::BackTab => poem.modify(EditOperation::Newline),
                        Key::Left => {
                            if poem.cursor == 0 {
                            } else {
                                poem.cursor -= 1
                            }
                        }
                        Key::Right => {
                            if poem.cursor == poem.buffer.len() {
                            } else {
                                poem.cursor += 1
                            }
                        }
                        Key::Up => {
                            let current_pos = poem.get_cursor_offset();
                            poem.target_line_pos = current_pos.0 as usize;

                            if current_pos.1 == 0 {
                                poem.cursor = 0; // atp equivalent to poem.cursor_start_line();
                            } else {
                                let prev_line_len = poem
                                    .buffer
                                    .lines()
                                    .nth((current_pos.1 - 1).into())
                                    .unwrap()
                                    .len();

                                if prev_line_len >= poem.target_line_pos {
                                    poem.cursor -= 1;
                                    while poem.get_cursor_offset().0 as usize
                                        != poem.target_line_pos
                                    {
                                        poem.cursor -= 1;
                                    }
                                } else {
                                    poem.cursor -= current_pos.0 as usize + 1;
                                }
                            }
                        }
                        Key::Down => {
                            let current_pos = poem.get_cursor_offset();
                            poem.target_line_pos = current_pos.0 as usize;

                            if current_pos.1 == poem.buffer.lines().count() as u16 - 1 {
                                poem.cursor_end_line();
                            } else {
                                let next_line_len = poem
                                    .buffer
                                    .lines()
                                    .nth((current_pos.1 + 1).into())
                                    .unwrap()
                                    .len();

                                if next_line_len >= poem.target_line_pos {
                                    poem.cursor += 1;
                                    while poem.get_cursor_offset().0 as usize
                                        != poem.target_line_pos
                                    {
                                        poem.cursor += 1;
                                    }
                                } else {
                                    // move until we get to the next line...
                                    while current_pos.1 == poem.get_cursor_offset().1 {
                                        poem.cursor += 1;
                                    }
                                    // and then move to the end of that line.
                                    poem.cursor_end_line();
                                }
                            }
                        }
                        Key::End => poem.cursor_end_line(),
                        Key::Home => poem.cursor_start_line(),
                        _ => {}
                    }
                    draw_screen(&mut stdout, &poem);
                }
                _ => {}
            },
        }
    }

    write!(
        stdout,
        "{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    );
}
