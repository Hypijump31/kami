//! Output formatting for CLI responses.

/// Prints a success message.
pub fn print_success(message: &str) {
    println!("[OK] {message}");
}

/// Prints a warning message.
pub fn print_warning(message: &str) {
    println!("[WARN] {message}");
}

/// Prints an error message.
pub fn print_error(message: &str) {
    eprintln!("[ERROR] {message}");
}

/// Prints an informational message.
pub fn print_info(message: &str) {
    println!("[INFO] {message}");
}

/// Default database path for the tool registry.
pub fn default_db_path() -> String {
    let home = std::env::var("KAMI_DATA_DIR").unwrap_or_else(|_| ".kami".to_string());
    format!("{home}/registry.db")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_success_does_not_panic() {
        print_success("ok");
    }

    #[test]
    fn print_warning_does_not_panic() {
        print_warning("warn");
    }

    #[test]
    fn print_error_does_not_panic() {
        print_error("err");
    }

    #[test]
    fn print_info_does_not_panic() {
        print_info("inf");
    }

    #[test]
    fn default_db_path_contains_registry_db() {
        let path = default_db_path();
        assert!(path.ends_with("registry.db"));
    }
}
