use std::env::split_paths;
#[allow(unused_imports)]
use std::fs::metadata;
use std::io::{self, Write};
use std::path::Path;
use std::{env, os::unix::process, process::exit};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let parts: Vec<&str> = input.as_str().split_whitespace().collect();
        let (command, args) = (parts[0], &parts[1..]);
        let path = env::var("PATH").unwrap_or_default();

        match command {
            "exit" => exit(0),
            "echo" => {
                println!("{}", args.join(" "));
            }
            "type" => {
                if !args.is_empty() {
                    let sub_command = args[0].trim();
                    let directories = split_paths(&path);
                    let mut found = false;
                    for dir in directories {
                        let new_path = dir.join(sub_command);
                        let metadata = match metadata(&new_path) {
                            Ok(m) => m,
                            Err(_) => continue,
                        };
                        if new_path.exists() && metadata.is_file() {
                            println!("{} is {}", sub_command, new_path.display());
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        match sub_command {
                            "exit" | "echo" | "type" => {
                                println!("{} is a shell builtin", sub_command)
                            }
                            _ => println!("{}: not found", sub_command),
                        }
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
