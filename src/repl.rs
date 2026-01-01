use crate::commands::{handle_command_with_clap, run_external_command};
use crate::completion::ShellCompleter;
use crate::errors::{ShellError, ShellResult};
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
    // Parse the command using shell-words
    let args = shell_words::split(input)
        .map_err(|_| ShellError::InvalidQuoting)?;

    if args.is_empty() {
        return Ok(());
    }

    // Check if this is a builtin command by trying to parse with clap
    match handle_command_with_clap(&args) {
        Ok(_) => Ok(()),
        Err(_) => {
            // Not a builtin, try to run as external command
            run_external_command(&args)
        }
    }
}
