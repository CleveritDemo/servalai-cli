//! Terminal progress feedback: an animated spinner for network waits and the
//! ServalAI/Cleverit/Raven text wordmark shown on `serval` launch.
//!
//! Fully inert (no ANSI, no background thread) when stdout/stderr aren't a
//! TTY or `NO_COLOR` is set — critical for CI logs and piped output.
#![allow(dead_code)]

const ACCENT: &str = "\x1b[36m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

/// Pure decision: should we render an animated, colored spinner? False when
/// either stream isn't a TTY, or `NO_COLOR` is set to a non-empty value.
pub fn should_animate(stdout_is_tty: bool, stderr_is_tty: bool, no_color_env: bool) -> bool {
    stdout_is_tty && stderr_is_tty && !no_color_env
}

/// Reads real terminal/env state. Thin IO wrapper — `should_animate` carries
/// the tested decision logic; this just supplies live inputs to it.
pub fn is_interactive() -> bool {
    use std::io::IsTerminal;
    let no_color = std::env::var("NO_COLOR")
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    should_animate(
        std::io::stdout().is_terminal(),
        std::io::stderr().is_terminal(),
        no_color,
    )
}

/// The ServalAI/Cleverit/Raven wordmark, shown once on plain `serval` launch.
pub fn banner(colored: bool) -> String {
    if colored {
        format!("{ACCENT}ServalAI{RESET}\n{DIM}a Cleverit company · powered by Raven{RESET}")
    } else {
        "ServalAI\na Cleverit company · powered by Raven".to_string()
    }
}

/// Formats the muted fallback note — visually distinct from the "serval:
/// <fatal error>" format used by `main.rs::run()` on real failures.
pub fn note_line(message: &str, colored: bool) -> String {
    if colored {
        format!("{DIM}○ {message}{RESET}")
    } else {
        format!("○ {message}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animates_when_both_streams_are_ttys_and_color_allowed() {
        assert!(should_animate(true, true, false));
    }

    #[test]
    fn does_not_animate_when_stdout_is_not_a_tty() {
        assert!(!should_animate(false, true, false));
    }

    #[test]
    fn does_not_animate_when_stderr_is_not_a_tty() {
        assert!(!should_animate(true, false, false));
    }

    #[test]
    fn does_not_animate_when_no_color_env_is_set() {
        assert!(!should_animate(true, true, true));
    }

    #[test]
    fn banner_colored_wraps_lines_in_ansi_codes() {
        let b = banner(true);
        assert!(b.contains("ServalAI"));
        assert!(b.contains("a Cleverit company"));
        assert!(b.contains("powered by Raven"));
        assert!(b.contains("\x1b["));
    }

    #[test]
    fn banner_plain_has_no_ansi_codes() {
        let b = banner(false);
        assert_eq!(b, "ServalAI\na Cleverit company · powered by Raven");
        assert!(!b.contains("\x1b["));
    }

    #[test]
    fn note_line_plain_has_neutral_marker_and_no_ansi() {
        let n = note_line("using cached config", false);
        assert_eq!(n, "○ using cached config");
        assert!(!n.contains("\x1b["));
    }

    #[test]
    fn note_line_colored_still_contains_message_and_ansi() {
        let n = note_line("using cached config", true);
        assert!(n.contains("using cached config"));
        assert!(n.contains("\x1b["));
    }
}
