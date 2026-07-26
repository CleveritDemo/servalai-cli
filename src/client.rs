//! Talks to the Worker's read-only `/cli/config` route to get the user's
//! ServalAI provider config. Transport is behind the `Http` trait so tests
//! inject a fake and no network is needed.

#![allow(dead_code)]

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FetchedConfig {
    pub email: String,
    #[serde(default)]
    pub models: Vec<String>,
    pub provider: serde_json::Value,
}

pub trait Http {
    fn get_json(&self, url: &str, bearer: &str) -> Result<serde_json::Value, String>;
}

pub struct UreqHttp;

impl Http for UreqHttp {
    fn get_json(&self, url: &str, bearer: &str) -> Result<serde_json::Value, String> {
        let resp = ureq::get(url)
            .set("Authorization", &format!("Bearer {bearer}"))
            .set(
                "User-Agent",
                &format!("serval/{}", env!("CARGO_PKG_VERSION")),
            )
            .call()
            .map_err(|e| format!("request to {url} failed: {e}"))?;
        resp.into_json::<serde_json::Value>()
            .map_err(|e| format!("invalid JSON from {url}: {e}"))
    }
}

pub fn fetch_config(
    http: &dyn Http,
    worker_url: &str,
    token: &str,
) -> Result<FetchedConfig, String> {
    let url = format!("{}/cli/config", worker_url.trim_end_matches('/'));
    let value = http.get_json(&url, token)?;
    serde_json::from_value::<FetchedConfig>(value)
        .map_err(|e| format!("unexpected /cli/config shape: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeHttp {
        body: serde_json::Value,
    }
    impl Http for FakeHttp {
        fn get_json(&self, url: &str, bearer: &str) -> Result<serde_json::Value, String> {
            assert!(url.ends_with("/cli/config"));
            assert_eq!(bearer, "aig_token");
            Ok(self.body.clone())
        }
    }

    #[test]
    fn parses_well_formed_config() {
        let http = FakeHttp {
            body: serde_json::json!({
                "email": "dev@cleveritgroup.com",
                "models": ["dynamic/balanced", "dynamic/light"],
                "provider": { "npm": "@ai-sdk/openai-compatible", "name": "ServalAI" }
            }),
        };
        let cfg = fetch_config(&http, "https://w.example.dev/", "aig_token").unwrap();
        assert_eq!(cfg.email, "dev@cleveritgroup.com");
        assert_eq!(cfg.models.len(), 2);
        assert_eq!(cfg.provider["name"], "ServalAI");
    }

    #[test]
    fn errors_on_bad_shape() {
        let http = FakeHttp {
            body: serde_json::json!({ "nope": true }),
        };
        let err = fetch_config(&http, "https://w.example.dev", "aig_token").unwrap_err();
        assert!(err.contains("unexpected /cli/config shape"));
    }
}
