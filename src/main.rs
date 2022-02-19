use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

fn string_index(s: String, i: usize) -> char {
    let b: u8 = s.as_bytes()[i];
    return b as char;
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

            words.retain(|&word| word.chars().count() > 0);

            for i in 0..5 {
                let mut alphabet: Vec<&str> = "abcdefghijklmnopqrstuvwxyz".split("").collect();
                let mut pattern = String::new();
                println!(
                    "\x1b[97;1mAll possible letters in position \x1b[33m{}\x1b[97m:\x1b[22m",
                    i + 1
                );
                io::stdin()
                    .read_line(&mut pattern)
                    .expect("Couldn't read pattern input.");
                pattern = pattern.trim().to_string();
                if string_index(pattern.clone(), 0) == '^' {
                    alphabet.retain(|&letter| !pattern.contains(letter));
                    pattern = alphabet.join("");
                }
                // only keep elements whose letter at index i is in the pattern for index i
                words.retain(|&word| pattern.contains(string_index(word.to_string(), i)))
            }

            if words.len() == 0 {
                print!("\x1b[91mNo solutions...\x1b[0m")
            } else {
                println!("");

                for i in 0..words.len() {
                    println!("\x1b[97m\x1b[1m{}. \x1b[32m{}", i + 1, words[i])
                }
            }
        }
    }
}
