//! Filesystem locations. Runtime asset paths (opencode, bundle) resolve relative
//! to the running binary so a symlinked `serval` still finds its siblings.

#![allow(dead_code)]

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

pub fn bundle_dir() -> PathBuf {
    install_root().join("bundle")
}

pub fn config_file() -> PathBuf {
    dirs::config_dir()
        .expect("no config dir")
        .join("serval")
        .join("config.toml")
}

pub fn data_dir() -> PathBuf {
    dirs::data_dir().expect("no data dir").join("serval")
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
}
