use regex::Regex;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::{os::unix::process, process::exit};
fn main() {
    // Uncomment this block to pass the first stage

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let parts: Vec<&str> = input.as_str().split(" ").collect();
        let (command, args) = (parts[0], &parts[1..]);

        match input.trim() {
            "exit 0" => exit(0),
            command => {
                if command.starts_with("echo") {
                    print!("{}", args.join(" "));
                } else {
                    println!("{}: command not found", input.trim())
                }
            }
        }
    }
}
