use clap::Command as ClapCommand;
#[allow(unused_imports)]
use clap::{Parser, Subcommand};
use shell_words;
use std::env::split_paths;
use std::fs;
use std::fs::metadata;
use std::fs::File;
use std::io::{self, Write};
use std::process;
use std::process::Command;
use std::os::unix::process::CommandExt;  // Import the Unix-specific CommandExt trait
use std::vec;
use std::{env, process::exit, process::Stdio};

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
    // Display the file contents
    Cat {
        // A file to concatenatet and print filesl
        files: Vec<String>,
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
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        // Use shell-words to parse the input following shell quoting rules
        let parts = match shell_words::split(input) {
            Ok(parts) => parts,
            Err(_) => {
                println!("Error: Invalid quoting in command");
                continue;
            }
        };

        if parts.is_empty() {
            continue;
        }

        // Check if the command includes redirection operators
        let has_redirection = parts.iter().any(|part| *part == ">" || *part == "1>");

        // If there's redirection, skip built-in command handling for echo
        if has_redirection {
            run_external_command(
                &parts[0],
                &parts[1..].iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
            );
            continue;
        }
        // Try to parse as built-in command with clap
        let mut clap_args = vec!["your_shell".to_string()];
        clap_args.extend(parts.iter().cloned());

        let parse_result = ShellArgs::try_parse_from(&clap_args);
        match parse_result {
            Ok(parsed_args) => {
                // Process built-in commands
                match parsed_args.command {
                    Some(ShellCommand::Echo { text }) => {
                        // Just print the text arguments directly
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
                            if target == "~" {
                                let home_dir = env::var("HOME").unwrap_or_default();
                                if let Err(_) = env::set_current_dir(&home_dir) {
                                    println!("cd: {}: No such file or directory", home_dir);
                                }
                            } else if let Err(_) = env::set_current_dir(&target) {
                                println!("cd: {}: No such file or directory", target);
                            }
                        } else {
                            // Default to home directory or do nothing
                            println!("Usage: cd <directory>");
                        }
                    }
                    Some(ShellCommand::Cat { files }) => {
                        for path in &files {
                            match fs::read_to_string(&path) {
                                Ok(content) => {
                                    print!("{}", content);
                                }
                                Err(_) => {
                                    eprintln!("cat: {}: No such file or directory", path);
                                }
                            }
                        }
                        io::stdout().flush().unwrap();
                    }
                    None => {
                        // If not a built-in command, try to execute it
                        run_external_command(
                            &parts[0],
                            &parts[1..].iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
                        );
                    }
                }
            }
            Err(_) => {
                // Not a built-in command, try to execute it
                run_external_command(
                    &parts[0],
                    &parts[1..].iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
                );
            }
        }
    }
}

/// Fallback logic to run external commands (not built-ins)
fn run_external_command(command: &str, args: &[&str]) {
    //  Check for redirection
    let mut redirect_file: Option<&str> = None;
    let mut filtered_args: Vec<&str> = Vec::new();

    // Find redirection operator
    let mut i = 0;
    while i < args.len() {
        if (args[i] == ">" || args[i] == "1>") && i + 1 < args.len() {
            redirect_file = Some(args[i + 1]);
            i += 2; // Skip the '>' and the filename
        } else {
            filtered_args.push(args[i]);
            i += 1;
        }
    }

    let path_var = env::var("PATH").unwrap_or_default();
    let directories = env::split_paths(&path_var);

    // constructing the Command:
    if let Some(filename) = redirect_file {
        // Set up redirection
        // You'll need to open the file and set it as stdout for the command
        // Look at std::fs::File::create and Command::stdout
        match File::create(filename) {
            Ok(f) => {
                let output = Stdio::from(f);

                let mut cmd = Command::new(command); // Use the full path to execute
                cmd.args(filtered_args);
                cmd.stdout(output);
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
            Err(e) => {
                println!("Failed to create output file: {}", e);
                return;
            }
        }
    }

    // First try direct execution (if command has path separators)
    if command.contains('/') {
        let mut cmd = Command::new(command);
        cmd.args(filtered_args);
        match cmd.spawn() {
            Ok(mut child) => {
                child.wait().unwrap();
                return;
            }
            Err(_) => {
                println!("{}: command not found", command);
                return;
            }
        }
    }

    // If no path separators, search in PATH
    let found = false;
    for dir in directories {
        let new_path = dir.join(command);

        if new_path.exists() {
            let mut cmd = Command::new(&new_path); // Use the full path to execute
            cmd.arg0(command);

            cmd.args(filtered_args);
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
