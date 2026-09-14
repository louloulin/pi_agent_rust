//! Shared startup banner constants for Pi frontends.
//!
//! Kept independent from terminal/TUI implementations so CLI and interactive
//! frontends present the same product identity without depending on the
//! coding-agent crate.

/// Title used by first-run setup and other startup surfaces.
pub const WELCOME_TITLE: &str = "Welcome to Pi";

/// Greeting used by the interactive empty-session surface.
pub const WELCOME_GREETING: &str = "Welcome to Pi!";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn banner_identity_is_stable() {
        assert_eq!(WELCOME_TITLE, "Welcome to Pi");
        assert_eq!(WELCOME_GREETING, "Welcome to Pi!");
    }
}
