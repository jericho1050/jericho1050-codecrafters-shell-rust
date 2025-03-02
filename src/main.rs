#[allow(unused_imports)]
use clap::{Parser, Subcommand};
use shell_words;
use std::env::split_paths;
use std::fs;
use std::fs::metadata;
use std::fs::File;
use std::io::{self, Write};
use std::os::unix::process::CommandExt; // Import the Unix-specific CommandExt trait
use std::process::Command;
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

enum RedirectionMode {
    Overwrite,
    Append,
}

struct Redirection {
    file: String,
    mode: RedirectionMode,
}

fn main() {
    loop {
        // Display prompt and get user input
        if let Err(_) = handle_command_input() {
            // Handle any errors from command execution
            // For now we just continue the loop
        }
    }
}

// Process a single command input
/// Process a single command input
fn handle_command_input() -> Result<(), ()> {
    print!("$ ");
    io::stdout().flush().unwrap();

    // Wait for user input
    let input = match read_input() {
        Some(input) => input,
        None => return Ok(()),
    };

    // Parse input into command parts
    let parts = match shell_words::split(&input) {
        Ok(parts) => parts,
        Err(_) => {
            println!("Error: Invalid quoting in command");
            return Ok(());
        }
    };

    if parts.is_empty() {
        return Ok(());
    }

    // Check if the command includes redirection operators
    let has_redirection = parts.iter().any(|part| {
        *part == ">"
            || *part == ">>"
            || *part == "1>"
            || *part == "1>>"
            || *part == "2>"
            || *part == "2>>"
    });

    // If there's redirection, skip built-in command handling
    if has_redirection {
        let args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();
        run_external_command(&parts[0], &args);
        return Ok(());
    }

    // Try to parse as built-in command with clap
    handle_command_with_clap(&parts)
}

/// Read user input from stdin
fn read_input() -> Option<String> {
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return None;
    }

    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    Some(input.to_string())
}

/// Parse and execute a command using clap
fn handle_command_with_clap(parts: &[String]) -> Result<(), ()> {
    let mut clap_args = vec!["your_shell".to_string()];
    clap_args.extend(parts.iter().cloned());

    match ShellArgs::try_parse_from(&clap_args) {
        Ok(parsed_args) => execute_builtin_command(parsed_args, parts),
        Err(_) => {
            // Not a built-in command, try to execute it
            let args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();
            run_external_command(&parts[0], &args);
            Ok(())
        }
    }
}

/// Execute a built-in shell command
fn execute_builtin_command(parsed_args: ShellArgs, parts: &[String]) -> Result<(), ()> {
    match parsed_args.command {
        Some(ShellCommand::Echo { text }) => {
            println!("{}", text.join(" "));
        }
        Some(ShellCommand::Pwd) => {
            let path = env::current_dir().unwrap();
            println!("{}", path.to_string_lossy());
        }
        Some(ShellCommand::Exit { code }) => exit(code.unwrap_or(0)),
        Some(ShellCommand::Type { name }) => {
            handle_type_command(&name);
        }
        Some(ShellCommand::Cd { dir }) => {
            handle_cd_command(dir);
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
            let args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();
            run_external_command(&parts[0], &args);
        }
    }
    Ok(())
}

/// Handle the 'type' built-in command
fn handle_type_command(name: &str) {
    match name {
        "exit" | "echo" | "type" | "pwd" | "cd" => {
            println!("{} is a shell builtin", name)
        }
        _ => {
            let path = env::var("PATH").unwrap_or_default();
            let directories = split_paths(&path);
            let mut found = false;
            for dir in directories {
                let new_path = dir.join(name);
                if let Ok(meta) = metadata(&new_path) {
                    if new_path.exists() && meta.is_file() {
                        println!("{} is {}", name, new_path.display());
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                println!("{}: not found", name)
            }
        }
    }
}

/// Handle the 'cd' built-in command
fn handle_cd_command(dir: Option<String>) {
    match dir {
        Some(target) => {
            if target == "~" {
                let home_dir = env::var("HOME").unwrap_or_default();
                if let Err(_) = env::set_current_dir(&home_dir) {
                    println!("cd: {}: No such file or directory", home_dir);
                }
            } else if let Err(_) = env::set_current_dir(&target) {
                println!("cd: {}: No such file or directory", target);
            }
        }
        None => {
            println!("Usage: cd <directory>");
        }
    }
}

/// Parse command args for redirection operators and build filtered arguments
fn parse_redirection<'a>(
    args: &'a [&'a str],
) -> (Vec<&'a str>, Option<Redirection>, Option<Redirection>) {
    let mut filtered_args = Vec::new();
    let mut stdout_redirect = None;
    let mut stderr_redirect = None;

    let mut i = 0;
    while i < args.len() {
        if (args[i] == "1>" || args[i] == ">") && i + 1 < args.len() {
            stdout_redirect = Some(Redirection {
                file: args[i + 1].to_string(),
                mode: RedirectionMode::Overwrite,
            });
            i += 2;
        } else if (args[i] == "1>>" || args[i] == ">>") && i + 1 < args.len() {
            stdout_redirect = Some(Redirection {
                file: args[i + 1].to_string(),
                mode: RedirectionMode::Append,
            });
            i += 2;
        } else if args[i] == "2>" && i + 1 < args.len() {
            stderr_redirect = Some(Redirection {
                file: args[i + 1].to_string(),
                mode: RedirectionMode::Overwrite,
            });
            i += 2;
        } else if args[i] == "2>>" && i + 1 < args.len() {
            stderr_redirect = Some(Redirection {
                file: args[i + 1].to_string(),
                mode: RedirectionMode::Append,
            });
            i += 2;
        } else {
            filtered_args.push(args[i]);
            i += 1;
        }
    }

    (filtered_args, stdout_redirect, stderr_redirect)
}

