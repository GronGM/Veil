#![forbid(unsafe_code)]

//! Core orchestration skeleton for Veil.

use veil_adapter_api::{DataplaneBackend, DryRunPlan};

/// Minimal session orchestration entrypoint for early dry-run wiring.
#[derive(Debug, Clone, Default)]
pub struct SessionEngine;

/// Human-readable summary returned by the control plane for dry-run flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DryRunReport {
    pub backend_name: String,
    pub command_preview: String,
    pub config_summary: String,
}

impl SessionEngine {
    /// Build a dry-run report from a backend adapter without touching the real network.
    pub fn dry_run<B: DataplaneBackend>(backend: &B) -> DryRunReport {
        DryRunReport::from(backend.build_dry_run_plan())
    }
}

impl From<DryRunPlan> for DryRunReport {
    fn from(plan: DryRunPlan) -> Self {
        Self {
            backend_name: plan.backend_name.to_string(),
            command_preview: plan.command_preview,
            config_summary: plan.config_summary,
        }
    }
}

impl DryRunReport {
    /// Render a compact operator-facing report for CLI output.
    pub fn render(&self) -> String {
        format!(
            "Veil dry-run\nbackend: {}\ncommand: {}\nconfig: {}",
            self.backend_name, self.command_preview, self.config_summary
        )
    }
}
