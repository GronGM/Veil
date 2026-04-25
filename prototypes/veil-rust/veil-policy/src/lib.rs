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
pub struct SessionHealthPolicy {
    pub failure_threshold: u8,
}

#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("failure_threshold must be positive")]
    InvalidFailureThreshold,
}

pub fn validate_session_health_policy(policy: &SessionHealthPolicy) -> Result<(), PolicyError> {
    if policy.failure_threshold == 0 {
        return Err(PolicyError::InvalidFailureThreshold);
    }
    Ok(())
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
}

