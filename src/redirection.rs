use crate::errors::{ShellError, ShellResult};
use std::fs::{File, OpenOptions};
use std::io;
use std::os::unix::io::{AsRawFd, FromRawFd};
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

/// Guard that restores stdout/stderr when dropped
pub struct RedirectionGuard {
    saved_stdout: Option<File>,
    saved_stderr: Option<File>,
}

impl Drop for RedirectionGuard {
    fn drop(&mut self) {
        // Flush stdout/stderr before restoring
        let _ = io::Write::flush(&mut io::stdout());
        let _ = io::Write::flush(&mut io::stderr());

        if let Some(saved) = self.saved_stdout.take() {
            unsafe {
                libc::dup2(saved.as_raw_fd(), libc::STDOUT_FILENO);
            }
        }
        if let Some(saved) = self.saved_stderr.take() {
            unsafe {
                libc::dup2(saved.as_raw_fd(), libc::STDERR_FILENO);
            }
        }
    }
}

/// Set up file redirection for builtins (redirects process stdout/stderr)
/// Returns a guard that restores the original file descriptors when dropped
pub fn setup_builtin_redirection(
    stdout_redir: &Option<Redirection>,
    stderr_redir: &Option<Redirection>,
) -> ShellResult<RedirectionGuard> {
    let mut guard = RedirectionGuard {
        saved_stdout: None,
        saved_stderr: None,
    };

    if let Some(redir) = stdout_redir {
        // Save current stdout
        let saved_fd = unsafe { libc::dup(libc::STDOUT_FILENO) };
        if saved_fd >= 0 {
            guard.saved_stdout = Some(unsafe { File::from_raw_fd(saved_fd) });
        }

        // Open the redirect file
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(matches!(redir.mode, RedirectionMode::Overwrite))
            .append(matches!(redir.mode, RedirectionMode::Append))
            .open(&redir.file)
            .map_err(|e| ShellError::RedirectionError(format!("Failed to open '{}': {}", redir.file, e)))?;

        // Redirect stdout to file
        unsafe {
            libc::dup2(file.as_raw_fd(), libc::STDOUT_FILENO);
        }
    }

    if let Some(redir) = stderr_redir {
        // Save current stderr
        let saved_fd = unsafe { libc::dup(libc::STDERR_FILENO) };
        if saved_fd >= 0 {
            guard.saved_stderr = Some(unsafe { File::from_raw_fd(saved_fd) });
        }

        // Open the redirect file
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(matches!(redir.mode, RedirectionMode::Overwrite))
            .append(matches!(redir.mode, RedirectionMode::Append))
            .open(&redir.file)
            .map_err(|e| ShellError::RedirectionError(format!("Failed to open '{}': {}", redir.file, e)))?;

        // Redirect stderr to file
        unsafe {
            libc::dup2(file.as_raw_fd(), libc::STDERR_FILENO);
        }
    }

    Ok(guard)
}
