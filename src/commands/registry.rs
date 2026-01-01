use crate::errors::ShellResult;
use once_cell::sync::Lazy;

/// Trait that all builtin commands must implement
pub trait BuiltinCommand: Send + Sync {
    /// The command name (e.g., "echo", "cd", "pwd")
    fn name(&self) -> &'static str;

    /// Help text / description for the command
    fn description(&self) -> &'static str;

    /// Execute the command with the given arguments
    /// args[0] is the command name itself
    fn execute(&self, args: &[String]) -> ShellResult<()>;

    /// Whether this command should cause the shell to exit
    /// Returns Some(exit_code) if shell should exit, None otherwise
    fn exit_code(&self, _args: &[String]) -> Option<i32> {
        None
    }
}

/// Central registry for all builtin commands
pub struct BuiltinRegistry {
    commands: Vec<Box<dyn BuiltinCommand>>,
}

impl BuiltinRegistry {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn register<C: BuiltinCommand + 'static>(&mut self, cmd: C) {
        self.commands.push(Box::new(cmd));
    }

    /// Check if a command name is a builtin
    pub fn is_builtin(&self, name: &str) -> bool {
        self.commands.iter().any(|c| c.name() == name)
    }

    /// Get all builtin command names (for completion and type command)
    pub fn builtin_names(&self) -> Vec<&'static str> {
        self.commands.iter().map(|c| c.name()).collect()
    }

    /// Execute a builtin command by name
    pub fn execute(&self, name: &str, args: &[String]) -> Option<ShellResult<()>> {
        self.commands
            .iter()
            .find(|c| c.name() == name)
            .map(|c| c.execute(args))
    }

    /// Check if command should exit the shell
    pub fn check_exit(&self, name: &str, args: &[String]) -> Option<i32> {
        self.commands
            .iter()
            .find(|c| c.name() == name)
            .and_then(|c| c.exit_code(args))
    }
}

/// Global registry instance
pub static BUILTINS: Lazy<BuiltinRegistry> = Lazy::new(|| {
    let mut registry = BuiltinRegistry::new();

    // Register all builtins here - SINGLE POINT OF REGISTRATION
    registry.register(super::builtins::EchoCommand);
    registry.register(super::builtins::PwdCommand);
    registry.register(super::builtins::CdCommand);
    registry.register(super::builtins::TypeCommand);
    registry.register(super::builtins::ExitCommand);
    registry.register(super::builtins::HistoryCommand);

    registry
});
