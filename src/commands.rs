use crate::client::{fetch_config, fetch_usage, health_check, resolve_config, UreqHttp};
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
    let spinner = crate::progress::Spinner::start("Contacting gateway…");
    let (provider, email, fallback_note) = resolve_config(
        &UreqHttp,
        &cfg.worker_url,
        &token,
        cfg.cached_provider.as_ref(),
    );
    match fallback_note {
        Some(note) => spinner.finish_note(&note),
        None => spinner.finish_silent(),
    }
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
    let spinner = crate::progress::Spinner::start("Contacting gateway…");
    let (provider, email, fallback_note) = resolve_config(
        &UreqHttp,
        &cfg.worker_url,
        &token,
        cfg.cached_provider.as_ref(),
    );
    match fallback_note {
        Some(note) => spinner.finish_note(&note),
        None => spinner.finish_silent(),
    }
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

pub fn ping() -> Result<(), String> {
    let cfg = load();
    let spinner = crate::progress::Spinner::start("Contacting gateway…");
    let status = health_check(&UreqHttp, &cfg.worker_url)
        .map_err(|e| format!("gateway unreachable: {e}"))?;
    spinner.finish_silent();
    println!("Gateway: {}", cfg.worker_url);
    println!("Status:  {status}");

    // Try to fetch provider config to show available models
    if let Ok(token) = require_token(&cfg) {
        match fetch_config(&UreqHttp, &cfg.worker_url, &token) {
            Ok(fc) if !fc.models.is_empty() => {
                println!("\nAvailable models:");
                for m in &fc.models {
                    println!("  {m}");
                }
                println!("\nIdentified as: {}", fc.email);
            }
            _ => {}
        }
    } else {
        println!("\nRun `serval auth` to authenticate and see your models.");
    }
    Ok(())
}

pub fn models() -> Result<(), String> {
    let cfg = load();
    let token = require_token(&cfg)?;
    let spinner = crate::progress::Spinner::start("Contacting gateway…");
    let fc = fetch_config(&UreqHttp, &cfg.worker_url, &token)
        .map_err(|e| format!("could not fetch model list: {e}"))?;
    spinner.finish_silent();

    println!("ServalAI Models");
    println!("━━━━━━━━━━━━━━━");
    println!("Account: {}", fc.email);
    println!();
    for m in &fc.models {
        let description = match m.as_str() {
            s if s.contains("dynamic/power") => "Hard tasks, architecture, large refactors",
            s if s.contains("dynamic/balanced") => "Everyday work (default)",
            s if s.contains("dynamic/light") => "Quick, mechanical tasks",
            _ => "",
        };
        println!("  {m}");
        if !description.is_empty() {
            println!("    {description}");
        }
    }
    Ok(())
}

pub fn usage() -> Result<(), String> {
    let cfg = load();
    let token = require_token(&cfg)?;
    let spinner = crate::progress::Spinner::start("Contacting gateway…");
    let data = fetch_usage(&UreqHttp, &cfg.worker_url, &token)?;
    spinner.finish_silent();

    println!("ServalAI Usage");
    println!("━━━━━━━━━━━━━━");
    if let Some(tokens) = data.get("total_tokens").and_then(|v| v.as_u64()) {
        println!("  Total tokens:   {tokens}");
    }
    if let Some(sessions) = data.get("session_count").and_then(|v| v.as_u64()) {
        println!("  Sessions:       {sessions}");
    }
    if let Some(model) = data.get("last_model").and_then(|v| v.as_str()) {
        println!("  Last model:     {model}");
    }
    if let Some(at) = data.get("last_used_at").and_then(|v| v.as_str()) {
        println!("  Last used:      {at}");
    }
    Ok(())
}

pub fn doctor() -> Result<(), String> {
    let cfg = load();
    println!("ServalAI Doctor");
    println!("━━━━━━━━━━━━━━━");

    // 1. Config file
    let config_path = paths::config_file();
    println!("  Config file:  {}", config_path.display());
    if config_path.exists() {
        println!("    ✓ exists");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = std::fs::metadata(&config_path) {
                let mode = meta.permissions().mode() & 0o777;
                if mode == 0o600 {
                    println!("    ✓ permissions 0600");
                } else {
                    println!("    ⚠ permissions {mode:03o} (should be 0600)");
                }
            }
        }
    } else {
        println!("    — not created yet");
    }

    // 2. Token
    let token_status = match &cfg.token {
        Some(t) if t.len() > 8 => {
            let masked = format!("{}…{}", &t[..4], &t[t.len() - 4..]);
            format!("✓ stored ({masked})")
        }
        Some(t) => format!("✓ stored ({} chars)", t.len()),
        None => "✗ missing — run `serval auth`".to_string(),
    };
    println!("  Token:        {token_status}");

    // 3. Gateway
    println!("  Gateway:      {}", cfg.worker_url);
    let spinner = crate::progress::Spinner::start("Checking gateway…");
    let health = health_check(&UreqHttp, &cfg.worker_url);
    spinner.finish_silent();
    match health {
        Ok(s) => println!("    ✓ reachable (status: {s})"),
        Err(e) => println!("    ✗ unreachable: {e}"),
    }

    // 4. Bundled binaries
    println!("\n  Bundled binaries:");
    check_binary("opencode", &paths::opencode_bin());
    check_binary("pi", &paths::pi_bin());
    println!("  Bundle dir:   {}", paths::bundle_dir().display());
    if paths::bundle_dir().exists() {
        println!("    ✓ exists");
    } else {
        println!("    ✗ missing — reinstall with `serval update` or the install script");
    }

    Ok(())
}

