use crate::commands::registry::BuiltinCommand;
use crate::errors::{ShellError, ShellResult};
use std::env;

pub struct PwdCommand;

impl BuiltinCommand for PwdCommand {
    fn name(&self) -> &'static str {
        "pwd"
    }

    fn description(&self) -> &'static str {
        "Print current working directory"
    }

    fn execute(&self, _args: &[String]) -> ShellResult<()> {
        let current_dir = env::current_dir().map_err(ShellError::IoError)?;
        println!("{}", current_dir.display());
        Ok(())
    }
}
