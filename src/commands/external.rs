use crate::errors::{ShellError, ShellResult};
use crate::redirection::{parse_redirection, setup_redirection};
use std::env;
use std::path::Path;
use std::process::Command;

/// Execute an external command
pub fn run_external_command(args: &[String]) -> ShellResult<()> {
    if args.is_empty() {
        return Ok(());
    }

    let command_name = &args[0];

    // Parse redirection operators
    let (filtered_args, stdout_redir, stderr_redir) = parse_redirection(args)?;

    // If command contains '/', execute directly
    if command_name.contains('/') {
        return execute_command_direct(command_name, &filtered_args[1..], &stdout_redir, &stderr_redir);
    }

    // Try to execute as a simple command first
    match execute_command_simple(&filtered_args[0], &filtered_args[1..], &stdout_redir, &stderr_redir) {
        Ok(_) => return Ok(()),
        Err(_) => {
            // Fall back to PATH search
            execute_command_from_path(command_name, &filtered_args[1..], &stdout_redir, &stderr_redir)
        }
    }
}

/// Execute a command directly (with absolute or relative path)
fn execute_command_direct(
    command_path: &str,
    args: &[String],
    stdout_redir: &Option<crate::redirection::Redirection>,
    stderr_redir: &Option<crate::redirection::Redirection>,
) -> ShellResult<()> {
    let mut cmd = Command::new(command_path);
    cmd.args(args);

    setup_redirection(&mut cmd, stdout_redir, stderr_redir)?;

    let status = cmd.status().map_err(|e| {
        ShellError::ExecutionError(format!("Failed to execute {}: {}", command_path, e))
    })?;

    if !status.success() {
        return Err(ShellError::ExecutionError(format!(
            "Command exited with status: {}",
            status.code().unwrap_or(-1)
        )));
    }

    Ok(())
}

/// Execute a command without PATH search
fn execute_command_simple(
    command_name: &str,
    args: &[String],
    stdout_redir: &Option<crate::redirection::Redirection>,
    stderr_redir: &Option<crate::redirection::Redirection>,
) -> ShellResult<()> {
    let mut cmd = Command::new(command_name);
    cmd.args(args);

    setup_redirection(&mut cmd, stdout_redir, stderr_redir)?;

    let status = cmd
        .status()
        .map_err(|_| ShellError::CommandNotFound(command_name.to_string()))?;

    if !status.success() {
        return Err(ShellError::ExecutionError(format!(
            "Command exited with status: {}",
            status.code().unwrap_or(-1)
        )));
    }

    Ok(())
}

/// Execute a command by searching PATH directories
fn execute_command_from_path(
    command_name: &str,
    args: &[String],
    stdout_redir: &Option<crate::redirection::Redirection>,
    stderr_redir: &Option<crate::redirection::Redirection>,
) -> ShellResult<()> {
    let path_var = env::var("PATH").map_err(|_| {
        ShellError::CommandNotFound(format!("{}: command not found", command_name))
    })?;

    for path_dir in path_var.split(':') {
        let full_path = format!("{}/{}", path_dir, command_name);
        if Path::new(&full_path).exists() {
            let mut cmd = Command::new(&full_path);
            cmd.args(args);

            setup_redirection(&mut cmd, stdout_redir, stderr_redir)?;

            match cmd.status() {
                Ok(status) => {
                    if !status.success() {
                        return Err(ShellError::ExecutionError(format!(
                            "Command exited with status: {}",
                            status.code().unwrap_or(-1)
                        )));
                    }
                    return Ok(());
                }
                Err(e) => {
                    return Err(ShellError::ExecutionError(format!(
                        "Failed to execute {}: {}",
                        full_path, e
                    )));
                }
            }
        }
    }

    Err(ShellError::CommandNotFound(format!(
        "{}: command not found",
        command_name
    )))
}