fn check_binary(name: &str, path: &std::path::Path) {
    if path.exists() {
        if let Ok(meta) = path.metadata() {
            let size = meta.len();
            let kb = size / 1024;
            let mb = kb / 1024;
            if mb > 0 {
                println!("    ✓ {name} ({mb} MB)");
            } else {
                println!("    ✓ {name} ({kb} KB)");
            }
        } else {
            println!("    ✓ {name}");
        }
    } else {
        println!("    — {name} not bundled");
    }
}

pub fn init() -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| format!("current dir: {e}"))?;
    let dest = cwd.join(".serval.jsonc");
    if dest.exists() {
        return Err(format!(
            "{dest:?} already exists. Delete it first if you want to re-initialize."
        ));
    }
    let content = r#"{
  // ServalAI project config
  // Model tier for this project: "dynamic/power", "dynamic/balanced" (default), or "dynamic/light"
  // "model": "dynamic/balanced"
}
"#;
    std::fs::write(&dest, content).map_err(|e| format!("write {dest:?}: {e}"))?;
    println!("Created {dest:?}");
    println!("Edit it to pin a model tier for this project.");
    Ok(())
}

pub fn report() -> Result<(), String> {
    let cfg = load();
    let cwd = std::env::current_dir().map_err(|e| format!("current dir: {e}"))?;
    let identity = cfg.cached_email.as_deref().unwrap_or("unknown");

    println!("ServalAI Session Report");
    println!("━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Identity:   {identity}");
    println!("  Directory:  {}", cwd.display());
    println!();

    println!("  Recent sessions (opencode):");
    let code_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("~/.local/share"))
        .join("opencode");
    if code_dir.exists() && code_dir.is_dir() {
        let sessions = std::fs::read_dir(&code_dir);
        let count = match sessions {
            Ok(entries) => entries.flatten().count(),
            Err(_) => 0,
        };
        println!("    {count} entries in {code_dir:?}");
    } else {
        println!("    — no opencode data directory found");
    }

    println!();
    println!("  Tools available:");
    if paths::opencode_bin().exists() {
        println!("    ✓ opencode");
    }
    if paths::pi_bin().exists() {
        println!("    ✓ pi");
    }
    if which::which("aider").is_ok() {
        println!("    ✓ aider");
    }
    Ok(())
}

pub fn code(passthrough: Vec<String>) -> Result<(), String> {
    let cfg = load();
    let token = require_token(&cfg)?;
    println!(
        "{}",
        crate::progress::banner(crate::progress::is_interactive())
    );
    let spinner = crate::progress::Spinner::start("Connecting to gateway…");
    let (provider, _, fallback_note) = resolve_config(
        &UreqHttp,
        &cfg.worker_url,
        &token,
        cfg.cached_provider.as_ref(),
    );
    match fallback_note {
        Some(note) => spinner.finish_note(&note),
        None => spinner.finish_silent(),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_read_only_agent_to_pi() {
        let input = "---\ndescription: \"System Architect, never writes code\"\nmode: subagent\npermission:\n  edit: deny\n  bash: deny\n---\n\nDo design work.";
        let result = convert_agent_to_pi("architect", input);
        assert!(result.contains("name: architect"));
        assert!(result.contains("description: \"System Architect, never writes code\""));
        assert!(!result.contains("edit"));
        assert!(result.contains("Do design work."));
    }

    #[test]
    fn convert_writer_agent_to_pi() {
        let input = "---\ndescription: \"Developer\"\nmode: subagent\npermission:\n  edit: allow\n  bash: ask\n---\n\nYou implement code.";
        let result = convert_agent_to_pi("developer", input);
        assert!(result.contains("name: developer"));
        assert!(result.contains("edit"));
        assert!(result.contains("bash"));
        assert!(result.contains("You implement code."));
    }

    #[test]
    fn pi_agent_dir_skipped_when_no_agents() {
        let dir = tempfile::tempdir().unwrap();
        let bundle = dir.path().join("bundle");
        std::fs::create_dir_all(&bundle).unwrap();
        sync_pi_agents(&bundle).unwrap();
    }
}
