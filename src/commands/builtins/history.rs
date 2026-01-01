use crate::commands::registry::BuiltinCommand;
use crate::errors::{ShellError, ShellResult};
use std::fs;

pub struct HistoryCommand;

impl BuiltinCommand for HistoryCommand {
    fn name(&self) -> &'static str {
        "history"
    }

    fn description(&self) -> &'static str {
        "Display or read command history"
    }

    fn execute(&self, args: &[String]) -> ShellResult<()> {
        // Check for -r flag to read history from file
        if args.len() >= 3 && args[1] == "-r" {
            let file_path = &args[2];
            let content = fs::read_to_string(file_path).map_err(ShellError::IoError)?;

            for (i, line) in content.lines().enumerate() {
                println!("{:4} {}", i + 1, line);
            }
        }

        Ok(())
    }
}
