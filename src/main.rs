use std::fs::File;
use std::io;
use std::cmp;
use std::time;
use std::thread;
use std::io::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::iter::Iterator;
use termion::cursor;
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

fn handle_correct_wrong_position(word: &Vec<Letter>, words: &mut Vec<String>, known_present: &mut HashMap<char, u16>, known_correct: &mut HashMap<u16, char>) {
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
                known_correct.insert(i as u16, letter.character);
            },
        }
    }
}

fn handle_incorrect(word: &Vec<Letter>, words: &mut Vec<String>, known_present: &HashMap<char, u16>, known_counts: &HashMap<char, u16>) {
    for i in 0..5 {
        let letter: Letter = word[i];
        match letter.letter_type {
            LetterType::INCORRECT => {
                match known_present.get(&letter.character) {
                    None => {
                        words.retain(|word| !word.contains(letter.character));
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

                words.retain(|word| word.chars().filter(|c| *c == letter.character).collect::<Vec<_>>().len() >= count as usize);
            },
        }
    }
}

fn parse(word: &mut Vec<Letter>, words: &mut Vec<String>, known_correct: &mut HashMap<u16, char>, known_present: &mut HashMap<char, u16>, known_counts: &mut HashMap<char, u16>) {
    count_letters(word, known_counts);
    handle_correct_wrong_position(word, words, known_present, known_correct);
    handle_incorrect(word, words, known_present, known_counts);
}

fn main() {
    let mut words: Vec<String> = get_words();

    words.retain(|word| word.chars().count() == 5);

    let mut known_correct: HashMap<u16, char> = HashMap::new();
    let mut known_present: HashMap<char, u16> = HashMap::new(); // uses bitwise operations; 0x11111 would be for all positions (IMPOSSIBLE!!!1!)
    let mut known_counts: HashMap<char, u16> = HashMap::new(); // known counts for each letter; only include words that contain >= count of letter

    let mut index: usize = 1;
    let mut line: u16 = 1;

    let mut stdout = io::stdout().into_raw_mode().unwrap();
    let mut stdin = termion::async_stdin().keys();

    cursor::Goto(1, 1);
    write!(stdout, "{}{}", termion::clear::All, cursor::Goto(1, 1)).unwrap();

    stdout.flush().unwrap();

    let mut word: Vec<Letter> = Vec::new();
    for _ in 0..5 {
        word.push(Letter { character: 'a', letter_type: LetterType::INCORRECT });
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
                    word[index - 1].letter_type = previous_type(word[index - 1].letter_type);
                    write!(stdout, "{}{}{}", cursor::Goto(index as u16, line), type_color_string(word[index - 1].letter_type), word[index - 1].character).unwrap();
                },
                Key::Down => {
                    word[index - 1].letter_type = next_type(word[index - 1].letter_type);
                    write!(stdout, "{}{}{}", cursor::Goto(index as u16, line), type_color_string(word[index - 1].letter_type), word[index - 1].character).unwrap();
                },
                Key::Ctrl('c') => {
                    break;
                },
                Char('\n') => {
                    line += 1;
                    parse(&mut word, &mut words, &mut known_correct, &mut known_present, &mut known_counts);
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
                        word[i].letter_type = LetterType::INCORRECT;
                    }
                },
                Char(c) => {
                    word[index - 1].character = c;
                    let idx = &((index - 1) as u16);
                    if known_correct.contains_key(idx) {
                        match known_correct.get(idx) {
                            Some(other_c) => {
                                if other_c == &c {
                                    word[index - 1].letter_type = LetterType::CORRECT;
                                }
                            },
                            _ => {},
                        }
                    } else {
                        word[index - 1].letter_type = LetterType::INCORRECT;
                    }
                    write!(stdout, "{}", cursor::Goto(index as u16, line)).unwrap();
                    write!(stdout, "{}{}", type_color_string(word[index - 1].letter_type), c).unwrap();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_letter_with_triple_guess() {
        let mut words = get_words();
        let mut word: Vec<Letter> = Vec::from([
            Letter{character: 'c', letter_type: LetterType::CORRECT},
            Letter{character: 'c', letter_type: LetterType::WRONG_POSITION},
            Letter{character: 'n', letter_type: LetterType::CORRECT},
            Letter{character: 'y', letter_type: LetterType::WRONG_POSITION},
            Letter{character: 'm', letter_type: LetterType::INCORRECT},
        ]);
        parse(&mut word, &mut words, &mut HashMap::new(), &mut HashMap::new(), &mut HashMap::new());
        println!("{:?}", words);
        assert_eq!(words.len(), 1);
    }
}
