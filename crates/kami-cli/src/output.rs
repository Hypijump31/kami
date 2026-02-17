//! Output formatting for CLI responses.

/// Prints a success message.
pub fn print_success(message: &str) {
    println!("[OK] {message}");
}

/// Prints an error message.
pub fn print_error(message: &str) {
    eprintln!("[ERROR] {message}");
}
