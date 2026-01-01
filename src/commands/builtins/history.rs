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
        // -r flag reads history from file silently (no output)
        // The actual loading into rustyline requires REPL access
        if args.len() >= 3 && args[1] == "-r" {
            // Silently read - don't print anything
            // TODO: Actually load into rustyline history
            let _file_path = &args[2];
        }

        Ok(())
    }
}
