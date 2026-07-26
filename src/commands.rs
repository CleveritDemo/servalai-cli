//! One function per subcommand. Each loads Config from `paths::config_file()`,
//! does its work, and returns Result<(), String> (main maps Err → exit 1).

use crate::client::{resolve_config, UreqHttp};
use crate::config::Config;
use crate::launch::{build_env, ExecLauncher, Launcher};
use crate::{paths, update};
use std::io::Write;

fn load() -> Config {
    Config::load(&paths::config_file())
}

fn require_token(cfg: &Config) -> Result<String, String> {
    cfg.token
        .clone()
        .ok_or_else(|| "no token — run `serval auth` (get yours from Mi Portal)".to_string())
}

pub fn auth(token: Option<String>) -> Result<(), String> {
    let mut cfg = load();
    let token = match token {
        Some(t) => t,
        None => {
            eprint!("Paste your ServalAI token: ");
            std::io::stderr().flush().ok();
            let mut s = String::new();
            std::io::stdin()
                .read_line(&mut s)
                .map_err(|e| format!("read token: {e}"))?;
            s.trim().to_string()
        }
    };
    if token.is_empty() {
        return Err("empty token".to_string());
    }
    cfg.token = Some(token.clone());
    // Validate + warm the cache against the Worker.
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
        Some(e) => println!("Authenticated as {e}."),
        None => println!("Token saved (could not reach the Worker to confirm identity)."),
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
    println!("Config synced.");
    Ok(())
}

pub fn status() -> Result<(), String> {
    let cfg = load();
    println!("serval        {}", env!("CARGO_PKG_VERSION"));
    println!("opencode      {}", opencode_version());
    println!("worker        {}", cfg.worker_url);
    println!("token         {}", cfg.masked_token());
    println!(
        "identity      {}",
        cfg.cached_email.as_deref().unwrap_or("—")
    );
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
    println!("Logged out.");
    Ok(())
}

pub fn update_cmd() -> Result<(), String> {
    use crate::client::Http;
    eprintln!("Checking for updates...");
    let (tag, url) = update::latest_release(&UreqHttp)?;
    let current = format!("v{}", env!("CARGO_PKG_VERSION"));
    if !update::needs_update(&current, &tag) {
        println!("Already up to date ({current}).");
        return Ok(());
    }
    println!("Update available: {current} -> {tag}");
    let bytes = UreqHttp.get_bytes(&url)?;
    let dest = paths::versions_dir().join(&tag);
    update::extract_tar_gz(&bytes, &dest)?;
    update::repoint_current(&paths::versions_dir(), &paths::current_link(), &tag)?;
    println!("Updated to {tag}. Run `serval` to use it.");
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
