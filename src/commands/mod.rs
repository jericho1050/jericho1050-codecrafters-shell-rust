pub mod builtin;
pub mod cli;
pub mod external;

use crate::errors::ShellResult;
use clap::Parser;

pub use builtin::{
    handle_cat_command, handle_cd_command, handle_echo_command, handle_pwd_command,
    handle_type_command, handle_tail_command, handle_head_command, handle_wc_command,
};
pub use cli::{ShellArgs, ShellCommand};
pub use external::run_external_command;

/// Parse and execute a command using clap
pub fn handle_command_with_clap(args: &[String]) -> ShellResult<()> {
    // Prepend a dummy program name since clap expects arg[0] to be the program name
    let mut clap_args = vec!["shell".to_string()];
    clap_args.extend(args.iter().cloned());

    let shell_args = ShellArgs::try_parse_from(&clap_args).map_err(|e| {
        crate::errors::ShellError::InputError(format!("Failed to parse command: {}", e))
    })?;

    match shell_args.command {
        ShellCommand::Echo { args } => handle_echo_command(&args),
        ShellCommand::Pwd => handle_pwd_command(),
        ShellCommand::Exit { code } => {
            std::process::exit(code);
        }
        ShellCommand::Type { name } => handle_type_command(&name),
        ShellCommand::Cd { path } => handle_cd_command(path.as_deref()),
        ShellCommand::Cat { files } => handle_cat_command(&files),
        ShellCommand::Tail { files, lines } => handle_tail_command(&files, lines),
        ShellCommand::Head { files, lines } => handle_head_command(&files, lines),
        ShellCommand::Wc { files } => handle_wc_command(&files),

    }
}
