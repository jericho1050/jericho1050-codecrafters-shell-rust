use crate::commands::registry::BuiltinCommand;
use crate::errors::ShellResult;

pub struct ExitCommand;

impl BuiltinCommand for ExitCommand {
    fn name(&self) -> &'static str {
        "exit"
    }

    fn description(&self) -> &'static str {
        "Exit the shell"
    }

    fn execute(&self, _args: &[String]) -> ShellResult<()> {
        // Exit is handled via exit_code(), this won't normally be called
        Ok(())
    }

    fn exit_code(&self, args: &[String]) -> Option<i32> {
        let code = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        Some(code)
    }
}
