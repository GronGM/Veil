#![forbid(unsafe_code)]

//! Core orchestration skeleton for Veil.

use veil_adapter_api::{DataplaneBackend, DryRunPlan};
use veil_manifest::ProviderManifest;
use veil_policy::{PolicyDecision, RoutePolicy};

/// Minimal session orchestration entrypoint for early dry-run wiring.
#[derive(Debug, Clone, Default)]
pub struct SessionEngine;

/// Human-readable summary returned by the control plane for dry-run flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DryRunReport {
    pub provider_name: String,
    pub profile_name: String,
    pub backend_name: String,
    pub command_preview: String,
    pub config_summary: String,
    pub decision_summary: String,
    pub allowed: bool,
}

impl SessionEngine {
    /// Build a dry-run report from manifest, policy, and backend inputs without touching the real network.
    pub fn dry_run<B: DataplaneBackend>(
        manifest: &ProviderManifest,
        policy: &RoutePolicy,
        backend: &B,
    ) -> DryRunReport {
        let plan = backend.build_dry_run_plan();
        let decision = policy.evaluate(manifest, plan.backend_name);
        DryRunReport::from_parts(manifest, plan, decision)
    }
}

impl DryRunReport {
    fn from_parts(
        manifest: &ProviderManifest,
        plan: DryRunPlan,
        decision: PolicyDecision,
    ) -> Self {
        Self {
            provider_name: manifest.provider_name.clone(),
            profile_name: manifest.profile_name.clone(),
            backend_name: plan.backend_name.to_string(),
            command_preview: plan.command_preview,
            config_summary: plan.config_summary,
            decision_summary: decision.summary,
            allowed: decision.allowed,
        }
    }

    /// Render a compact operator-facing report for CLI output.
    pub fn render(&self) -> String {
        format!(
            "Veil dry-run\nprovider: {}\nprofile: {}\nbackend: {}\nallowed: {}\ndecision: {}\ncommand: {}\nconfig: {}",
            self.provider_name,
            self.profile_name,
            self.backend_name,
            self.allowed,
            self.decision_summary,
            self.command_preview,
            self.config_summary
        )
    }
}
