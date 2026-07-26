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

/// The provider block embedded at compile time — used as a last-resort fallback.
pub fn default_provider() -> serde_json::Value {
    const RAW: &str = include_str!("../assets/default-bundle/opencode.jsonc");
    let full: serde_json::Value =
        serde_json::from_str(RAW).expect("embedded default opencode.jsonc must be valid JSON");
    full["provider"][crate::constants::PROVIDER_KEY].clone()
}

/// Resolve the provider config without ever failing, and always return a JSON object:
/// Worker (if it returns an object) → cache (if an object) → embedded default.
pub fn resolve_config(
    http: &dyn Http,
    worker_url: &str,
    token: &str,
    cached: Option<&serde_json::Value>,
) -> (serde_json::Value, Option<String>) {
    match fetch_config(http, worker_url, token) {
        Ok(fc) if fc.provider.is_object() => (fc.provider, Some(fc.email)),
        Ok(_) => {
            eprintln!(
                "serval: Worker returned a non-object provider; using {} config",
                fallback_source(cached)
            );
            (fallback_provider(cached), None)
        }
        Err(e) => {
            eprintln!("serval: using {} config ({e})", fallback_source(cached));
            (fallback_provider(cached), None)
        }
    }
}

fn fallback_source(cached: Option<&serde_json::Value>) -> &'static str {
    match cached {
        Some(v) if v.is_object() => "cached",
        _ => "default",
    }
}

fn fallback_provider(cached: Option<&serde_json::Value>) -> serde_json::Value {
    match cached {
        Some(v) if v.is_object() => v.clone(),
        _ => default_provider(),
    }
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

    struct FailingHttp;
    impl Http for FailingHttp {
        fn get_json(&self, _u: &str, _b: &str) -> Result<serde_json::Value, String> {
            Err("network down".to_string())
        }
    }

    #[test]
    fn default_provider_has_three_tiers() {
        let p = default_provider();
        assert_eq!(p["name"], "ServalAI");
        assert!(p["models"]["dynamic/balanced"]["limit"]["context"].is_number());
    }

    #[test]
    fn resolve_falls_back_to_cache_then_default() {
        // Worker fails, cache present → use cache.
        let cache = serde_json::json!({ "name": "cached" });
        let (p, email) = resolve_config(&FailingHttp, "https://w.dev", "t", Some(&cache));
        assert_eq!(p["name"], "cached");
        assert!(email.is_none());
        // Worker fails, no cache → embedded default.
        let (p2, _) = resolve_config(&FailingHttp, "https://w.dev", "t", None);
        assert_eq!(p2["name"], "ServalAI");
    }

    #[test]
    fn resolve_falls_back_when_worker_provider_not_object() {
        // Worker "succeeds" but returns provider as a non-object → must not be used;
        // with no cache we get the embedded default (an object), never a panic downstream.
        struct StringProviderHttp;
        impl Http for StringProviderHttp {
            fn get_json(&self, _u: &str, _b: &str) -> Result<serde_json::Value, String> {
                Ok(
                    serde_json::json!({ "email": "x@y.com", "models": [], "provider": "oops-not-an-object" }),
                )
            }
        }
        let (p, email) = resolve_config(&StringProviderHttp, "https://w.dev", "t", None);
        assert!(p.is_object());
        assert_eq!(p["name"], "ServalAI");
        assert!(email.is_none());
    }
}
