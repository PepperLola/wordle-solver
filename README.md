# Wordle Solver
*A TUI program that solves a Wordle problem.*
---

[![Rust](https://github.com/PepperLola/wordle-solver/actions/workflows/rust.yml/badge.svg)](https://github.com/PepperLola/wordle-solver/actions/workflows/rust.yml)

### Installation

To install the Wordle solver, just run
```sh
curl https://raw.githubusercontent.com/PepperLola/wordle-solver/main/install.sh | sh
```

### Usage

Once the executable is installed and added to the $PATH, run it in the same directory as the words.txt file. Until the program is able to download the words list, this will mean cloning the repository and cd-ing in to the folder.
Guess a word in Wordle and enter the letters in the solver. Use the left and right arrows to move between letters, and the up and down arrows to change the letter type between correct, present, and absent.
Once the letters and types match what is in Wordle, press <kbd>Enter</kbd>. The program will print out 10 words that match all of the previously entered requirements.  
***Note:** These words are ordered alphabetically, not by the quality of the guess. Some of them will also not be recognized by Wordle as actual words. It's up to the user to choose which of the words to guess.*

### Planned Features
* [ ] Program downloads word list and puts it in a universal path so the user can use the program from any directory
* [ ] Web UI?

---

License: MIT
