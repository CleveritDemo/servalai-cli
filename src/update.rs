//! Self-update: locate the latest GitHub release, download this platform's bundle,
//! extract it into a versioned dir, and atomically repoint the `current` symlink.

#![allow(dead_code)]

use crate::client::Http;
use crate::constants::RELEASES_LATEST_API;
use flate2::read::GzDecoder;
use std::path::Path;

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub const CURRENT_TARGET: &str = "x86_64-unknown-linux-musl";
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub const CURRENT_TARGET: &str = "aarch64-unknown-linux-musl";
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub const CURRENT_TARGET: &str = "x86_64-apple-darwin";
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub const CURRENT_TARGET: &str = "aarch64-apple-darwin";
#[cfg(not(any(
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "linux", target_arch = "aarch64"),
    all(target_os = "macos", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "aarch64"),
)))]
compile_error!("serval: unsupported target — add a CURRENT_TARGET for this platform");

pub fn asset_name(target: &str) -> String {
    format!("serval-{target}.tar.gz")
}

pub fn needs_update(current_tag: &str, latest_tag: &str) -> bool {
    current_tag != latest_tag
}

pub fn extract_tar_gz(bytes: &[u8], dest: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dest).map_err(|e| format!("mkdir {dest:?}: {e}"))?;
    let gz = GzDecoder::new(bytes);
    let mut ar = tar::Archive::new(gz);
    ar.unpack(dest)
        .map_err(|e| format!("extract to {dest:?}: {e}"))
}

#[cfg(unix)]
pub fn repoint_current(versions: &Path, current: &Path, tag: &str) -> Result<(), String> {
    use std::os::unix::fs::symlink;
    let target = versions.join(tag);
    if !target.exists() {
        return Err(format!("version dir missing: {target:?}"));
    }
    // Write a temp symlink then rename over `current` — rename is atomic.
    let tmp = current.with_extension("tmp");
    let _ = std::fs::remove_file(&tmp);
    symlink(&target, &tmp).map_err(|e| format!("symlink {tmp:?}: {e}"))?;
    std::fs::rename(&tmp, current).map_err(|e| format!("swap {current:?}: {e}"))?;
    Ok(())
}

/// Fetch the latest release tag + this platform's asset download URL.
pub fn latest_release(http: &dyn Http) -> Result<(String, String), String> {
    let release = http.get_json(RELEASES_LATEST_API, "")?;
    let tag = release["tag_name"]
        .as_str()
        .ok_or("release JSON missing tag_name")?
        .to_string();
    let want = asset_name(CURRENT_TARGET);
    let assets = release["assets"]
        .as_array()
        .ok_or("release JSON missing assets")?;
    for a in assets {
        if a["name"].as_str() == Some(want.as_str()) {
            let url = a["browser_download_url"]
                .as_str()
                .ok_or("asset missing browser_download_url")?
                .to_string();
            return Ok((tag, url));
        }
    }
    Err(format!("no asset named {want} in latest release"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn asset_name_matches_release_convention() {
        assert_eq!(
            asset_name("x86_64-apple-darwin"),
            "serval-x86_64-apple-darwin.tar.gz"
        );
    }

    #[test]
    fn needs_update_compares_tags() {
        assert!(needs_update("v0.1.0", "v0.2.0"));
        assert!(!needs_update("v0.2.0", "v0.2.0"));
    }

    #[test]
    fn latest_release_finds_matching_asset() {
        struct FakeHttp;
        impl Http for FakeHttp {
            fn get_json(&self, _u: &str, _b: &str) -> Result<serde_json::Value, String> {
                Ok(serde_json::json!({
                    "tag_name": "v9.9.9",
                    "assets": [
                        { "name": asset_name(CURRENT_TARGET),
                          "browser_download_url": "https://example.com/x.tar.gz" }
                    ]
                }))
            }
        }
        let (tag, url) = latest_release(&FakeHttp).unwrap();
        assert_eq!(tag, "v9.9.9");
        assert_eq!(url, "https://example.com/x.tar.gz");
    }

    #[cfg(unix)]
    #[test]
    fn repoint_current_points_symlink_at_version() {
        let dir = tempdir().unwrap();
        let versions = dir.path().join("versions");
        std::fs::create_dir_all(versions.join("v1.0.0")).unwrap();
        let current = dir.path().join("current");
        repoint_current(&versions, &current, "v1.0.0").unwrap();
        assert_eq!(
            std::fs::read_link(&current).unwrap(),
            versions.join("v1.0.0")
        );
        // Repoint again — must overwrite atomically.
        std::fs::create_dir_all(versions.join("v1.1.0")).unwrap();
        repoint_current(&versions, &current, "v1.1.0").unwrap();
        assert_eq!(
            std::fs::read_link(&current).unwrap(),
            versions.join("v1.1.0")
        );
    }

    #[test]
    fn extract_tar_gz_roundtrip() {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;
        // Build a tiny .tar.gz containing one file "hello".
        let mut tar_buf = Vec::new();
        {
            let mut b = tar::Builder::new(&mut tar_buf);
            let data = b"hi";
            let mut h = tar::Header::new_gnu();
            h.set_size(data.len() as u64);
            h.set_mode(0o644);
            h.set_cksum();
            b.append_data(&mut h, "hello", &data[..]).unwrap();
            b.finish().unwrap();
        }
        let mut gz = GzEncoder::new(Vec::new(), Compression::default());
        gz.write_all(&tar_buf).unwrap();
        let bytes = gz.finish().unwrap();

        let dir = tempdir().unwrap();
        extract_tar_gz(&bytes, dir.path()).unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.path().join("hello")).unwrap(),
            "hi"
        );
    }
}
