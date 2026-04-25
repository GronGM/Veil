//! Minimal control-plane skeleton for Veil.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use veil_adapter_api::{AdapterContext, AdapterRegistrySnapshot, StaticAdapterRegistry};
use veil_adapter_xray::XrayDryRunBackend;
use veil_diagnostics::{build_incident_report, build_support_bundle, IncidentReport, SupportBundle};
use veil_manifest::{demo_provider_manifest, validate_manifest, ProviderManifest};
use veil_policy::{IncidentGuidance, RuntimeSupportAssessment};
use veil_routing::{demo_runtime_state, select_best_route, RetryBudget, SelectionContext};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionPhase {
    Idle,
    Loading,
    Selecting,
    Connecting,
    Connected,
    Degraded,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionOutcome {
    Planned,
    ReadyToConnect,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionEvent {
    pub phase: SessionPhase,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionPlan {
    pub session_id: Uuid,
    pub created_at: OffsetDateTime,
    pub phase: SessionPhase,
    pub outcome: SessionOutcome,
    pub selected_endpoint_id: Option<String>,
    pub selected_backend_name: Option<String>,
    pub diagnostics_reason: String,
    pub route_summary: String,
    pub lifecycle: Vec<SessionEvent>,
    pub adapter_registry: AdapterRegistrySnapshot,
    pub incident_report: IncidentReport,
    pub support_bundle: SupportBundle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionReport {
    pub session_id: Uuid,
    pub outcome: SessionOutcome,
    pub selected_endpoint_id: Option<String>,
    pub selected_backend_name: Option<String>,
    pub route_summary: String,
    pub incident_severity: String,
    pub event_count: usize,
}

pub fn build_dry_run_plan(
    manifest: &ProviderManifest,
    runtime_support: RuntimeSupportAssessment,
    guidance: IncidentGuidance,
) -> SessionPlan {
    let manifest_is_valid = validate_manifest(manifest).is_ok();
    let session_id = Uuid::new_v4();
    let adapter_context = AdapterContext {
        client_platform: "linux".to_string(),
        dry_run: true,
        session_id: session_id.to_string(),
    };
    let mut registry = StaticAdapterRegistry::new();
    registry.register(Box::new(XrayDryRunBackend));
    let adapter_registry = registry.metadata_snapshot();
    let mut lifecycle = vec![SessionEvent {
        phase: SessionPhase::Loading,
        detail: "loaded dry-run manifest into core".to_string(),
    }];
    let route_selection = select_best_route(
        manifest,
        &SelectionContext {
            client_platform: "linux".to_string(),
            last_known_good_endpoint_id: None,
            retry_budget: RetryBudget {
                max_candidates: usize::from(manifest.transport_policy.retry_budget),
            },
            runtime_state: demo_runtime_state(),
        },
    );
    lifecycle.push(SessionEvent {
        phase: SessionPhase::Selecting,
        detail: "evaluated route candidates using routing state".to_string(),
    });
    let selected_endpoint_id = route_selection
        .as_ref()
        .map(|selection| selection.selected.endpoint.id.clone());
    let selected_backend_name = route_selection.as_ref().and_then(|selection| {
        registry
            .resolve_backend_name_for_endpoint(&selection.selected.endpoint)
            .ok()
    });
    if let Some(backend_name) = &selected_backend_name {
        lifecycle.push(SessionEvent {
            phase: SessionPhase::Connecting,
            detail: format!("resolved backend '{backend_name}' from adapter registry"),
        });
        let _ = &adapter_context;
    }
    let diagnostics_reason = format!(
        "manifest_valid={} runtime_support={} guidance={}",
        manifest_is_valid, runtime_support.tier, guidance.recommended_action
    );
    let route_summary = route_selection
        .as_ref()
        .map(|selection| selection.summary())
        .unwrap_or_else(|| "no candidate route available".to_string());
    let phase = if selected_endpoint_id.is_some() {
        SessionPhase::Selecting
    } else {
        SessionPhase::Failed
    };
    let outcome = if manifest_is_valid && selected_endpoint_id.is_some() {
        SessionOutcome::ReadyToConnect
    } else if manifest_is_valid {
        SessionOutcome::Planned
    } else {
        SessionOutcome::Blocked
    };
    lifecycle.push(SessionEvent {
        phase,
        detail: if selected_endpoint_id.is_some() {
            "selected route is ready for a future adapter start".to_string()
        } else {
            "no route was selected for the current dry-run context".to_string()
        },
    });
    let incident_report = build_incident_report(
        manifest_is_valid,
        runtime_support.clone(),
        guidance.clone(),
        selected_endpoint_id.clone(),
        route_summary.clone(),
    );
    let support_bundle = build_support_bundle(
        manifest_is_valid,
        manifest,
        runtime_support,
        selected_endpoint_id.clone(),
        route_summary.clone(),
        incident_report.clone(),
    );

    SessionPlan {
        session_id,
        created_at: OffsetDateTime::now_utc(),
        phase,
        outcome,
        selected_endpoint_id,
        selected_backend_name,
        diagnostics_reason,
        route_summary,
        lifecycle,
        adapter_registry,
        incident_report,
        support_bundle,
    }
}

pub fn build_session_report(plan: &SessionPlan) -> SessionReport {
    SessionReport {
        session_id: plan.session_id,
        outcome: plan.outcome,
        selected_endpoint_id: plan.selected_endpoint_id.clone(),
        selected_backend_name: plan.selected_backend_name.clone(),
        route_summary: plan.route_summary.clone(),
        incident_severity: plan.incident_report.severity.clone(),
        event_count: plan.lifecycle.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use veil_policy::{IncidentGuidance, RuntimeSupportAssessment};

    #[test]
    fn dry_run_plan_uses_first_endpoint() {
        let manifest: ProviderManifest = demo_provider_manifest();

        let plan = build_dry_run_plan(
            &manifest,
            RuntimeSupportAssessment::mvp_supported(),
            IncidentGuidance::default(),
        );

        assert_eq!(plan.phase, SessionPhase::Selecting);
        assert_eq!(plan.outcome, SessionOutcome::ReadyToConnect);
        assert_eq!(plan.selected_endpoint_id.as_deref(), Some("edge-1"));
        assert_eq!(plan.selected_backend_name.as_deref(), Some("xray-core"));
        assert!(plan.diagnostics_reason.contains("manifest_valid=true"));
        assert!(plan.route_summary.contains("selected=edge-1"));
        assert_eq!(plan.incident_report.severity, "ok");
        assert_eq!(plan.support_bundle.selected_endpoint_id.as_deref(), Some("edge-1"));
        assert_eq!(plan.lifecycle.len(), 4);
        assert_eq!(plan.adapter_registry.entries.len(), 1);
    }

    #[test]
    fn session_report_summarizes_plan() {
        let manifest: ProviderManifest = demo_provider_manifest();
        let plan = build_dry_run_plan(
            &manifest,
            RuntimeSupportAssessment::mvp_supported(),
            IncidentGuidance::default(),
        );
        let report = build_session_report(&plan);

        assert_eq!(report.outcome, SessionOutcome::ReadyToConnect);
        assert_eq!(report.selected_endpoint_id.as_deref(), Some("edge-1"));
        assert_eq!(report.selected_backend_name.as_deref(), Some("xray-core"));
        assert_eq!(report.incident_severity, "ok");
        assert_eq!(report.event_count, 4);
    }
}
