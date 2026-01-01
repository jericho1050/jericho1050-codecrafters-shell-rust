use crate::commands::registry::BuiltinCommand;
use crate::errors::{ShellError, ShellResult};
use crate::history;

pub struct HistoryCommand;

impl BuiltinCommand for HistoryCommand {
    fn name(&self) -> &'static str {
        "history"
    }

    fn description(&self) -> &'static str {
        "Display or read command history"
    }

    fn execute(&self, args: &[String]) -> ShellResult<()> {
        if args.len() >= 3 && args[1] == "-r" {
            // Load history from file (silently)
            let file_path = &args[2];
            history::load_history_from_file(file_path).map_err(ShellError::IoError)?;
        } else {
            // Display history
            let entries = history::get_history();
            for (i, entry) in entries.iter().enumerate() {
                println!("{:>5}  {}", i + 1, entry);
            }
        }

        Ok(())
    }
}
