//! Output formatting for CLI responses.

/// Prints a success message.
pub fn print_success(message: &str) {
    println!("[OK] {message}");
}

/// Prints an error message.
pub fn print_error(message: &str) {
    eprintln!("[ERROR] {message}");
}

/// Default database path for the tool registry.
pub fn default_db_path() -> String {
    let home = std::env::var("KAMI_DATA_DIR")
        .unwrap_or_else(|_| ".kami".to_string());
    format!("{home}/registry.db")
}
