pub mod builtins;
pub mod external;
pub mod registry;

pub use external::run_external_command;
pub use registry::BUILTINS;

use crate::errors::ShellResult;

/// Execute a command (checks builtins first, then external)
pub fn handle_command(args: &[String]) -> ShellResult<()> {
    if args.is_empty() {
        return Ok(());
    }

    let cmd_name = &args[0];

    // Check for exit command first
    if let Some(code) = BUILTINS.check_exit(cmd_name, args) {
        std::process::exit(code);
    }

    // Try builtin
    if let Some(result) = BUILTINS.execute(cmd_name, args) {
        return result;
    }

    // Fall back to external command
    run_external_command(args)
}
