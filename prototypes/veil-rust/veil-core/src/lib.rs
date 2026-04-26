#![forbid(unsafe_code)]

//! Core orchestration skeleton for Veil.

use veil_adapter_api::{
    validate_capability_requirements, validate_transport_support, AdapterCapabilities,
    AdapterCapabilityDecision, AdapterTransportDecision, DataplaneBackend, DryRunPlan,
};
use veil_diagnostics::RedactedDryRunDiagnostics;
use veil_dry_run::{DryRunOutcome, DryRunReport};
use veil_manifest::ProviderManifest;
use veil_policy::{PolicyDecision, RoutePolicy, RuntimeSupportAssessment, TransportPolicyDecision};

/// Minimal session orchestration entrypoint for early dry-run wiring.
#[derive(Debug, Clone, Default)]
pub struct SessionEngine;

impl SessionEngine {
    /// Build a dry-run report from manifest, policy, and backend inputs without touching the real network.
    pub fn dry_run<B: DataplaneBackend>(
        manifest: &ProviderManifest,
        policy: &RoutePolicy,
        backend: &B,
    ) -> DryRunReport {
        let plan = backend.build_dry_run_plan();
        let backend_decision = policy.evaluate(manifest, plan.backend_name);
        let transport_decision = policy.evaluate_transport(manifest);
        let capabilities = backend.capabilities();
        let adapter_transport_decision =
            validate_transport_support(&capabilities, transport_decision.transport_profile);
        let adapter_capability_decision =
            validate_capability_requirements(&capabilities, policy.required_capabilities);
        let runtime_support_assessment =
            policy.assess_runtime_support(manifest, plan.backend_name);
        Self::from_parts(
            manifest,
            plan,
            backend_decision,
            transport_decision,
            adapter_transport_decision,
            adapter_capability_decision,
            runtime_support_assessment,
            capabilities,
        )
    }
}

impl SessionEngine {
    fn from_parts(
        manifest: &ProviderManifest,
        plan: DryRunPlan,
        backend_decision: PolicyDecision,
        transport_decision: TransportPolicyDecision,
        adapter_transport_decision: AdapterTransportDecision,
        adapter_capability_decision: AdapterCapabilityDecision,
        runtime_support_assessment: RuntimeSupportAssessment,
        capabilities: AdapterCapabilities,
    ) -> DryRunReport {
        let allowed = backend_decision.allowed
            && transport_decision.allowed
            && adapter_transport_decision.supported
            && adapter_capability_decision.compatible;

        let outcome = if !backend_decision.allowed || !transport_decision.allowed {
            DryRunOutcome::BlockedByPolicy
        } else if !adapter_transport_decision.supported || !adapter_capability_decision.compatible {
            DryRunOutcome::BlockedByAdapter
        } else {
            DryRunOutcome::Allowed
        };

        DryRunReport {
            provider_name: manifest.provider_name.clone(),
            profile_name: manifest.profile_name.clone(),
            backend_name: plan.backend_name.to_string(),
            transport_profile: transport_decision.transport_profile.as_str().to_string(),
            outcome,
            backend_policy_reason: backend_decision.reason,
            transport_policy_reason: transport_decision.reason,
            adapter_transport_reason: adapter_transport_decision.reason,
            adapter_capability_reason: adapter_capability_decision.reason,
            runtime_support_tier: runtime_support_assessment.tier,
            runtime_support_reason: runtime_support_assessment.reason,
            command_preview: plan.command_preview,
            config_summary: plan.config_summary,
            decision_summary: backend_decision.summary,
            transport_summary: transport_decision.summary,
            adapter_transport_summary: adapter_transport_decision.summary,
            adapter_capability_summary: adapter_capability_decision.summary,
            runtime_support_summary: runtime_support_assessment.summary,
            runtime_support_caveats: runtime_support_assessment.caveats,
            client_platform: manifest.client_platform,
            platform_adapter: manifest.platform_adapter,
            routing_summary: "routing eligibility has not been evaluated yet".to_string(),
            routing_candidates: Vec::new(),
            capabilities_summary: render_capabilities(&capabilities),
            allowed,
        }
    }
}

impl DryRunReport {
    /// Build a redacted diagnostics view for support-facing CLI output.
    pub fn redacted_diagnostics(&self) -> RedactedDryRunDiagnostics {
        RedactedDryRunDiagnostics::from_report(self)
    }
}

