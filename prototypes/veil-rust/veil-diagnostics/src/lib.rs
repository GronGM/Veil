//! Incident and support-facing diagnostics skeleton for Veil.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use veil_manifest::{ProviderManifest, XrayEndpointMetadata};
use veil_policy::{IncidentGuidance, RouteSelectionPolicy, RuntimeSupportAssessment};
use veil_routing::RejectedRouteCandidate;

const REDACTED_TOKEN: &str = "[REDACTED_TOKEN]";
const REDACTED_PRIVATE_KEY: &str = "[REDACTED_PRIVATE_KEY]";
const REDACTED_SECRET: &str = "[REDACTED_SECRET]";
const REDACTED_QUERY_VALUE: &str = "[REDACTED_QUERY_VALUE]";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncidentReport {
    pub severity: String,
    pub headline: String,
    pub recommended_action: String,
    pub selected_endpoint_id: Option<String>,
    pub route_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactedIncidentReport {
    pub severity: String,
    pub headline: String,
    pub recommended_action: String,
    pub selected_endpoint_id: Option<String>,
    pub route_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteDiagnostics {
    pub selected_endpoint_id: Option<String>,
    pub selected_backend_name: Option<String>,
    pub route_summary: String,
    pub fallback_triggered: bool,
    pub rejected_routes: Vec<RejectedRouteCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactedRejectedRoute {
    pub endpoint_id: String,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactedRouteDiagnostics {
    pub selected_endpoint_id: Option<String>,
    pub selected_backend_name: Option<String>,
    pub route_summary: String,
    pub fallback_triggered: bool,
    pub rejected_routes: Vec<RedactedRejectedRoute>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactedStringListPolicyField {
    pub values: Vec<String>,
    pub redaction_applied: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactedPolicyDiagnostics {
    pub route_preference: String,
    pub fallback_mode: String,
    pub allow_metered_networks: bool,
    pub allow_expensive_networks: bool,
    pub preferred_regions: RedactedStringListPolicyField,
    pub allowed_regions: RedactedStringListPolicyField,
    pub denied_regions: RedactedStringListPolicyField,
    pub allowed_backends: RedactedStringListPolicyField,
    pub denied_backends: RedactedStringListPolicyField,
    pub allowed_transports: RedactedStringListPolicyField,
    pub denied_transports: RedactedStringListPolicyField,
    pub retry_budget_max_candidates: u8,
    pub forbid_active_cooldown: bool,
    pub known_good_enabled: bool,
    pub known_good_bonus_points: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactedManifestMetadata {
    pub schema_version: u32,
    pub provider_profile_schema_version: Option<u32>,
    pub generated_at: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactedManifestCapability {
    pub platform: String,
    pub supported_dataplanes: Vec<String>,
    pub network_adapter: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactedEndpointTransportDetails {
    pub protocol: String,
    pub stream: String,
    pub security: String,
    pub server_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactedManifestEndpoint {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub transport: String,
    pub region: String,
    pub dataplane: Option<String>,
    pub supported_client_platforms: Vec<String>,
    pub logical_server: Option<String>,
    pub provider_profile_schema_version: Option<u32>,
    pub xray: Option<RedactedEndpointTransportDetails>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactedManifestDiagnostics {
    pub metadata: RedactedManifestMetadata,
    pub capabilities: Vec<RedactedManifestCapability>,
    pub endpoint_count: usize,
    pub endpoints: Vec<RedactedManifestEndpoint>,
    pub preferred_transports: Vec<String>,
    pub transport_retry_budget: u8,
    pub profile_kind: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendPreflightCommandDiagnostics {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendPreflightDiagnostics {
    pub backend_name: String,
    pub ready_for_dry_run_connect: bool,
    pub binary_path: String,
    pub config_path: String,
    pub binary_present: bool,
    pub readiness_note: String,
    pub command: BackendPreflightCommandDiagnostics,
    pub rendered_config: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactedBackendPreflightDiagnostics {
    pub backend_name: String,
    pub ready_for_dry_run_connect: bool,
    pub binary_path: String,
    pub config_path: String,
    pub binary_present: bool,
    pub readiness_note: String,
    pub command: BackendPreflightCommandDiagnostics,
    pub rendered_config: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupportBundle {
    pub manifest_valid: bool,
    pub runtime_support_tier: String,
    pub selected_endpoint_id: Option<String>,
    pub route_summary: String,
    pub endpoint_count: usize,
    pub redacted_manifest_diagnostics: RedactedManifestDiagnostics,
    pub backend_preflight_diagnostics: Option<BackendPreflightDiagnostics>,
    pub redacted_backend_preflight_diagnostics: Option<RedactedBackendPreflightDiagnostics>,
    pub route_diagnostics: RouteDiagnostics,
    pub redacted_route_diagnostics: RedactedRouteDiagnostics,
    pub redacted_policy_diagnostics: RedactedPolicyDiagnostics,
    pub incident: IncidentReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactedSupportBundle {
    pub manifest_valid: bool,
    pub runtime_support_tier: String,
    pub selected_endpoint_id: Option<String>,
    pub route_summary: String,
    pub endpoint_count: usize,
    pub redacted_manifest_diagnostics: RedactedManifestDiagnostics,
    pub redacted_backend_preflight_diagnostics: Option<RedactedBackendPreflightDiagnostics>,
    pub redacted_route_diagnostics: RedactedRouteDiagnostics,
    pub redacted_policy_diagnostics: RedactedPolicyDiagnostics,
    pub incident: RedactedIncidentReport,
}

pub fn build_incident_report(
    manifest_valid: bool,
    runtime_support: RuntimeSupportAssessment,
    guidance: IncidentGuidance,
    selected_endpoint_id: Option<String>,
    route_summary: String,
) -> IncidentReport {
    let severity = if manifest_valid && runtime_support.in_mvp_scope && selected_endpoint_id.is_some()
    {
        "ok".to_string()
    } else if manifest_valid {
        "warning".to_string()
    } else {
        "critical".to_string()
    };

    let headline = match severity.as_str() {
        "ok" => "session plan is inside the current MVP contour".to_string(),
        "warning" => "session plan was built with caveats".to_string(),
        _ => "session plan is blocked by manifest or runtime contract issues".to_string(),
    };

    IncidentReport {
        severity,
        headline,
        recommended_action: guidance.recommended_action,
        selected_endpoint_id,
        route_summary,
    }
}

pub fn build_route_diagnostics(
    selected_endpoint_id: Option<String>,
    selected_backend_name: Option<String>,
    route_summary: String,
    fallback_triggered: bool,
    rejected_routes: Vec<RejectedRouteCandidate>,
) -> RouteDiagnostics {
    RouteDiagnostics {
        selected_endpoint_id,
        selected_backend_name,
        route_summary,
        fallback_triggered,
        rejected_routes,
    }
}

pub fn build_redacted_route_diagnostics(
    route_diagnostics: &RouteDiagnostics,
) -> RedactedRouteDiagnostics {
    RedactedRouteDiagnostics {
        selected_endpoint_id: route_diagnostics.selected_endpoint_id.clone(),
        selected_backend_name: route_diagnostics.selected_backend_name.clone(),
        route_summary: redact_sensitive_text(&route_diagnostics.route_summary),
        fallback_triggered: route_diagnostics.fallback_triggered,
        rejected_routes: route_diagnostics
            .rejected_routes
            .iter()
            .map(|rejected| RedactedRejectedRoute {
                endpoint_id: rejected.endpoint_id.clone(),
                reasons: rejected
                    .reasons
                    .iter()
                    .map(|reason| redact_sensitive_text(reason))
                    .collect(),
            })
            .collect(),
    }
}

pub fn build_redacted_policy_diagnostics(
    policy: &RouteSelectionPolicy,
) -> RedactedPolicyDiagnostics {
    RedactedPolicyDiagnostics {
        route_preference: format!("{:?}", policy.route_preference),
        fallback_mode: format!("{:?}", policy.fallback),
        allow_metered_networks: policy.network_cost.allow_metered_networks,
        allow_expensive_networks: policy.network_cost.allow_expensive_networks,
        preferred_regions: redact_policy_string_list(&policy.regions.preferred),
        allowed_regions: redact_policy_string_list(&policy.regions.allow),
        denied_regions: redact_policy_string_list(&policy.regions.deny),
        allowed_backends: redact_policy_string_list(&policy.backends.allow),
        denied_backends: redact_policy_string_list(&policy.backends.deny),
        allowed_transports: redact_policy_string_list(&policy.transports.allow),
        denied_transports: redact_policy_string_list(&policy.transports.deny),
        retry_budget_max_candidates: policy.retry_budget.max_candidates,
        forbid_active_cooldown: policy.cooldown.forbid_active_cooldown,
        known_good_enabled: policy.known_good.enabled,
        known_good_bonus_points: policy.known_good.bonus_points,
    }
}

pub fn build_redacted_manifest_diagnostics(
    manifest: &ProviderManifest,
) -> RedactedManifestDiagnostics {
    RedactedManifestDiagnostics {
        metadata: RedactedManifestMetadata {
            schema_version: manifest.metadata.schema_version,
            provider_profile_schema_version: manifest.metadata.provider_profile_schema_version,
            generated_at: manifest.metadata.generated_at.to_string(),
            expires_at: manifest.metadata.expires_at.to_string(),
        },
        capabilities: manifest
            .capabilities
            .iter()
            .map(|capability| RedactedManifestCapability {
                platform: capability.platform.clone(),
                supported_dataplanes: capability.supported_dataplanes.clone(),
                network_adapter: capability.network_adapter.clone(),
                status: capability.status.clone(),
            })
            .collect(),
        endpoint_count: manifest.endpoints.len(),
        endpoints: manifest
            .endpoints
            .iter()
            .map(|endpoint| RedactedManifestEndpoint {
                id: endpoint.id.clone(),
                host: redact_sensitive_text(&endpoint.host),
                port: endpoint.port,
                transport: endpoint.transport.clone(),
                region: endpoint.region.clone(),
                dataplane: endpoint.dataplane.clone(),
                supported_client_platforms: endpoint.supported_client_platforms.clone(),
                logical_server: endpoint.logical_server.clone(),
                provider_profile_schema_version: endpoint.provider_profile_schema_version,
                xray: endpoint.xray.as_ref().map(redact_xray_details),
            })
            .collect(),
        preferred_transports: manifest.transport_policy.preferred_order.clone(),
        transport_retry_budget: manifest.transport_policy.retry_budget,
        profile_kind: manifest
            .features
            .profile_kind
            .map(|kind| format!("{kind:?}")),
    }
}

pub fn build_backend_preflight_diagnostics(
    backend_name: impl Into<String>,
    binary_path: impl Into<String>,
    config_path: impl Into<String>,
    binary_present: bool,
    readiness_note: impl Into<String>,
    command_program: impl Into<String>,
    command_args: Vec<String>,
    rendered_config: Value,
) -> BackendPreflightDiagnostics {
    let readiness_note = readiness_note.into();
    BackendPreflightDiagnostics {
        backend_name: backend_name.into(),
        ready_for_dry_run_connect: true,
        binary_path: binary_path.into(),
        config_path: config_path.into(),
        binary_present,
        readiness_note,
        command: BackendPreflightCommandDiagnostics {
            program: command_program.into(),
            args: command_args,
        },
        rendered_config,
    }
}

pub fn build_redacted_backend_preflight_diagnostics(
    preflight: &BackendPreflightDiagnostics,
) -> RedactedBackendPreflightDiagnostics {
    RedactedBackendPreflightDiagnostics {
        backend_name: preflight.backend_name.clone(),
        ready_for_dry_run_connect: preflight.ready_for_dry_run_connect,
        binary_path: redact_sensitive_text(&preflight.binary_path),
        config_path: redact_sensitive_text(&preflight.config_path),
        binary_present: preflight.binary_present,
        readiness_note: redact_sensitive_text(&preflight.readiness_note),
        command: BackendPreflightCommandDiagnostics {
            program: redact_sensitive_text(&preflight.command.program),
            args: preflight
                .command
                .args
                .iter()
                .map(|arg| redact_sensitive_text(arg))
                .collect(),
        },
        rendered_config: redact_json_value(&preflight.rendered_config),
    }
}

pub fn build_support_bundle(
    manifest_valid: bool,
    manifest: &ProviderManifest,
    runtime_support: RuntimeSupportAssessment,
    backend_preflight_diagnostics: Option<BackendPreflightDiagnostics>,
    route_diagnostics: RouteDiagnostics,
    route_policy: &RouteSelectionPolicy,
    incident: IncidentReport,
) -> SupportBundle {
    let redacted_manifest_diagnostics = build_redacted_manifest_diagnostics(manifest);
    let redacted_backend_preflight_diagnostics = backend_preflight_diagnostics
        .as_ref()
        .map(build_redacted_backend_preflight_diagnostics);
    let redacted_route_diagnostics = build_redacted_route_diagnostics(&route_diagnostics);
    let redacted_policy_diagnostics = build_redacted_policy_diagnostics(route_policy);

    SupportBundle {
        manifest_valid,
        runtime_support_tier: runtime_support.tier,
        selected_endpoint_id: route_diagnostics.selected_endpoint_id.clone(),
        route_summary: route_diagnostics.route_summary.clone(),
        endpoint_count: manifest.endpoints.len(),
        redacted_manifest_diagnostics,
        backend_preflight_diagnostics,
        redacted_backend_preflight_diagnostics,
        route_diagnostics,
        redacted_route_diagnostics,
        redacted_policy_diagnostics,
        incident,
    }
}

pub fn build_redacted_support_bundle(bundle: &SupportBundle) -> RedactedSupportBundle {
    RedactedSupportBundle {
        manifest_valid: bundle.manifest_valid,
        runtime_support_tier: bundle.runtime_support_tier.clone(),
        selected_endpoint_id: bundle.selected_endpoint_id.clone(),
        route_summary: bundle.redacted_route_diagnostics.route_summary.clone(),
        endpoint_count: bundle.endpoint_count,
        redacted_manifest_diagnostics: bundle.redacted_manifest_diagnostics.clone(),
        redacted_backend_preflight_diagnostics: bundle
            .redacted_backend_preflight_diagnostics
            .clone(),
        redacted_route_diagnostics: bundle.redacted_route_diagnostics.clone(),
        redacted_policy_diagnostics: bundle.redacted_policy_diagnostics.clone(),
        incident: RedactedIncidentReport {
            severity: bundle.incident.severity.clone(),
            headline: bundle.incident.headline.clone(),
            recommended_action: redact_sensitive_text(&bundle.incident.recommended_action),
            selected_endpoint_id: bundle.incident.selected_endpoint_id.clone(),
            route_summary: redact_sensitive_text(&bundle.incident.route_summary),
        },
    }
}

pub fn redact_sensitive_text(input: &str) -> String {
    let without_private_keys = redact_private_key_blocks(input);
    let without_bearer = redact_bearer_tokens(&without_private_keys);
    let without_url_userinfo = redact_url_userinfo(&without_bearer);
    let without_query_tokens = redact_query_tokens(&without_url_userinfo);
    redact_uuid_like_values(&without_query_tokens)
}

fn redact_private_key_blocks(input: &str) -> String {
    let begin = "-----BEGIN PRIVATE KEY-----";
    let end = "-----END PRIVATE KEY-----";

    if let (Some(start), Some(end_index)) = (input.find(begin), input.find(end)) {
        let suffix_start = end_index + end.len();
        let mut result = String::new();
        result.push_str(&input[..start]);
        result.push_str(REDACTED_PRIVATE_KEY);
        result.push_str(&input[suffix_start..]);
        result
    } else {
        input.to_string()
    }
}

fn redact_bearer_tokens(input: &str) -> String {
    let marker = "Bearer ";
    let mut result = input.to_string();

    while let Some(start) = result.find(marker) {
        let token_start = start + marker.len();
        let token_end = result[token_start..]
            .find(char::is_whitespace)
            .map(|offset| token_start + offset)
            .unwrap_or(result.len());
        result.replace_range(token_start..token_end, REDACTED_TOKEN);
    }

    result
}

fn redact_url_userinfo(input: &str) -> String {
    let mut output = Vec::new();

    for part in input.split_whitespace() {
        if let Some(scheme_index) = part.find("://") {
            let authority_start = scheme_index + 3;
            if let Some(at_index) = part[authority_start..].find('@') {
                let absolute_at = authority_start + at_index;
                if let Some(colon_index) = part[authority_start..absolute_at].find(':') {
                    let absolute_colon = authority_start + colon_index;
                    let mut redacted = String::new();
                    redacted.push_str(&part[..absolute_colon + 1]);
                    redacted.push_str(REDACTED_SECRET);
                    redacted.push_str(&part[absolute_at..]);
                    output.push(redacted);
                    continue;
                }
            }
        }

        output.push(part.to_string());
    }

    output.join(" ")
}

fn redact_query_tokens(input: &str) -> String {
    let mut output = Vec::new();

    for part in input.split_whitespace() {
        if let Some(query_index) = part.find('?') {
            let prefix = &part[..query_index];
            let query = &part[query_index + 1..];
            let mut redacted_pairs = Vec::new();

            for pair in query.split('&') {
                let mut pieces = pair.splitn(2, '=');
                let key = pieces.next().unwrap_or_default();
                let value = pieces.next().unwrap_or_default();
                if matches!(
                    key,
                    "token" | "sig" | "signature" | "credential" | "auth" | "authorization"
                ) {
                    redacted_pairs.push(format!("{key}={REDACTED_QUERY_VALUE}"));
                } else if value.is_empty() {
                    redacted_pairs.push(key.to_string());
                } else {
                    redacted_pairs.push(format!("{key}={value}"));
                }
            }

            output.push(format!("{prefix}?{}", redacted_pairs.join("&")));
        } else {
            output.push(part.to_string());
        }
    }

    output.join(" ")
}

fn redact_uuid_like_values(input: &str) -> String {
    let mut result = input.to_string();
    for token in input.split(|character: char| {
        character.is_whitespace()
            || matches!(character, ',' | ';' | ')' | '(' | '[' | ']' | '"' | '\'')
    }) {
        let trimmed = token
            .trim_matches(|character: char| matches!(character, '.' | ':' | '=' | '/'));
        if looks_like_uuid(trimmed) {
            result = result.replace(trimmed, REDACTED_SECRET);
        }
    }
    result
}

fn redact_json_value(value: &Value) -> Value {
    match value {
        Value::String(content) => Value::String(redact_sensitive_text(content)),
        Value::Array(items) => Value::Array(items.iter().map(redact_json_value).collect()),
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| (key.clone(), redact_json_value(value)))
                .collect(),
        ),
        _ => value.clone(),
    }
}

fn looks_like_uuid(value: &str) -> bool {
    if value.len() != 36 {
        return false;
    }

    for (index, character) in value.chars().enumerate() {
        if matches!(index, 8 | 13 | 18 | 23) {
            if character != '-' {
                return false;
            }
        } else if !character.is_ascii_hexdigit() {
            return false;
        }
    }

    true
}

fn redact_xray_details(xray: &XrayEndpointMetadata) -> RedactedEndpointTransportDetails {
    RedactedEndpointTransportDetails {
        protocol: xray.protocol.clone(),
        stream: xray.stream.clone(),
        security: xray.security.clone(),
        server_name: xray
            .server_name
            .as_ref()
            .map(|server_name| redact_sensitive_text(server_name)),
    }
}

fn redact_policy_string_list(values: &[String]) -> RedactedStringListPolicyField {
    let redacted_values = values
        .iter()
        .map(|value| redact_sensitive_text(value))
        .collect::<Vec<_>>();
    let redaction_applied = values
        .iter()
        .zip(redacted_values.iter())
        .any(|(original, redacted)| original != redacted);

    RedactedStringListPolicyField {
        values: redacted_values,
        redaction_applied,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use veil_manifest::demo_provider_manifest;

    #[test]
    fn incident_report_is_ok_for_supported_demo_path() {
        let report = build_incident_report(
            true,
            RuntimeSupportAssessment::mvp_supported(),
            IncidentGuidance::default(),
            Some("edge-1".to_string()),
            "selected=edge-1".to_string(),
        );

        assert_eq!(report.severity, "ok");
        assert!(report.headline.contains("MVP contour"));
    }

    #[test]
    fn support_bundle_tracks_endpoint_count() {
        let manifest = demo_provider_manifest();
        let incident = build_incident_report(
            true,
            RuntimeSupportAssessment::mvp_supported(),
            IncidentGuidance::default(),
            Some("edge-1".to_string()),
            "selected=edge-1".to_string(),
        );
        let route_diagnostics = build_route_diagnostics(
            Some("edge-1".to_string()),
            Some("xray-core".to_string()),
            "selected=edge-1".to_string(),
            false,
            Vec::new(),
        );

        let bundle = build_support_bundle(
            true,
            &manifest,
            RuntimeSupportAssessment::mvp_supported(),
            None,
            route_diagnostics,
            &RouteSelectionPolicy::default(),
            incident,
        );

        assert_eq!(bundle.endpoint_count, 1);
        assert_eq!(bundle.runtime_support_tier, "mvp-supported");
        assert_eq!(bundle.redacted_manifest_diagnostics.endpoint_count, 1);
        assert_eq!(
            bundle.route_diagnostics.selected_backend_name.as_deref(),
            Some("xray-core")
        );
        assert_eq!(
            bundle.redacted_policy_diagnostics.retry_budget_max_candidates,
            2
        );
    }

    #[test]
    fn route_diagnostics_tracks_rejections_and_fallback() {
        let route_diagnostics = build_route_diagnostics(
            None,
            None,
            "route-selection-failed".to_string(),
            true,
            vec![RejectedRouteCandidate {
                endpoint_id: "edge-1".to_string(),
                reasons: vec!["backend 'xray-core' is denylisted by policy".to_string()],
            }],
        );

        assert!(route_diagnostics.fallback_triggered);
        assert_eq!(route_diagnostics.rejected_routes.len(), 1);
        assert!(
            route_diagnostics.rejected_routes[0]
                .reasons
                .iter()
                .any(|reason| reason.contains("denylisted"))
        );
    }

    #[test]
    fn redacts_bearer_tokens() {
        let redacted = redact_sensitive_text(
            "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.example.payload",
        );

        assert!(redacted.contains(REDACTED_TOKEN));
        assert!(!redacted.contains("eyJhbGci"));
    }

    #[test]
    fn redacts_private_keys() {
        let redacted = redact_sensitive_text(
            "-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BAQEFAASCBK...\n-----END PRIVATE KEY-----",
        );

        assert_eq!(redacted, REDACTED_PRIVATE_KEY);
    }

    #[test]
    fn redacts_uuid_like_credentials() {
        let redacted = redact_sensitive_text("credential=550e8400-e29b-41d4-a716-446655440000");

        assert!(redacted.contains(REDACTED_SECRET));
        assert!(!redacted.contains("550e8400-e29b-41d4-a716-446655440000"));
    }

    #[test]
    fn redacts_url_userinfo_and_query_tokens() {
        let redacted = redact_sensitive_text(
            "https://user:supersecret@example.internal:8443/connect?token=abc123&sig=xyz789&trace_id=visible-value",
        );

        assert!(redacted.contains("user:[REDACTED_SECRET]@example.internal"));
        assert!(redacted.contains("token=[REDACTED_QUERY_VALUE]"));
        assert!(redacted.contains("sig=[REDACTED_QUERY_VALUE]"));
        assert!(redacted.contains("trace_id=visible-value"));
        assert!(!redacted.contains("supersecret"));
        assert!(!redacted.contains("abc123"));
    }

    #[test]
    fn redacted_route_diagnostics_keep_safe_metadata() {
        let route_diagnostics = build_route_diagnostics(
            Some("edge-1".to_string()),
            Some("xray-core".to_string()),
            "Authorization: Bearer secret-token".to_string(),
            true,
            vec![RejectedRouteCandidate {
                endpoint_id: "edge-1".to_string(),
                reasons: vec![
                    "https://example.test/api/v1/connect?token=abc123&sig=xyz789&trace_id=visible-value"
                        .to_string(),
                ],
            }],
        );

        let redacted = build_redacted_route_diagnostics(&route_diagnostics);

        assert_eq!(redacted.selected_backend_name.as_deref(), Some("xray-core"));
        assert!(redacted.route_summary.contains(REDACTED_TOKEN));
        assert_eq!(redacted.rejected_routes.len(), 1);
        assert!(
            redacted.rejected_routes[0].reasons[0].contains("trace_id=visible-value")
        );
        assert!(
            redacted.rejected_routes[0].reasons[0].contains(REDACTED_QUERY_VALUE)
        );
    }

    #[test]
    fn redacted_policy_diagnostics_mask_secret_like_policy_values() {
        let mut policy = RouteSelectionPolicy::default();
        policy.backends.deny = vec!["Authorization: Bearer example.secret".to_string()];
        policy.transports.allow = vec![
            "https://user:supersecret@example.internal:8443/connect?token=abc123&trace_id=visible"
                .to_string(),
        ];
        policy.regions.preferred = vec!["credential=550e8400-e29b-41d4-a716-446655440000".to_string()];

        let redacted = build_redacted_policy_diagnostics(&policy);

        assert!(redacted.denied_backends.redaction_applied);
        assert!(
            redacted.denied_backends.values[0].contains(REDACTED_TOKEN)
        );
        assert!(redacted.allowed_transports.redaction_applied);
        assert!(
            redacted.allowed_transports.values[0].contains(REDACTED_SECRET)
        );
        assert!(
            redacted.allowed_transports.values[0].contains("trace_id=visible")
        );
        assert!(redacted.preferred_regions.redaction_applied);
        assert!(
            redacted.preferred_regions.values[0].contains(REDACTED_SECRET)
        );
    }

    #[test]
    fn redacted_manifest_diagnostics_keep_safe_shape() {
        let mut manifest = demo_provider_manifest();
        manifest.endpoints[0].host = "user:supersecret@example.internal".to_string();
        manifest.endpoints[0].xray = Some(XrayEndpointMetadata {
            protocol: "vless".to_string(),
            stream: "tcp".to_string(),
            security: "tls".to_string(),
            server_name: Some("Authorization: Bearer top.secret".to_string()),
        });

        let redacted = build_redacted_manifest_diagnostics(&manifest);

        assert_eq!(redacted.endpoint_count, 1);
        assert!(
            redacted.endpoints[0].host.contains(REDACTED_SECRET)
                || redacted.endpoints[0].host.contains("example.internal")
        );
        assert_eq!(redacted.endpoints[0].transport, "https");
        assert_eq!(redacted.endpoints[0].region, "eu-central");
        assert!(
            redacted.endpoints[0]
                .xray
                .as_ref()
                .expect("xray")
                .server_name
                .as_ref()
                .expect("server_name")
                .contains(REDACTED_TOKEN)
        );
    }

    #[test]
    fn redacted_support_bundle_uses_only_safe_views() {
        let manifest = demo_provider_manifest();
        let incident = build_incident_report(
            true,
            RuntimeSupportAssessment::mvp_supported(),
            IncidentGuidance {
                severity: "info".to_string(),
                recommended_action: "Authorization: Bearer unsafe-token".to_string(),
            },
            Some("edge-1".to_string()),
            "https://example.test/connect?token=abc123".to_string(),
        );
        let route_diagnostics = build_route_diagnostics(
            Some("edge-1".to_string()),
            Some("xray-core".to_string()),
            "https://example.test/connect?token=abc123".to_string(),
            false,
            Vec::new(),
        );
        let bundle = build_support_bundle(
            true,
            &manifest,
            RuntimeSupportAssessment::mvp_supported(),
            None,
            route_diagnostics,
            &RouteSelectionPolicy::default(),
            incident,
        );

        let redacted = build_redacted_support_bundle(&bundle);

        assert!(redacted.route_summary.contains(REDACTED_QUERY_VALUE));
        assert!(redacted.incident.recommended_action.contains(REDACTED_TOKEN));
        assert_eq!(redacted.redacted_manifest_diagnostics.endpoint_count, 1);
    }

    #[test]
    fn redacted_backend_preflight_masks_sensitive_strings_inside_rendered_config() {
        let preflight = build_backend_preflight_diagnostics(
            "xray-core",
            "/tmp/runtime.json",
            "/tmp/runtime.json",
            false,
            "Authorization: Bearer unsafe-token",
            "xray",
            vec![
                "run".to_string(),
                "-config".to_string(),
                "/tmp/runtime.json".to_string(),
            ],
            serde_json::json!({
                "outbounds": [{
                    "settings": {
                        "address": "user:supersecret@example.internal",
                        "server_name": "https://example.test/connect?token=abc123"
                    }
                }]
            }),
        );

        let redacted = build_redacted_backend_preflight_diagnostics(&preflight);

        assert!(redacted.readiness_note.contains(REDACTED_TOKEN));
        assert!(
            redacted.rendered_config["outbounds"][0]["settings"]["address"]
                .as_str()
                .expect("address")
                .contains(REDACTED_SECRET)
        );
        assert!(
            redacted.rendered_config["outbounds"][0]["settings"]["server_name"]
                .as_str()
                .expect("server_name")
                .contains(REDACTED_QUERY_VALUE)
        );
    }
}
