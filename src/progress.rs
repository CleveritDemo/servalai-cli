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

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

const FRAMES: [&str; 8] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];

/// An animated "still working" indicator for network waits. Renders on
/// stderr only — stdout stays reserved for actual command output.
///
/// Fully inert when the terminal isn't interactive (see `is_interactive`):
/// `start` prints the message once as a static line instead of animating,
/// and `finish_note` still prints its note — same information, no ANSI.
pub struct Spinner {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    colored: bool,
}

impl Spinner {
    /// Starts the spinner using live terminal/env detection.
    pub fn start(message: &str) -> Spinner {
        Spinner::start_with(message, is_interactive())
    }

    fn start_with(message: &str, animated: bool) -> Spinner {
        if !animated {
            eprintln!("{message}");
            return Spinner {
                stop: Arc::new(AtomicBool::new(false)),
                handle: None,
                colored: false,
            };
        }
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = Arc::clone(&stop);
        let msg = message.to_string();
        let handle = std::thread::spawn(move || {
            let mut i = 0usize;
            while !stop_clone.load(Ordering::Relaxed) {
                eprint!("\r{ACCENT}{}{RESET} {msg}", FRAMES[i % FRAMES.len()]);
                let _ = std::io::stderr().flush();
                i += 1;
                std::thread::sleep(Duration::from_millis(80));
            }
        });
        Spinner {
            stop,
            handle: Some(handle),
            colored: true,
        }
    }

    fn stop_thread(&mut self) {
        if let Some(handle) = self.handle.take() {
            self.stop.store(true, Ordering::Relaxed);
            let _ = handle.join();
            eprint!("\x1b[2K\r");
            let _ = std::io::stderr().flush();
        }
    }

    /// Stops the spinner and clears its line without printing anything
    /// further. Used on success, right before opencode's own output (or
    /// `exec`) takes over the terminal.
    pub fn finish_silent(mut self) {
        self.stop_thread();
    }

    /// Stops the spinner and prints a muted fallback note in its place.
    pub fn finish_note(mut self, message: &str) {
        self.stop_thread();
        eprintln!("{}", note_line(message, self.colored));
    }
}

impl Drop for Spinner {
    /// Safety net for early returns via `?` between `start` and a `finish_*`
    /// call: still stops the thread and clears the line so a fatal error
    /// printed right after doesn't land on top of a stuck spinner frame.
    fn drop(&mut self) {
        self.stop_thread();
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

    #[test]
    fn non_animated_start_and_finish_silent_does_not_panic() {
        let spinner = Spinner::start_with("test message", false);
        spinner.finish_silent();
    }

    #[test]
    fn non_animated_start_and_finish_note_does_not_panic() {
        let spinner = Spinner::start_with("test message", false);
        spinner.finish_note("fallback happened");
    }

    #[test]
    fn animated_start_and_finish_silent_stops_the_thread_cleanly() {
        let spinner = Spinner::start_with("test message", true);
        std::thread::sleep(std::time::Duration::from_millis(50));
        spinner.finish_silent();
    }

    #[test]
    fn animated_start_and_finish_note_stops_the_thread_cleanly() {
        let spinner = Spinner::start_with("test message", true);
        std::thread::sleep(std::time::Duration::from_millis(50));
        spinner.finish_note("using cached config");
    }

    #[test]
    fn dropping_an_unfinished_spinner_stops_the_thread() {
        let spinner = Spinner::start_with("test message", true);
        std::thread::sleep(std::time::Duration::from_millis(50));
        drop(spinner);
    }
}
