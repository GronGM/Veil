//! Explainable route selection skeleton for Veil.

use serde::{Deserialize, Serialize};
use veil_manifest::{Endpoint, ProviderManifest};
use veil_policy::{
    evaluate_route_policy, FallbackMode, PolicyEvaluation, PolicyEvaluationInput,
    RouteSelectionPolicy,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryBudget {
    pub max_candidates: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FailureBucket {
    DnsInterference,
    TlsInterference,
    EndpointDown,
    TransportRejected,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CooldownState {
    pub cooling_down: bool,
    pub remaining_seconds: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReenableState {
    pub pending: bool,
    pub ready: bool,
    pub backoff_seconds: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndpointRuntimeState {
    pub endpoint_id: String,
    pub cooldown: CooldownState,
    pub locally_disabled: bool,
    pub health_score: i32,
    pub last_failure_bucket: Option<FailureBucket>,
    pub reenable: ReenableState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RouteRuntimeState {
    pub endpoints: Vec<EndpointRuntimeState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionContext {
    pub client_platform: String,
    pub last_known_good_endpoint_id: Option<String>,
    pub retry_budget: RetryBudget,
    pub runtime_state: RouteRuntimeState,
    pub policy: RouteSelectionPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    pub endpoint_id: String,
    pub client_platform_match: bool,
    pub dataplane_preferred: bool,
    pub last_known_good_bonus: bool,
    pub locally_disabled: bool,
    pub cooling_down: bool,
    pub cooldown_remaining_seconds: u32,
    pub health_score: i32,
    pub reenable_pending: bool,
    pub reenable_ready: bool,
    pub reenable_backoff_seconds: u32,
    pub last_failure_bucket: Option<FailureBucket>,
    pub total_score: i32,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteCandidate {
    pub endpoint: Endpoint,
    pub score: ScoreBreakdown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RejectedRouteCandidate {
    pub endpoint_id: String,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteSelection {
    pub selected: RouteCandidate,
    pub candidates: Vec<RouteCandidate>,
    pub rejected: Vec<RejectedRouteCandidate>,
    pub fallback_triggered: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteSelectionError {
    pub detail: String,
    pub rejected: Vec<RejectedRouteCandidate>,
}

impl RouteSelection {
    pub fn summary(&self) -> String {
        let ordered_ids = self
            .candidates
            .iter()
            .map(|candidate| candidate.endpoint.id.clone())
            .collect::<Vec<_>>()
            .join("->");
        format!(
            "selected={} score={} retry_budget={} order={} rejected={} fallback={}",
            self.selected.endpoint.id,
            self.selected.score.total_score,
            self.candidates.len(),
            ordered_ids,
            self.rejected.len(),
            self.fallback_triggered
        )
    }
}

pub fn select_best_route(
    manifest: &ProviderManifest,
    context: &SelectionContext,
) -> Result<RouteSelection, RouteSelectionError> {
    let mut candidates = Vec::new();
    let mut rejected = Vec::new();
    let mut fallback_triggered = false;

    for endpoint in &manifest.endpoints {
        let runtime_state = context
            .runtime_state
            .endpoints
            .iter()
            .find(|state| state.endpoint_id == endpoint.id);
        let evaluation = evaluate_route_policy(
            &context.policy,
            &PolicyEvaluationInput {
                endpoint_id: endpoint.id.clone(),
                backend_name: endpoint
                    .dataplane
                    .clone()
                    .unwrap_or_else(|| "xray-core".to_string()),
                transport: endpoint.transport.clone(),
                region: endpoint.region.clone(),
                is_known_good: context.last_known_good_endpoint_id.as_deref()
                    == Some(endpoint.id.as_str()),
                cooling_down: runtime_state
                    .map(|state| state.cooldown.cooling_down)
                    .unwrap_or(false),
                locally_disabled: runtime_state
                    .map(|state| state.locally_disabled)
                    .unwrap_or(false),
            },
        );

        if evaluation.is_hard_rejected() {
            rejected.push(RejectedRouteCandidate {
                endpoint_id: endpoint.id.clone(),
                reasons: evaluation.hard_rejections,
            });
            continue;
        }

        if evaluation.requires_fallback() {
            let is_known_good = context.last_known_good_endpoint_id.as_deref()
                == Some(endpoint.id.as_str());
            let allow_known_good_fallback =
                context.policy.fallback == FallbackMode::AllowKnownGood && is_known_good;

            if !allow_known_good_fallback {
                rejected.push(RejectedRouteCandidate {
                    endpoint_id: endpoint.id.clone(),
                    reasons: evaluation.soft_rejections,
                });
                continue;
            }

            fallback_triggered = true;
        }

        candidates.push(RouteCandidate {
            score: score_endpoint(endpoint, context, evaluation),
            endpoint: endpoint.clone(),
        });
    }

    candidates.sort_by(|left, right| {
        right
            .score
            .total_score
            .cmp(&left.score.total_score)
            .then_with(|| left.endpoint.id.cmp(&right.endpoint.id))
    });

    let max_candidates = context.retry_budget.max_candidates.max(1);
    candidates.truncate(max_candidates);

    let selected = candidates
        .first()
        .cloned()
        .ok_or_else(|| RouteSelectionError {
            detail: "no route satisfied the active policy and runtime constraints".to_string(),
            rejected: rejected.clone(),
        })?;

    Ok(RouteSelection {
        selected,
        candidates,
        rejected,
        fallback_triggered,
    })
}

fn score_endpoint(
    endpoint: &Endpoint,
    context: &SelectionContext,
    evaluation: PolicyEvaluation,
) -> ScoreBreakdown {
    let runtime_state = context
        .runtime_state
        .endpoints
        .iter()
        .find(|state| state.endpoint_id == endpoint.id);
    let client_platform_match = endpoint.supported_client_platforms.is_empty()
        || endpoint
            .supported_client_platforms
            .iter()
            .any(|platform| platform == &context.client_platform);
    let dataplane_preferred = endpoint.dataplane.as_deref() == Some("xray-core");
    let last_known_good_bonus =
        context.last_known_good_endpoint_id.as_deref() == Some(endpoint.id.as_str());
    let locally_disabled = runtime_state.map(|state| state.locally_disabled).unwrap_or(false);
    let cooldown_remaining_seconds = runtime_state
        .map(|state| state.cooldown.remaining_seconds)
        .unwrap_or(0);
    let cooling_down = runtime_state
        .map(|state| state.cooldown.cooling_down)
        .unwrap_or(false);
    let health_score = runtime_state.map(|state| state.health_score).unwrap_or(0);
    let reenable_pending = runtime_state
        .map(|state| state.reenable.pending)
        .unwrap_or(false);
    let reenable_ready = runtime_state
        .map(|state| state.reenable.ready)
        .unwrap_or(false);
    let reenable_backoff_seconds = runtime_state
        .map(|state| state.reenable.backoff_seconds)
        .unwrap_or(0);
    let last_failure_bucket = runtime_state.and_then(|state| state.last_failure_bucket.clone());

    let mut total_score = 0;
    let mut reasons = Vec::new();

    if client_platform_match {
        total_score += 10;
        reasons.push("client-platform-match".to_string());
    } else {
        reasons.push("client-platform-mismatch".to_string());
    }

    if dataplane_preferred {
        total_score += 5;
        reasons.push("preferred-dataplane".to_string());
    }

    total_score += health_score;
    reasons.push(format!("health-score={health_score}"));

    if cooling_down {
        total_score -= 20;
        reasons.push(format!("cooldown-active={cooldown_remaining_seconds}s"));
    }

    if locally_disabled {
        total_score -= 50;
        reasons.push("locally-disabled".to_string());
    }

    if reenable_pending {
        if reenable_ready {
            total_score -= 5;
            reasons.push("reenable-probe-ready".to_string());
        } else {
            total_score -= 15;
            reasons.push(format!("reenable-backoff={}s", reenable_backoff_seconds));
        }
    }

    if let Some(bucket) = &last_failure_bucket {
        reasons.push(format!("last-failure-bucket={}", failure_bucket_name(bucket)));
    }

    if last_known_good_bonus {
        reasons.push("last-known-good".to_string());
    }

    total_score += evaluation.score_adjustment;
    reasons.extend(evaluation.reasons);

    ScoreBreakdown {
        endpoint_id: endpoint.id.clone(),
        client_platform_match,
        dataplane_preferred,
        last_known_good_bonus,
        locally_disabled,
        cooling_down,
        cooldown_remaining_seconds,
        health_score,
        reenable_pending,
        reenable_ready,
        reenable_backoff_seconds,
        last_failure_bucket,
        total_score,
        reasons,
    }
}

fn failure_bucket_name(bucket: &FailureBucket) -> &'static str {
    match bucket {
        FailureBucket::DnsInterference => "dns-interference",
        FailureBucket::TlsInterference => "tls-interference",
        FailureBucket::EndpointDown => "endpoint-down",
        FailureBucket::TransportRejected => "transport-rejected",
        FailureBucket::Unknown => "unknown",
    }
}

pub fn demo_runtime_state() -> RouteRuntimeState {
    RouteRuntimeState {
        endpoints: vec![EndpointRuntimeState {
            endpoint_id: "edge-1".to_string(),
            cooldown: CooldownState {
                cooling_down: false,
                remaining_seconds: 0,
            },
            locally_disabled: false,
            health_score: 2,
            last_failure_bucket: None,
            reenable: ReenableState {
                pending: false,
                ready: false,
                backoff_seconds: 0,
            },
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use veil_manifest::{demo_provider_manifest, Endpoint};

    #[test]
    fn selection_prefers_matching_xray_candidate() {
        let manifest = demo_provider_manifest();
        let selection = select_best_route(
            &manifest,
            &SelectionContext {
                client_platform: "linux".to_string(),
                last_known_good_endpoint_id: None,
                retry_budget: RetryBudget { max_candidates: 2 },
                runtime_state: demo_runtime_state(),
                policy: RouteSelectionPolicy::default(),
            },
        )
        .expect("selection should exist");

        assert_eq!(selection.selected.endpoint.id, "edge-1");
        assert!(selection.selected.score.client_platform_match);
        assert!(selection.selected.score.dataplane_preferred);
        assert_eq!(selection.selected.score.health_score, 2);
        assert!(!selection.selected.score.reenable_pending);
    }

    #[test]
    fn summary_mentions_selected_endpoint() {
        let manifest = demo_provider_manifest();
        let selection = select_best_route(
            &manifest,
            &SelectionContext {
                client_platform: "linux".to_string(),
                last_known_good_endpoint_id: Some("edge-1".to_string()),
                retry_budget: RetryBudget { max_candidates: 2 },
                runtime_state: demo_runtime_state(),
                policy: RouteSelectionPolicy::default(),
            },
        )
        .expect("selection should exist");

        assert!(selection.summary().contains("selected=edge-1"));
    }

    #[test]
    fn denylisted_backend_is_not_selected() {
        let manifest = demo_provider_manifest();
        let mut policy = RouteSelectionPolicy::default();
        policy.backends.deny = vec!["xray-core".to_string()];

        let error = select_best_route(
            &manifest,
            &SelectionContext {
                client_platform: "linux".to_string(),
                last_known_good_endpoint_id: None,
                retry_budget: RetryBudget { max_candidates: 1 },
                runtime_state: demo_runtime_state(),
                policy,
            },
        )
        .expect_err("selection should fail");

        assert!(
            error
                .rejected
                .iter()
                .flat_map(|candidate| candidate.reasons.iter())
                .any(|reason| reason.contains("denylisted"))
        );
    }

    #[test]
    fn known_good_influences_selection() {
        let mut manifest = demo_provider_manifest();
        manifest.endpoints.push(Endpoint {
            id: "edge-2".to_string(),
            host: "198.51.100.20".to_string(),
            port: 443,
            transport: "https".to_string(),
            region: "us-east".to_string(),
            dataplane: Some("xray-core".to_string()),
            supported_client_platforms: vec!["linux".to_string()],
            logical_server: Some("edge".to_string()),
            provider_profile_schema_version: Some(1),
            xray: manifest.endpoints[0].xray.clone(),
        });

        let selection = select_best_route(
            &manifest,
            &SelectionContext {
                client_platform: "linux".to_string(),
                last_known_good_endpoint_id: Some("edge-2".to_string()),
                retry_budget: RetryBudget { max_candidates: 2 },
                runtime_state: RouteRuntimeState {
                    endpoints: vec![
                        EndpointRuntimeState {
                            endpoint_id: "edge-1".to_string(),
                            cooldown: CooldownState {
                                cooling_down: false,
                                remaining_seconds: 0,
                            },
                            locally_disabled: false,
                            health_score: 2,
                            last_failure_bucket: None,
                            reenable: ReenableState {
                                pending: false,
                                ready: false,
                                backoff_seconds: 0,
                            },
                        },
                        EndpointRuntimeState {
                            endpoint_id: "edge-2".to_string(),
                            cooldown: CooldownState {
                                cooling_down: false,
                                remaining_seconds: 0,
                            },
                            locally_disabled: false,
                            health_score: 0,
                            last_failure_bucket: None,
                            reenable: ReenableState {
                                pending: false,
                                ready: false,
                                backoff_seconds: 0,
                            },
                        },
                    ],
                },
                policy: RouteSelectionPolicy::default(),
            },
        )
        .expect("selection should exist");

        assert_eq!(selection.selected.endpoint.id, "edge-2");
        assert!(
            selection
                .selected
                .score
                .reasons
                .iter()
                .any(|reason| reason == "known-good-bonus=3")
        );
    }

    #[test]
    fn fallback_applies_predictably_for_known_good_endpoint() {
        let manifest = demo_provider_manifest();
        let selection = select_best_route(
            &manifest,
            &SelectionContext {
                client_platform: "linux".to_string(),
                last_known_good_endpoint_id: Some("edge-1".to_string()),
                retry_budget: RetryBudget { max_candidates: 1 },
                runtime_state: RouteRuntimeState {
                    endpoints: vec![EndpointRuntimeState {
                        endpoint_id: "edge-1".to_string(),
                        cooldown: CooldownState {
                            cooling_down: true,
                            remaining_seconds: 90,
                        },
                        locally_disabled: false,
                        health_score: 0,
                        last_failure_bucket: Some(FailureBucket::EndpointDown),
                        reenable: ReenableState {
                            pending: false,
                            ready: false,
                            backoff_seconds: 0,
                        },
                    }],
                },
                policy: RouteSelectionPolicy::default(),
            },
        )
        .expect("known-good fallback should allow selection");

        assert!(selection.fallback_triggered);
        assert_eq!(selection.selected.endpoint.id, "edge-1");
    }

    #[test]
    fn explainable_error_is_returned_when_everything_is_rejected() {
        let manifest = demo_provider_manifest();
        let mut policy = RouteSelectionPolicy::default();
        policy.transports.allow = vec!["quic".to_string()];

        let error = select_best_route(
            &manifest,
            &SelectionContext {
                client_platform: "linux".to_string(),
                last_known_good_endpoint_id: None,
                retry_budget: RetryBudget { max_candidates: 1 },
                runtime_state: demo_runtime_state(),
                policy,
            },
        )
        .expect_err("selection should fail");

        assert!(error.detail.contains("no route satisfied"));
        assert!(
            error
                .rejected
                .iter()
                .flat_map(|candidate| candidate.reasons.iter())
                .any(|reason| reason.contains("allowlist"))
        );
    }
}
