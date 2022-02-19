use std::fs::File;
use std::io;
use std::time;
use std::thread;
use std::io::prelude::*;
use std::path::Path;
use termion::{clear, cursor, color, style};
use termion::raw::IntoRawMode;
use termion::input::TermRead;
use termion::event::Key;

fn string_index(s: String, i: usize) -> char {
    let b: u8 = s.as_bytes()[i];
    return b as char;
}

enum LetterType {
    INCORRECT,
    WRONG_POSITION,
    CORRECT
}

struct Letter {
    character: String,
    letterType: LetterType,
}

fn main() {
    // Create a path to the desired file
    let path = Path::new("words.txt");
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display, why),
        Ok(_) => {
            let mut words: Vec<&str> = s.split("\n").collect();

            words.retain(|&word| word.chars().count() == 5);

            let mut index: i32 = 0;

            let mut stdout = io::stdout().into_raw_mode().unwrap();
            let mut stdin = termion::async_stdin().keys();

            cursor::Goto(1, 1);
            write!(stdout, "{}{}", termion::clear::All, cursor::Goto(1, 1)).unwrap();

            stdout.flush().unwrap();

            let mut word: Vec<&char> = Vec::from(['', '', '', '', '']);

            loop {
                let b = stdin.next();
                use termion::event::Key::*;

                if let Some(Ok(key)) = b {
                    match key {
                        Key::Left => {
                            index -= 1;
                            while index < 1 {
                                index += 5;
                            }
                            write!(stdout, "{}", cursor::Goto(index as u16, 1)).unwrap();
                            stdout.lock().flush().unwrap();
                        },
                        Key::Right => {
                            index += 1;
                            index %= 6;
                            if index < 1 {
                                index = 1;
                            }
                            write!(stdout, "{}", cursor::Goto(index as u16, 1)).unwrap();
                            stdout.lock().flush().unwrap();
                        },
                        Key::Ctrl('c') => {
                            break;
                        },
                        Char(c) => {
                            write!(stdout, "{}", c).unwrap();
                            index += 1;
                            index %= 6;
                            if index < 1 {
                                index = 1;
                            }
                            word[(index as usize) - 1] = &c;
                            write!(stdout, "{}", cursor::Goto(index as u16, 1)).unwrap();
                            stdout.lock().flush().unwrap();
                        },
                        _ => {},
                    }
                }

                thread::sleep(time::Duration::from_millis(50));
            }
        }
    }
}
