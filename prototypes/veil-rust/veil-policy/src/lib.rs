#![forbid(unsafe_code)]

//! Policy skeleton for Veil.

use veil_manifest::ProviderManifest;

/// Minimal policy model for early dry-run wiring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutePolicy {
    pub allow_backend: String,
    pub allow_fallback: bool,
}

/// Minimal decision summary returned by policy evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyDecision {
    pub allowed: bool,
    pub summary: String,
}

impl RoutePolicy {
    /// Build a small demo policy for the public dry-run path.
    pub fn demo() -> Self {
        Self {
            allow_backend: "xray-core".to_string(),
            allow_fallback: false,
        }
    }

    /// Build a demo policy mismatch scenario for CLI testing.
    pub fn mismatch_demo() -> Self {
        Self {
            allow_backend: "mock-backend".to_string(),
            allow_fallback: false,
        }
    }

    /// Evaluate the manifest against the current backend choice.
    pub fn evaluate(&self, manifest: &ProviderManifest, backend_name: &str) -> PolicyDecision {
        let allowed = backend_name == self.allow_backend;
        let summary = if allowed {
            format!(
                "policy allows backend '{}' for provider '{}' profile '{}'",
                backend_name, manifest.provider_name, manifest.profile_name
            )
        } else if self.allow_fallback {
            format!(
                "policy prefers '{}' but allows fallback from '{}'",
                self.allow_backend, backend_name
            )
        } else {
            format!(
                "policy blocks backend '{}' for provider '{}' profile '{}' because only '{}' is allowed",
                backend_name, manifest.provider_name, manifest.profile_name, self.allow_backend
            )
        };

        PolicyDecision { allowed, summary }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_policy_allows_xray_backend() {
        let manifest = ProviderManifest::demo();
        let policy = RoutePolicy::demo();
        let decision = policy.evaluate(&manifest, "xray-core");

        assert!(decision.allowed);
        assert!(decision.summary.contains("policy allows backend 'xray-core'"));
    }

    #[test]
    fn mismatch_policy_blocks_xray_backend() {
        let manifest = ProviderManifest::demo();
        let policy = RoutePolicy::mismatch_demo();
        let decision = policy.evaluate(&manifest, "xray-core");

        assert!(!decision.allowed);
        assert!(decision.summary.contains("policy blocks backend 'xray-core'"));
        assert!(decision.summary.contains("only 'mock-backend' is allowed"));
    }
}
