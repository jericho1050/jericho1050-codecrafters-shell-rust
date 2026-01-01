use clap::{Parser, Subcommand};

/// Shell command line arguments
#[derive(Parser, Debug)]
#[command(name = "")]
#[command(about = "A simple shell", long_about = None)]
pub struct ShellArgs {
    #[command(subcommand)]
    pub command: ShellCommand,
}

/// Shell commands
#[derive(Subcommand, Debug)]
pub enum ShellCommand {
    /// Echo arguments to stdout
    Echo {
        /// Arguments to echo
        args: Vec<String>,
    },
    /// Print the current working directory
    Pwd,
    /// Exit the shell
    Exit {
        /// Exit code
        #[arg(default_value = "0")]
        code: i32,
    },
    /// Print the type of a command
    Type {
        /// Command name to check
        name: String,
    },
    /// Change the current directory
    Cd {
        /// Directory to change to
        path: Option<String>,
    },
    /// Concatenate and print files
    Cat {
        /// Files to concatenate
        files: Vec<String>,
    },
}
