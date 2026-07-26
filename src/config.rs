//! Persistent CLI config (`~/.config/serval/config.toml`). Holds the token
//! (0600) and the last provider config fetched from the Worker.

#![allow(dead_code)]

use crate::constants::DEFAULT_WORKER_URL;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default = "default_worker_url")]
    pub worker_url: String,
    #[serde(default)]
    pub cached_email: Option<String>,
    #[serde(default)]
    pub cached_provider: Option<serde_json::Value>,
}

fn default_worker_url() -> String {
    DEFAULT_WORKER_URL.to_string()
}

impl Default for Config {
    fn default() -> Self {
        Config {
            token: None,
            worker_url: default_worker_url(),
            cached_email: None,
            cached_provider: None,
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Config {
        match std::fs::read_to_string(path) {
            Ok(s) => toml::from_str(&s).unwrap_or_default(),
            Err(_) => Config::default(),
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("mkdir {parent:?}: {e}"))?;
        }
        let body = toml::to_string_pretty(self).map_err(|e| format!("serialize config: {e}"))?;
        #[cfg(unix)]
        {
            use std::io::Write;
            use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(path)
                .map_err(|e| format!("open {path:?}: {e}"))?;
            // Tighten perms before writing any secret content — mode() above only
            // applies when the file is newly created, so a pre-existing file that
            // was created with looser perms must be chmod'd while still empty.
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
                .map_err(|e| format!("chmod 0600 {path:?}: {e}"))?;
            f.write_all(body.as_bytes())
                .map_err(|e| format!("write {path:?}: {e}"))?;
        }
        #[cfg(not(unix))]
        {
            std::fs::write(path, body).map_err(|e| format!("write {path:?}: {e}"))?;
        }
        Ok(())
    }

    pub fn masked_token(&self) -> String {
        match &self.token {
            None => "—".to_string(),
            Some(t) if t.len() <= 4 => "••••".to_string(),
            Some(t) => format!("••••{}", &t[t.len() - 4..]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn load_missing_returns_defaults() {
        let dir = tempdir().unwrap();
        let cfg = Config::load(&dir.path().join("nope.toml"));
        assert_eq!(cfg.worker_url, DEFAULT_WORKER_URL);
        assert!(cfg.token.is_none());
    }

    #[test]
    fn save_then_load_roundtrips_token() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("config.toml");
        let mut cfg = Config::default();
        cfg.token = Some("aig_secret1234".to_string());
        cfg.save(&p).unwrap();
        let back = Config::load(&p);
        assert_eq!(back.token.as_deref(), Some("aig_secret1234"));
    }

    #[cfg(unix)]
    #[test]
    fn save_sets_0600() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempdir().unwrap();
        let p = dir.path().join("config.toml");
        Config::default().save(&p).unwrap();
        let mode = std::fs::metadata(&p).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[cfg(unix)]
    #[test]
    fn save_tightens_preexisting_loose_perms() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempdir().unwrap();
        let p = dir.path().join("config.toml");
        std::fs::write(&p, "old = true").unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o644)).unwrap();
        let mut cfg = Config::default();
        cfg.token = Some("aig_secret1234".to_string());
        cfg.save(&p).unwrap();
        let mode = std::fs::metadata(&p).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn masked_token_shows_last4_only() {
        let mut cfg = Config::default();
        cfg.token = Some("aig_abcd1234".to_string());
        assert_eq!(cfg.masked_token(), "••••1234");
        assert_eq!(Config::default().masked_token(), "—");
    }
}
