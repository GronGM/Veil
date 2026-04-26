#![forbid(unsafe_code)]

//! Mock adapter skeleton for Veil.

use veil_adapter_api::{DataplaneBackend, DryRunPlan};

/// Placeholder mock backend adapter for generic contract testing.
#[derive(Debug, Clone, Default)]
pub struct MockBackend;

impl DataplaneBackend for MockBackend {
    fn backend_name(&self) -> &'static str {
        "mock-backend"
    }

    fn build_dry_run_plan(&self) -> DryRunPlan {
        DryRunPlan {
            backend_name: self.backend_name(),
            command_preview: "veil-mock --dry-run --profile generated/mock-profile.json".to_string(),
            config_summary: "mock backend preview for contract and diagnostics testing".to_string(),
        }
    }
}