fn render_capabilities(capabilities: &AdapterCapabilities) -> String {
    let supported_transports = capabilities
        .supported_transports
        .iter()
        .map(|transport| transport.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "dry_run={}, typed_config={}, real_binary={}, transports=[{}]",
        capabilities.supports_dry_run,
        capabilities.renders_typed_config,
        capabilities.requires_real_binary,
        supported_transports
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use veil_adapter_mock::MockBackend;
    use veil_adapter_xray::XrayBackend;
    use veil_transport::TransportProfile;

    #[test]
    fn dry_run_report_shows_xray_capabilities() {
        let manifest = ProviderManifest::demo();
        let policy = RoutePolicy::demo();
        let report = SessionEngine::dry_run(&manifest, &policy, &XrayBackend::default());

        assert!(report.allowed);
        assert_eq!(report.outcome.as_str(), "allowed");
        assert_eq!(report.transport_profile, "tls-tcp");
        assert_eq!(report.backend_policy_reason.as_str(), "preferred_allowed");
        assert_eq!(report.transport_policy_reason.as_str(), "preferred_allowed");
        assert_eq!(report.adapter_transport_reason.as_str(), "supported");
        assert_eq!(report.adapter_capability_reason.as_str(), "supported");
        assert_eq!(report.runtime_support_tier.as_str(), "mvp-supported");
        assert_eq!(report.runtime_support_reason.as_str(), "explicit_mvp_contour");
        assert!(report.transport_summary.contains("policy allows transport 'tls-tcp'"));
        assert!(report.adapter_transport_summary.contains("supports transport 'tls-tcp'"));
        assert!(report.adapter_capability_summary.contains("satisfies the requested capability requirements"));
        assert!(report.runtime_support_summary.contains("honest MVP runtime target"));
        assert!(report.capabilities_summary.contains("typed_config=true"));
        assert!(report.capabilities_summary.contains("real_binary=true"));
    }

    #[test]
    fn dry_run_report_changes_with_mock_backend() {
        let manifest = ProviderManifest::demo();
        let policy = RoutePolicy::mismatch_demo();
        let report = SessionEngine::dry_run(&manifest, &policy, &MockBackend::default());

        assert!(report.allowed);
        assert_eq!(report.outcome.as_str(), "allowed");
        assert_eq!(report.backend_name, "mock-backend");
        assert_eq!(report.backend_policy_reason.as_str(), "required_by_policy");
        assert_eq!(report.adapter_transport_reason.as_str(), "supported");
        assert_eq!(report.adapter_capability_reason.as_str(), "supported");
        assert_eq!(report.runtime_support_tier.as_str(), "contract-mismatch");
        assert!(report.adapter_transport_summary.contains("supports transport 'tls-tcp'"));
        assert!(report.capabilities_summary.contains("typed_config=false"));
        assert!(report.capabilities_summary.contains("real_binary=false"));
    }

    #[test]
    fn dry_run_report_can_mark_linux_foundation_contour() {
        let manifest = ProviderManifest::linux_foundation_demo();
        let policy = RoutePolicy::mismatch_demo();
        let report = SessionEngine::dry_run(&manifest, &policy, &MockBackend::default());

        assert!(report.allowed);
        assert_eq!(report.runtime_support_tier.as_str(), "foundation-only");
        assert_eq!(report.runtime_support_reason.as_str(), "linux_non_mvp_contour");
        assert!(report.runtime_support_caveats.iter().any(|caveat| caveat.contains("first MVP target")));
    }

    #[test]
    fn dry_run_report_blocks_transport_mismatch() {
        let manifest = ProviderManifest::demo();
        let policy = RoutePolicy::transport_mismatch_demo();
        let report = SessionEngine::dry_run(&manifest, &policy, &XrayBackend::default());

        assert!(!report.allowed);
        assert_eq!(report.outcome.as_str(), "blocked_by_policy");
        assert_eq!(report.transport_profile, "grpc");
        assert_eq!(report.transport_policy_reason.as_str(), "required_by_policy");
        assert_eq!(report.adapter_transport_reason.as_str(), "supported");
        assert_eq!(report.adapter_capability_reason.as_str(), "supported");
        assert_eq!(report.runtime_support_tier.as_str(), "mvp-supported");
        assert!(report.transport_summary.contains("requires transport 'grpc'"));
        assert!(report.adapter_transport_summary.contains("supports transport 'grpc'"));
    }

    #[test]
    fn dry_run_report_blocks_unsupported_adapter_transport() {
        let manifest = ProviderManifest::grpc_demo();
        let policy = RoutePolicy {
            allow_backend: "mock-backend".to_string(),
            allow_fallback: false,
            allow_transport: TransportProfile::Grpc,
            required_capabilities: veil_adapter_api::AdapterCapabilityRequirements::dry_run_only(),
        };
        let report = SessionEngine::dry_run(&manifest, &policy, &MockBackend::default());

        assert!(!report.allowed);
        assert_eq!(report.outcome.as_str(), "blocked_by_adapter");
        assert_eq!(report.backend_name, "mock-backend");
        assert_eq!(report.transport_profile, "grpc");
        assert_eq!(report.backend_policy_reason.as_str(), "required_by_policy");
        assert_eq!(report.transport_policy_reason.as_str(), "preferred_allowed");
        assert_eq!(report.adapter_transport_reason.as_str(), "unsupported_transport");
        assert_eq!(report.adapter_capability_reason.as_str(), "supported");
        assert_eq!(report.runtime_support_tier.as_str(), "contract-mismatch");
        assert!(report.transport_summary.contains("policy allows transport 'grpc'"));
        assert!(report.adapter_transport_summary.contains("does not support transport 'grpc'"));
        assert!(report.capabilities_summary.contains("transports=[tls-tcp]"));
    }

    #[test]
    fn dry_run_report_blocks_missing_typed_config_requirement() {
        let manifest = ProviderManifest::demo();
        let policy = RoutePolicy::typed_config_demo();
        let report = SessionEngine::dry_run(&manifest, &policy, &MockBackend::default());

        assert!(!report.allowed);
        assert_eq!(report.outcome.as_str(), "blocked_by_adapter");
        assert_eq!(report.adapter_transport_reason.as_str(), "supported");
        assert_eq!(report.adapter_capability_reason.as_str(), "missing_typed_config");
        assert_eq!(report.runtime_support_tier.as_str(), "contract-mismatch");
        assert!(report.adapter_capability_summary.contains("capability requirement 'typed_config'"));
    }
}
