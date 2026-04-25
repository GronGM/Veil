//! Incident and support-facing diagnostics skeleton for Veil.

use serde::{Deserialize, Serialize};
use veil_manifest::ProviderManifest;
use veil_policy::{IncidentGuidance, RuntimeSupportAssessment};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncidentReport {
    pub severity: String,
    pub headline: String,
    pub recommended_action: String,
    pub selected_endpoint_id: Option<String>,
    pub route_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupportBundle {
    pub manifest_valid: bool,
    pub runtime_support_tier: String,
    pub selected_endpoint_id: Option<String>,
    pub route_summary: String,
    pub endpoint_count: usize,
    pub incident: IncidentReport,
}

pub fn build_incident_report(
    manifest_valid: bool,
    runtime_support: RuntimeSupportAssessment,
    guidance: IncidentGuidance,
    selected_endpoint_id: Option<String>,
    route_summary: String,
) -> IncidentReport {
    let severity = if manifest_valid && runtime_support.in_mvp_scope && selected_endpoint_id.is_some() {
        "ok".to_string()
    } else if manifest_valid {
        "warning".to_string()
    } else {
        "critical".to_string()
    };

    let headline = match severity.as_str() {
        "ok" => "session plan is inside the current MVP contour".to_string(),
        "warning" => "session plan was built with caveats".to_string(),
        _ => "session plan is blocked by manifest or runtime contract issues".to_string(),
    };

    IncidentReport {
        severity,
        headline,
        recommended_action: guidance.recommended_action,
        selected_endpoint_id,
        route_summary,
    }
}

pub fn build_support_bundle(
    manifest_valid: bool,
    manifest: &ProviderManifest,
    runtime_support: RuntimeSupportAssessment,
    selected_endpoint_id: Option<String>,
    route_summary: String,
    incident: IncidentReport,
) -> SupportBundle {
    SupportBundle {
        manifest_valid,
        runtime_support_tier: runtime_support.tier,
        selected_endpoint_id,
        route_summary,
        endpoint_count: manifest.endpoints.len(),
        incident,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use veil_manifest::demo_provider_manifest;

    #[test]
    fn incident_report_is_ok_for_supported_demo_path() {
        let report = build_incident_report(
            true,
            RuntimeSupportAssessment::mvp_supported(),
            IncidentGuidance::default(),
            Some("edge-1".to_string()),
            "selected=edge-1".to_string(),
        );

        assert_eq!(report.severity, "ok");
        assert!(report.headline.contains("MVP contour"));
    }

    #[test]
    fn support_bundle_tracks_endpoint_count() {
        let manifest = demo_provider_manifest();
        let incident = build_incident_report(
            true,
            RuntimeSupportAssessment::mvp_supported(),
            IncidentGuidance::default(),
            Some("edge-1".to_string()),
            "selected=edge-1".to_string(),
        );

        let bundle = build_support_bundle(
            true,
            &manifest,
            RuntimeSupportAssessment::mvp_supported(),
            Some("edge-1".to_string()),
            "selected=edge-1".to_string(),
            incident,
        );

        assert_eq!(bundle.endpoint_count, 1);
        assert_eq!(bundle.runtime_support_tier, "mvp-supported");
    }
}
