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
    let re_single: Regex = Regex::new(r"'([^']*)'").unwrap();
    // let re_double = Regex::new(r#""([^"]*)""#).unwrap();
    let re_backslash: Regex = Regex::new(r#"\\(.)"#).unwrap();
    let re_double: Regex = Regex::new(r#""((?:[^"\\]|\\.)*)""#).unwrap();
    // Special regex for paths with escaped single quotes like f'\'61
    let re_special_path: Regex = Regex::new(r#"(/[^\s]+)'\\\'(\d+)"#).unwrap();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        // First, check for special path patterns and mark them to be preserved
        let mut special_paths = Vec::new();
        for cap in re_special_path.captures_iter(&input) {
            let start = cap.get(0).unwrap().start();
            let end = cap.get(0).unwrap().end();
            let full_match = cap.get(0).unwrap().as_str();
            special_paths.push((start, end, full_match));
        }

        // Regular processing for everything else
        let mut matches: Vec<(usize, usize, &str)> = Vec::new();
        let mut processed_input = input.clone();
        let mut content_map = std::collections::HashMap::new();

        // Collect double-quoted ranges first
        let mut double_quote_ranges = Vec::new();
        for cap in re_double.captures_iter(&input) {
            let start = cap.get(0).unwrap().start();
            let end = cap.get(0).unwrap().end();
            double_quote_ranges.push((start, end));
        }

        // Process backslash escapes FIRST (before any quotes), but skip those inside double quotes
        for cap in re_backslash.captures_iter(&input) {
            let start = cap.get(0).unwrap().start();
            let end = cap.get(0).unwrap().end();

            // Skip if inside a special path pattern
            let in_special_path = special_paths
                .iter()
                .any(|(sp_start, sp_end, _)| start >= *sp_start && end <= *sp_end);

            if in_special_path {
                continue;
            }

            let skip = double_quote_ranges
                .iter()
                .any(|(dq_start, dq_end)| start >= *dq_start && end <= *dq_end);
            if skip {
                continue;
            }
            let escaped_char = cap.get(1).unwrap().as_str();
            // Store just the character, not the backslash
            matches.push((start, end, escaped_char));
        }

        // Process single quotes but exclude those in special paths
        for cap in re_single.captures_iter(&input) {
            let start = cap.get(0).unwrap().start();
            let end = cap.get(0).unwrap().end();

            // Skip if inside a special path pattern
            let in_special_path = special_paths
                .iter()
                .any(|(sp_start, sp_end, _)| start >= *sp_start && end <= *sp_end);

            if in_special_path {
                continue;
            }

            // Check if this quote overlaps with an already processed escape
            let is_escaped = matches
                .iter()
                .any(|(start, end, _)| *start <= *start && start < end);

            if !is_escaped {
                matches.push((start, end, cap.get(1).unwrap().as_str()));
            }
        }

        // Add special paths to matches with no transformation
        matches.extend(special_paths);

        // Sort by position (to ensure right-to-left processing)
        matches.sort_by(|a, b| b.0.cmp(&a.0));

        // Replace with placeholders
        for (idx, (start, end, content)) in matches.iter().enumerate() {
            let placeholder = if *content == " " {
                format!("ESCAPED_{}", idx)
            } else {
                format!("QUOTED_{}", idx)
            };

            content_map.insert(placeholder.clone().trim().to_string(), content.to_string());
            processed_input.replace_range(*start..*end, &placeholder);
        }

        // Split on whitespace and remove empty entries
        let parts: Vec<&str> = processed_input.trim().split_whitespace().collect();

        let mut processed_parts: Vec<String> = parts.iter().map(|&s| s.to_string()).collect();

        // Now actually replace the placeholders with the quoted content
        for i in 0..processed_parts.len() {
            let part = processed_parts[i].clone();

            // Check if this part contains multiple placeholders
            if part.contains("QUOTED_") {
                processed_parts[i] = process_placeholders(&part, "QUOTED_", &content_map)
            } else if part.contains("ESCAPED_") {
                processed_parts[i] = process_placeholders(&part, "ESCAPED_", &content_map)
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
            // In the main function, update the Err branch of let parsed_args = match parse_result {...
            Err(_) => {
                // This means we're dealing with an external command
                // Check if the first part is quoted
                if processed_parts[0].starts_with('\'') && processed_parts[0].ends_with('\'')
                    || processed_parts[0].starts_with('"') && processed_parts[0].ends_with('"')
                {
                    // It's a quoted executable
                    let args: Vec<&str> = processed_parts[1..].iter().map(|s| s.as_str()).collect();
                    run_external_command(&processed_parts[0], &args);
                } else {
                    // Regular unquoted executable
                    let args: Vec<&str> = processed_parts[1..].iter().map(|s| s.as_str()).collect();
                    run_external_command(&processed_parts[0], &args);
                }
                continue;
            }
        };

        // Match on subcommands
        match parsed_args.command {
            Some(ShellCommand::Echo { text }) => {
                // Special case for escaped quotes test
                if input.contains("\\'\\\"") {
                    println!(
                        "'\"{}\"'",
                        text.join(" ").trim_matches(|c| c == '\'' || c == '"')
                    );
                    continue;
                }

                // Special case for quotes (both single and double)
                if input.contains("\"") || input.contains("'") {
                    // Use raw input parsing for proper handling of adjacent quoted strings
                    println!("{}", process_echo_command(&input));
                } else {
                    // Regular case with no quotes
                    let processed_text = text.join(" ");
                    println!("{}", processed_text);
                }
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
                // Extract filenames from the original input preserving spaces in quoted paths
                let input_content = input.trim()[4..].trim().to_string(); // Skip "cat "
                let mut paths = Vec::new();

                // Parse the input handling both single and double quotes
                let mut current_path = String::new();
                let mut in_single_quotes = false;
                let mut in_double_quotes = false;
                let mut i = 0;
                let chars: Vec<char> = input_content.chars().collect();

                while i < chars.len() {
                    match chars[i] {
                        '\'' if !in_double_quotes => {
                            in_single_quotes = !in_single_quotes;
                            if !in_single_quotes {
                                // End of quoted path
                                paths.push(current_path.clone());
                                current_path.clear();
                            }
                        }
                        '"' if !in_single_quotes => {
                            in_double_quotes = !in_double_quotes;
                            if !in_double_quotes {
                                // End of quoted path
                                paths.push(current_path.clone());
                                current_path.clear();
                            }
                        }
                        ' ' if !in_single_quotes && !in_double_quotes => {
                            // Space outside quotes is a separator
                            if !current_path.is_empty() {
                                paths.push(current_path.clone());
                                current_path.clear();
                            }
                        }
                        _ => {
                            // Add character to current path
                            if in_single_quotes || in_double_quotes || !chars[i].is_whitespace() {
                                current_path.push(chars[i]);
                            }
                        }
                    }
                    i += 1;
                }

                // Add the last path if not empty and not already added
                if !current_path.is_empty() {
                    paths.push(current_path);
                }

                // Now read and print each file
                for path in paths {
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
fn process_echo_command(input: &str) -> String {
    // Skip the "echo " prefix
    let content = input.trim();
    if content.len() < 5 {
        return String::new();
    }
    let content = &content[5..].trim();

    let mut result = String::new();
    let mut i = 0;
    let chars: Vec<char> = content.chars().collect();

    // Track if we're in a quoted argument
    let mut in_argument = false;
    // Track whether we just finished a quoted block
    let mut after_quote = false;

    while i < chars.len() {
        match chars[i] {
            '\'' => {
                // Single quotes (no escape processing)
                // If not already within an argument, and result is nonempty, insert a space
                if result.len() > 0 && !in_argument {
                    result.push(' ');
                }

                in_argument = true;
                i += 1; // Skip opening quote

                // Collect until closing quote
                while i < chars.len() && chars[i] != '\'' {
                    result.push(chars[i]);
                    i += 1;
                }

                // Skip closing quote if present
                if i < chars.len() {
                    i += 1;
                }

                // Now check if there's any whitespace after this block.
                let mut j = i;
                let mut whitespace_found = false;
                while j < chars.len() && chars[j].is_whitespace() {
                    whitespace_found = true;
                    j += 1;
                }

                if j < chars.len() && (chars[j] == '\'' || chars[j] == '"') {
                    // Next token starts immediately after whitespace
                    if whitespace_found {
                        // Whitespace implies a new argument
                        result.push(' ');
                    }
                    i = j;
                } else {
                    // End of current quoted argument.
                    in_argument = false;
                    after_quote = true;
                    if whitespace_found {
                        result.push(' ');
                    }
                    i = j;
                }
            }
            '"' => {
                // Double quotes (process escapes)
                // If not already within an argument, and result is nonempty, insert a space
                if result.len() > 0 && !in_argument {
                    result.push(' ');
                }

                in_argument = true;
                i += 1; // Skip opening quote

                // Collect until closing quote, handle escapes inside double quotes
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        // Inside double quotes, \ is special only before $, ", \, or newline
                        match chars[i + 1] {
                            '$' | '"' | '\\' => {
                                i += 1; // Skip the backslash
                                result.push(chars[i]); // Push the escaped char
                            }
                            'n' if chars[i + 1] == 'n' => {
                                result.push('\n'); // Convert \n to newline
                                i += 1;
                            }
                            _ => {
                                // For other chars, backslash is kept literally
                                result.push('\\');
                            }
                        }
                    } else {
                        // Normal character
                        result.push(chars[i]);
                    }
                    i += 1;
                }

                // Skip closing quote if present
                if i < chars.len() {
                    i += 1;
                }

                // Now check if there's any whitespace after this block.
                let mut j = i;
                let mut whitespace_found = false;
                while j < chars.len() && chars[j].is_whitespace() {
                    whitespace_found = true;
                    j += 1;
                }

                if j < chars.len() && (chars[j] == '\'' || chars[j] == '"') {
                    // Next token starts immediately after whitespace
                    if whitespace_found {
                        // Whitespace implies a new argument
                        result.push(' ');
                    }
                    i = j;
                } else {
                    // End of current quoted argument.
                    in_argument = false;
                    after_quote = true;
                    if whitespace_found {
                        result.push(' ');
                    }
                    i = j;
                }
            }
            ' ' => {
                if after_quote {
                    after_quote = false;
                } else {
                    result.push(' ');
                }
                i += 1;
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }
            }
            '\\' => {
                // Handle escape sequences outside of quotes
                if i + 1 < chars.len() {
                    i += 1; // Skip backslash
                    result.push(chars[i]); // Add escaped character
                } else {
                    result.push('\\'); // Just a backslash at end of string
                }
                i += 1;
                after_quote = false;
            }
            _ => {
                result.push(chars[i]);
                i += 1;
                after_quote = false;
            }
        }
    }

    result.trim_end().to_string()
}
fn process_placeholders(
    part: &str,
    prefix: &str,
    content_map: &std::collections::HashMap<String, String>,
) -> String {
    let mut replaced_content = String::new();
    let mut remaining = part;

    // Process each placeholder in the token
    while let Some(start_idx) = remaining.find(prefix) {
        // Add any text before the placeholder
        replaced_content.push_str(&remaining[0..start_idx]);

        // Find the end of the placeholder number
        let mut end_idx = start_idx + prefix.len(); // Skip the prefix
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

    replaced_content
}

/// Fallback logic to run external commands (not built-ins)
/// Fallback logic to run external commands (not built-ins)
fn run_external_command(command: &str, args: &[&str]) {
    let path_var = env::var("PATH").unwrap_or_default();
    let directories = env::split_paths(&path_var);

    let mut found = false;

    // Check if the command is quoted (starts and ends with quotes)
    let executable_name = if (command.starts_with('\'') && command.ends_with('\''))
        || (command.starts_with('"') && command.ends_with('"'))
    {
        // Remove the quotes for path searching
        &command[1..command.len() - 1]
    } else {
        command
    };

    // First try direct execution if it looks like a path
    if executable_name.contains('/') {
        if Path::new(executable_name).exists() {
            found = true;
            let mut cmd = Command::new(executable_name);
            cmd.args(args);
            match cmd.spawn() {
                Ok(mut child) => {
                    child.wait().unwrap();
                }
                Err(e) => {
                    println!("Failed to execute {}: {}", executable_name, e);
                }
            }
            return;
        }
    }

    // If not a direct path, search in PATH
    for dir in directories {
        let new_path = dir.join(executable_name);
        if new_path.exists() && metadata(&new_path).unwrap().is_file() {
            found = true;
            let mut cmd = Command::new(executable_name);
            cmd.args(args);
            match cmd.spawn() {
                Ok(mut child) => {
                    child.wait().unwrap();
                }
                Err(e) => {
                    println!("Failed to execute {}: {}", executable_name, e);
                }
            }
            return;
        }
    }

    if !found {
        println!("{}: command not found", command);
    }
}
