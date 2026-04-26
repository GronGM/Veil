#![forbid(unsafe_code)]

//! Adapter boundary skeleton for Veil.

/// Minimal operator-facing capability snapshot for an adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterCapabilities {
    pub backend_name: &'static str,
    pub supports_dry_run: bool,
    pub renders_typed_config: bool,
    pub requires_real_binary: bool,
}

/// Minimal dry-run plan returned by a backend adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DryRunPlan {
    pub backend_name: &'static str,
    pub command_preview: String,
    pub config_summary: String,
}

/// Minimal backend contract marker for early workspace scaffolding.
pub trait DataplaneBackend {
    /// Stable backend name used in diagnostics and selection.
    fn backend_name(&self) -> &'static str;

    /// Build a safe dry-run plan without touching the real network.
    fn build_dry_run_plan(&self) -> DryRunPlan;

    /// Describe adapter capabilities for operator-facing diagnostics.
    fn capabilities(&self) -> AdapterCapabilities;
}
