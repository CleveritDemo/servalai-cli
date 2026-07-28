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
    after_help = "Additional commands: ping, models, usage, doctor, init, report.\nRun `serval status` to see your current configuration.",
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
        long_about = "Store your ServalAI token and validate it against the gateway.\n\nIf --token is omitted, you'll be prompted interactively.\nYour token is stored in your OS keychain (not a plain file).\nGet your token at https://cleverit-support.cleveritgroup.com"
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
    /// Ping the gateway and show available models.
    Ping,
    /// List available models from the gateway.
    Models,
    /// Show token usage and session statistics.
    Usage,
    /// Run diagnostics on your serval installation.
    #[command(
        long_about = "Run diagnostics on your serval installation.\n\nChecks: config file permissions, token validity,\ngateway reachability, bundled binary health,\nand bundle directory integrity."
    )]
    Doctor,
    /// Initialize a .serval.jsonc project config in the current directory.
    Init,
    /// Generate a summary report of recent activity.
    Report,
    /// Launch your preconfigured opencode session (this is the default action).
    #[command(
        long_about = "Launch opencode preconfigured with ServalAI as the provider.\n\nRunning `serval` without a subcommand does the same thing.\nAll arguments after `--` are forwarded to opencode.\n\nExamples:\n  serval code\n  serval code -- --print-logs\n  serval"
    )]
    Code {
        /// Extra arguments forwarded to opencode.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Launch oh-my-pi preconfigured with ServalAI.
    #[command(
        long_about = "Launch oh-my-pi preconfigured with ServalAI as the provider.\n\nAll arguments after `--` are forwarded to pi.\n\nExamples:\n  serval pi\n  serval pi -- --help"
    )]
    Pi {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Launch aider preconfigured with ServalAI (requires aider on PATH).
    #[command(
        long_about = "Launch aider preconfigured with ServalAI as the provider.\n\nRequires aider to be installed (pip install aider-chat or brew install aider).\nAll arguments after `--` are forwarded to aider.\n\nExamples:\n  serval aider\n  serval aider -- --help"
    )]
    Aider {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Print serval version and exit.
    #[command(name = "version", hide = true)]
    Version,
    /// Print this help and exit.
    #[command(name = "help", hide = true)]
    Help,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Auth { token }) => run(commands::auth(token)),
        Some(Command::Sync) => run(commands::sync()),
        Some(Command::Status) => run(commands::status()),
        Some(Command::Logout) => run(commands::logout()),
        Some(Command::Update) => run(commands::update_cmd()),
        Some(Command::Ping) => run(commands::ping()),
        Some(Command::Models) => run(commands::models()),
        Some(Command::Usage) => run(commands::usage()),
        Some(Command::Doctor) => run(commands::doctor()),
        Some(Command::Init) => run(commands::init()),
        Some(Command::Report) => run(commands::report()),
        Some(Command::Code { args }) => run(commands::code(args)),
        Some(Command::Pi { args }) => run(commands::pi(args)),
        Some(Command::Aider { args }) => run(commands::aider(args)),
        Some(Command::Version) => {
            println!(
                "serval {}\n\nServalAI — Cleverit's company-funded model gateway CLI\n\n\
                 Tip: run `serval status` to see your configuration and bundled opencode version.\n\
                 Run `serval auth` to get started.",
                env!("CARGO_PKG_VERSION")
            );
        }
        Some(Command::Help) => {
            let mut cmd = <Cli as clap::CommandFactory>::command();
            cmd.print_help().ok();
            println!();
        }
        None => match cli.args.first().map(String::as_str) {
            Some("--") => run(commands::code(cli.args[1..].to_vec())),
            Some(first) if !first.starts_with('-') => {
                eprintln!(
                    "serval: '{first}' is not a serval command.\n\n\
                     Did you mean one of these?\n\n\
                     \x20 serval auth       Store your token and get started\n\
                     \x20 serval             Start coding (opencode)\n\
                     \x20 serval pi          Start oh-my-pi\n\
                     \x20 serval aider       Start aider\n\
                     \x20 serval ping        Test gateway connectivity\n\
                     \x20 serval doctor      Run installation diagnostics\n\
                     \x20 serval status      Show your configuration\n\n\
                     Run `serval --help` to see all commands."
                );
                std::process::exit(1);
            }
            _ => run(commands::code(cli.args)),
        },
    }
}

fn run(result: Result<(), String>) {
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

    #[test]
    fn version_subcommand_is_recognized() {
        let cli = Cli::try_parse_from(["serval", "version"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Version)));
    }

    #[test]
    fn help_subcommand_is_recognized() {
        let cli = Cli::try_parse_from(["serval", "help"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Help)));
    }

    #[test]
    fn unknown_word_captured_as_passthrough() {
        let cli = Cli::try_parse_from(["serval", "opencode"]).unwrap();
        assert!(cli.command.is_none());
        assert_eq!(cli.args, vec!["opencode".to_string()]);
    }

    #[test]
    fn pi_subcommand_is_recognized() {
        let cli = Cli::try_parse_from(["serval", "pi"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Pi { .. })));
    }

    #[test]
    fn aider_subcommand_is_recognized() {
        let cli = Cli::try_parse_from(["serval", "aider"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Aider { .. })));
    }

    #[test]
    fn pi_subcommand_captures_trailing_args() {
        let cli = Cli::try_parse_from(["serval", "pi", "--", "--model", "gpt4"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Pi { args }) if args == vec!["--model".to_string(), "gpt4".to_string()]
        ));
    }

    #[test]
    fn aider_subcommand_captures_trailing_args() {
        let cli = Cli::try_parse_from(["serval", "aider", "--", "--help"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Aider { args }) if args == vec!["--help".to_string()]
        ));
    }

    #[test]
    fn ping_subcommand_is_recognized() {
        let cli = Cli::try_parse_from(["serval", "ping"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Ping)));
    }

    #[test]
    fn models_subcommand_is_recognized() {
        let cli = Cli::try_parse_from(["serval", "models"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Models)));
    }

    #[test]
    fn usage_subcommand_is_recognized() {
        let cli = Cli::try_parse_from(["serval", "usage"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Usage)));
    }

    #[test]
    fn doctor_subcommand_is_recognized() {
        let cli = Cli::try_parse_from(["serval", "doctor"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Doctor)));
    }

    #[test]
    fn init_subcommand_is_recognized() {
        let cli = Cli::try_parse_from(["serval", "init"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Init)));
    }

    #[test]
    fn report_subcommand_is_recognized() {
        let cli = Cli::try_parse_from(["serval", "report"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Report)));
    }
}
