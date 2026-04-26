#![forbid(unsafe_code)]

//! Routing skeleton for Veil.

use veil_dry_run::{DryRunOutcome, RoutingCandidateReport, RoutingEligibilityReason};
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

/// Dry-run-derived eligibility snapshot used by routing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendEligibility {
    pub backend_name: &'static str,
    pub outcome: DryRunOutcome,
}

/// Routing result that can explain why no backend was selected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EligibilityRoutingResult {
    pub candidates: Vec<RoutingCandidateReport>,
    pub selection: Option<BackendSelection>,
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
        .find(|candidate| {
            candidate.backend_name == manifest.preferred_backend
                && candidate.backend_name == policy.allow_backend
        })
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

/// Select a backend only if dry-run eligibility says that backend is routable.
pub fn select_backend_with_eligibility(
    candidates: &[BackendCandidate],
    manifest: &ProviderManifest,
    policy: &RoutePolicy,
    eligibility: &[BackendEligibility],
) -> EligibilityRoutingResult {
    let selected = select_backend(candidates, manifest, policy);
    let mut classified = candidates
        .iter()
        .map(|candidate| classify_candidate(candidate, selected.as_ref(), eligibility))
        .collect::<Vec<_>>();

    match selected {
        Some(selection) => {
            let matched = eligibility
                .iter()
                .find(|candidate| candidate.backend_name == selection.backend_name);

            match matched {
                Some(candidate) if candidate.outcome == DryRunOutcome::Allowed => {
                    let summary = format!(
                        "{} and eligibility confirmed outcome '{}'",
                        selection.summary,
                        candidate.outcome.as_str()
                    );

                    EligibilityRoutingResult {
                        candidates: classified,
                        selection: Some(BackendSelection {
                            backend_name: selection.backend_name,
                            summary,
                        }),
                        summary: format!(
                            "routing accepted backend '{}' because dry-run outcome is '{}'",
                            selection.backend_name,
                            candidate.outcome.as_str()
                        ),
                    }
                }
                Some(candidate) => EligibilityRoutingResult {
                    candidates: classified,
                    selection: None,
                    summary: format!(
                        "routing rejected backend '{}' because dry-run outcome is '{}'",
                        selection.backend_name,
                        candidate.outcome.as_str()
                    ),
                },
                None => EligibilityRoutingResult {
                    candidates: classified,
                    selection: None,
                    summary: format!(
                        "routing could not confirm eligibility for backend '{}'",
                        selection.backend_name
                    ),
                },
            }
        }
        None => {
            classified.iter_mut().for_each(|candidate| {
                candidate.eligible = false;
                candidate.reason = RoutingEligibilityReason::NotSelectedByPolicy;
            });

            EligibilityRoutingResult {
                selection: None,
                candidates: classified,
                summary: format!(
                    "routing found no backend matching policy allow_backend '{}'",
                    policy.allow_backend
                ),
            }
        }
    }
}

