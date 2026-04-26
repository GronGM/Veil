#![forbid(unsafe_code)]

//! Manifest and provider input skeleton for Veil.

/// Minimal typed manifest input for early dry-run wiring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderManifest {
    pub provider_name: String,
    pub profile_name: String,
    pub preferred_backend: String,
}

impl ProviderManifest {
    /// Build a small demo manifest for the public dry-run path.
    pub fn demo() -> Self {
        Self {
            provider_name: "demo-provider".to_string(),
            profile_name: "stable-eu".to_string(),
            preferred_backend: "xray-core".to_string(),
        }
    }
}
