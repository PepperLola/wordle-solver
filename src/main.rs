use std::{
    cmp,
    collections::HashMap,
    env,
    fs::File,
    io::{self, prelude::*, Write},
    iter::Iterator,
    path::Path,
    thread, time,
};

pub use crossterm::{
    cursor,
    event::{self, read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue, style,
    terminal::{self, ClearType},
    Command, Result,
};

fn string_index(s: String, i: usize) -> char {
    let b: u8 = s.as_bytes()[i];
    b as char
}

fn next_type(t: LetterType) -> LetterType {
    match t {
        LetterType::Incorrect => LetterType::WrongPosition,
        LetterType::WrongPosition => LetterType::Correct,
        LetterType::Correct => LetterType::Incorrect,
    }
}

fn previous_type(t: LetterType) -> LetterType {
    match t {
        LetterType::Incorrect => LetterType::Correct,
        LetterType::WrongPosition => LetterType::Incorrect,
        LetterType::Correct => LetterType::WrongPosition,
    }
}

fn type_color_string(t: LetterType) -> String {
    match t {
        LetterType::Incorrect => String::from("\x1b[100m"),
        LetterType::WrongPosition => String::from("\x1b[43m"),
        LetterType::Correct => String::from("\x1b[42;1m"),
    }
}

#[derive(Copy, Clone)]
enum LetterType {
    Incorrect,
    WrongPosition,
    Correct,
}

#[derive(Copy, Clone)]
struct Letter {
    character: char,
    letter_type: LetterType,
}

fn get_words() -> Vec<String> {
    let home: String = if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
        env::var("HOME").unwrap()
    } else {
        env::var("APPDATA").unwrap()
    };
    let raw_path = home + "/.wordle_solver/words.txt";
    let path = Path::new(&raw_path);

    let mut file = match File::open(path) {
        Err(_) => {
            std::process::Command::new("curl")
                .arg("https://gist.githubusercontent.com/PepperLola/bbc3512ba937a56572219536b60da4fd/raw/1295fc03cabfcc7d2bd290dc75fc28a8569dc3fe/words.txt")
                .arg("-o")
                .arg(raw_path)
                .output()
                .expect("failed to execute process");
            return get_words();
        }
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();
    s.split("\n")
        .map(|s| s.to_string().replace("\r", ""))
        .collect()
}

fn count_letters(word: &[Letter], known_counts: &mut HashMap<char, u16>) {
    let mut temp_counts: HashMap<char, u16> = HashMap::new();
    // count letters
    for letter in word.iter().take(5) {
        let mut count = 0;
        match letter.letter_type {
            LetterType::Incorrect => {}
            _ => match temp_counts.get(&letter.character) {
                None => {
                    count = 1;
                }
                Some(stored_count) => {
                    count = stored_count + 1;
                }
            },
        }
        if count > 0 {
            temp_counts.insert(letter.character, count);
        }
    }

    for (key, value) in &temp_counts {
        let mut new_value: u16;
        match known_counts.get(key) {
            None => {
                new_value = *value;
            }
            Some(val) => {
                new_value = *val;
                if value > val {
                    new_value = *val;
                }
            }
        }

        known_counts.insert(*key, new_value);
    }
}

fn handle_correct_wrong_position(
    word: &[Letter],
    words: &mut Vec<String>,
    known_present: &mut HashMap<char, u16>,
    known_correct: &mut HashMap<char, u16>,
) {
    for i in 0..5 {
        let letter: Letter = word[i];
        match letter.letter_type {
            LetterType::Incorrect => {}
            LetterType::WrongPosition => {
                let saved: u16 = match known_present.get(&letter.character) {
                    None => 0b00000,
                    Some(data) => *data,
                };
                known_present.insert(letter.character, saved | 1 << i);
                words.retain(|word| {
                    string_index(word.to_string(), i) != letter.character
                        && word.contains(letter.character)
                });
            }
            LetterType::Correct => {
                words.retain(|word| string_index(word.to_string(), i) == letter.character);
                let mut data: u16 = 1 << i;
                match known_correct.get(&letter.character) {
                    None => {}
                    Some(stored) => {
                        data |= stored;
                    }
                }
                known_correct.insert(letter.character, data);
            }
        }
    }
}

fn handle_incorrect(
    word: &[Letter],
    words: &mut Vec<String>,
    known_correct: &HashMap<char, u16>,
    known_present: &HashMap<char, u16>,
    known_counts: &HashMap<char, u16>,
) {
    for i in 0..5 {
        let letter: Letter = word[i];
        match letter.letter_type {
            LetterType::Incorrect => match known_present.get(&letter.character) {
                None => match known_correct.get(&letter.character) {
                    None => {
                        words.retain(|word| !word.contains(letter.character));
                    }
                    Some(data) => {
                        words.retain(|word| !word.contains(letter.character) || *data > 0);
                    }
                },
                Some(data) => {
                    words.retain(|word| {
                        string_index(word.to_string(), i) != letter.character
                            && (data & (1 << i)) == 0
                    });
                }
            },
            _ => {
                let mut count = 1;
                match known_counts.get(&letter.character) {
                    None => {}
                    Some(stored_count) => {
                        count = *stored_count;
                    }
                }

                words.retain(|word| {
                    word.chars()
                        .filter(|c| *c == letter.character)
                        .collect::<Vec<_>>()
                        .len()
                        >= count as usize
                });
            }
        }
    }
}

fn parse(
    word: &mut [Letter],
    words: &mut Vec<String>,
    known_correct: &mut HashMap<char, u16>,
    known_present: &mut HashMap<char, u16>,
    known_counts: &mut HashMap<char, u16>,
) {
    count_letters(word, known_counts);
    handle_correct_wrong_position(word, words, known_present, known_correct);
    handle_incorrect(word, words, known_correct, known_present, known_counts);
}

fn main() -> Result<()> {
    let mut words: Vec<String> = get_words();

    words.retain(|word| word.chars().count() == 5);

    let mut known_correct: HashMap<char, u16> = HashMap::new();
    let mut known_present: HashMap<char, u16> = HashMap::new(); // uses bitwise operations; 0x11111 would be for all positions (IMPOSSIBLE!!!1!)
    let mut known_counts: HashMap<char, u16> = HashMap::new(); // known counts for each letter; only include words that contain >= count of letter

    let mut index: usize = 1;
    let mut line: u16 = 1;

    terminal::enable_raw_mode()?;

    let mut stdout = io::stdout();

    queue!(
        stdout,
        style::ResetColor,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(1, 1)
    )?;

    write!(stdout, "╭─┬─┬─┬─┬─╮")?;

    queue!(stdout, cursor::MoveTo(1, 2))?;

    write!(stdout, "│ │ │ │ │ │")?;

    queue!(stdout, cursor::MoveTo(1, 3))?;

    write!(stdout, "╰─┴─┴─┴─┴─╯")?;

    queue!(stdout, cursor::MoveTo(2, 2))?;

    stdout.flush()?;

    let mut word: Vec<Letter> = Vec::new();
    for _ in 0..5 {
        word.push(Letter {
            character: 'a',
            letter_type: LetterType::Incorrect,
        });
    }

    loop {
        let event = read()?;

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                ..
            }) => {
                index -= 1;
                while index < 1 {
                    index += 5;
                }
                queue!(stdout, cursor::MoveTo(index as u16 * 2, line + 1))?;
                stdout.lock().flush()?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                ..
            }) => {
                index += 1;
                index %= 6;
                if index < 1 {
                    index = 1;
                }
                queue!(stdout, cursor::MoveTo(index as u16 * 2, line + 1))?;
                stdout.lock().flush()?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up, ..
            }) => {
                word[index - 1].letter_type = previous_type(word[index - 1].letter_type);
                queue!(stdout, cursor::MoveTo(index as u16 * 2, line + 1),)?;
                write!(
                    stdout,
                    "{}{}",
                    type_color_string(word[index - 1].letter_type),
                    word[index - 1].character
                )?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                ..
            }) => {
                word[index - 1].letter_type = next_type(word[index - 1].letter_type);
                queue!(stdout, cursor::MoveTo(index as u16 * 2, line + 1))?;
                write!(
                    stdout,
                    "{}{}",
                    type_color_string(word[index - 1].letter_type),
                    word[index - 1].character
                )?;
            }
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('c'),
            }) => {
                terminal::disable_raw_mode()?;
                break;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => {
                line += 1;
                parse(
                    &mut word,
                    &mut words,
                    &mut known_correct,
                    &mut known_present,
                    &mut known_counts,
                );
                let length = cmp::min(10, words.len());
                for (i, item) in words.iter().enumerate().take(length) {
                    queue!(stdout, cursor::MoveTo(20, i as u16 + 1))?;
                    write!(stdout, "        ")?;
                    queue!(stdout, cursor::MoveTo(20, i as u16 + 1),)?;
                    write!(stdout, "{}.{}", (i + 1), item)?;
                }
                for i in 0..(10 - length) {
                    queue!(stdout, cursor::MoveTo(20, (i + length + 1) as u16))?;
                    write!(stdout, "        ")?;
                }
                queue!(stdout, cursor::MoveTo(1, line + 1))?;
                write!(stdout, "│ │ │ │ │ │")?;
                queue!(stdout, cursor::MoveTo(1, line + 2))?;
                write!(stdout, "╰─┴─┴─┴─┴─╯")?;
                queue!(stdout, cursor::MoveTo(index as u16 * 2, line + 1))?;
                stdout.lock().flush()?;
                for item in word.iter_mut().take(5) {
                    item.letter_type = LetterType::Incorrect;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                ..
            }) => {
                word[index - 1].character = c;
                match known_correct.get(&c) {
                    Some(data) => {
                        if data & (1 << (index - 1)) > 0 {
                            word[index - 1].letter_type = LetterType::Correct;
                        } else {
                            word[index - 1].letter_type = LetterType::Incorrect;
                        }
                    }
                    _ => {
                        word[index - 1].letter_type = LetterType::Incorrect;
                    }
                }

                queue!(stdout, cursor::MoveTo(index as u16 * 2, line + 1))?;
                write!(
                    stdout,
                    "{}{}",
                    type_color_string(word[index - 1].letter_type),
                    c
                )?;
                index += 1;
                index %= 6;
                if index < 1 {
                    index = 1;
                }
                queue!(stdout, cursor::MoveTo(index as u16 * 2, line + 1))?;
                stdout.lock().flush()?;
            }
            _ => {}
        }
        write!(stdout, "\x1b[0m")?;

        thread::sleep(time::Duration::from_millis(50));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrambled() {
        let mut words = get_words();
        let mut word: Vec<Letter> = vec![
            Letter {
                character: 's',
                letter_type: LetterType::WrongPosition,
            },
            Letter {
                character: 'w',
                letter_type: LetterType::WrongPosition,
            },
            Letter {
                character: 'k',
                letter_type: LetterType::WrongPosition,
            },
            Letter {
                character: 'o',
                letter_type: LetterType::WrongPosition,
            },
            Letter {
                character: 'r',
                letter_type: LetterType::WrongPosition,
            },
        ];
        parse(
            &mut word,
            &mut words,
            &mut HashMap::new(),
            &mut HashMap::new(),
            &mut HashMap::new(),
        );
        assert_eq!(words.len(), 1);
        assert_eq!(words.get(0), Some(&String::from("works")));
    }

    #[test]
    fn test_double_letter_with_triple_guess() {
        let mut words = get_words();
        let mut word: Vec<Letter> = vec![
            Letter {
                character: 'c',
                letter_type: LetterType::Correct,
            },
            Letter {
                character: 'c',
                letter_type: LetterType::WrongPosition,
            },
            Letter {
                character: 'n',
                letter_type: LetterType::Correct,
            },
            Letter {
                character: 'y',
                letter_type: LetterType::WrongPosition,
            },
            Letter {
                character: 'm',
                letter_type: LetterType::Incorrect,
            },
        ];
        parse(
            &mut word,
            &mut words,
            &mut HashMap::new(),
            &mut HashMap::new(),
            &mut HashMap::new(),
        );
        assert_eq!(words.len(), 1);
        assert_eq!(words.get(0), Some(&String::from("cynic")));
    }

    #[test]
    fn test_incorrect_then_correct_letter() {
        let mut words = get_words();
        let mut word: Vec<Letter> = vec![
            Letter {
                character: 'e',
                letter_type: LetterType::Incorrect,
            },
            Letter {
                character: 'e',
                letter_type: LetterType::Correct,
            },
            Letter {
                character: 'm',
                letter_type: LetterType::Correct,
            },
            Letter {
                character: 'r',
                letter_type: LetterType::WrongPosition,
            },
            Letter {
                character: 'l',
                letter_type: LetterType::WrongPosition,
            },
        ];
        parse(
            &mut word,
            &mut words,
            &mut HashMap::new(),
            &mut HashMap::new(),
            &mut HashMap::new(),
        );
        assert_eq!(words.len(), 1);
        assert_eq!(words.get(0), Some(&String::from("lemur")));
    }

    #[test]
    fn test_multiple_guesses() {
        let mut words = get_words();
        let mut word = vec![
            Letter {
                character: 't',
                letter_type: LetterType::Incorrect,
            },
            Letter {
                character: 'r',
                letter_type: LetterType::WrongPosition,
            },
            Letter {
                character: 'a',
                letter_type: LetterType::Incorrect,
            },
            Letter {
                character: 'c',
                letter_type: LetterType::Incorrect,
            },
            Letter {
                character: 'e',
                letter_type: LetterType::Incorrect,
            },
        ];
        let mut known_correct: HashMap<char, u16> = HashMap::new();
        let mut known_present: HashMap<char, u16> = HashMap::new();
        let mut known_counts: HashMap<char, u16> = HashMap::new();
        parse(
            &mut word,
            &mut words,
            &mut known_correct,
            &mut known_present,
            &mut known_counts,
        );
        word = vec![
            Letter {
                character: 'r',
                letter_type: LetterType::WrongPosition,
            },
            Letter {
                character: 'o',
                letter_type: LetterType::Correct,
            },
            Letter {
                character: 'u',
                letter_type: LetterType::Correct,
            },
            Letter {
                character: 'n',
                letter_type: LetterType::WrongPosition,
            },
            Letter {
                character: 'd',
                letter_type: LetterType::Incorrect,
            },
        ];
        parse(
            &mut word,
            &mut words,
            &mut known_correct,
            &mut known_present,
            &mut known_counts,
        );
        word = vec![
            Letter {
                character: 'a',
                letter_type: LetterType::Incorrect,
            },
            Letter {
                character: 'm',
                letter_type: LetterType::WrongPosition,
            },
            Letter {
                character: 'a',
                letter_type: LetterType::Incorrect,
            },
            Letter {
                character: 'a',
                letter_type: LetterType::Incorrect,
            },
            Letter {
                character: 'a',
                letter_type: LetterType::Incorrect,
            },
        ];
        parse(
            &mut word,
            &mut words,
            &mut known_correct,
            &mut known_present,
            &mut known_counts,
        );
        assert_eq!(words.len(), 1);
        assert_eq!(words.get(0), Some(&String::from("mourn")));
    }
}
