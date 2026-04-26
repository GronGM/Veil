#![forbid(unsafe_code)]

//! Xray adapter skeleton for Veil.

use veil_adapter_api::{DataplaneBackend, DryRunPlan};

/// Placeholder Xray backend adapter.
#[derive(Debug, Clone, Default)]
pub struct XrayBackend;

impl DataplaneBackend for XrayBackend {
    fn backend_name(&self) -> &'static str {
        "xray-core"
    }

    fn build_dry_run_plan(&self) -> DryRunPlan {
        DryRunPlan {
            backend_name: self.backend_name(),
            command_preview: "xray -test -config generated/veil-xray.json".to_string(),
            config_summary: "typed Xray config preview for a future generated runtime artifact".to_string(),
        }
    }
}
