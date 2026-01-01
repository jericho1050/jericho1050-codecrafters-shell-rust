use crate::commands::registry::BuiltinCommand;
use crate::errors::ShellResult;

pub struct EchoCommand;

impl BuiltinCommand for EchoCommand {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn description(&self) -> &'static str {
        "Echo arguments to stdout"
    }

    fn execute(&self, args: &[String]) -> ShellResult<()> {
        // args[0] is "echo", actual args start at [1]
        println!("{}", args[1..].join(" "));
        Ok(())
    }
}
