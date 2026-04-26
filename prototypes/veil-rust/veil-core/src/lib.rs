//! Minimal control-plane skeleton for Veil.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use veil_adapter_api::{AdapterContext, AdapterRegistrySnapshot, StaticAdapterRegistry};
use veil_adapter_mock::MockDryRunBackend;
use veil_adapter_xray::XrayDryRunBackend;
use veil_diagnostics::{
    build_backend_preflight_diagnostics, build_incident_report, build_route_diagnostics,
    build_support_bundle, BackendPreflightDiagnostics, IncidentReport, SupportBundle,
};
use veil_manifest::{demo_provider_manifest, validate_manifest, ProviderManifest};
use veil_policy::{
    validate_route_selection_policy, IncidentGuidance, RouteSelectionPolicy,
    RuntimeSupportAssessment,
};
use veil_routing::{
    demo_runtime_state, select_best_route, RejectedRouteCandidate, RetryBudget, SelectionContext,
};

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
    pub fallback_triggered: bool,
    pub rejected_routes: Vec<RejectedRouteCandidate>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DryRunOverrides {
    pub selected_endpoint_id: Option<String>,
    pub selected_backend_name: Option<String>,
}

pub fn build_dry_run_plan(
    manifest: &ProviderManifest,
    runtime_support: RuntimeSupportAssessment,
    guidance: IncidentGuidance,
) -> SessionPlan {
    build_dry_run_plan_with_policy(
        manifest,
        runtime_support,
        guidance,
        RouteSelectionPolicy::default(),
    )
}

pub fn build_dry_run_plan_with_policy(
    manifest: &ProviderManifest,
    runtime_support: RuntimeSupportAssessment,
    guidance: IncidentGuidance,
    route_policy: RouteSelectionPolicy,
) -> SessionPlan {
    build_dry_run_plan_with_policy_and_overrides(
        manifest,
        runtime_support,
        guidance,
        route_policy,
        DryRunOverrides::default(),
    )
}

