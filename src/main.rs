#[allow(unused_imports)]
use clap::{Parser, Subcommand};
use regex::Regex;
use std::env::{set_current_dir, split_paths};
use std::fmt::format;
use std::fs;
use std::fs::metadata;
use std::io::{self, Write};
use std::ops::Not;
use std::path::Path;
use std::{env, os::unix::process, process::exit, process::Command};
use std::{iter, vec};

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
    let re = Regex::new(r"'([^']*)'").unwrap();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        // process the entire input
        let mut matches: Vec<(usize, usize, &str)> = Vec::new();

        // find all matches and record their positions
        for cap in re.captures_iter(&input) {
            matches.push((
                cap.get(0).unwrap().start(),
                cap.get(0).unwrap().end(),
                cap.get(1).unwrap().as_str(),
            ));
        }

        let mut processed_input = input.clone();
        let mut content_map = std::collections::HashMap::new();

        // Sort by position (to ensure right-to-left processing)
        matches.sort_by(|a, b| b.0.cmp(&a.0));

        // Replace with space-padded placeholders
        for (idx, (start, end, content)) in matches.iter().enumerate() {
            // Special handling for adjacent quotes - don't add spaces around placeholder
            let is_adjacent_to_next = idx > 0 && matches[idx - 1].0 == *end;
            let is_adjacent_to_prev = idx < matches.len() - 1 && matches[idx + 1].1 == *start;
            // Use different placeholder format based on adjacency
            let placeholder = if is_adjacent_to_next || is_adjacent_to_prev {
                format!("QUOTED_{}", idx) // No spaces for adjacent quotes
            } else {
                format!(" QUOTED_{} ", idx) // Space-padded for non-adjacent
            };
            content_map.insert(format!("QUOTED_{}", idx), content.to_string());
            processed_input.replace_range(*start..*end, &placeholder);
        }

        // Split on whitespace and remove empty entries
        let parts: Vec<&str> = processed_input.trim().split_whitespace().collect();

        let mut processed_parts: Vec<String> = parts.iter().map(|&s| s.to_string()).collect();

        // Now actually replace the placeholders with the quoted content
        for i in 0..processed_parts.len() {
            let mut part = processed_parts[i].clone();

            // Check if this part contains multiple placeholders
            if part.contains("QUOTED_") {
                let mut replaced_content = String::new();
                let mut remaining = part.as_str();

                // Process each placeholder in the token
                while let Some(start_idx) = remaining.find("QUOTED_") {
                    // Add any text before the placeholder
                    replaced_content.push_str(&remaining[0..start_idx]);

                    // Find the end of the placeholder number
                    let mut end_idx = start_idx + 7; // "QUOTED_" is 7 chars
                    while end_idx < remaining.len()
                        && remaining[end_idx..end_idx + 1]
                            .chars()
                            .next()
                            .unwrap()
                            .is_digit(10)
                    {
                        end_idx += 1;
                    }

                    // Extract the placeholder
                    let placeholder = &remaining[start_idx..end_idx];

                    // Add the content for this placeholder
                    if let Some(content) = content_map.get(placeholder) {
                        replaced_content.push_str(content);
                    } else {
                        replaced_content.push_str(placeholder);
                    }

                    // Move past this placeholder
                    remaining = &remaining[end_idx..];
                }

                // Add any remaining text
                replaced_content.push_str(remaining);
                processed_parts[i] = replaced_content;
            } else if let Some(content) = content_map.get(&part) {
                // Simple case - entire token is a placeholder
                processed_parts[i] = content.clone();
            }
        }

        if processed_parts.is_empty() {
            continue;
        }

        // Try to parse the parts with clap; if parsing fails, treat as unknown
        let mut clap_args = vec!["your_shell".to_string()]; // Add program name first
        clap_args.extend(processed_parts.iter().cloned());
        let parse_result = ShellArgs::try_parse_from(&clap_args);
        let parsed_args = match parse_result {
            Ok(args) => args,
            Err(_) => {
                let args: Vec<&str> = processed_parts[1..].iter().map(|s| s.as_str()).collect();
                run_external_command(&processed_parts[0], &args);
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
                    if target == "~" {
                        let home_dir = env::var("HOME").unwrap_or_default();
                        if let Err(e) = env::set_current_dir(&home_dir) {
                            println!("cd: {}: No such file or directory", home_dir);
                        }
                    } else if let Err(e) = env::set_current_dir(&target) {
                        println!("cd: {}: No such file or directory", target);
                    }
                } else {
                    // Dfault to home directory or do nothing
                    println!("Usage: cd <directory>");
                }
            }
            Some(ShellCommand::Cat { files }) => {
                let mut contents: Vec<String> = Vec::new();
                for file_path in files {
                    let content = fs::read_to_string(file_path)
                        .expect("Should have been able to read the file");
                    contents.push(content);
                }

                // Print contents without an additional newline
                print!("{}", contents.concat());

                // Make sure the output is flushed
                io::stdout().flush().unwrap();
            }
            None => {
                // If the user typed something that doesn't match a subcommand
                run_external_command(
                    &processed_parts[0],
                    &processed_parts[1..]
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<&str>>(),
                );
            }
        }
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
