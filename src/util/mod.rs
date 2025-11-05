use std::io::{self, Write};

/// Helpful when diagnosing an infinite loop, or just want to step through
/// a certain block of code.
pub fn wait_for_input() {
    print!("Press Enter to continue...");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}
