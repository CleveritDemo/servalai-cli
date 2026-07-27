//! One function per subcommand. Each loads Config from `paths::config_file()`,
//! does its work, and returns Result<(), String> (main maps Err → exit 1).

use crate::client::{resolve_config, UreqHttp};
use crate::config::Config;
use crate::launch::{build_ai_env, build_env, ExecLauncher, Launcher};
use crate::{paths, update};
use std::io::Write;

const TOKEN_URL: &str = "https://cleverit-support.cleveritgroup.com";

fn load() -> Config {
    Config::load(&paths::config_file())
}

fn require_token(cfg: &Config) -> Result<String, String> {
    cfg.token.clone().ok_or_else(|| {
        format!(
            "you haven't authenticated yet. Run `serval auth` to get started.\n\n\
             Get your token at {TOKEN_URL}"
        )
    })
}

pub fn auth(token: Option<String>) -> Result<(), String> {
    let mut cfg = load();
    let token = match token {
        Some(t) => t,
        None => {
            eprint!("Paste your ServalAI token (from {TOKEN_URL}): ");
            std::io::stderr().flush().ok();
            let mut s = String::new();
            std::io::stdin()
                .read_line(&mut s)
                .map_err(|e| format!("could not read token: {e}"))?;
            s.trim().to_string()
        }
    };
    if token.is_empty() {
        return Err(format!(
            "no token provided.\nGet yours at {TOKEN_URL} and run `serval auth` again."
        ));
    }
    cfg.token = Some(token.clone());
    let (provider, email) = resolve_config(
        &UreqHttp,
        &cfg.worker_url,
        &token,
        cfg.cached_provider.as_ref(),
    );
    cfg.cached_provider = Some(provider);
    cfg.cached_email = email.clone();
    cfg.save(&paths::config_file())?;
    match email {
        Some(e) => println!("Authenticated as {e}. Ready — run `serval` to start coding."),
        None => println!(
            "Token saved, but the gateway could not be reached to confirm your identity.\n\
             Your provider config uses the built-in defaults. Run `serval sync` later to refresh."
        ),
    }
    Ok(())
}

pub fn sync() -> Result<(), String> {
    let mut cfg = load();
    let token = require_token(&cfg)?;
    let (provider, email) = resolve_config(
        &UreqHttp,
        &cfg.worker_url,
        &token,
        cfg.cached_provider.as_ref(),
    );
    cfg.cached_provider = Some(provider);
    if email.is_some() {
        cfg.cached_email = email;
    }
    cfg.save(&paths::config_file())?;
    println!("Provider config synced from the gateway.");
    Ok(())
}

pub fn status() -> Result<(), String> {
    let cfg = load();
    println!("ServalAI CLI");
    println!("━━━━━━━━━━━━");
    println!("  version      v{}", env!("CARGO_PKG_VERSION"));
    println!("  opencode     {}", opencode_version());
    println!("  gateway      {}", cfg.worker_url);
    println!("  token        {}", cfg.masked_token());
    println!(
        "  identity     {}",
        cfg.cached_email.as_deref().unwrap_or("—")
    );
    if cfg.token.is_none() {
        println!("\nRun `serval auth` to get started.");
    }
    Ok(())
}

fn opencode_version() -> String {
    std::process::Command::new(paths::opencode_bin())
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "not bundled".to_string())
}

pub fn logout() -> Result<(), String> {
    let mut cfg = load();
    cfg.token = None;
    cfg.save(&paths::config_file())?;
    println!(
        "Logged out. Your token has been cleared from this machine.\n\
         Run `serval auth` again to reconnect."
    );
    Ok(())
}

pub fn update_cmd() -> Result<(), String> {
    use crate::client::Http;
    eprintln!("Checking for updates…");
    let (tag, url) = update::latest_release(&UreqHttp)?;
    let current = format!("v{}", env!("CARGO_PKG_VERSION"));
    if !update::needs_update(&current, &tag) {
        println!("You're on the latest version ({tag}).");
        return Ok(());
    }
    println!(
        "Update available: {current} → {tag}\n\
         Downloading…"
    );
    let bytes = UreqHttp.get_bytes(&url)?;
    let dest = paths::versions_dir().join(&tag);
    update::extract_tar_gz(&bytes, &dest)?;
    update::repoint_current(&paths::versions_dir(), &paths::current_link(), &tag)?;
    println!("Updated to {tag}. Run `serval` to start your new session.");
    Ok(())
}

