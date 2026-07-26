mod client;
mod commands;
mod config;
mod constants;
mod launch;
mod paths;
mod update;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "serval", version, about = "ServalAI CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Store your ServalAI token.
    Auth {
        #[arg(long)]
        token: Option<String>,
    },
    /// Refresh provider config from the Worker.
    Sync,
    /// Show version, pinned opencode, and resolved identity.
    Status,
    /// Clear the stored token.
    Logout,
    /// Self-update to the latest release.
    Update,
    /// Launch opencode preconfigured (default action).
    Code,
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Some(Command::Auth { token }) => commands::auth(token),
        Some(Command::Sync) => commands::sync(),
        Some(Command::Status) => commands::status(),
        Some(Command::Logout) => commands::logout(),
        Some(Command::Update) => commands::update_cmd(),
        Some(Command::Code) | None => commands::code(vec![]),
    };
    if let Err(e) = result {
        eprintln!("serval: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parses_auth_with_token_flag() {
        let cli = Cli::try_parse_from(["serval", "auth", "--token", "aig_x"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Auth { token: Some(_) })
        ));
    }

    #[test]
    fn no_subcommand_is_none() {
        let cli = Cli::try_parse_from(["serval"]).unwrap();
        assert!(cli.command.is_none());
    }
}
