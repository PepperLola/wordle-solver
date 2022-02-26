use std::fs::File;
use std::io;
use std::cmp;
use std::time;
use std::thread;
use std::io::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::iter::Iterator;
use std::iter::FromIterator;
use termion::{clear, cursor, color, style};
use termion::raw::IntoRawMode;
use termion::input::TermRead;
use termion::event::Key;

fn string_index(s: String, i: usize) -> char {
    let b: u8 = s.as_bytes()[i];
    return b as char;
}

fn next_type(t: LetterType) -> LetterType {
    match t {
        LetterType::INCORRECT => return LetterType::WRONG_POSITION,
        LetterType::WRONG_POSITION => return LetterType::CORRECT,
        LetterType::CORRECT => return LetterType::INCORRECT,
    }
}

fn previous_type(t: LetterType) -> LetterType {
    match t {
        LetterType::INCORRECT => return LetterType::CORRECT,
        LetterType::WRONG_POSITION => return LetterType::INCORRECT,
        LetterType::CORRECT => return LetterType::WRONG_POSITION,
    }
}

fn type_color_string(t: LetterType) -> String {
    match t {
        LetterType::INCORRECT => return String::from("\x1b[100m"),
        LetterType::WRONG_POSITION => return String::from("\x1b[43m"),
        LetterType::CORRECT => return String::from("\x1b[42;1m"),
    }
}

#[derive(Copy, Clone)]
enum LetterType {
    INCORRECT,
    WRONG_POSITION,
    CORRECT,
}

#[derive(Copy, Clone)]
struct Letter {
    character: char,
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

            let mut known_correct: HashMap<u16, char> = HashMap::new();
            let mut known_present: HashMap<char, u16> = HashMap::new(); // uses bitwise operations; 0x11111 would be for all positions (IMPOSSIBLE!!!1!)

            let mut index: usize = 1;
            let mut line: u16 = 1;

            let mut stdout = io::stdout().into_raw_mode().unwrap();
            let mut stdin = termion::async_stdin().keys();

            cursor::Goto(1, 1);
            write!(stdout, "{}{}", termion::clear::All, cursor::Goto(1, 1)).unwrap();

            stdout.flush().unwrap();

            let mut word: Vec<Letter> = Vec::new();
            for _ in 0..5 {
                word.push(Letter { character: 'a', letterType: LetterType::INCORRECT });
            }

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
                            write!(stdout, "{}", cursor::Goto(index as u16, line)).unwrap();
                            stdout.lock().flush().unwrap();
                        },
                        Key::Right => {
                            index += 1;
                            index %= 6;
                            if index < 1 {
                                index = 1;
                            }
                            write!(stdout, "{}", cursor::Goto(index as u16, line)).unwrap();
                            stdout.lock().flush().unwrap();
                        },
                        Key::Up => {
                            word[index - 1].letterType = previous_type(word[index - 1].letterType);
                            write!(stdout, "{}{}{}", cursor::Goto(index as u16, line), type_color_string(word[index - 1].letterType), word[index - 1].character).unwrap();
                        },
                        Key::Down => {
                            word[index - 1].letterType = next_type(word[index - 1].letterType);
                            write!(stdout, "{}{}{}", cursor::Goto(index as u16, line), type_color_string(word[index - 1].letterType), word[index - 1].character).unwrap();
                        },
                        Key::Ctrl('c') => {
                            break;
                        },
                        Char('\n') => {
                            line += 1;
                            for i in 0..5 {
                                let letter: Letter = word[i];
                                match letter.letterType {
                                    LetterType::INCORRECT => {
                                    },
                                    LetterType::WRONG_POSITION => {
                                        let saved: u16;
                                        match known_present.get(&letter.character) {
                                            None => {
                                                saved = 0b00000;
                                            },
                                            Some(data) => {
                                                saved = *data;
                                            }
                                        }
                                        known_present.insert(letter.character, saved | 1 << i);
                                        words.retain(|&word| string_index(word.to_string(), i) != letter.character && word.contains(letter.character));
                                    },
                                    LetterType::CORRECT => {
                                        words.retain(|&word| string_index(word.to_string(), i) == letter.character);
                                        known_correct.insert(i as u16, letter.character);
                                    },
                                }
                            }
                            for i in 0..5 {
                                let letter: Letter = word[i];
                                match letter.letterType {
                                    LetterType::INCORRECT => {
                                        words.retain(|&word| {
                                            match known_present.get(&letter.character) {
                                                None => {
                                                    return !word.contains(letter.character);
                                                },
                                                Some(data) => {
                                                    return !word.contains(letter.character) && (data & 1 << i) <= 0;
                                                }
                                            }
                                        });
                                    },
                                    LetterType::WRONG_POSITION => {
                                    },
                                    LetterType::CORRECT => {
                                    },
                                }
                            }
                            let length = cmp::min(10, words.len());
                            for i in 0..length {
                                write!(stdout, "{}        ", cursor::Goto(20, (i + 1) as u16)).unwrap();
                                write!(stdout, "{}{}.{}", cursor::Goto(20, (i + 1) as u16), (i + 1).to_string(), words[i]).unwrap();
                            }
                            for i in 0..(10 - length) {
                                write!(stdout, "{}        ", cursor::Goto(20, (i + length + 1) as u16)).unwrap();
                            }
                            write!(stdout, "{}", cursor::Goto(index as u16, line)).unwrap();
                            stdout.lock().flush().unwrap();
                            for i in 0..5 {
                                word[i].letterType = LetterType::INCORRECT;
                            }
                        },
                        Char(c) => {
                            word[index - 1].character = c;
                            let idx = &((index - 1) as u16);
                            if known_correct.contains_key(idx) {
                                match known_correct.get(idx) {
                                    Some(other_c) => {
                                        if other_c == &c {
                                            word[index - 1].letterType = LetterType::CORRECT;
                                        }
                                    },
                                    _ => {},
                                }
                            } else {
                                word[index - 1].letterType = LetterType::INCORRECT;
                            }
                            write!(stdout, "{}", cursor::Goto(index as u16, line)).unwrap();
                            write!(stdout, "{}{}", type_color_string(word[index - 1].letterType), c).unwrap();
                            index += 1;
                            index %= 6;
                            if index < 1 {
                                index = 1;
                            }
                            write!(stdout, "{}", cursor::Goto(index as u16, line)).unwrap();
                            stdout.lock().flush().unwrap();
                        },
                        _ => {},
                    }
                    write!(stdout, "\x1b[0m").unwrap();
                }


                thread::sleep(time::Duration::from_millis(50));
            }
        }
    }
}
