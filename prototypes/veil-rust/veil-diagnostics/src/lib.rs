#![forbid(unsafe_code)]

//! Diagnostics skeleton for Veil.

use veil_dry_run::DryRunReport;

/// Minimal redacted diagnostics view for dry-run reporting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedactedDryRunDiagnostics {
    pub provider_label: String,
    pub profile_label: String,
    pub backend_name: String,
    pub transport_profile: String,
    pub client_platform: String,
    pub platform_adapter: String,
    pub outcome: String,
    pub allowed: bool,
    pub backend_policy_reason: String,
    pub transport_policy_reason: String,
    pub adapter_transport_reason: String,
    pub adapter_capability_reason: String,
    pub runtime_support_tier: String,
    pub runtime_support_reason: String,
    pub decision_summary: String,
    pub transport_summary: String,
    pub adapter_transport_summary: String,
    pub adapter_capability_summary: String,
    pub runtime_support_summary: String,
    pub runtime_support_caveats: Vec<String>,
    pub routing_summary: String,
    pub routing_candidates: Vec<String>,
    pub capabilities_summary: String,
}

impl RedactedDryRunDiagnostics {
    pub fn from_report(report: &DryRunReport) -> Self {
        Self {
            provider_label: redact_name(&report.provider_name),
            profile_label: redact_name(&report.profile_name),
            backend_name: report.backend_name.clone(),
            transport_profile: report.transport_profile.clone(),
            client_platform: report.client_platform.as_str().to_string(),
            platform_adapter: report.platform_adapter.as_str().to_string(),
            outcome: report.outcome.as_str().to_string(),
            allowed: report.allowed,
            backend_policy_reason: report.backend_policy_reason.as_str().to_string(),
            transport_policy_reason: report.transport_policy_reason.as_str().to_string(),
            adapter_transport_reason: report.adapter_transport_reason.as_str().to_string(),
            adapter_capability_reason: report.adapter_capability_reason.as_str().to_string(),
            runtime_support_tier: report.runtime_support_tier.as_str().to_string(),
            runtime_support_reason: report.runtime_support_reason.as_str().to_string(),
            decision_summary: report.decision_summary.clone(),
            transport_summary: report.transport_summary.clone(),
            adapter_transport_summary: report.adapter_transport_summary.clone(),
            adapter_capability_summary: report.adapter_capability_summary.clone(),
            runtime_support_summary: report.runtime_support_summary.clone(),
            runtime_support_caveats: report.runtime_support_caveats.clone(),
            routing_summary: report.routing_summary.clone(),
            routing_candidates: report
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
                .collect(),
            capabilities_summary: report.capabilities_summary.clone(),
        }
    }

    /// Render a compact redacted diagnostics block for CLI output.
    pub fn render(&self) -> String {
        format!(
            "Veil diagnostics\nprovider: {}\nprofile: {}\nbackend: {}\ntransport: {}\nclient_platform: {}\nplatform_adapter: {}\noutcome: {}\nallowed: {}\nbackend_reason: {}\ntransport_reason: {}\nadapter_transport_reason: {}\nadapter_capability_reason: {}\nruntime_support_tier: {}\nruntime_support_reason: {}\ndecision: {}\ntransport_decision: {}\nadapter_transport: {}\nadapter_capability: {}\nruntime_support: {}\nruntime_support_caveats: {}\nrouting: {}\nrouting_candidates: {}\ncapabilities: {}",
            self.provider_label,
            self.profile_label,
            self.backend_name,
            self.transport_profile,
            self.client_platform,
            self.platform_adapter,
            self.outcome,
            self.allowed,
            self.backend_policy_reason,
            self.transport_policy_reason,
            self.adapter_transport_reason,
            self.adapter_capability_reason,
            self.runtime_support_tier,
            self.runtime_support_reason,
            self.decision_summary,
            self.transport_summary,
            self.adapter_transport_summary,
            self.adapter_capability_summary,
            self.runtime_support_summary,
            self.runtime_support_caveats.join(" | "),
            self.routing_summary,
            self.routing_candidates.join(", "),
            self.capabilities_summary
        )
    }

    /// Render a small JSON-like diagnostics artifact for machine-readable workflows.
    pub fn render_json(&self) -> String {
        format!(
            concat!(
                "{\n",
                "  \"provider_label\": \"{}\",\n",
                "  \"profile_label\": \"{}\",\n",
                "  \"backend_name\": \"{}\",\n",
                "  \"transport_profile\": \"{}\",\n",
                "  \"client_platform\": \"{}\",\n",
                "  \"platform_adapter\": \"{}\",\n",
                "  \"outcome\": \"{}\",\n",
                "  \"allowed\": {},\n",
                "  \"backend_policy_reason\": \"{}\",\n",
                "  \"transport_policy_reason\": \"{}\",\n",
                "  \"adapter_transport_reason\": \"{}\",\n",
                "  \"adapter_capability_reason\": \"{}\",\n",
                "  \"runtime_support_tier\": \"{}\",\n",
                "  \"runtime_support_reason\": \"{}\",\n",
                "  \"decision_summary\": \"{}\",\n",
                "  \"transport_summary\": \"{}\",\n",
                "  \"adapter_transport_summary\": \"{}\",\n",
                "  \"adapter_capability_summary\": \"{}\",\n",
                "  \"runtime_support_summary\": \"{}\",\n",
                "  \"runtime_support_caveats\": \"{}\",\n",
                "  \"routing_summary\": \"{}\",\n",
                "  \"routing_candidates\": \"{}\",\n",
                "  \"capabilities_summary\": \"{}\"\n",
                "}}"
            ),
            self.provider_label,
            self.profile_label,
            self.backend_name,
            self.transport_profile,
            self.client_platform,
            self.platform_adapter,
            self.outcome,
            self.allowed,
            self.backend_policy_reason,
            self.transport_policy_reason,
            self.adapter_transport_reason,
            self.adapter_capability_reason,
            self.runtime_support_tier,
            self.runtime_support_reason,
            escape_json(&self.decision_summary),
            escape_json(&self.transport_summary),
            escape_json(&self.adapter_transport_summary),
            escape_json(&self.adapter_capability_summary),
            escape_json(&self.runtime_support_summary),
            escape_json(&self.runtime_support_caveats.join(" | ")),
            escape_json(&self.routing_summary),
            escape_json(&self.routing_candidates.join(", ")),
            escape_json(&self.capabilities_summary)
        )
    }
}

fn redact_name(value: &str) -> String {
    if value.len() <= 3 {
        "***".to_string()
    } else {
        format!("{}***", &value[..3])
    }
}

fn escape_json(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