pub fn code(passthrough: Vec<String>) -> Result<(), String> {
    let cfg = load();
    let token = require_token(&cfg)?;
    let (provider, _) = resolve_config(
        &UreqHttp,
        &cfg.worker_url,
        &token,
        cfg.cached_provider.as_ref(),
    );
    let env = build_env(&provider, &cfg.worker_url, &token, &paths::bundle_dir());
    ExecLauncher.exec(&paths::opencode_bin(), &passthrough, &env)
}

pub fn pi(passthrough: Vec<String>) -> Result<(), String> {
    let cfg = load();
    let token = require_token(&cfg)?;
    sync_pi_agents(&paths::bundle_dir())?;
    let env = build_ai_env(&cfg.worker_url, &token);
    ExecLauncher.exec(&paths::pi_bin(), &passthrough, &env)
}

pub fn aider(passthrough: Vec<String>) -> Result<(), String> {
    let cfg = load();
    let token = require_token(&cfg)?;
    let aider_bin = which::which("aider").map_err(|_| {
        "aider not found on PATH.\n\n\
         Install it first:\n\
         \x20 pip install aider-chat\n\
         or\n\
         \x20 brew install aider\n\n\
         Then run `serval aider` again."
    })?;
    let env = build_ai_env(&cfg.worker_url, &token);
    ExecLauncher.exec(&aider_bin, &passthrough, &env)
}

fn sync_pi_agents(bundle_dir: &std::path::Path) -> Result<(), String> {
    let src = bundle_dir.join("agents");
    let dest = dirs::home_dir()
        .ok_or("no home directory")?
        .join(".omp")
        .join("agents");
    if !src.exists() || !src.is_dir() {
        return Ok(());
    }
    std::fs::create_dir_all(&dest).map_err(|e| format!("mkdir {dest:?}: {e}"))?;
    for entry in std::fs::read_dir(&src).map_err(|e| format!("read agents dir {src:?}: {e}"))? {
        let entry = entry.map_err(|e| format!("read entry: {e}"))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let body = std::fs::read_to_string(&path)
            .map_err(|e| format!("read {:?}: {e}", path.file_name().unwrap_or_default()))?;
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("agent");
        let converted = convert_agent_to_pi(name, &body);
        let dest_path = dest.join(path.file_name().unwrap());
        std::fs::write(&dest_path, converted).map_err(|e| format!("write {dest_path:?}: {e}"))?;
    }
    Ok(())
}

fn convert_agent_to_pi(name: &str, opencode_md: &str) -> String {
    let desc = opencode_md
        .lines()
        .find(|l| l.starts_with("description:"))
        .and_then(|l| l.split("description:").nth(1))
        .map(|s| s.trim().trim_matches('"'))
        .unwrap_or(name);

    let is_read_only =
        opencode_md.contains("edit: deny") || opencode_md.contains("Never writes production code");

    let tools = if is_read_only {
        "  - read\n  - grep\n  - glob\n  - web_search"
    } else {
        "  - read\n  - grep\n  - glob\n  - bash\n  - edit\n  - write\n  - lsp\n  - web_search"
    };

    // Strip opencode-specific frontmatter and use Pi's YAML format
    let body_lines: Vec<&str> = opencode_md
        .lines()
        .skip_while(|l| l != &"---")
        .skip(1)
        .skip_while(|l| l != &"---")
        .skip(1)
        .collect();
    let body = body_lines.join("\n").trim().to_string();

    format!(
        "---\n\
         name: {name}\n\
         description: \"{desc}\"\n\
         tools:\n{tools}\n\
         model:\n  - \"@balanced\"\n\
         ---\n\n\
         {body}\n"
    )
}
