use crate::errors::{ShellError, ShellResult};
use crate::redirection::parse_redirection;
use crate::commands::BUILTINS;
use std::process::{Command, Stdio, Child};
use std::env;
use std::path::Path;

/// Check if a command is a builtin
fn is_builtin(name: &str) -> bool {
    BUILTINS.is_builtin(name)
}

/// Split input into pipeline stages
pub fn split_pipeline(input: &str) -> Vec<&str> {
    input.split('|').map(|s| s.trim()).collect()
}

/// Check if input contains a pipeline
pub fn is_pipeline(input: &str) -> bool {
    input.contains('|')
}

/// Execute a pipeline of commands
pub fn execute_pipeline(stages: Vec<Vec<String>>) -> ShellResult<()> {
    if stages.is_empty() {
        return Ok(());
    }

    if stages.len() == 1 {
        // Single command, no pipeline needed
        return crate::commands::run_external_command(&stages[0]);
    }

    let mut previous_stdout: Option<std::process::ChildStdout> = None;
    let mut children: Vec<Child> = Vec::new();

    for (i, args) in stages.iter().enumerate() {
        if args.is_empty() {
            continue;
        }

        let is_last = i == stages.len() - 1;
        let command_name = &args[0];

        // Parse any redirections (only meaningful for last command's stdout/stderr)
        let (filtered_args, stdout_redir, stderr_redir) = parse_redirection(args)?;

        // Check if this is a builtin command AND it's the last stage
        // (only handle builtins specially at the end; in the middle, use external versions for piping)
        if is_last && is_builtin(command_name) {
            // Wait for all previous children first
            for mut child in children.drain(..) {
                let _ = child.wait();
            }

            // Drain previous stdout (builtins like type/pwd don't read stdin)
            if let Some(mut prev_stdout) = previous_stdout.take() {
                use std::io::Read;
                let mut buf = Vec::new();
                let _ = prev_stdout.read_to_end(&mut buf);
            }

            // Check for exit command
            if let Some(code) = BUILTINS.check_exit(command_name, &filtered_args) {
                std::process::exit(code);
            }

            // Execute builtin using registry
            if let Some(result) = BUILTINS.execute(command_name, &filtered_args) {
                result?;
            }

            // Builtins don't produce piped output, so clear previous_stdout
            previous_stdout = None;
            continue;
        }

        // Find the command path
        let cmd_path = find_command_path(command_name)?;

        let mut cmd = Command::new(&cmd_path);
        cmd.args(&filtered_args[1..]);

        // Set up stdin from previous command's stdout
        if let Some(prev_stdout) = previous_stdout.take() {
            cmd.stdin(Stdio::from(prev_stdout));
        }

        // Set up stdout
        if is_last {
            // Last command: output to stdout or redirect to file
            if let Some(redir) = &stdout_redir {
                let file = std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(matches!(redir.mode, crate::redirection::RedirectionMode::Overwrite))
                    .append(matches!(redir.mode, crate::redirection::RedirectionMode::Append))
                    .open(&redir.file)
                    .map_err(|e| ShellError::RedirectionError(format!("Failed to open '{}': {}", redir.file, e)))?;
                cmd.stdout(Stdio::from(file));
            } else {
                cmd.stdout(Stdio::inherit());
            }
        } else {
            // Not last: pipe stdout to next command
            cmd.stdout(Stdio::piped());
        }

        // Set up stderr (redirect if specified, otherwise inherit)
        if let Some(redir) = &stderr_redir {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(matches!(redir.mode, crate::redirection::RedirectionMode::Overwrite))
                .append(matches!(redir.mode, crate::redirection::RedirectionMode::Append))
                .open(&redir.file)
                .map_err(|e| ShellError::RedirectionError(format!("Failed to open '{}': {}", redir.file, e)))?;
            cmd.stderr(Stdio::from(file));
        } else {
            cmd.stderr(Stdio::inherit());
        }

        let mut child = cmd.spawn().map_err(|e| {
            ShellError::ExecutionError(format!("Failed to execute {}: {}", command_name, e))
        })?;

        // Capture stdout for next command in pipeline
        if !is_last {
            previous_stdout = child.stdout.take();
        }

        children.push(child);
    }

    // Wait for all children to complete
    for mut child in children {
        let _ = child.wait();
    }

    Ok(())
}

/// Find command path (check if it's a path or search in PATH)
fn find_command_path(command_name: &str) -> ShellResult<String> {
    // If command contains '/', use it directly
    if command_name.contains('/') {
        if Path::new(command_name).exists() {
            return Ok(command_name.to_string());
        }
        return Err(ShellError::CommandNotFound(format!("{}: command not found", command_name)));
    }

    // Try to find in PATH
    if let Ok(path_var) = env::var("PATH") {
        for path_dir in path_var.split(':') {
            let full_path = format!("{}/{}", path_dir, command_name);
            if Path::new(&full_path).exists() {
                return Ok(full_path);
            }
        }
    }

    // Try as-is (might work for commands in current dir or system knows about)
    Ok(command_name.to_string())
}