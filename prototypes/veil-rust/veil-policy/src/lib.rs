#![forbid(unsafe_code)]

//! Policy skeleton for Veil.

use veil_adapter_api::AdapterCapabilityRequirements;
use veil_manifest::{ClientPlatform, PlatformAdapterKind, PlatformSupportStatus, ProviderManifest};
use veil_transport::TransportProfile;

/// Minimal policy model for early dry-run wiring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutePolicy {
    pub allow_backend: String,
    pub allow_fallback: bool,
    pub allow_transport: TransportProfile,
    pub required_capabilities: AdapterCapabilityRequirements,
}

/// Minimal decision summary returned by policy evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyDecision {
    pub allowed: bool,
    pub reason: BackendPolicyReason,
    pub summary: String,
}

/// Stable backend policy reason code for dry-run and diagnostics flows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendPolicyReason {
    PreferredAllowed,
    RequiredByPolicy,
    FallbackAllowed,
    BlockedByAllowlist,
}

impl BackendPolicyReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PreferredAllowed => "preferred_allowed",
            Self::RequiredByPolicy => "required_by_policy",
            Self::FallbackAllowed => "fallback_allowed",
            Self::BlockedByAllowlist => "blocked_by_allowlist",
        }
    }
}

/// Minimal transport decision summary returned by policy evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportPolicyDecision {
    pub allowed: bool,
    pub transport_profile: TransportProfile,
    pub reason: TransportPolicyReason,
    pub summary: String,
}

/// Runtime support assessment derived from the manifest contour and selected backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSupportAssessment {
    pub tier: RuntimeSupportTier,
    pub reason: RuntimeSupportReason,
    pub summary: String,
    pub in_mvp_scope: bool,
    pub caveats: Vec<String>,
}

/// Stable support tier for the current runtime contour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeSupportTier {
    MvpSupported,
    FoundationOnly,
    Planned,
    BridgeOnly,
    ContractMismatch,
    DevelopmentOnly,
}

impl RuntimeSupportTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MvpSupported => "mvp-supported",
            Self::FoundationOnly => "foundation-only",
            Self::Planned => "planned",
            Self::BridgeOnly => "bridge-only",
            Self::ContractMismatch => "contract-mismatch",
            Self::DevelopmentOnly => "development-only",
        }
    }
}

/// Stable reason code for runtime support assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeSupportReason {
    ExplicitMvpContour,
    LinuxNonMvpContour,
    PlannedPlatform,
    BridgeOnlyPlatform,
    ManifestContractMismatch,
    SimulatedContour,
}

impl RuntimeSupportReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ExplicitMvpContour => "explicit_mvp_contour",
            Self::LinuxNonMvpContour => "linux_non_mvp_contour",
            Self::PlannedPlatform => "planned_platform",
            Self::BridgeOnlyPlatform => "bridge_only_platform",
            Self::ManifestContractMismatch => "manifest_contract_mismatch",
            Self::SimulatedContour => "simulated_contour",
        }
    }
}

/// Stable transport policy reason code for dry-run and diagnostics flows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportPolicyReason {
    PreferredAllowed,
    RequiredByPolicy,
}

impl TransportPolicyReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PreferredAllowed => "preferred_allowed",
            Self::RequiredByPolicy => "required_by_policy",
        }
    }
}

impl RoutePolicy {
    /// Build a small demo policy for the public dry-run path.
    pub fn demo() -> Self {
        Self {
            allow_backend: "xray-core".to_string(),
            allow_fallback: false,
            allow_transport: TransportProfile::TlsTcp,
            required_capabilities: AdapterCapabilityRequirements::dry_run_only(),
        }
    }

    /// Build a demo policy mismatch scenario for CLI testing.
    pub fn mismatch_demo() -> Self {
        Self {
            allow_backend: "mock-backend".to_string(),
            allow_fallback: false,
            allow_transport: TransportProfile::TlsTcp,
            required_capabilities: AdapterCapabilityRequirements::dry_run_only(),
        }
    }

    /// Build a demo transport mismatch scenario for CLI testing.
    pub fn transport_mismatch_demo() -> Self {
        Self {
            allow_backend: "xray-core".to_string(),
            allow_fallback: false,
            allow_transport: TransportProfile::Grpc,
            required_capabilities: AdapterCapabilityRequirements::dry_run_only(),
        }
    }

