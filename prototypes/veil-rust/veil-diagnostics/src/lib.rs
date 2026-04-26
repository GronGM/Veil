#![forbid(unsafe_code)]

//! Diagnostics skeleton for Veil.

/// Minimal redacted diagnostics view for dry-run reporting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedactedDryRunDiagnostics {
    pub provider_label: String,
    pub profile_label: String,
    pub backend_name: String,
    pub allowed: bool,
    pub decision_summary: String,
}

impl RedactedDryRunDiagnostics {
    pub fn new(
        provider_name: &str,
        profile_name: &str,
        backend_name: &str,
        allowed: bool,
        decision_summary: &str,
    ) -> Self {
        Self {
            provider_label: redact_name(provider_name),
            profile_label: redact_name(profile_name),
            backend_name: backend_name.to_string(),
            allowed,
            decision_summary: decision_summary.to_string(),
        }
    }

    /// Render a compact redacted diagnostics block for CLI output.
    pub fn render(&self) -> String {
        format!(
            "Veil diagnostics\nprovider: {}\nprofile: {}\nbackend: {}\nallowed: {}\ndecision: {}",
            self.provider_label,
            self.profile_label,
            self.backend_name,
            self.allowed,
            self.decision_summary
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
                "  \"allowed\": {},\n",
                "  \"decision_summary\": \"{}\"\n",
                "}}"
            ),
            self.provider_label,
            self.profile_label,
            self.backend_name,
            self.allowed,
            escape_json(&self.decision_summary)
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
