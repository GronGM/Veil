//! Explainable route selection skeleton for Veil.

use serde::{Deserialize, Serialize};
use veil_manifest::{Endpoint, ProviderManifest};

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
pub struct RouteSelection {
    pub selected: RouteCandidate,
    pub candidates: Vec<RouteCandidate>,
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
            "selected={} score={} retry_budget={} order={}",
            self.selected.endpoint.id,
            self.selected.score.total_score,
            self.candidates.len(),
            ordered_ids
        )
    }
}

pub fn select_best_route(
    manifest: &ProviderManifest,
    context: &SelectionContext,
) -> Option<RouteSelection> {
    let mut candidates = manifest
        .endpoints
        .iter()
        .cloned()
        .map(|endpoint| RouteCandidate {
            score: score_endpoint(&endpoint, context),
            endpoint,
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| {
        right
            .score
            .total_score
            .cmp(&left.score.total_score)
            .then_with(|| left.endpoint.id.cmp(&right.endpoint.id))
    });

    let max_candidates = context.retry_budget.max_candidates.max(1);
    candidates.truncate(max_candidates);

    let selected = candidates.first().cloned()?;
    Some(RouteSelection { selected, candidates })
}

fn score_endpoint(endpoint: &Endpoint, context: &SelectionContext) -> ScoreBreakdown {
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

    if last_known_good_bonus {
        total_score += 3;
        reasons.push("last-known-good".to_string());
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
    use veil_manifest::demo_provider_manifest;

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
            },
        )
        .expect("selection should exist");

        assert!(selection.summary().contains("selected=edge-1"));
    }

    #[test]
    fn locally_disabled_endpoint_is_penalized() {
        let manifest = demo_provider_manifest();
        let selection = select_best_route(
            &manifest,
            &SelectionContext {
                client_platform: "linux".to_string(),
                last_known_good_endpoint_id: None,
                retry_budget: RetryBudget { max_candidates: 1 },
                runtime_state: RouteRuntimeState {
                    endpoints: vec![EndpointRuntimeState {
                        endpoint_id: "edge-1".to_string(),
                        cooldown: CooldownState {
                            cooling_down: false,
                            remaining_seconds: 0,
                        },
                        locally_disabled: true,
                        health_score: 0,
                        last_failure_bucket: Some(FailureBucket::TransportRejected),
                        reenable: ReenableState {
                            pending: false,
                            ready: false,
                            backoff_seconds: 0,
                        },
                    }],
                },
            },
        )
        .expect("selection should exist");

        assert!(selection.selected.score.locally_disabled);
        assert!(selection.selected.score.total_score < 0);
        assert_eq!(
            selection.selected.score.last_failure_bucket,
            Some(FailureBucket::TransportRejected)
        );
    }

    #[test]
    fn pending_reenable_is_explained_in_score_breakdown() {
        let manifest = demo_provider_manifest();
        let selection = select_best_route(
            &manifest,
            &SelectionContext {
                client_platform: "linux".to_string(),
                last_known_good_endpoint_id: None,
                retry_budget: RetryBudget { max_candidates: 1 },
                runtime_state: RouteRuntimeState {
                    endpoints: vec![EndpointRuntimeState {
                        endpoint_id: "edge-1".to_string(),
                        cooldown: CooldownState {
                            cooling_down: false,
                            remaining_seconds: 0,
                        },
                        locally_disabled: false,
                        health_score: 0,
                        last_failure_bucket: Some(FailureBucket::TlsInterference),
                        reenable: ReenableState {
                            pending: true,
                            ready: false,
                            backoff_seconds: 120,
                        },
                    }],
                },
            },
        )
        .expect("selection should exist");

        assert!(selection.selected.score.reenable_pending);
        assert!(!selection.selected.score.reenable_ready);
        assert_eq!(selection.selected.score.reenable_backoff_seconds, 120);
        assert!(
            selection
                .selected
                .score
                .reasons
                .iter()
                .any(|reason| reason == "reenable-backoff=120s")
        );
    }
}
