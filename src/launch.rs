//! Builds the environment that injects ServalAI into opencode, then execs the
//! bundled opencode binary. The env-building logic is pure and unit-tested; the
//! exec itself is behind a trait so tests never spawn a real process.

use crate::constants::{PROVIDER_KEY, TOKEN_ENV};
use std::path::Path;

pub fn build_env(
    provider: &serde_json::Value,
    worker_url: &str,
    token: &str,
    bundle_dir: &Path,
) -> Vec<(String, String)> {
    let skills_dir = bundle_dir.join("skills");
    let mut provider = provider.clone();
    provider["options"] = serde_json::json!({
        "baseURL": worker_url.trim_end_matches('/'),
        "apiKey": token,
    });
    let content = serde_json::json!({
        "provider": { PROVIDER_KEY: provider },
        "skills": { "paths": [ skills_dir.to_string_lossy() ] }
    });
    vec![
        (
            "OPENCODE_CONFIG_DIR".to_string(),
            bundle_dir.to_string_lossy().to_string(),
        ),
        ("OPENCODE_CONFIG_CONTENT".to_string(), content.to_string()),
        (TOKEN_ENV.to_string(), token.to_string()),
    ]
}

pub trait Launcher {
    fn exec(&self, program: &Path, args: &[String], env: &[(String, String)])
        -> Result<(), String>;
}

pub struct ExecLauncher;

impl Launcher for ExecLauncher {
    #[cfg(unix)]
    fn exec(
        &self,
        program: &Path,
        args: &[String],
        env: &[(String, String)],
    ) -> Result<(), String> {
        use std::os::unix::process::CommandExt;
        let mut cmd = std::process::Command::new(program);
        cmd.args(args);
        for (k, v) in env {
            cmd.env(k, v);
        }
        // exec() only returns if it FAILED.
        Err(format!("failed to exec {program:?}: {}", cmd.exec()))
    }

    #[cfg(not(unix))]
    fn exec(
        &self,
        _program: &Path,
        _args: &[String],
        _env: &[(String, String)],
    ) -> Result<(), String> {
        Err("serval only supports Unix (Linux/macOS/WSL2)".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn build_env_injects_provider_dir_content_and_token() {
        let provider =
            serde_json::json!({ "npm": "@ai-sdk/openai-compatible", "name": "ServalAI" });
        let env = build_env(
            &provider,
            "https://w.example.dev/",
            "aig_tok",
            &PathBuf::from("/b"),
        );

        let map: std::collections::HashMap<_, _> = env.into_iter().collect();
        assert_eq!(map["OPENCODE_CONFIG_DIR"], "/b");
        assert_eq!(map[TOKEN_ENV], "aig_tok");

        let content: serde_json::Value =
            serde_json::from_str(&map["OPENCODE_CONFIG_CONTENT"]).unwrap();
        let p = &content["provider"][PROVIDER_KEY];
        assert_eq!(p["name"], "ServalAI");
        assert_eq!(p["options"]["baseURL"], "https://w.example.dev");
        assert_eq!(p["options"]["apiKey"], "aig_tok");
        assert_eq!(content["skills"]["paths"][0].as_str().unwrap(), "/b/skills");
    }
}
