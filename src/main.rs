mod client;
mod commands;
mod config;
mod constants;
mod launch;
mod paths;
mod update;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "serval",
    version,
    about = "ServalAI — Cleverit's company-funded model gateway CLI",
    long_about = "One install, one token, and you're coding with a fully-configured opencode\nsession backed by ServalAI.  No config files, no env vars — everything is\ninjected at launch without touching your personal opencode setup.\n\nGet your token at https://cleverit-support.cleveritgroup.com, then run:\n\n    serval auth\n    serval",
    after_help = "Run `serval status` to see your current configuration.",
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Extra arguments forwarded to opencode when no subcommand is given.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Store your ServalAI token and validate it against the gateway.
    #[command(
        long_about = "Store your ServalAI token and validate it against the gateway.\n\nIf --token is omitted, you'll be prompted interactively.\nGet your token at https://cleverit-support.cleveritgroup.com"
    )]
    Auth {
        /// Token string (if not provided you'll be prompted).
        #[arg(long)]
        token: Option<String>,
    },
    /// Refresh your provider config from the ServalAI gateway.
    Sync,
    /// Show CLI version, bundled opencode, gateway URL, and identity.
    Status,
    /// Clear your stored token from this machine.
    Logout,
    /// Download and install the latest serval release.
    Update,
    /// Launch your preconfigured opencode session (this is the default action).
    #[command(
        long_about = "Launch opencode preconfigured with ServalAI as the provider.\n\nRunning `serval` without a subcommand does the same thing.\nAll arguments after `--` are forwarded to opencode.\n\nExamples:\n  serval code\n  serval code -- --print-logs\n  serval"
    )]
    Code {
        /// Extra arguments forwarded to opencode.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Some(Command::Auth { token }) => commands::auth(token),
        Some(Command::Sync) => commands::sync(),
        Some(Command::Status) => commands::status(),
        Some(Command::Logout) => commands::logout(),
        Some(Command::Update) => commands::update_cmd(),
        Some(Command::Code { args }) => commands::code(args),
        None => commands::code(cli.args),
    };
    if let Err(e) = result {
        let hint = if e.contains("haven't authenticated") || e.contains("no token") {
            "\nHint: get your token at https://cleverit-support.cleveritgroup.com, then run `serval auth`.\n"
        } else {
            "\n"
        };
        eprintln!("serval: {e}{hint}");
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
        assert!(cli.args.is_empty());
    }

    #[test]
    fn code_subcommand_captures_trailing_args() {
        let cli = Cli::try_parse_from(["serval", "code", "--", "--foo", "bar"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Code { args }) if args == vec!["--foo".to_string(), "bar".to_string()]
        ));
    }

    #[test]
    fn version_flag_still_parses() {
        let err = Cli::try_parse_from(["serval", "--version"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayVersion);
    }

    #[test]
    fn help_flag_still_parses() {
        let err = Cli::try_parse_from(["serval", "--help"]).unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn bare_serval_passthrough_captures_trailing_args() {
        let cli = Cli::try_parse_from(["serval", "--", "--foo", "bar"]).unwrap();
        assert!(cli.command.is_none());
        assert_eq!(cli.args, vec!["--foo".to_string(), "bar".to_string()]);
    }

    #[test]
    fn known_subcommands_still_route_correctly_alongside_top_level_args() {
        let cli = Cli::try_parse_from(["serval", "status"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Status)));
        assert!(cli.args.is_empty());
    }
}
