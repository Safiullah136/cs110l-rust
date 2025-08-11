// Simple Hangman Program
// User gets five incorrect guesses
// Word chosen randomly from words.txt
// Inspiration from: https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html
// This assignment will introduce you to some fundamental syntax in Rust:
// - variable declaration
// - string manipulation
// - conditional statements
// - loops
// - vectors
// - files
// - user input
// We've tried to limit/hide Rust's quirks since we'll discuss those details
// more in depth in the coming lectures.
extern crate rand;
use rand::Rng;
use std::fs;
use std::io;
use std::io::Write;

const NUM_INCORRECT_GUESSES: u32 = 5;
const WORDS_PATH: &str = "../words.txt";

fn pick_a_random_word() -> String {
    let file_string = fs::read_to_string(WORDS_PATH).expect("Unable to read file.");
    let words: Vec<&str> = file_string.split('\n').collect();
    String::from(words[rand::thread_rng().gen_range(0, words.len())].trim())
}

fn main() {
    let secret_word = pick_a_random_word();
    // Note: given what you know about Rust so far, it's easier to pull characters out of a
    // vector than it is to pull them out of a string. You can get the ith character of
    // secret_word by doing secret_word_chars[i].

    let secret_word_chars: Vec<char> = secret_word.chars().collect();
    // Uncomment for debugging:
    // println!("random word: {}", secret_word);

    // Your code here! :)

    let mut guesses = NUM_INCORRECT_GUESSES;
    let mut word_so_far : String;
    let mut guessed : Vec<char> = Vec::new(); 
    let mut guessed_chars : String = String::from("");

    println!("Welcome to CS110L Hangman!");
    loop {
        word_so_far = String::from("");
        let mut remaining = secret_word_chars.len();
    
        for i in secret_word_chars.iter() {
            if guessed.contains(i) {
                word_so_far.push(i.clone());
                remaining -= 1;
            } else {
                word_so_far.push('-');
            }
        }

        if remaining == 0 {
            println!("Congratulations you guessed the secret word: {}!", word_so_far);
            break;
        }

        if guesses == 0 {
            println!("Sorry, you ran out of guesses!");
            break;
        }

        println!("The word so far is {}", word_so_far);

        println!("You have guessed the following letters: {}", guessed_chars);
        println!("You have {} guesses left", guesses);
        print!("Please guess a letter: ");
        io::stdout().flush().expect("Error flushing stdout.");
        let mut input_guess = String::new();
        let mut guess: char = char::default();
        io::stdin().read_line(&mut input_guess).expect("Error reading line.");

        if let Some(first_char) = input_guess.chars().next() {
            guess = first_char;
        }

        if secret_word_chars.contains(&guess) && !guessed.contains(&guess) {
            guessed.push(guess.clone());
            guessed_chars.push(guess.clone());
        } else {
            println!("\nSorry, that letter is not in the word");
            guesses -= 1;
        }
        println!("\n");
    }
}
