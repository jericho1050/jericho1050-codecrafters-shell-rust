use crate::commands::{handle_command_with_clap, run_external_command};
use crate::completion::ShellCompleter;
use crate::errors::{ShellError, ShellResult};
use crate::pipeline::{execute_pipeline, is_pipeline, split_pipeline};
use crate::redirection::{parse_redirection, setup_builtin_redirection};
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::{ColorMode, Config, Editor};

/// Read input from the user using rustyline
pub fn read_input() -> ShellResult<String> {
    let config = Config::builder()
        .color_mode(ColorMode::Enabled)
        .auto_add_history(true)
        .build();

    let mut rl = Editor::with_config(config).map_err(|e| {
        ShellError::InputError(format!("Failed to create readline editor: {}", e))
    })?;

    rl.set_helper(Some(ShellCompleter));
    rl.set_completion_type(rustyline::CompletionType::List);

    loop {
        match rl.readline("$ ") {
            Ok(line) => {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    return Ok(trimmed.to_string());
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C pressed, continue loop
                continue;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D pressed, exit gracefully
                return Err(ShellError::Interrupted);
            }
            Err(e) => {
                return Err(ShellError::InputError(format!("Readline error: {}", e)));
            }
        }
    }
}

/// Handle a command input line
pub fn handle_command_input(input: &str) -> ShellResult<()> {
    // Check for pipeline
    if is_pipeline(input) {
        let stages: Vec<&str> = split_pipeline(input);
        let parsed_stages: Result<Vec<Vec<String>>, _> = stages
            .into_iter()
            .map(|stage| shell_words::split(stage).map_err(|_| ShellError::InvalidQuoting))
            .collect();
        
        return execute_pipeline(parsed_stages?);
    }

    // Parse the command using shell-words
    let args = shell_words::split(input)
        .map_err(|_| ShellError::InvalidQuoting)?;

    if args.is_empty() {
        return Ok(());
    }

    // Parse redirections first (for both builtins and external commands)
    let (filtered_args, stdout_redir, stderr_redir) = parse_redirection(&args)?;

    if filtered_args.is_empty() {
        return Ok(());
    }

    // Set up redirection for builtins (will be restored when guard is dropped)
    let _guard = setup_builtin_redirection(&stdout_redir, &stderr_redir)?;

    // Check if this is a builtin command by trying to parse with clap
    match handle_command_with_clap(&filtered_args) {
        Ok(_) => Ok(()),
        Err(_) => {
            // Not a builtin, try to run as external command (with original args for redirection)
            run_external_command(&args)
        }
    }
}