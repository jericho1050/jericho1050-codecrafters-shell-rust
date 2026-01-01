use rustyline::completion::{Completer, Pair};
use rustyline::hint::Hinter;
use rustyline::highlight::{Highlighter, CmdKind};
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use std::env;
use std::fs;

/// Shell completer for tab completion of commands
#[derive(Clone)]
pub struct ShellCompleter;

impl Completer for ShellCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let input = &line[..pos];
        let parts: Vec<&str> = input.split_whitespace().collect();

        // Only complete the first word (command name)
        if parts.len() <= 1 && !input.ends_with(' ') {
            let prefix = parts.first().map(|s| *s).unwrap_or("");
            let mut candidates = Vec::new();

            // Add builtin commands
            let builtins = ["echo", "type", "exit", "pwd", "cd", "cat"];
            for builtin in builtins {
                if builtin.starts_with(prefix) {
                    candidates.push(Pair {
                        display: builtin.to_string(),
                        replacement: builtin.to_string(),
                    });
                }
            }

            // Add executables from PATH
            if let Ok(path_var) = env::var("PATH") {
                for path_dir in path_var.split(':') {
                    if let Ok(entries) = fs::read_dir(path_dir) {
                        for entry in entries.flatten() {
                            if let Ok(file_name) = entry.file_name().into_string() {
                                if file_name.starts_with(prefix) {
                                    // Check if executable
                                    if let Ok(metadata) = entry.metadata() {
                                        #[cfg(unix)]
                                        {
                                            use std::os::unix::fs::PermissionsExt;
                                            let is_executable = metadata.permissions().mode() & 0o111 != 0;
                                            if metadata.is_file() && is_executable {
                                                candidates.push(Pair {
                                                    display: file_name.clone(),
                                                    replacement: file_name,
                                                });
                                            }
                                        }
                                        #[cfg(not(unix))]
                                        {
                                            if metadata.is_file() {
                                                candidates.push(Pair {
                                                    display: file_name.clone(),
                                                    replacement: file_name,
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Remove duplicates and sort
            candidates.sort_by(|a, b| a.display.cmp(&b.display));
            candidates.dedup_by(|a, b| a.display == b.display);

            // If there's exactly one match, add a trailing space
            if candidates.len() == 1 {
                candidates[0].replacement.push(' ');
            }

            Ok((pos - prefix.len(), candidates))
        } else {
            Ok((pos, vec![]))
        }
    }
}

impl Hinter for ShellCompleter {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for ShellCompleter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> std::borrow::Cow<'l, str> {
        std::borrow::Cow::Borrowed(line)
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> std::borrow::Cow<'b, str> {
        std::borrow::Cow::Borrowed(prompt)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> std::borrow::Cow<'h, str> {
        std::borrow::Cow::Borrowed(hint)
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: CmdKind) -> bool {
        false
    }
}

impl Validator for ShellCompleter {}

impl Helper for ShellCompleter {}
