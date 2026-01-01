use crate::errors::{ShellError, ShellResult};
use std::fs::OpenOptions;
use std::process::{Command, Stdio};

/// Redirection mode (overwrite or append)
#[derive(Debug, Clone, Copy)]
pub enum RedirectionMode {
    Overwrite,
    Append,
}

/// Represents a redirection specification
#[derive(Debug, Clone)]
pub struct Redirection {
    pub file: String,
    pub mode: RedirectionMode,
}

/// Parse redirection operators from command arguments
/// Returns (filtered_args, stdout_redir, stderr_redir)
pub fn parse_redirection(
    args: &[String],
) -> ShellResult<(Vec<String>, Option<Redirection>, Option<Redirection>)> {
    let mut filtered_args = Vec::new();
    let mut stdout_redir = None;
    let mut stderr_redir = None;
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        if arg == ">" || arg == "1>" {
            if i + 1 < args.len() {
                stdout_redir = Some(Redirection {
                    file: args[i + 1].clone(),
                    mode: RedirectionMode::Overwrite,
                });
                i += 2;
                continue;
            } else {
                return Err(ShellError::RedirectionError(
                    "Expected filename after '>'".to_string(),
                ));
            }
        } else if arg == ">>" || arg == "1>>" {
            if i + 1 < args.len() {
                stdout_redir = Some(Redirection {
                    file: args[i + 1].clone(),
                    mode: RedirectionMode::Append,
                });
                i += 2;
                continue;
            } else {
                return Err(ShellError::RedirectionError(
                    "Expected filename after '>>'".to_string(),
                ));
            }
        } else if arg == "2>" {
            if i + 1 < args.len() {
                stderr_redir = Some(Redirection {
                    file: args[i + 1].clone(),
                    mode: RedirectionMode::Overwrite,
                });
                i += 2;
                continue;
            } else {
                return Err(ShellError::RedirectionError(
                    "Expected filename after '2>'".to_string(),
                ));
            }
        } else if arg == "2>>" {
            if i + 1 < args.len() {
                stderr_redir = Some(Redirection {
                    file: args[i + 1].clone(),
                    mode: RedirectionMode::Append,
                });
                i += 2;
                continue;
            } else {
                return Err(ShellError::RedirectionError(
                    "Expected filename after '2>>'".to_string(),
                ));
            }
        }

        filtered_args.push(arg.clone());
        i += 1;
    }

    Ok((filtered_args, stdout_redir, stderr_redir))
}

/// Set up file redirection for stdout/stderr
pub fn setup_redirection(
    cmd: &mut Command,
    stdout_redir: &Option<Redirection>,
    stderr_redir: &Option<Redirection>,
) -> ShellResult<()> {
    if let Some(redir) = stdout_redir {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(matches!(redir.mode, RedirectionMode::Overwrite))
            .append(matches!(redir.mode, RedirectionMode::Append))
            .open(&redir.file)
            .map_err(|e| ShellError::RedirectionError(format!("Failed to open '{}': {}", redir.file, e)))?;
        cmd.stdout(Stdio::from(file));
    }

    if let Some(redir) = stderr_redir {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(matches!(redir.mode, RedirectionMode::Overwrite))
            .append(matches!(redir.mode, RedirectionMode::Append))
            .open(&redir.file)
            .map_err(|e| ShellError::RedirectionError(format!("Failed to open '{}': {}", redir.file, e)))?;
        cmd.stderr(Stdio::from(file));
    }

    Ok(())
}
