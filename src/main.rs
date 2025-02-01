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
        let parts: Vec<&str> = input.as_str().split_whitespace().collect();
        let (command, args) = (parts[0], &parts[1..]);

        match command {
            "exit" => exit(0),
            "echo" => {
                println!("{}", args.join(" "));
            }
            "type" => {
                if !args.is_empty() {
                    let sub_command = args[0].trim();
                    match sub_command {
                        "exit" | "echo" | "type" => println!("{} is a shell builtin", sub_command),
                        _ => println!("{}: not found", sub_command),
                    }
                }
            }
            _ => {
                if input.trim().is_empty() {
                    println!();
                } else {
                    println!("{}: command not found", input.trim());
                }
            }
        }
    }
}
