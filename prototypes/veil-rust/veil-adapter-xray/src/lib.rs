#![forbid(unsafe_code)]

//! Xray adapter skeleton for Veil.

use veil_adapter_api::DataplaneBackend;

/// Placeholder Xray backend adapter.
#[derive(Debug, Clone, Default)]
pub struct XrayBackend;

impl DataplaneBackend for XrayBackend {
    fn backend_name(&self) -> &'static str {
        "xray-core"
    }
}
