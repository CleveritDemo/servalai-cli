use crate::constants::{DEFAULT_WORKER_URL, SERVAL_KEYCHAIN_SERVICE};
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
    #[serde(default)]
    pub use_keychain: bool,
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
            use_keychain: true,
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Config {
        let mut cfg: Config = match std::fs::read_to_string(path) {
            Ok(s) => toml::from_str(&s).unwrap_or_default(),
            Err(_) => Config::default(),
        };
        if cfg.use_keychain && cfg.token.is_none() {
            cfg.token = keychain_get().ok().flatten();
        }
        cfg
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if self.use_keychain {
            if let Some(t) = &self.token {
                keychain_set(t)?;
            } else {
                keychain_delete().ok();
            }
        }
        let stripped = Config {
            token: if self.use_keychain {
                None
            } else {
                self.token.clone()
            },
            cached_provider: self.cached_provider.clone(),
            cached_email: self.cached_email.clone(),
            worker_url: self.worker_url.clone(),
            use_keychain: self.use_keychain,
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("mkdir {parent:?}: {e}"))?;
        }
        let body =
            toml::to_string_pretty(&stripped).map_err(|e| format!("serialize config: {e}"))?;
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

fn keychain_get() -> Result<Option<String>, String> {
    let entry = keyring::Entry::new(SERVAL_KEYCHAIN_SERVICE, "token")
        .map_err(|e| format!("keyring entry: {e}"))?;
    match entry.get_password() {
        Ok(p) => Ok(Some(p)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("keyring get: {e}")),
    }
}

fn keychain_set(token: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(SERVAL_KEYCHAIN_SERVICE, "token")
        .map_err(|e| format!("keyring entry: {e}"))?;
    entry
        .set_password(token)
        .map_err(|e| format!("keyring set: {e}"))
}

fn keychain_delete() -> Result<(), String> {
    let entry = keyring::Entry::new(SERVAL_KEYCHAIN_SERVICE, "token")
        .map_err(|e| format!("keyring entry: {e}"))?;
    entry
        .delete_credential()
        .map_err(|e| format!("keyring delete: {e}"))
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
    fn save_then_load_roundtrips_token_file_only() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("config.toml");
        let cfg = Config {
            token: Some("aig_secret1234".to_string()),
            use_keychain: false,
            ..Config::default()
        };
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
        Config {
            use_keychain: false,
            ..Config::default()
        }
        .save(&p)
        .unwrap();
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
        let cfg = Config {
            token: Some("aig_secret1234".to_string()),
            use_keychain: false,
            ..Config::default()
        };
        cfg.save(&p).unwrap();
        let mode = std::fs::metadata(&p).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn masked_token_shows_last4_only() {
        let cfg = Config {
            token: Some("aig_abcd1234".to_string()),
            ..Config::default()
        };
        assert_eq!(cfg.masked_token(), "••••1234");
        assert_eq!(Config::default().masked_token(), "—");
    }
}