    /// Build a demo policy that requires typed config support.
    pub fn typed_config_demo() -> Self {
        Self {
            allow_backend: "mock-backend".to_string(),
            allow_fallback: false,
            allow_transport: TransportProfile::TlsTcp,
            required_capabilities: AdapterCapabilityRequirements {
                require_dry_run: true,
                require_typed_config: true,
                require_real_binary: false,
            },
        }
    }

    /// Build a demo policy that requires a real backend binary.
    pub fn real_binary_demo() -> Self {
        Self {
            allow_backend: "mock-backend".to_string(),
            allow_fallback: false,
            allow_transport: TransportProfile::TlsTcp,
            required_capabilities: AdapterCapabilityRequirements {
                require_dry_run: true,
                require_typed_config: false,
                require_real_binary: true,
            },
        }
    }

    /// Evaluate the manifest against the current backend choice.
    pub fn evaluate(&self, manifest: &ProviderManifest, backend_name: &str) -> PolicyDecision {
        let allowed = backend_name == self.allow_backend;
        let (reason, summary) = if allowed && backend_name == manifest.preferred_backend {
            (
                BackendPolicyReason::PreferredAllowed,
                format!(
                    "policy allows backend '{}' for provider '{}' profile '{}'",
                    backend_name, manifest.provider_name, manifest.profile_name
                ),
            )
        } else if allowed {
            (
                BackendPolicyReason::RequiredByPolicy,
                format!(
                    "policy requires backend '{}' for provider '{}' profile '{}'",
                    backend_name, manifest.provider_name, manifest.profile_name
                ),
            )
        } else if self.allow_fallback {
            (
                BackendPolicyReason::FallbackAllowed,
                format!(
                    "policy prefers '{}' but allows fallback from '{}'",
                    self.allow_backend, backend_name
                ),
            )
        } else {
            (
                BackendPolicyReason::BlockedByAllowlist,
                format!(
                    "policy blocks backend '{}' for provider '{}' profile '{}' because only '{}' is allowed",
                    backend_name, manifest.provider_name, manifest.profile_name, self.allow_backend
                ),
            )
        };

        PolicyDecision {
            allowed,
            reason,
            summary,
        }
    }

    /// Evaluate whether the manifest transport intent is allowed by policy.
    pub fn evaluate_transport(&self, manifest: &ProviderManifest) -> TransportPolicyDecision {
        let allowed = manifest.preferred_transport == self.allow_transport;
        let (reason, summary) = if allowed {
            (
                TransportPolicyReason::PreferredAllowed,
                format!(
                    "policy allows transport '{}' for provider '{}' profile '{}'",
                    manifest.preferred_transport.as_str(),
                    manifest.provider_name,
                    manifest.profile_name
                ),
            )
        } else {
            (
                TransportPolicyReason::RequiredByPolicy,
                format!(
                    "policy requires transport '{}' instead of preferred '{}'",
                    self.allow_transport.as_str(),
                    manifest.preferred_transport.as_str()
                ),
            )
        };

        TransportPolicyDecision {
            allowed,
            transport_profile: if allowed {
                manifest.preferred_transport
            } else {
                self.allow_transport
            },
            reason,
            summary,
        }
    }

