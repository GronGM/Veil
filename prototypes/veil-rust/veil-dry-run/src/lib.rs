#![forbid(unsafe_code)]

//! Shared dry-run domain types for Veil.

use veil_adapter_api::{AdapterCapabilityReason, AdapterTransportReason};
use veil_manifest::{ClientPlatform, PlatformAdapterKind};
use veil_policy::{
    BackendPolicyReason, RuntimeSupportReason, RuntimeSupportTier, TransportPolicyReason,
};

/// Human-readable and machine-friendly dry-run snapshot shared across layers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DryRunReport {
    pub provider_name: String,
    pub profile_name: String,
    pub backend_name: String,
    pub transport_profile: String,
    pub outcome: DryRunOutcome,
    pub backend_policy_reason: BackendPolicyReason,
    pub transport_policy_reason: TransportPolicyReason,
    pub adapter_transport_reason: AdapterTransportReason,
    pub adapter_capability_reason: AdapterCapabilityReason,
    pub runtime_support_tier: RuntimeSupportTier,
    pub runtime_support_reason: RuntimeSupportReason,
    pub command_preview: String,
    pub config_summary: String,
    pub decision_summary: String,
    pub transport_summary: String,
    pub adapter_transport_summary: String,
    pub adapter_capability_summary: String,
    pub runtime_support_summary: String,
    pub runtime_support_caveats: Vec<String>,
    pub client_platform: ClientPlatform,
    pub platform_adapter: PlatformAdapterKind,
    pub routing_summary: String,
    pub routing_candidates: Vec<RoutingCandidateReport>,
    pub capabilities_summary: String,
    pub allowed: bool,
}

/// Stable overall dry-run status derived from the typed decision chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DryRunOutcome {
    Allowed,
    BlockedByPolicy,
    BlockedByAdapter,
}

impl DryRunOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
            Self::BlockedByPolicy => "blocked_by_policy",
            Self::BlockedByAdapter => "blocked_by_adapter",
        }
    }
}

impl DryRunReport {
    pub fn set_routing(
        &mut self,
        routing_summary: String,
        routing_candidates: Vec<RoutingCandidateReport>,
    ) {
        self.routing_summary = routing_summary;
        self.routing_candidates = routing_candidates;
    }

    /// Render a compact operator-facing report for CLI output.
    pub fn render(&self) -> String {
        let routing_candidates = self
            .routing_candidates
            .iter()
            .map(|candidate| {
                format!(
                    "{}:{}:{}",
                    candidate.backend_name,
                    candidate.eligible,
                    candidate.reason.as_str()
                )
            })
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            concat!(
                "Veil dry-run\n",
                "provider: {}\n",
                "profile: {}\n",
                "backend: {}\n",
                "transport: {}\n",
                "client_platform: {}\n",
                "platform_adapter: {}\n",
                "outcome: {}\n",
                "allowed: {}\n",
                "backend_reason: {}\n",
                "transport_reason: {}\n",
                "adapter_transport_reason: {}\n",
                "adapter_capability_reason: {}\n",
                "runtime_support_tier: {}\n",
                "runtime_support_reason: {}\n",
                "decision: {}\n",
                "transport_decision: {}\n",
                "adapter_transport: {}\n",
                "adapter_capability: {}\n",
                "runtime_support: {}\n",
                "runtime_support_caveats: {}\n",
                "routing: {}\n",
                "routing_candidates: {}\n",
                "capabilities: {}\n",
                "command: {}\n",
                "config: {}"
            ),
            self.provider_name,
            self.profile_name,
            self.backend_name,
            self.transport_profile,
            self.client_platform.as_str(),
            self.platform_adapter.as_str(),
            self.outcome.as_str(),
            self.allowed,
            self.backend_policy_reason.as_str(),
            self.transport_policy_reason.as_str(),
            self.adapter_transport_reason.as_str(),
            self.adapter_capability_reason.as_str(),
            self.runtime_support_tier.as_str(),
            self.runtime_support_reason.as_str(),
            self.decision_summary,
            self.transport_summary,
            self.adapter_transport_summary,
            self.adapter_capability_summary,
            self.runtime_support_summary,
            self.runtime_support_caveats.join(" | "),
            self.routing_summary,
            routing_candidates,
            self.capabilities_summary,
            self.command_preview,
            self.config_summary
        )
    }
}

/// Stable routing-level reason code for candidate eligibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingEligibilityReason {
    Eligible,
    RejectedByOutcome,
    MissingEligibility,
    NotSelectedByPolicy,
}

impl RoutingEligibilityReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Eligible => "eligible",
            Self::RejectedByOutcome => "rejected_by_outcome",
            Self::MissingEligibility => "missing_eligibility",
            Self::NotSelectedByPolicy => "not_selected_by_policy",
        }
    }
}

/// Shared routing-level classification for each backend candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingCandidateReport {
    pub backend_name: &'static str,
    pub eligible: bool,
    pub reason: RoutingEligibilityReason,
    pub summary: String,
}
