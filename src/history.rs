use once_cell::sync::Lazy;
use std::sync::Mutex;

/// Shared command history state
pub static HISTORY: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Add a command to history
pub fn add_to_history(command: &str) {
    if let Ok(mut history) = HISTORY.lock() {
        history.push(command.to_string());
    }
}

/// Load history from a file (replaces current history)
pub fn load_history_from_file(path: &str) -> std::io::Result<()> {
    let content = std::fs::read_to_string(path)?;
    if let Ok(mut history) = HISTORY.lock() {
        history.clear();
        for line in content.lines() {
            if !line.is_empty() {
                history.push(line.to_string());
            }
        }
    }
    Ok(())
}

/// Get all history entries
pub fn get_history() -> Vec<String> {
    HISTORY.lock().map(|h| h.clone()).unwrap_or_default()
}
