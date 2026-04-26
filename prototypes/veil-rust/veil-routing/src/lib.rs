#![forbid(unsafe_code)]

//! Routing skeleton for Veil.

use veil_manifest::ProviderManifest;
use veil_policy::RoutePolicy;

/// Minimal backend candidate used for early selection flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendCandidate {
    pub backend_name: &'static str,
}

/// Minimal backend selection result for early routing flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendSelection {
    pub backend_name: &'static str,
    pub summary: String,
}

/// Select a backend from the available candidates using manifest preference and policy allowlist.
pub fn select_backend(
    candidates: &[BackendCandidate],
    manifest: &ProviderManifest,
    policy: &RoutePolicy,
) -> Option<BackendSelection> {
    let preferred = candidates
        .iter()
        .find(|candidate| candidate.backend_name == manifest.preferred_backend && candidate.backend_name == policy.allow_backend)
        .or_else(|| {
            candidates
                .iter()
                .find(|candidate| candidate.backend_name == policy.allow_backend)
        })?;

    let summary = if preferred.backend_name == manifest.preferred_backend {
        format!(
            "routing selected preferred backend '{}' for provider '{}' profile '{}'",
            preferred.backend_name, manifest.provider_name, manifest.profile_name
        )
    } else {
        format!(
            "routing selected backend '{}' because policy requires it instead of preferred '{}'",
            preferred.backend_name, manifest.preferred_backend
        )
    };

    Some(BackendSelection {
        backend_name: preferred.backend_name,
        summary,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_backend_prefers_manifest_backend_when_allowed() {
        let manifest = ProviderManifest::demo();
        let policy = RoutePolicy::demo();
        let candidates = [
            BackendCandidate { backend_name: "xray-core" },
            BackendCandidate { backend_name: "mock-backend" },
        ];

        let selection = select_backend(&candidates, &manifest, &policy).expect("selection");
        assert_eq!(selection.backend_name, "xray-core");
        assert!(selection.summary.contains("routing selected preferred backend 'xray-core'"));
    }

    #[test]
    fn select_backend_falls_back_to_policy_backend() {
        let manifest = ProviderManifest::demo();
        let policy = RoutePolicy::mismatch_demo();
        let candidates = [
            BackendCandidate { backend_name: "xray-core" },
            BackendCandidate { backend_name: "mock-backend" },
        ];

        let selection = select_backend(&candidates, &manifest, &policy).expect("selection");
        assert_eq!(selection.backend_name, "mock-backend");
        assert!(selection.summary.contains("policy requires it instead of preferred 'xray-core'"));
    }
}
