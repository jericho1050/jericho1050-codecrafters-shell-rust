use codecrafters_shell::run_shell;

fn main() {
    if let Err(e) = run_shell() {
        eprintln!("Shell error: {}", e);
        std::process::exit(1);
    }
}
