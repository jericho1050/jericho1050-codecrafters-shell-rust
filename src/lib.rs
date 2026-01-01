pub mod commands;
pub mod completion;
pub mod errors;
pub mod history;
pub mod pipeline;
pub mod redirection;
pub mod repl;

use errors::{ShellError, ShellResult};
use repl::{handle_command_input, read_input};

/// Main entry point for the shell REPL
pub fn run_shell() -> ShellResult<()> {
    loop {
        match read_input() {
            Ok(input) => {
                // Add command to history before executing
                history::add_to_history(&input);

                if let Err(e) = handle_command_input(&input) {
                    match e {
                        ShellError::CommandNotFound(msg) => eprintln!("{}", msg),
                        ShellError::InvalidDirectory(msg) => eprintln!("{}", msg),
                        ShellError::ExecutionError(msg) => eprintln!("{}", msg),
                        ShellError::RedirectionError(msg) => eprintln!("{}", msg),
                        _ => eprintln!("Error: {}", e),
                    }
                }
            }
            Err(ShellError::Interrupted) => {
                // Ctrl-D pressed, exit gracefully
                break;
            }
            Err(e) => {
                eprintln!("Input error: {}", e);
            }
        }
    }
    Ok(())
}