    /// Assess whether the selected backend fits the manifest-declared runtime support contour.
    pub fn assess_runtime_support(
        &self,
        manifest: &ProviderManifest,
        backend_name: &str,
    ) -> RuntimeSupportAssessment {
        let mut caveats = Vec::new();
        let capability = &manifest.platform_capability;

        if capability.platform != manifest.client_platform {
            caveats.push(format!(
                "manifest platform capability targets '{}' instead of '{}'",
                capability.platform.as_str(),
                manifest.client_platform.as_str()
            ));
        }
        if !capability
            .supported_backends
            .iter()
            .any(|backend| backend == backend_name)
        {
            caveats.push(format!(
                "manifest platform capability for '{}' does not declare backend '{}'",
                manifest.client_platform.as_str(),
                backend_name
            ));
        }
        if capability.platform_adapter != manifest.platform_adapter {
            caveats.push(format!(
                "manifest platform capability for '{}' expects adapter '{}', not '{}'",
                manifest.client_platform.as_str(),
                capability.platform_adapter.as_str(),
                manifest.platform_adapter.as_str()
            ));
        }

        if manifest.platform_adapter == PlatformAdapterKind::Simulated {
            return RuntimeSupportAssessment {
                tier: RuntimeSupportTier::DevelopmentOnly,
                reason: RuntimeSupportReason::SimulatedContour,
                summary: "simulated adapter is intended for modeling and dry-run validation rather than MVP runtime claims"
                    .to_string(),
                in_mvp_scope: false,
                caveats,
            };
        }

        if manifest.client_platform == ClientPlatform::Linux
            && backend_name == "xray-core"
            && manifest.platform_adapter == PlatformAdapterKind::Linux
        {
            if capability.status != PlatformSupportStatus::MvpSupported {
                caveats.push(
                    "manifest platform capability does not mark the linux xray contour as mvp-supported"
                        .to_string(),
                );
            }

            if caveats.is_empty() {
                return RuntimeSupportAssessment {
                    tier: RuntimeSupportTier::MvpSupported,
                    reason: RuntimeSupportReason::ExplicitMvpContour,
                    summary:
                        "linux xray contour matches the current honest MVP runtime target".to_string(),
                    in_mvp_scope: true,
                    caveats,
                };
            }

            return RuntimeSupportAssessment {
                tier: RuntimeSupportTier::ContractMismatch,
                reason: RuntimeSupportReason::ManifestContractMismatch,
                summary: "selected linux xray contour matches the repository MVP target, but the manifest contract does not declare the same contour"
                    .to_string(),
                in_mvp_scope: false,
                caveats,
            };
        }

        if manifest.client_platform == ClientPlatform::Linux
            && manifest.platform_adapter == PlatformAdapterKind::Linux
        {
            if !caveats.is_empty() {
                return RuntimeSupportAssessment {
                    tier: RuntimeSupportTier::ContractMismatch,
                    reason: RuntimeSupportReason::ManifestContractMismatch,
                    summary:
                        "selected linux contour does not match the manifest-declared support contract"
                            .to_string(),
                    in_mvp_scope: false,
                    caveats,
                };
            }

            return RuntimeSupportAssessment {
                tier: RuntimeSupportTier::FoundationOnly,
                reason: RuntimeSupportReason::LinuxNonMvpContour,
                summary:
                    "linux remains the reference runtime, but this contour is outside the first MVP target"
                        .to_string(),
                in_mvp_scope: false,
                caveats: vec![
                    "the first MVP target is the explicit linux + xray-core contour".to_string(),
                ],
            };
        }

        if manifest.client_platform == ClientPlatform::Ios
            || capability.status == PlatformSupportStatus::BridgeOnly
        {
            if capability.status != PlatformSupportStatus::BridgeOnly
                && manifest.client_platform == ClientPlatform::Ios
            {
                caveats.push(
                    "ios should stay on a bridge-only contract until a dedicated runtime exists"
                        .to_string(),
                );
            }

            return RuntimeSupportAssessment {
                tier: if caveats.is_empty() {
                    RuntimeSupportTier::BridgeOnly
                } else {
                    RuntimeSupportTier::ContractMismatch
                },
                reason: if caveats.is_empty() {
                    RuntimeSupportReason::BridgeOnlyPlatform
                } else {
                    RuntimeSupportReason::ManifestContractMismatch
                },
                summary:
                    "ios remains on a bridge-oriented contract path and is outside the first MVP runtime target"
                        .to_string(),
                in_mvp_scope: false,
                caveats,
            };
        }

        if matches!(
            manifest.client_platform,
            ClientPlatform::Windows | ClientPlatform::Macos | ClientPlatform::Android
        ) {
            return RuntimeSupportAssessment {
                tier: if caveats.is_empty() {
                    RuntimeSupportTier::Planned
                } else {
                    RuntimeSupportTier::ContractMismatch
                },
                reason: if caveats.is_empty() {
                    RuntimeSupportReason::PlannedPlatform
                } else {
                    RuntimeSupportReason::ManifestContractMismatch
                },
                summary: "this runtime contour follows the shared product model but remains outside the first hardened MVP target"
                    .to_string(),
                in_mvp_scope: false,
                caveats,
            };
        }

        RuntimeSupportAssessment {
            tier: if caveats.is_empty() {
                RuntimeSupportTier::DevelopmentOnly
            } else {
                RuntimeSupportTier::ContractMismatch
            },
            reason: if caveats.is_empty() {
                RuntimeSupportReason::SimulatedContour
            } else {
                RuntimeSupportReason::ManifestContractMismatch
            },
            summary:
                "this runtime contour is intended for modeling and local experimentation rather than MVP claims"
                    .to_string(),
            in_mvp_scope: false,
            caveats,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_policy_allows_xray_backend() {
        let manifest = ProviderManifest::demo();
        let policy = RoutePolicy::demo();
        let decision = policy.evaluate(&manifest, "xray-core");

        assert!(decision.allowed);
        assert_eq!(decision.reason, BackendPolicyReason::PreferredAllowed);
        assert!(decision.summary.contains("policy allows backend 'xray-core'"));
    }

    #[test]
    fn mismatch_policy_blocks_xray_backend() {
        let manifest = ProviderManifest::demo();
        let policy = RoutePolicy::mismatch_demo();
        let decision = policy.evaluate(&manifest, "xray-core");

        assert!(!decision.allowed);
        assert_eq!(decision.reason, BackendPolicyReason::BlockedByAllowlist);
        assert!(decision.summary.contains("policy blocks backend 'xray-core'"));
        assert!(decision.summary.contains("only 'mock-backend' is allowed"));
    }

    #[test]
    fn transport_policy_allows_manifest_transport() {
        let manifest = ProviderManifest::demo();
        let policy = RoutePolicy::demo();
        let decision = policy.evaluate_transport(&manifest);

        assert!(decision.allowed);
        assert_eq!(decision.transport_profile, TransportProfile::TlsTcp);
        assert_eq!(decision.reason, TransportPolicyReason::PreferredAllowed);
        assert!(decision.summary.contains("policy allows transport 'tls-tcp'"));
    }

    #[test]
    fn transport_policy_reports_required_transport() {
        let manifest = ProviderManifest::demo();
        let policy = RoutePolicy::transport_mismatch_demo();
        let decision = policy.evaluate_transport(&manifest);

        assert!(!decision.allowed);
        assert_eq!(decision.transport_profile, TransportProfile::Grpc);
        assert_eq!(decision.reason, TransportPolicyReason::RequiredByPolicy);
        assert!(decision.summary.contains("requires transport 'grpc'"));
    }

    #[test]
    fn runtime_support_marks_linux_xray_as_mvp_supported() {
        let manifest = ProviderManifest::demo();
        let policy = RoutePolicy::demo();
        let assessment = policy.assess_runtime_support(&manifest, "xray-core");

        assert_eq!(assessment.tier, RuntimeSupportTier::MvpSupported);
        assert_eq!(assessment.reason, RuntimeSupportReason::ExplicitMvpContour);
        assert!(assessment.in_mvp_scope);
        assert!(assessment.caveats.is_empty());
    }

    #[test]
    fn runtime_support_marks_windows_contour_as_planned() {
        let manifest = ProviderManifest::planned_windows_demo();
        let policy = RoutePolicy::demo();
        let assessment = policy.assess_runtime_support(&manifest, "xray-core");

        assert_eq!(assessment.tier, RuntimeSupportTier::Planned);
        assert_eq!(assessment.reason, RuntimeSupportReason::PlannedPlatform);
        assert!(!assessment.in_mvp_scope);
    }

    #[test]
    fn runtime_support_marks_valid_linux_non_mvp_contour_as_foundation_only() {
        let manifest = ProviderManifest::linux_foundation_demo();
        let policy = RoutePolicy::mismatch_demo();
        let assessment = policy.assess_runtime_support(&manifest, "mock-backend");

        assert_eq!(assessment.tier, RuntimeSupportTier::FoundationOnly);
        assert_eq!(assessment.reason, RuntimeSupportReason::LinuxNonMvpContour);
        assert!(!assessment.in_mvp_scope);
    }

    #[test]
    fn runtime_support_reports_contract_mismatch() {
        let manifest = ProviderManifest::contract_mismatch_demo();
        let policy = RoutePolicy::demo();
        let assessment = policy.assess_runtime_support(&manifest, "xray-core");

        assert_eq!(assessment.tier, RuntimeSupportTier::ContractMismatch);
        assert_eq!(assessment.reason, RuntimeSupportReason::ManifestContractMismatch);
        assert!(!assessment.caveats.is_empty());
    }
}
