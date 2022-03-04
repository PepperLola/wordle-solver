use std::fs::File;
use std::cmp;
use std::time;
use std::thread;
use std::io::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::iter::Iterator;

use std::io::{self, Write};

pub use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, read, KeyModifiers},
    execute, queue, style,
    terminal::{self, ClearType},
    Command, Result,
};

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
    letter_type: LetterType,
}

fn get_words() -> Vec<String> {
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
    file.read_to_string(&mut s).unwrap();
    return s.split("\n").map(|s| s.to_string()).collect();
}

fn count_letters(word: &Vec<Letter>, known_counts: &mut HashMap<char, u16>) {
    let mut temp_counts: HashMap<char, u16> = HashMap::new();
    // count letters
    for i in 0..5 {
        let letter: Letter = word[i];
        let mut count = 0;
        match letter.letter_type {
            LetterType::INCORRECT => {},
            _ => {
                match temp_counts.get(&letter.character) {
                    None => {
                        count = 1;
                    },
                    Some(stored_count) => {
                        count = stored_count + 1;
                    }
                }
            }
        }
        if count > 0 {
            temp_counts.insert(letter.character, count);
        }
    }

    for (key, value) in &temp_counts {
        let mut new_value: u16;
        match known_counts.get(&key) {
            None => {
                new_value = *value;
            },
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

fn handle_correct_wrong_position(word: &Vec<Letter>, words: &mut Vec<String>, known_present: &mut HashMap<char, u16>, known_correct: &mut HashMap<char, u16>) {
    for i in 0..5 {
        let letter: Letter = word[i];
        match letter.letter_type {
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
                words.retain(|word| {
                    return string_index(word.to_string(), i) != letter.character && word.contains(letter.character);
                });
            },
            LetterType::CORRECT => {
                words.retain(|word| {
                    return string_index(word.to_string(), i) == letter.character
                });
                let mut data: u16 = 1 << i;
                match known_correct.get(&letter.character) {
                    None => {},
                    Some(stored) => {
                        data = stored | data;
                    }
                }
                known_correct.insert(letter.character, data);
            },
        }
    }
}

fn handle_incorrect(word: &Vec<Letter>, words: &mut Vec<String>, known_correct: &HashMap<char, u16>, known_present: &HashMap<char, u16>, known_counts: &HashMap<char, u16>) {
    for i in 0..5 {
        let letter: Letter = word[i];
        match letter.letter_type {
            LetterType::INCORRECT => {
                match known_present.get(&letter.character) {
                    None => {
                        match known_correct.get(&letter.character) {
                            None => {
                                words.retain(|word| !word.contains(letter.character));
                            },
                            Some(data) => {
                                words.retain(|word| !word.contains(letter.character) || *data > 0);
                            }
                        }
                    },
                    Some(data) => {
                        words.retain(|word| string_index(word.to_string(), i) != letter.character && (data & (1 << i)) <= 0);
                    }
                }
            },
            _ => {
                let mut count = 1;
                match known_counts.get(&letter.character) {
                    None => {},
                    Some(stored_count) => {
                        count = *stored_count;
                    }
                }

                words.retain(|word| {
                    return word.chars().filter(|c| *c == letter.character).collect::<Vec<_>>().len() >= count as usize;
                });
            },
        }
    }
}

fn parse(word: &mut Vec<Letter>, words: &mut Vec<String>, known_correct: &mut HashMap<char, u16>, known_present: &mut HashMap<char, u16>, known_counts: &mut HashMap<char, u16>) {
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

    queue!(
        stdout,
        cursor::MoveTo(1, 2)
    )?;

    write!(stdout, "│ │ │ │ │ │")?;

    queue!(
        stdout,
        cursor::MoveTo(1, 3)
    )?;

    write!(stdout, "╰─┴─┴─┴─┴─╯")?;

    queue!(
        stdout,
        cursor::MoveTo(2,2)
    )?;

    stdout.flush()?;

    let mut word: Vec<Letter> = Vec::new();
    for _ in 0..5 {
        word.push(Letter { character: 'a', letter_type: LetterType::INCORRECT });
    }

    loop {
        let event = read()?;

        match event {
            Event::Key(KeyEvent { code: KeyCode::Left, .. }) => {
                index -= 1;
                while index < 1 {
                    index += 5;
                }
                queue!(
                    stdout,
                    cursor::MoveTo(index as u16 * 2, line + 1)
                )?;
                stdout.lock().flush()?;
            },
            Event::Key(KeyEvent { code: KeyCode::Right, .. }) => {
                index += 1;
                index %= 6;
                if index < 1 {
                    index = 1;
                }
                queue!(
                    stdout,
                    cursor::MoveTo(index as u16 * 2, line + 1)
                )?;
                stdout.lock().flush()?;
            },
            Event::Key(KeyEvent { code: KeyCode::Up, .. }) => {
                word[index - 1].letter_type = previous_type(word[index - 1].letter_type);
                queue!(
                    stdout,
                    cursor::MoveTo(index as u16 * 2, line + 1),
                )?;
                write!(stdout, "{}{}", type_color_string(word[index - 1].letter_type), word[index - 1].character)?;
            },
            Event::Key(KeyEvent { code: KeyCode::Down, .. }) => {
                word[index - 1].letter_type = next_type(word[index - 1].letter_type);
                queue!(
                    stdout,
                    cursor::MoveTo(index as u16 * 2, line + 1)
                )?;
                write!(stdout, "{}{}", type_color_string(word[index - 1].letter_type), word[index - 1].character)?;
            },
            Event::Key(KeyEvent {modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('c'),}) => {
                terminal::disable_raw_mode()?;
                break;
            },
            Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                line += 1;
                parse(&mut word, &mut words, &mut known_correct, &mut known_present, &mut known_counts);
                let length = cmp::min(10, words.len());
                for i in 0..length {
                    queue!(
                        stdout,
                        cursor::MoveTo(20, i as u16 + 1)
                    )?;
                    write!(stdout, "        ")?;
                    queue!(
                        stdout,
                        cursor::MoveTo(20, i as u16 + 1),
                    )?;
                    write!(stdout, "{}.{}", (i + 1).to_string(), words[i])?;
                }
                for i in 0..(10 - length) {
                    queue!(
                        stdout,
                        cursor::MoveTo(20, (i + length + 1) as u16)
                    )?;
                    write!(stdout, "        ")?;
                }
                queue!(
                    stdout,
                    cursor::MoveTo(1, line + 1)
                )?;
                write!(stdout, "│ │ │ │ │ │")?;
                queue!(
                    stdout,
                    cursor::MoveTo(1, line + 2)
                )?;
                write!(stdout, "╰─┴─┴─┴─┴─╯")?;
                queue!(
                    stdout,
                    cursor::MoveTo(index as u16 * 2, line + 1)
                )?;
                stdout.lock().flush()?;
                for i in 0..5 {
                    word[i].letter_type = LetterType::INCORRECT;
                }
            },
            Event::Key(KeyEvent { code: KeyCode::Char(c), .. }) => {
                word[index - 1].character = c;
                match known_correct.get(&c) {
                Some(data) => {
                        if data & (1 << (index - 1)) > 0 {
                            word[index - 1].letter_type = LetterType::CORRECT;
                        } else {
                            word[index - 1].letter_type = LetterType::INCORRECT;
                        }
                    },
                    _ => {
                        word[index - 1].letter_type = LetterType::INCORRECT;
                    },
                }

                queue!(
                    stdout,
                    cursor::MoveTo(index as u16 * 2, line + 1)
                )?;
                write!(stdout, "{}{}", type_color_string(word[index - 1].letter_type), c)?;
                index += 1;
                index %= 6;
                if index < 1 {
                    index = 1;
                }
                queue!(
                    stdout,
                    cursor::MoveTo(index as u16 * 2, line + 1)
                )?;
                stdout.lock().flush()?;
            },
            _ => {},
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
            Letter{character: 's', letter_type: LetterType::WRONG_POSITION},
            Letter{character: 'w', letter_type: LetterType::WRONG_POSITION},
            Letter{character: 'k', letter_type: LetterType::WRONG_POSITION},
            Letter{character: 'o', letter_type: LetterType::WRONG_POSITION},
            Letter{character: 'r', letter_type: LetterType::WRONG_POSITION},
        ];
        parse(&mut word, &mut words, &mut HashMap::new(), &mut HashMap::new(), &mut HashMap::new());
        assert_eq!(words.len(), 1);
        assert_eq!(words.get(0), Some(&String::from("works")));
    }

    #[test]
    fn test_double_letter_with_triple_guess() {
        let mut words = get_words();
        let mut word: Vec<Letter> = vec![
            Letter{character: 'c', letter_type: LetterType::CORRECT},
            Letter{character: 'c', letter_type: LetterType::WRONG_POSITION},
            Letter{character: 'n', letter_type: LetterType::CORRECT},
            Letter{character: 'y', letter_type: LetterType::WRONG_POSITION},
            Letter{character: 'm', letter_type: LetterType::INCORRECT},
        ];
        parse(&mut word, &mut words, &mut HashMap::new(), &mut HashMap::new(), &mut HashMap::new());
        assert_eq!(words.len(), 1);
        assert_eq!(words.get(0), Some(&String::from("cynic")));
    }

    #[test]
    fn test_incorrect_then_correct_letter() {
        let mut words = get_words();
        let mut word: Vec<Letter> = vec![
            Letter{character: 'e', letter_type: LetterType::INCORRECT},
            Letter{character: 'e', letter_type: LetterType::CORRECT},
            Letter{character: 'm', letter_type: LetterType::CORRECT},
            Letter{character: 'r', letter_type: LetterType::WRONG_POSITION},
            Letter{character: 'l', letter_type: LetterType::WRONG_POSITION}
        ];
        parse(&mut word, &mut words, &mut HashMap::new(), &mut HashMap::new(), &mut HashMap::new());
        assert_eq!(words.len(), 1);
        assert_eq!(words.get(0), Some(&String::from("lemur")));
    }

    #[test]
    fn test_multiple_guesses() {
        let mut words = get_words();
        let mut word = vec![
            Letter{character: 't', letter_type: LetterType::INCORRECT},
            Letter{character: 'r', letter_type: LetterType::WRONG_POSITION},
            Letter{character: 'a', letter_type: LetterType::INCORRECT},
            Letter{character: 'c', letter_type: LetterType::INCORRECT},
            Letter{character: 'e', letter_type: LetterType::INCORRECT}
        ];
        let mut known_correct: HashMap<char, u16> = HashMap::new();
        let mut known_present: HashMap<char, u16> = HashMap::new();
        let mut known_counts: HashMap<char, u16> = HashMap::new();
        parse(&mut word, &mut words, &mut known_correct, &mut known_present, &mut known_counts);
        word = vec![
            Letter{character: 'r', letter_type: LetterType::WRONG_POSITION},
            Letter{character: 'o', letter_type: LetterType::CORRECT},
            Letter{character: 'u', letter_type: LetterType::CORRECT},
            Letter{character: 'n', letter_type: LetterType::WRONG_POSITION},
            Letter{character: 'd', letter_type: LetterType::INCORRECT}
        ];
        parse(&mut word, &mut words, &mut known_correct, &mut known_present, &mut known_counts);
        word = vec![
            Letter{character: 'a', letter_type: LetterType::INCORRECT},
            Letter{character: 'm', letter_type: LetterType::WRONG_POSITION},
            Letter{character: 'a', letter_type: LetterType::INCORRECT},
            Letter{character: 'a', letter_type: LetterType::INCORRECT},
            Letter{character: 'a', letter_type: LetterType::INCORRECT}
        ];
        parse(&mut word, &mut words, &mut known_correct, &mut known_present, &mut known_counts);
        assert_eq!(words.len(), 1);
        assert_eq!(words.get(0), Some(&String::from("mourn")));
    }
}
