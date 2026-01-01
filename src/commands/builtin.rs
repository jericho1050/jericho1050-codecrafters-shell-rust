use crate::errors::{ShellError, ShellResult};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Check if a command is a builtin or exists in PATH
pub fn handle_type_command(name: &str) -> ShellResult<()> {
    let builtins = ["echo", "exit", "type", "pwd", "cd"];

    if builtins.contains(&name) {
        println!("{} is a shell builtin", name);
        return Ok(());
    }

    // Check PATH
    if let Ok(path) = env::var("PATH") {
        for dir in path.split(':') {
            let full_path = format!("{}/{}", dir, name);
            let path = Path::new(&full_path);
            if path.exists() {
                // Check if file has execute permissions
                if let Ok(metadata) = path.metadata() {
                    use std::os::unix::fs::PermissionsExt;
                    let is_executable = metadata.permissions().mode() & 0o111 != 0;
                    if metadata.is_file() && is_executable {
                        println!("{} is {}", name, full_path);
                        return Ok(());
                    }
                }
            }
        }
    }

    println!("{}: not found", name);
    Ok(())
}

/// Change the current working directory
pub fn handle_cd_command(path: Option<&str>) -> ShellResult<()> {
    let target = match path {
        Some(p) if p.starts_with('~') => {
            // Expand ~ to home directory
            match env::var("HOME") {
                Ok(home) => p.replacen('~', &home, 1),
                Err(_) => {
                    return Err(ShellError::InvalidDirectory(
                        "HOME environment variable not set".to_string(),
                    ));
                }
            }
        }
        Some(p) => p.to_string(),
        None => {
            // No argument provided - go to HOME directory
            match env::var("HOME") {
                Ok(home) => home,
                Err(_) => {
                    return Err(ShellError::InvalidDirectory(
                        "HOME environment variable not set".to_string(),
                    ));
                }
            }
        }
    };

    env::set_current_dir(&target).map_err(|_| {
        ShellError::InvalidDirectory(format!("cd: {}: No such file or directory", target))
    })
}

/// Execute the echo command
pub fn handle_echo_command(args: &[String]) -> ShellResult<()> {
    println!("{}", args.join(" "));
    Ok(())
}

/// Execute the pwd command
pub fn handle_pwd_command() -> ShellResult<()> {
    let current_dir = env::current_dir().map_err(|e| ShellError::IoError(e))?;
    println!("{}", current_dir.display());
    Ok(())
}

/// Execute the cat command
pub fn handle_cat_command(files: &[String]) -> ShellResult<()> {
    for file in files {
        match fs::read_to_string(file) {
            Ok(content) => {
                print!("{}", content);
                io::stdout().flush().map_err(|e| ShellError::IoError(e))?;
            }
            Err(_) => {
                // Print error to stderr and continue (like real cat)
                eprintln!("cat: {}: No such file or directory", file);
            }
        }
    }
    Ok(())
}


pub fn handle_tail_command(files: &[String], lines: Option<usize>) -> ShellResult<()> {
    for file in files {
        let content = fs::read_to_string(file)
            .map_err(|e| ShellError::IoError(e))?;
        print!("{}", content);
        io::stdout().flush().map_err(|e| ShellError::IoError(e))?;
    }
    Ok(())
}

pub fn handle_head_command(files: &[String], lines: Option<usize>) -> ShellResult<()> {
    for file in files {
        let content = fs::read_to_string(file)
            .map_err(|e| ShellError::IoError(e))?;
        print!("{}", content);
        io::stdout().flush().map_err(|e| ShellError::IoError(e))?;
    }
    Ok(())
}

pub fn handle_wc_command(files: &[String]) -> ShellResult<()> {
    for file in files {
        let content = fs::read_to_string(file)
            .map_err(|e| ShellError::IoError(e))?;
        print!("{}", content);
        io::stdout().flush().map_err(|e| ShellError::IoError(e))?;
    }
    Ok(())
}