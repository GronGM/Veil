#![forbid(unsafe_code)]

//! Adapter boundary skeleton for Veil.

use veil_transport::TransportProfile;

/// Minimal operator-facing capability snapshot for an adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterCapabilities {
    pub backend_name: &'static str,
    pub supports_dry_run: bool,
    pub renders_typed_config: bool,
    pub requires_real_binary: bool,
    pub supported_transports: Vec<TransportProfile>,
}

/// Minimal dry-run plan returned by a backend adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DryRunPlan {
    pub backend_name: &'static str,
    pub command_preview: String,
    pub config_summary: String,
}

/// Adapter-level validation result for a requested transport profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterTransportDecision {
    pub supported: bool,
    pub reason: AdapterTransportReason,
    pub summary: String,
}

/// Stable adapter transport reason code for dry-run and diagnostics flows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterTransportReason {
    Supported,
    UnsupportedTransport,
}

impl AdapterTransportReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::UnsupportedTransport => "unsupported_transport",
        }
    }
}

/// Minimal capability requirements used by dry-run compatibility checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdapterCapabilityRequirements {
    pub require_dry_run: bool,
    pub require_typed_config: bool,
    pub require_real_binary: bool,
}

impl AdapterCapabilityRequirements {
    pub fn dry_run_only() -> Self {
        Self {
            require_dry_run: true,
            require_typed_config: false,
            require_real_binary: false,
        }
    }
}

/// Adapter-level validation result for capability requirements beyond transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterCapabilityDecision {
    pub compatible: bool,
    pub reason: AdapterCapabilityReason,
    pub summary: String,
}

/// Stable adapter capability reason code for dry-run and diagnostics flows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterCapabilityReason {
    Supported,
    MissingDryRun,
    MissingTypedConfig,
    MissingRealBinary,
}

impl AdapterCapabilityReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::MissingDryRun => "missing_dry_run",
            Self::MissingTypedConfig => "missing_typed_config",
            Self::MissingRealBinary => "missing_real_binary",
        }
    }
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

/// Evaluate whether an adapter can serve the requested transport profile.
pub fn validate_transport_support(
    capabilities: &AdapterCapabilities,
    requested_transport: TransportProfile,
) -> AdapterTransportDecision {
    let supported = capabilities
        .supported_transports
        .contains(&requested_transport);

    let summary = if supported {
        format!(
            "adapter '{}' supports transport '{}'",
            capabilities.backend_name,
            requested_transport.as_str()
        )
    } else {
        let supported_transports = capabilities
            .supported_transports
            .iter()
            .map(TransportProfile::as_str)
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            "adapter '{}' does not support transport '{}'; supported transports: {}",
            capabilities.backend_name,
            requested_transport.as_str(),
            supported_transports
        )
    };

    AdapterTransportDecision {
        supported,
        reason: if supported {
            AdapterTransportReason::Supported
        } else {
            AdapterTransportReason::UnsupportedTransport
        },
        summary,
    }
}

/// Evaluate whether an adapter satisfies the requested non-transport capabilities.
pub fn validate_capability_requirements(
    capabilities: &AdapterCapabilities,
    requirements: AdapterCapabilityRequirements,
) -> AdapterCapabilityDecision {
    let (compatible, reason, summary) = if requirements.require_dry_run && !capabilities.supports_dry_run {
        (
            false,
            AdapterCapabilityReason::MissingDryRun,
            format!(
                "adapter '{}' does not satisfy capability requirement 'dry_run'",
                capabilities.backend_name
            ),
        )
    } else if requirements.require_typed_config && !capabilities.renders_typed_config {
        (
            false,
            AdapterCapabilityReason::MissingTypedConfig,
            format!(
                "adapter '{}' does not satisfy capability requirement 'typed_config'",
                capabilities.backend_name
            ),
        )
    } else if requirements.require_real_binary && !capabilities.requires_real_binary {
        (
            false,
            AdapterCapabilityReason::MissingRealBinary,
            format!(
                "adapter '{}' does not satisfy capability requirement 'real_binary'",
                capabilities.backend_name
            ),
        )
    } else {
        (
            true,
            AdapterCapabilityReason::Supported,
            format!(
                "adapter '{}' satisfies the requested capability requirements",
                capabilities.backend_name
            ),
        )
    };

    AdapterCapabilityDecision {
        compatible,
        reason,
        summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_validation_accepts_supported_transport() {
        let capabilities = AdapterCapabilities {
            backend_name: "xray-core",
            supports_dry_run: true,
            renders_typed_config: true,
            requires_real_binary: true,
            supported_transports: vec![TransportProfile::TlsTcp, TransportProfile::Grpc],
        };

        let decision = validate_transport_support(&capabilities, TransportProfile::Grpc);
        assert!(decision.supported);
        assert_eq!(decision.reason, AdapterTransportReason::Supported);
        assert!(decision.summary.contains("supports transport 'grpc'"));
    }

    #[test]
    fn transport_validation_reports_supported_set() {
        let capabilities = AdapterCapabilities {
            backend_name: "mock-backend",
            supports_dry_run: true,
            renders_typed_config: false,
            requires_real_binary: false,
            supported_transports: vec![TransportProfile::TlsTcp],
        };

        let decision = validate_transport_support(&capabilities, TransportProfile::Grpc);
        assert!(!decision.supported);
        assert_eq!(decision.reason, AdapterTransportReason::UnsupportedTransport);
        assert!(decision.summary.contains("does not support transport 'grpc'"));
        assert!(decision.summary.contains("supported transports: tls-tcp"));
    }

    #[test]
    fn capability_validation_accepts_dry_run_only_requirements() {
        let capabilities = AdapterCapabilities {
            backend_name: "mock-backend",
            supports_dry_run: true,
            renders_typed_config: false,
            requires_real_binary: false,
            supported_transports: vec![TransportProfile::TlsTcp],
        };

        let decision =
            validate_capability_requirements(&capabilities, AdapterCapabilityRequirements::dry_run_only());
        assert!(decision.compatible);
        assert_eq!(decision.reason, AdapterCapabilityReason::Supported);
    }

    #[test]
    fn capability_validation_reports_missing_typed_config() {
        let capabilities = AdapterCapabilities {
            backend_name: "mock-backend",
            supports_dry_run: true,
            renders_typed_config: false,
            requires_real_binary: false,
            supported_transports: vec![TransportProfile::TlsTcp],
        };

        let decision = validate_capability_requirements(
            &capabilities,
            AdapterCapabilityRequirements {
                require_dry_run: true,
                require_typed_config: true,
                require_real_binary: false,
            },
        );
        assert!(!decision.compatible);
        assert_eq!(decision.reason, AdapterCapabilityReason::MissingTypedConfig);
        assert!(decision.summary.contains("capability requirement 'typed_config'"));
    }
}