/// Create a file handle for redirection
fn setup_redirection(redirect: &Option<Redirection>) -> Option<Stdio> {
    match redirect {
        Some(redirection) => {
            let file_result = match redirection.mode {
                RedirectionMode::Overwrite => File::create(&redirection.file),
                RedirectionMode::Append => File::options()
                    .append(true)
                    .create(true)
                    .open(&redirection.file),
            };

            match file_result {
                Ok(file) => Some(Stdio::from(file)),
                Err(e) => {
                    println!("Failed to create output file: {}", e);
                    None
                }
            }
        }
        None => None,
    }
}

/// Run external (non-builtin) commands
fn run_external_command(command: &str, args: &[&str]) {
    let (filtered_args, stdout_redirect, stderr_redirect) = parse_redirection(args);

    // First try direct execution (if command has path separators)
    if command.contains('/') {
        // Set up redirections for direct execution
        let stdout_redirection = setup_redirection(&stdout_redirect);
        let stderr_redirection = setup_redirection(&stderr_redirect);
        
        if try_spawn_command_direct(
            command,
            &filtered_args,
            stdout_redirection,
            stderr_redirection,
        ) {
            return;
        }
        println!("{}: command not found", command);
        return;
    }

    // Set up redirections for normal command and PATH search
    let stdout_redirection = setup_redirection(&stdout_redirect);
    let stderr_redirection = setup_redirection(&stderr_redirect);

    // Try normal command execution
    if try_spawn_command(
        command,
        &filtered_args,
        stdout_redirection,
        stderr_redirection,
    ) {
        return;
    }

    // If no path separators, search in PATH
    let stdout_redirection = setup_redirection(&stdout_redirect);
    let stderr_redirection = setup_redirection(&stderr_redirect);
    
    if try_spawn_from_path(
        command,
        &filtered_args,
        stdout_redirection,
        stderr_redirection,
    ) {
        return;
    }

    println!("{}: command not found", command);
}

/// Try to spawn a command with the given name directly
fn try_spawn_command(
    command: &str,
    args: &[&str],
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
) -> bool {
    let mut cmd = Command::new(command);
    cmd.args(args);

    if let Some(stdout) = stdout {
        cmd.stdout(stdout);
    }
    if let Some(stderr) = stderr {
        cmd.stderr(stderr);
    }

    match cmd.spawn() {
        Ok(mut child) => {
            child.wait().unwrap();
            true
        }
        Err(_) => false,
    }
}

/// Try to spawn a command using a direct path
fn try_spawn_command_direct(
    path: &str,
    args: &[&str],
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
) -> bool {
    let mut cmd = Command::new(path);
    cmd.args(args);

    if let Some(stdout) = stdout {
        cmd.stdout(stdout);
    }
    if let Some(stderr) = stderr {
        cmd.stderr(stderr);
    }

    match cmd.spawn() {
        Ok(mut child) => {
            child.wait().unwrap();
            true
        }
        Err(_) => false,
    }
}

/// Try to spawn a command by searching in PATH
fn try_spawn_from_path(
    command: &str,
    args: &[&str],
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
) -> bool {
    let path_var = env::var("PATH").unwrap_or_default();
    let directories = env::split_paths(&path_var);

    for dir in directories {
        let command_path = dir.join(command);

        if command_path.exists() {
            let mut cmd = Command::new(&command_path);
            cmd.arg0(command);
            cmd.args(args);

            if let Some(stdout) = stdout {
                cmd.stdout(stdout);
                break; // Exit after using stdout once
            }
            if let Some(stderr) = stderr {
                cmd.stderr(stderr);
            }

            match cmd.spawn() {
                Ok(mut child) => {
                    child.wait().unwrap();
                    return true;
                }
                Err(e) => {
                    println!("Failed to execute {}: {}", command, e);
                    return false;
                }
            }
        }
    }

    false
}
