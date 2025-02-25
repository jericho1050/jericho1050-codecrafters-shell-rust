#[allow(unused_imports)]
use clap::{Parser, Subcommand};
use std::env::{set_current_dir, split_paths};
use std::fs::metadata;
use std::io::{self, Write};
use std::iter;
use std::path::Path;
use std::{env, os::unix::process, process::exit, process::Command};

/// A simple interactive shell that supports basic commands
#[derive(Parser, Debug)]
#[command(version, about = "A simple Rust shell using clap.")]
struct ShellArgs {
    /// Optional subcommands for built-in behavior
    #[command(subcommand)]
    command: Option<ShellCommand>,
}

#[derive(Subcommand, Debug)]
enum ShellCommand {
    /// Print text stdout
    Echo {
        /// The text o print
        text: Vec<String>,
    },
    Pwd,
    /// Exit the shell
    Exit {
        code: Option<i32>,
    },
    /// Print how a command name is interpreted
    Type {
        // The command name to inspect
        name: String,
    },
    /// Change Directory
    Cd {
        // The directory to switch to
        dir: Option<String>,
    },
}
fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        // Convert the line to a vec of &str for clap to parsel

        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        // Try to parse the parts with clap; if parsing fails, treat as unknown
        let parse_result = ShellArgs::try_parse_from(std::iter::once("myshell").chain(parts.iter().map(|s| *s)));
        let parsed_args = match parse_result {
            Ok(args) => args,
            Err(_) => {
                run_external_command(&parts[0], &parts[1..]);
                continue;
            }
        };

        // Match on subcommands
        match parsed_args.command {
            Some(ShellCommand::Echo { text }) => {
                println!("{}", text.join(" "));
            }
            Some(ShellCommand::Pwd) => {
                let path = env::current_dir().unwrap();
                let pwd = String::from(path.to_string_lossy());
                println!("{}", pwd);
            }
            Some(ShellCommand::Exit { code }) => exit(code.unwrap_or(0)),
            Some(ShellCommand::Type { name }) => match name.as_str() {
                "exit" | "echo" | "type" | "pwd" | "cd" => {
                    println!("{} is a shell builtin", name)
                }
                _ => {
                    let sub_command = &name;
                    let path = env::var("PATH").unwrap_or_default();
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
                        println!("{}: not found", sub_command)
                    }
                }
            },
            Some(ShellCommand::Cd { dir }) => {
                if let Some(target) = dir {
                    if let Err(e) = env::set_current_dir(&target) {
                        println!("cd: {}: No such file or directory",target);
                    }
                } else {
                    // Dfault to home directory or do nothing
                    println!("Usage: cd <directory>");
                }
            }
            None => {
                // If the user typed something that doesn't match a subcommand
                run_external_command(&parts[0], &parts[1..]);
            }
        }

        let (command, args) = (parts[0], &parts[1..]);
        let path = env::var("PATH").unwrap_or_default();
    }
}

/// Fallback logic to run external commands (not built-ins)
fn run_external_command(command: &str, args: &[&str]) {
    let path_var = env::var("PATH").unwrap_or_default();
    let directories = env::split_paths(&path_var);
    //
    let mut found = false;

    for dir in directories {
        let new_path = dir.join(command);
        if new_path.exists() && metadata(&new_path).unwrap().is_file() {
            found = true;
            let mut cmd = Command::new(command);
            cmd.args(args);
            match cmd.spawn() {
                Ok(mut child) => {
                    child.wait().unwrap();
                }
                Err(e) => {
                    println!("Failed to execute {}: {}", command, e);
                }
            }
            return;
        }
    }
    if !found {
        println!("{}: command not found", command);
    }
}
