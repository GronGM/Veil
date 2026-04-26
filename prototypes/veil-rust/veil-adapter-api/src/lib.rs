#![forbid(unsafe_code)]

//! Adapter boundary skeleton for Veil.

/// Minimal backend contract marker for early workspace scaffolding.
pub trait DataplaneBackend {
    /// Stable backend name used in diagnostics and selection.
    fn backend_name(&self) -> &'static str;
}
