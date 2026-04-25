//! Typed policy skeleton for Veil.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncidentGuidance {
    pub severity: String,
    pub recommended_action: String,
}

impl Default for IncidentGuidance {
    fn default() -> Self {
        Self {
            severity: "info".to_string(),
            recommended_action: "No immediate action required.".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSupportAssessment {
    pub tier: String,
    pub in_mvp_scope: bool,
}

impl RuntimeSupportAssessment {
    pub fn mvp_supported() -> Self {
        Self {
            tier: "mvp-supported".to_string(),
            in_mvp_scope: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RoutePreference {
    Balanced,
    PreferLatency,
    PreferStability,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FallbackMode {
    Strict,
    AllowKnownGood,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkCostPolicy {
    pub allow_metered_networks: bool,
    pub allow_expensive_networks: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RegionPolicy {
    #[serde(default)]
    pub preferred: Vec<String>,
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BackendPolicy {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TransportPolicy {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryBudgetPolicy {
    pub max_candidates: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CooldownPolicy {
    pub forbid_active_cooldown: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnownGoodPolicy {
    pub enabled: bool,
    pub bonus_points: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteSelectionPolicy {
    pub route_preference: RoutePreference,
    pub network_cost: NetworkCostPolicy,
    pub regions: RegionPolicy,
    pub backends: BackendPolicy,
    pub transports: TransportPolicy,
    pub retry_budget: RetryBudgetPolicy,
    pub cooldown: CooldownPolicy,
    pub known_good: KnownGoodPolicy,
    pub fallback: FallbackMode,
}

impl RouteSelectionPolicy {
    pub fn mvp_default() -> Self {
        Self {
            route_preference: RoutePreference::Balanced,
            network_cost: NetworkCostPolicy {
                allow_metered_networks: true,
                allow_expensive_networks: true,
            },
            regions: RegionPolicy::default(),
            backends: BackendPolicy::default(),
            transports: TransportPolicy::default(),
            retry_budget: RetryBudgetPolicy { max_candidates: 2 },
            cooldown: CooldownPolicy {
                forbid_active_cooldown: true,
            },
            known_good: KnownGoodPolicy {
                enabled: true,
                bonus_points: 3,
            },
            fallback: FallbackMode::AllowKnownGood,
        }
    }
}

impl Default for RouteSelectionPolicy {
    fn default() -> Self {
        Self::mvp_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionHealthPolicy {
    pub failure_threshold: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyEvaluationInput {
    pub endpoint_id: String,
    pub backend_name: String,
    pub transport: String,
    pub region: String,
    pub is_known_good: bool,
    pub cooling_down: bool,
    pub locally_disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PolicyEvaluation {
    pub hard_rejections: Vec<String>,
    pub soft_rejections: Vec<String>,
    pub score_adjustment: i32,
    pub reasons: Vec<String>,
}

impl PolicyEvaluation {
    pub fn is_hard_rejected(&self) -> bool {
        !self.hard_rejections.is_empty()
    }

    pub fn requires_fallback(&self) -> bool {
        !self.soft_rejections.is_empty()
    }
}

#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("failure_threshold must be positive")]
    InvalidFailureThreshold,
    #[error("retry_budget.max_candidates must be positive")]
    InvalidRetryBudget,
    #[error("known_good.bonus_points must not be negative")]
    InvalidKnownGoodBonus,
    #[error("policy '{policy_name}' has overlapping allow/deny entry '{value}'")]
    OverlappingAllowDeny { policy_name: String, value: String },
}

pub fn validate_session_health_policy(policy: &SessionHealthPolicy) -> Result<(), PolicyError> {
    if policy.failure_threshold == 0 {
        return Err(PolicyError::InvalidFailureThreshold);
    }
    Ok(())
}

pub fn validate_route_selection_policy(policy: &RouteSelectionPolicy) -> Result<(), PolicyError> {
    if policy.retry_budget.max_candidates == 0 {
        return Err(PolicyError::InvalidRetryBudget);
    }
    if policy.known_good.bonus_points < 0 {
        return Err(PolicyError::InvalidKnownGoodBonus);
    }

    validate_allow_deny_overlap("regions", &policy.regions.allow, &policy.regions.deny)?;
    validate_allow_deny_overlap("backends", &policy.backends.allow, &policy.backends.deny)?;
    validate_allow_deny_overlap("transports", &policy.transports.allow, &policy.transports.deny)?;

    Ok(())
}

pub fn evaluate_route_policy(
    policy: &RouteSelectionPolicy,
    input: &PolicyEvaluationInput,
) -> PolicyEvaluation {
    let mut evaluation = PolicyEvaluation::default();

    if input.locally_disabled {
        evaluation
            .hard_rejections
            .push("endpoint is locally disabled".to_string());
    }

    if is_denied(&policy.backends.deny, &input.backend_name) {
        evaluation.hard_rejections.push(format!(
            "backend '{}' is denylisted by policy",
            input.backend_name
        ));
    }
    if !policy.backends.allow.is_empty() && !is_allowed(&policy.backends.allow, &input.backend_name) {
        evaluation.hard_rejections.push(format!(
            "backend '{}' is not in the policy allowlist",
            input.backend_name
        ));
    }

    if is_denied(&policy.transports.deny, &input.transport) {
        evaluation.hard_rejections.push(format!(
            "transport '{}' is denylisted by policy",
            input.transport
        ));
    }
    if !policy.transports.allow.is_empty() && !is_allowed(&policy.transports.allow, &input.transport) {
        evaluation.hard_rejections.push(format!(
            "transport '{}' is not in the policy allowlist",
            input.transport
        ));
    }

    if is_denied(&policy.regions.deny, &input.region) {
        evaluation
            .hard_rejections
            .push(format!("region '{}' is denylisted by policy", input.region));
    }
    if !policy.regions.allow.is_empty() && !is_allowed(&policy.regions.allow, &input.region) {
        evaluation
            .hard_rejections
            .push(format!("region '{}' is not in the policy allowlist", input.region));
    }

    if policy.cooldown.forbid_active_cooldown && input.cooling_down {
        evaluation
            .soft_rejections
            .push("endpoint is still in cooldown".to_string());
    }

    if input.is_known_good && policy.known_good.enabled {
        evaluation.score_adjustment += policy.known_good.bonus_points;
        evaluation.reasons.push(format!(
            "known-good-bonus={}",
            policy.known_good.bonus_points
        ));
    }

    if policy.regions.preferred.iter().any(|region| region == &input.region) {
        evaluation.score_adjustment += 2;
        evaluation
            .reasons
            .push(format!("preferred-region={}", input.region));
    }

    match policy.route_preference {
        RoutePreference::Balanced => {}
        RoutePreference::PreferLatency => {
            evaluation.score_adjustment += 1;
            evaluation
                .reasons
                .push("policy-preference=prefer-latency".to_string());
        }
        RoutePreference::PreferStability => {
            evaluation.score_adjustment += 2;
            evaluation
                .reasons
                .push("policy-preference=prefer-stability".to_string());
        }
    }

    evaluation
}

fn validate_allow_deny_overlap(
    policy_name: &str,
    allow: &[String],
    deny: &[String],
) -> Result<(), PolicyError> {
    for value in allow {
        if deny.iter().any(|denied| denied == value) {
            return Err(PolicyError::OverlappingAllowDeny {
                policy_name: policy_name.to_string(),
                value: value.clone(),
            });
        }
    }
    Ok(())
}

fn is_allowed(allow: &[String], value: &str) -> bool {
    allow.iter().any(|allowed| allowed == value)
}

fn is_denied(deny: &[String], value: &str) -> bool {
    deny.iter().any(|denied| denied == value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_policy_rejects_zero_threshold() {
        let error = validate_session_health_policy(&SessionHealthPolicy { failure_threshold: 0 })
            .expect_err("policy should fail");

        assert!(matches!(error, PolicyError::InvalidFailureThreshold));
    }

    #[test]
    fn validate_route_policy_rejects_overlapping_allow_and_deny() {
        let mut policy = RouteSelectionPolicy::mvp_default();
        policy.backends.allow = vec!["xray-core".to_string()];
        policy.backends.deny = vec!["xray-core".to_string()];

        let error = validate_route_selection_policy(&policy).expect_err("policy should fail");

        assert!(matches!(error, PolicyError::OverlappingAllowDeny { .. }));
    }

    #[test]
    fn denylisted_backend_returns_explainable_error() {
        let mut policy = RouteSelectionPolicy::mvp_default();
        policy.backends.deny = vec!["xray-core".to_string()];

        let evaluation = evaluate_route_policy(
            &policy,
            &PolicyEvaluationInput {
                endpoint_id: "edge-1".to_string(),
                backend_name: "xray-core".to_string(),
                transport: "https".to_string(),
                region: "eu-central".to_string(),
                is_known_good: false,
                cooling_down: false,
                locally_disabled: false,
            },
        );

        assert!(evaluation.is_hard_rejected());
        assert!(
            evaluation
                .hard_rejections
                .iter()
                .any(|reason| reason.contains("denylisted"))
        );
    }

    #[test]
    fn known_good_adds_policy_bonus() {
        let policy = RouteSelectionPolicy::mvp_default();

        let evaluation = evaluate_route_policy(
            &policy,
            &PolicyEvaluationInput {
                endpoint_id: "edge-1".to_string(),
                backend_name: "xray-core".to_string(),
                transport: "https".to_string(),
                region: "eu-central".to_string(),
                is_known_good: true,
                cooling_down: false,
                locally_disabled: false,
            },
        );

        assert_eq!(evaluation.score_adjustment, 3);
        assert!(
            evaluation
                .reasons
                .iter()
                .any(|reason| reason == "known-good-bonus=3")
        );
    }

    #[test]
    fn cooldown_becomes_soft_rejection() {
        let policy = RouteSelectionPolicy::mvp_default();

        let evaluation = evaluate_route_policy(
            &policy,
            &PolicyEvaluationInput {
                endpoint_id: "edge-1".to_string(),
                backend_name: "xray-core".to_string(),
                transport: "https".to_string(),
                region: "eu-central".to_string(),
                is_known_good: false,
                cooling_down: true,
                locally_disabled: false,
            },
        );

        assert!(evaluation.requires_fallback());
        assert_eq!(evaluation.soft_rejections, vec!["endpoint is still in cooldown"]);
    }
}