pub fn build_dry_run_plan_with_policy_and_overrides(
    manifest: &ProviderManifest,
    runtime_support: RuntimeSupportAssessment,
    guidance: IncidentGuidance,
    route_policy: RouteSelectionPolicy,
    overrides: DryRunOverrides,
) -> SessionPlan {
    let filtered_manifest = apply_dry_run_overrides(manifest, &overrides);
    let manifest_is_valid = validate_manifest(&filtered_manifest).is_ok();
    let policy_is_valid = validate_route_selection_policy(&route_policy).is_ok();
    let session_id = Uuid::new_v4();
    let adapter_context = AdapterContext {
        client_platform: "linux".to_string(),
        dry_run: true,
        session_id: session_id.to_string(),
    };
    let mut registry = StaticAdapterRegistry::new();
    registry.register(Box::new(MockDryRunBackend));
    registry.register(Box::new(XrayDryRunBackend));
    let adapter_registry = registry.metadata_snapshot();
    let mut lifecycle = vec![SessionEvent {
        phase: SessionPhase::Loading,
        detail: "loaded dry-run manifest into core".to_string(),
    }];
    if let Some(endpoint_id) = &overrides.selected_endpoint_id {
        lifecycle.push(SessionEvent {
            phase: SessionPhase::Loading,
            detail: format!("applied endpoint override for '{endpoint_id}'"),
        });
    }
    if let Some(backend_name) = &overrides.selected_backend_name {
        lifecycle.push(SessionEvent {
            phase: SessionPhase::Loading,
            detail: format!("applied backend override for '{backend_name}'"),
        });
    }
    let route_selection = select_best_route(
        &filtered_manifest,
        &SelectionContext {
            client_platform: "linux".to_string(),
            last_known_good_endpoint_id: None,
            retry_budget: RetryBudget {
                max_candidates: usize::from(route_policy.retry_budget.max_candidates),
            },
            runtime_state: demo_runtime_state(),
            policy: route_policy.clone(),
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
    let backend_preflight_diagnostics = route_selection
        .as_ref()
        .ok()
        .and_then(|selection| {
            registry
                .resolve_backend_for_endpoint(&selection.selected.endpoint)
                .ok()
                .and_then(|backend| {
                    backend
                        .build_dry_run_preflight(&selection.selected.endpoint, &adapter_context)
                        .ok()
                })
                .map(build_backend_preflight_diagnostics_from_adapter)
        });
    if let Some(backend_name) = &selected_backend_name {
        lifecycle.push(SessionEvent {
            phase: SessionPhase::Connecting,
            detail: format!("resolved backend '{backend_name}' from adapter registry"),
        });
    }
    if let Some(preflight) = &backend_preflight_diagnostics {
        lifecycle.push(SessionEvent {
            phase: SessionPhase::Connecting,
            detail: format!(
                "prepared backend preflight for '{}' (binary_present={})",
                preflight.backend_name, preflight.binary_present
            ),
        });
    }
    let backend_preflight_ready = selected_endpoint_id.is_some()
        && backend_preflight_diagnostics
            .as_ref()
            .map(|preflight| preflight.ready_for_dry_run_connect)
            .unwrap_or(false);
    let diagnostics_reason = format!(
        "manifest_valid={} policy_valid={} backend_preflight_ready={} endpoint_override={} backend_override={} runtime_support={} guidance={}",
        manifest_is_valid,
        policy_is_valid,
        backend_preflight_ready,
        overrides
            .selected_endpoint_id
            .clone()
            .unwrap_or_else(|| "none".to_string()),
        overrides
            .selected_backend_name
            .clone()
            .unwrap_or_else(|| "none".to_string()),
        runtime_support.tier,
        guidance.recommended_action
    );
    let route_summary = route_selection
        .as_ref()
        .map(|selection| selection.summary())
        .unwrap_or_else(|error| format!("route-selection-failed: {}", error.detail));
    let fallback_triggered = route_selection
        .as_ref()
        .map(|selection| selection.fallback_triggered)
        .unwrap_or(false);
    let rejected_routes = match &route_selection {
        Ok(selection) => selection.rejected.clone(),
        Err(error) => error.rejected.clone(),
    };
    let phase = if selected_endpoint_id.is_some() {
        SessionPhase::Selecting
    } else {
        SessionPhase::Failed
    };
    let outcome = if manifest_is_valid && backend_preflight_ready {
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
    let route_diagnostics = build_route_diagnostics(
        selected_endpoint_id.clone(),
        selected_backend_name.clone(),
        route_summary.clone(),
        fallback_triggered,
        rejected_routes.clone(),
    );
    let adapter_compatibility = veil_diagnostics::build_adapter_compatibility_diagnostics(
        &filtered_manifest,
        &adapter_registry,
        selected_backend_name.as_deref(),
    );
    let diagnostics_reason = format!(
        "{} adapter_compatibility_ok={}",
        diagnostics_reason, adapter_compatibility.compatibility_ok
    );
    let support_bundle = build_support_bundle(
        manifest_is_valid,
        &filtered_manifest,
        &adapter_registry,
        runtime_support,
        backend_preflight_diagnostics,
        route_diagnostics,
        &route_policy,
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
        fallback_triggered,
        rejected_routes,
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

fn apply_dry_run_overrides(
    manifest: &ProviderManifest,
    overrides: &DryRunOverrides,
) -> ProviderManifest {
    let mut filtered_manifest = manifest.clone();

    if let Some(endpoint_id) = &overrides.selected_endpoint_id {
        filtered_manifest
            .endpoints
            .retain(|endpoint| &endpoint.id == endpoint_id);
    }

    if let Some(backend_name) = &overrides.selected_backend_name {
        filtered_manifest.endpoints.retain(|endpoint| {
            endpoint.dataplane.as_deref().unwrap_or("xray-core") == backend_name
        });
    }

    filtered_manifest
}

fn build_backend_preflight_diagnostics_from_adapter(
    preflight: veil_adapter_api::DryRunPreflight,
) -> BackendPreflightDiagnostics {
    build_backend_preflight_diagnostics(
        preflight.backend_name,
        preflight.binary_path,
        preflight.config_path,
        preflight.binary_present,
        preflight.readiness_note,
        preflight.command.program,
        preflight.command.args,
        preflight.rendered_config,
    )
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
        assert!(plan.diagnostics_reason.contains("policy_valid=true"));
        assert!(plan.diagnostics_reason.contains("backend_preflight_ready=true"));
        assert!(plan.diagnostics_reason.contains("endpoint_override=none"));
        assert!(plan.diagnostics_reason.contains("backend_override=none"));
        assert!(plan.route_summary.contains("selected=edge-1"));
        assert!(!plan.fallback_triggered);
        assert!(plan.rejected_routes.is_empty());
        assert_eq!(plan.incident_report.severity, "ok");
        assert_eq!(plan.support_bundle.selected_endpoint_id.as_deref(), Some("edge-1"));
        assert_eq!(
            plan.support_bundle
                .route_diagnostics
                .selected_backend_name
                .as_deref(),
            Some("xray-core")
        );
        assert_eq!(
            plan.support_bundle.redacted_policy_diagnostics.retry_budget_max_candidates,
            2
        );
        assert!(
            plan.support_bundle
                .backend_preflight_diagnostics
                .as_ref()
                .expect("backend preflight")
                .ready_for_dry_run_connect
        );
        assert_eq!(plan.lifecycle.len(), 5);
        assert_eq!(plan.adapter_registry.entries.len(), 2);
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
        assert_eq!(report.event_count, 5);
    }

    #[test]
    fn dry_run_plan_exposes_rejected_routes_for_policy_failure() {
        let manifest: ProviderManifest = demo_provider_manifest();
        let mut policy = RouteSelectionPolicy::default();
        policy.backends.deny = vec!["xray-core".to_string()];

        let plan = build_dry_run_plan_with_policy(
            &manifest,
            RuntimeSupportAssessment::mvp_supported(),
            IncidentGuidance::default(),
            policy,
        );

        assert_eq!(plan.outcome, SessionOutcome::Planned);
        assert!(plan.selected_endpoint_id.is_none());
        assert_eq!(plan.rejected_routes.len(), 2);
        assert_eq!(plan.support_bundle.route_diagnostics.rejected_routes.len(), 2);
        assert_eq!(
            plan.support_bundle
                .redacted_policy_diagnostics
                .denied_backends
                .values,
            vec!["xray-core"]
        );
        assert!(
            plan.rejected_routes[0]
                .reasons
                .iter()
                .any(|reason| reason.contains("denylisted"))
        );
    }

    #[test]
    fn dry_run_plan_can_fall_back_to_mock_backend_when_xray_is_denylisted() {
        let manifest: ProviderManifest = demo_provider_manifest();
        let mut policy = RouteSelectionPolicy::default();
        policy.backends.deny = vec!["xray-core".to_string()];

        let plan = build_dry_run_plan_with_policy(
            &manifest,
            RuntimeSupportAssessment::mvp_supported(),
            IncidentGuidance::default(),
            policy,
        );

        assert_eq!(plan.outcome, SessionOutcome::ReadyToConnect);
        assert_eq!(plan.selected_endpoint_id.as_deref(), Some("edge-mock-1"));
        assert_eq!(plan.selected_backend_name.as_deref(), Some("mock-backend"));
        assert!(
            plan.support_bundle
                .backend_preflight_diagnostics
                .as_ref()
                .expect("backend preflight")
                .backend_name
                == "mock-backend"
        );
    }

    #[test]
    fn dry_run_plan_honors_backend_override() {
        let manifest: ProviderManifest = demo_provider_manifest();

        let plan = build_dry_run_plan_with_policy_and_overrides(
            &manifest,
            RuntimeSupportAssessment::mvp_supported(),
            IncidentGuidance::default(),
            RouteSelectionPolicy::default(),
            DryRunOverrides {
                selected_endpoint_id: None,
                selected_backend_name: Some("mock-backend".to_string()),
            },
        );

        assert_eq!(plan.outcome, SessionOutcome::ReadyToConnect);
        assert_eq!(plan.selected_endpoint_id.as_deref(), Some("edge-mock-1"));
        assert_eq!(plan.selected_backend_name.as_deref(), Some("mock-backend"));
        assert!(plan.diagnostics_reason.contains("backend_override=mock-backend"));
    }

    #[test]
    fn dry_run_plan_honors_endpoint_override() {
        let manifest: ProviderManifest = demo_provider_manifest();

        let plan = build_dry_run_plan_with_policy_and_overrides(
            &manifest,
            RuntimeSupportAssessment::mvp_supported(),
            IncidentGuidance::default(),
            RouteSelectionPolicy::default(),
            DryRunOverrides {
                selected_endpoint_id: Some("edge-mock-1".to_string()),
                selected_backend_name: None,
            },
        );

        assert_eq!(plan.outcome, SessionOutcome::ReadyToConnect);
        assert_eq!(plan.selected_endpoint_id.as_deref(), Some("edge-mock-1"));
        assert_eq!(plan.selected_backend_name.as_deref(), Some("mock-backend"));
        assert!(plan.diagnostics_reason.contains("endpoint_override=edge-mock-1"));
    }
}
