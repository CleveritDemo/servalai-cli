//! Filesystem locations. Runtime asset paths (opencode, bundle) resolve relative
//! to the running binary so a symlinked `serval` still finds its siblings.

use std::path::PathBuf;

pub fn install_root() -> PathBuf {
    if let Ok(root) = std::env::var("SERVAL_INSTALL_ROOT") {
        return PathBuf::from(root);
    }
    let exe = std::env::current_exe().expect("cannot locate current exe");
    let exe = std::fs::canonicalize(&exe).unwrap_or(exe);
    exe.parent().expect("exe has no parent").to_path_buf()
}

pub fn opencode_bin() -> PathBuf {
    install_root().join("opencode")
}

pub fn pi_bin() -> PathBuf {
    install_root().join("pi")
}

pub fn bundle_dir() -> PathBuf {
    install_root().join("bundle")
}

/// Resolve an XDG base dir: the env var if set and non-empty, else `$HOME/<default_sub>`.
/// Pure and testable — no process-global state.
fn xdg_base(env_val: Option<String>, home: &std::path::Path, default_sub: &str) -> PathBuf {
    match env_val {
        Some(v) if !v.is_empty() => PathBuf::from(v),
        _ => home.join(default_sub),
    }
}

fn home() -> PathBuf {
    dirs::home_dir().expect("no home dir")
}

pub fn config_file() -> PathBuf {
    xdg_base(std::env::var("XDG_CONFIG_HOME").ok(), &home(), ".config")
        .join("serval")
        .join("config.toml")
}

pub fn data_dir() -> PathBuf {
    xdg_base(std::env::var("XDG_DATA_HOME").ok(), &home(), ".local/share").join("serval")
}

pub fn versions_dir() -> PathBuf {
    data_dir().join("versions")
}

pub fn current_link() -> PathBuf {
    data_dir().join("current")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_root_honors_env_override() {
        std::env::set_var("SERVAL_INSTALL_ROOT", "/tmp/serval-test-root");
        assert_eq!(install_root(), PathBuf::from("/tmp/serval-test-root"));
        assert_eq!(
            opencode_bin(),
            PathBuf::from("/tmp/serval-test-root/opencode")
        );
        assert_eq!(bundle_dir(), PathBuf::from("/tmp/serval-test-root/bundle"));
        std::env::remove_var("SERVAL_INSTALL_ROOT");
    }

    #[test]
    fn config_and_data_paths_end_in_serval() {
        assert!(config_file().ends_with("serval/config.toml"));
        assert!(data_dir().ends_with("serval"));
        assert!(current_link().ends_with("serval/current"));
    }

    #[test]
    fn xdg_base_uses_env_when_set_and_non_empty() {
        let home = PathBuf::from("/home/dev");
        assert_eq!(
            xdg_base(Some("/custom/data".to_string()), &home, ".local/share"),
            PathBuf::from("/custom/data")
        );
    }

    #[test]
    fn xdg_base_falls_back_to_home_when_env_absent() {
        let home = PathBuf::from("/home/dev");
        assert_eq!(
            xdg_base(None, &home, ".local/share"),
            PathBuf::from("/home/dev/.local/share")
        );
    }

    #[test]
    fn xdg_base_falls_back_to_home_when_env_empty() {
        let home = PathBuf::from("/home/dev");
        assert_eq!(
            xdg_base(Some(String::new()), &home, ".config"),
            PathBuf::from("/home/dev/.config")
        );
    }
}
