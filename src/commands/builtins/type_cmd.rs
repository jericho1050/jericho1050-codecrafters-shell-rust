use crate::commands::registry::{BuiltinCommand, BUILTINS};
use crate::errors::ShellResult;
use std::env;
use std::path::Path;

pub struct TypeCommand;

impl BuiltinCommand for TypeCommand {
    fn name(&self) -> &'static str {
        "type"
    }

    fn description(&self) -> &'static str {
        "Print the type of a command"
    }

    fn execute(&self, args: &[String]) -> ShellResult<()> {
        let name = args.get(1).map(|s| s.as_str()).unwrap_or("");

        // Check builtins using the registry
        if BUILTINS.is_builtin(name) {
            println!("{} is a shell builtin", name);
            return Ok(());
        }

        // Check PATH
        if let Ok(path) = env::var("PATH") {
            for dir in path.split(':') {
                let full_path = format!("{}/{}", dir, name);
                let path = Path::new(&full_path);
                if path.exists() {
                    if let Ok(metadata) = path.metadata() {
                        use std::os::unix::fs::PermissionsExt;
                        let is_executable = metadata.permissions().mode() & 0o111 != 0;
                        if metadata.is_file() && is_executable {
                            println!("{} is {}", name, full_path);
                            return Ok(());
                        }
                    }
                }
            }
        }

        println!("{}: not found", name);
        Ok(())
    }
}