fn classify_candidate(
    candidate: &BackendCandidate,
    selected: Option<&BackendSelection>,
    eligibility: &[BackendEligibility],
) -> RoutingCandidateReport {
    let matched = eligibility
        .iter()
        .find(|item| item.backend_name == candidate.backend_name);

    match (selected, matched) {
        (Some(selection), Some(eligibility_item))
            if selection.backend_name == candidate.backend_name
                && eligibility_item.outcome == DryRunOutcome::Allowed =>
        {
            RoutingCandidateReport {
                backend_name: candidate.backend_name,
                eligible: true,
                reason: RoutingEligibilityReason::Eligible,
                summary: format!(
                    "candidate '{}' is eligible because dry-run outcome is '{}'",
                    candidate.backend_name,
                    eligibility_item.outcome.as_str()
                ),
            }
        }
        (Some(selection), Some(eligibility_item))
            if selection.backend_name == candidate.backend_name =>
        {
            RoutingCandidateReport {
                backend_name: candidate.backend_name,
                eligible: false,
                reason: RoutingEligibilityReason::RejectedByOutcome,
                summary: format!(
                    "candidate '{}' is ineligible because dry-run outcome is '{}'",
                    candidate.backend_name,
                    eligibility_item.outcome.as_str()
                ),
            }
        }
        (_, Some(_)) => RoutingCandidateReport {
            backend_name: candidate.backend_name,
            eligible: false,
            reason: RoutingEligibilityReason::NotSelectedByPolicy,
            summary: format!(
                "candidate '{}' was not selected by policy routing",
                candidate.backend_name
            ),
        },
        (_, None) => RoutingCandidateReport {
            backend_name: candidate.backend_name,
            eligible: false,
            reason: RoutingEligibilityReason::MissingEligibility,
            summary: format!(
                "candidate '{}' is ineligible because no eligibility snapshot was available",
                candidate.backend_name
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_backend_prefers_manifest_backend_when_allowed() {
        let manifest = ProviderManifest::demo();
        let policy = RoutePolicy::demo();
        let candidates = [
            BackendCandidate {
                backend_name: "xray-core",
            },
            BackendCandidate {
                backend_name: "mock-backend",
            },
        ];

        let selection = select_backend(&candidates, &manifest, &policy).expect("selection");
        assert_eq!(selection.backend_name, "xray-core");
        assert!(selection
            .summary
            .contains("routing selected preferred backend 'xray-core'"));
    }

    #[test]
    fn select_backend_falls_back_to_policy_backend() {
        let manifest = ProviderManifest::demo();
        let policy = RoutePolicy::mismatch_demo();
        let candidates = [
            BackendCandidate {
                backend_name: "xray-core",
            },
            BackendCandidate {
                backend_name: "mock-backend",
            },
        ];

        let selection = select_backend(&candidates, &manifest, &policy).expect("selection");
        assert_eq!(selection.backend_name, "mock-backend");
        assert!(selection
            .summary
            .contains("policy requires it instead of preferred 'xray-core'"));
    }

    #[test]
    fn eligibility_selection_accepts_allowed_backend() {
        let manifest = ProviderManifest::demo();
        let policy = RoutePolicy::demo();
        let candidates = [
            BackendCandidate {
                backend_name: "xray-core",
            },
            BackendCandidate {
                backend_name: "mock-backend",
            },
        ];
        let eligibility = [
            BackendEligibility {
                backend_name: "xray-core",
                outcome: DryRunOutcome::Allowed,
            },
            BackendEligibility {
                backend_name: "mock-backend",
                outcome: DryRunOutcome::BlockedByAdapter,
            },
        ];

        let result =
            select_backend_with_eligibility(&candidates, &manifest, &policy, &eligibility);
        assert_eq!(result.candidates.len(), 2);
        let selection = result.selection.expect("selection");
        assert_eq!(selection.backend_name, "xray-core");
        assert!(result.summary.contains("accepted backend 'xray-core'"));
        assert_eq!(
            result.candidates[0].reason,
            RoutingEligibilityReason::Eligible
        );
    }

    #[test]
    fn eligibility_selection_rejects_blocked_backend() {
        let manifest = ProviderManifest::demo();
        let policy = RoutePolicy::mismatch_demo();
        let candidates = [
            BackendCandidate {
                backend_name: "xray-core",
            },
            BackendCandidate {
                backend_name: "mock-backend",
            },
        ];
        let eligibility = [
            BackendEligibility {
                backend_name: "xray-core",
                outcome: DryRunOutcome::Allowed,
            },
            BackendEligibility {
                backend_name: "mock-backend",
                outcome: DryRunOutcome::BlockedByAdapter,
            },
        ];

        let result =
            select_backend_with_eligibility(&candidates, &manifest, &policy, &eligibility);
        assert!(result.selection.is_none());
        assert!(result.summary.contains("rejected backend 'mock-backend'"));
        assert!(result.summary.contains("blocked_by_adapter"));
        assert_eq!(
            result.candidates[1].reason,
            RoutingEligibilityReason::RejectedByOutcome
        );
        assert_eq!(
            result.candidates[0].reason,
            RoutingEligibilityReason::NotSelectedByPolicy
        );
    }

    #[test]
    fn eligibility_classification_marks_missing_snapshot() {
        let manifest = ProviderManifest::demo();
        let policy = RoutePolicy::demo();
        let candidates = [
            BackendCandidate {
                backend_name: "xray-core",
            },
            BackendCandidate {
                backend_name: "mock-backend",
            },
        ];
        let eligibility = [BackendEligibility {
            backend_name: "mock-backend",
            outcome: DryRunOutcome::Allowed,
        }];

        let result =
            select_backend_with_eligibility(&candidates, &manifest, &policy, &eligibility);
        assert!(result.selection.is_none());
        assert_eq!(
            result.candidates[0].reason,
            RoutingEligibilityReason::MissingEligibility
        );
    }
}
