use std::env;
use std::process;
use std::fs::File;
use std::fs;
use std::io;
use std::io::BufRead;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Too few arguments.");
        process::exit(1);
    }
    let filename = &args[1];
    // Your code here :)
    let mut words_count = 0;
    let mut lines_count = 0;
    let mut chars_count = 0;

    let f = io::BufReader::new(File::open(filename).unwrap());

    for line in f.lines() {
        lines_count += 1;
        let lines_str = line.unwrap();
        chars_count += lines_str.len();
        words_count += lines_str.split_whitespace().count();
    }
    chars_count += lines_count - 1;

    println!("Number of lines: {}", lines_count);
    println!("Number of words: {}", words_count);
    println!("Number of characters: {}", chars_count);
}
