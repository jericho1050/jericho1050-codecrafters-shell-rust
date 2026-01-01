use crate::commands::registry::BuiltinCommand;
use crate::errors::{ShellError, ShellResult};
use std::env;

pub struct CdCommand;

impl BuiltinCommand for CdCommand {
    fn name(&self) -> &'static str {
        "cd"
    }

    fn description(&self) -> &'static str {
        "Change current working directory"
    }

    fn execute(&self, args: &[String]) -> ShellResult<()> {
        let path = args.get(1).map(|s| s.as_str());

        let target = match path {
            Some(p) if p.starts_with('~') => {
                // Expand ~ to home directory
                match env::var("HOME") {
                    Ok(home) => p.replacen('~', &home, 1),
                    Err(_) => {
                        return Err(ShellError::InvalidDirectory(
                            "HOME environment variable not set".to_string(),
                        ));
                    }
                }
            }
            Some(p) => p.to_string(),
            None => {
                // No argument provided - go to HOME directory
                match env::var("HOME") {
                    Ok(home) => home,
                    Err(_) => {
                        return Err(ShellError::InvalidDirectory(
                            "HOME environment variable not set".to_string(),
                        ));
                    }
                }
            }
        };

        env::set_current_dir(&target).map_err(|_| {
            ShellError::InvalidDirectory(format!("cd: {}: No such file or directory", target))
        })
    }
}
