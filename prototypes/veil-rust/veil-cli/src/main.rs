use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Deserialize;
use serde_json::to_string_pretty;
use std::fs;
use std::path::{Path, PathBuf};
use veil_core::{build_dry_run_plan_with_policy, build_session_report};
use veil_diagnostics::{
    build_redacted_support_bundle, RedactedBackendPreflightDiagnostics,
    RedactedManifestDiagnostics, RedactedPolicyDiagnostics, RedactedRejectedRoute,
    RedactedRouteDiagnostics, RedactedSupportBundle,
};
use veil_manifest::demo_provider_manifest;
use veil_policy::{
    FallbackMode, IncidentGuidance, RoutePreference, RouteSelectionPolicy,
    RuntimeSupportAssessment,
};

#[derive(Debug, Parser)]
#[command(name = "veil")]
#[command(about = "Veil Rust prototype CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Demo(DemoArgs),
}

#[derive(Debug, Args, Default)]
struct DemoArgs {
    #[arg(long = "raw-json")]
    raw_json: bool,
    #[arg(long = "export-redacted-bundle")]
    export_redacted_bundle: Option<String>,
    #[arg(long = "export-redacted-preflight")]
    export_redacted_preflight: Option<String>,
    #[arg(long = "policy-file")]
    policy_file: Option<String>,
    #[arg(long = "allow-backend")]
    allow_backends: Vec<String>,
    #[arg(long = "deny-backend")]
    deny_backends: Vec<String>,
    #[arg(long = "allow-transport")]
    allow_transports: Vec<String>,
    #[arg(long = "deny-transport")]
    deny_transports: Vec<String>,
    #[arg(long = "prefer-region")]
    preferred_regions: Vec<String>,
    #[arg(long = "allow-region")]
    allow_regions: Vec<String>,
    #[arg(long = "deny-region")]
    deny_regions: Vec<String>,
    #[arg(long = "retry-budget")]
    retry_budget: Option<u8>,
    #[arg(long = "disable-known-good")]
    disable_known_good: bool,
    #[arg(long = "strict-fallback")]
    strict_fallback: bool,
    #[arg(long = "route-preference", value_enum)]
    route_preference: Option<RoutePreferenceArg>,
}

#[derive(Debug, Clone, ValueEnum)]
enum RoutePreferenceArg {
    Balanced,
    PreferLatency,
    PreferStability,
}

impl From<RoutePreferenceArg> for RoutePreference {
    fn from(value: RoutePreferenceArg) -> Self {
        match value {
            RoutePreferenceArg::Balanced => RoutePreference::Balanced,
            RoutePreferenceArg::PreferLatency => RoutePreference::PreferLatency,
            RoutePreferenceArg::PreferStability => RoutePreference::PreferStability,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
struct PartialRegionPolicy {
    preferred: Option<Vec<String>>,
    allow: Option<Vec<String>>,
    deny: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct PartialBackendPolicy {
    allow: Option<Vec<String>>,
    deny: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct PartialTransportPolicy {
    allow: Option<Vec<String>>,
    deny: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct PartialRetryBudgetPolicy {
    max_candidates: Option<u8>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct PartialCooldownPolicy {
    forbid_active_cooldown: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct PartialKnownGoodPolicy {
    enabled: Option<bool>,
    bonus_points: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct PartialNetworkCostPolicy {
    allow_metered_networks: Option<bool>,
    allow_expensive_networks: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct PartialRouteSelectionPolicy {
    route_preference: Option<RoutePreference>,
    network_cost: Option<PartialNetworkCostPolicy>,
    regions: Option<PartialRegionPolicy>,
    backends: Option<PartialBackendPolicy>,
    transports: Option<PartialTransportPolicy>,
    retry_budget: Option<PartialRetryBudgetPolicy>,
    cooldown: Option<PartialCooldownPolicy>,
    known_good: Option<PartialKnownGoodPolicy>,
    fallback: Option<FallbackMode>,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command.unwrap_or(Commands::Demo(DemoArgs::default())) {
        Commands::Demo(args) => run_demo(args),
    };

    if let Err(error) = result {
        eprintln!("error={error}");
        std::process::exit(2);
    }
}

fn run_demo(args: DemoArgs) -> Result<(), String> {
    let manifest = demo_provider_manifest();
    let policy = build_effective_policy(&args)?;
    let plan = build_dry_run_plan_with_policy(
        &manifest,
        RuntimeSupportAssessment::mvp_supported(),
        IncidentGuidance::default(),
        policy.clone(),
    );
    let report = build_session_report(&plan);
    let redacted_bundle = build_redacted_support_bundle(&plan.support_bundle);
    let summary = build_compact_diagnostics_summary(
        &plan.support_bundle.redacted_manifest_diagnostics,
        plan.support_bundle
            .redacted_backend_preflight_diagnostics
            .as_ref(),
        &plan.support_bundle.redacted_route_diagnostics,
        &plan.support_bundle.redacted_policy_diagnostics,
    );
    let report_mode = if args.raw_json { "raw-json" } else { "redacted" };

    println!("diagnostics_summary:");
    for line in summary.lines() {
        println!("  {line}");
    }
    println!("report_mode={report_mode}");
    println!("session_phase={:?}", plan.phase);
    println!("session_outcome={:?}", plan.outcome);
    println!(
        "selected_endpoint_id={}",
        plan.selected_endpoint_id.unwrap_or_else(|| "none".to_string())
    );
    println!(
        "selected_backend_name={}",
        plan.selected_backend_name.unwrap_or_else(|| "none".to_string())
    );
    println!("diagnostics_reason={}", plan.diagnostics_reason);
    println!("route_summary={}", plan.route_summary);
    println!("fallback_triggered={}", plan.fallback_triggered);
    println!("rejected_route_count={}", plan.rejected_routes.len());
    for rejected in &plan.rejected_routes {
        println!(
            "rejected_route={} reasons={}",
            rejected.endpoint_id,
            rejected.reasons.join("|")
        );
    }
    println!("incident_severity={}", plan.incident_report.severity);
    println!("incident_headline={}", plan.incident_report.headline);
    println!("session_event_count={}", report.event_count);
    println!("adapter_registry_entries={}", plan.adapter_registry.entries.len());
    if let Some(path) = &args.export_redacted_bundle {
        export_redacted_bundle(path, &redacted_bundle)?;
        println!("redacted_bundle_exported_to={path}");
    }
    if let Some(path) = &args.export_redacted_preflight {
        export_redacted_preflight(
            path,
            plan.support_bundle
                .redacted_backend_preflight_diagnostics
                .as_ref(),
        )?;
        println!("redacted_preflight_exported_to={path}");
    }
    println!(
        "redacted_manifest_diagnostics={}",
        to_string_pretty(&plan.support_bundle.redacted_manifest_diagnostics)
            .unwrap_or_else(|_| "{}".to_string())
    );
    println!(
        "redacted_backend_preflight_diagnostics={}",
        to_string_pretty(&plan.support_bundle.redacted_backend_preflight_diagnostics)
            .unwrap_or_else(|_| "null".to_string())
    );
    println!(
        "redacted_route_diagnostics={}",
        to_string_pretty(&plan.support_bundle.redacted_route_diagnostics)
            .unwrap_or_else(|_| "{}".to_string())
    );
    println!(
        "redacted_policy_diagnostics={}",
        to_string_pretty(&plan.support_bundle.redacted_policy_diagnostics)
            .unwrap_or_else(|_| "{}".to_string())
    );

    if args.raw_json {
        println!(
            "effective_policy={}",
            to_string_pretty(&policy).unwrap_or_else(|_| "{}".to_string())
        );
        println!(
            "route_diagnostics={}",
            to_string_pretty(&plan.support_bundle.route_diagnostics)
                .unwrap_or_else(|_| "{}".to_string())
        );
        println!(
            "backend_preflight_diagnostics={}",
            to_string_pretty(&plan.support_bundle.backend_preflight_diagnostics)
                .unwrap_or_else(|_| "null".to_string())
        );
        println!(
            "support_bundle={}",
            to_string_pretty(&plan.support_bundle).unwrap_or_else(|_| "{}".to_string())
        );
    }

    Ok(())
}

fn export_redacted_bundle(path: &str, bundle: &RedactedSupportBundle) -> Result<(), String> {
    let output_path = PathBuf::from(path);
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create export directory '{}': {}",
                    parent.display(),
                    error
                )
            })?;
        }
    }

    let payload = to_string_pretty(bundle)
        .map_err(|error| format!("failed to serialize redacted bundle: {}", error))?;
    fs::write(&output_path, payload).map_err(|error| {
        format!(
            "failed to write redacted bundle '{}': {}",
            output_path.display(),
            error
        )
    })
}

fn export_redacted_preflight(
    path: &str,
    preflight: Option<&RedactedBackendPreflightDiagnostics>,
) -> Result<(), String> {
    let preflight = preflight.ok_or_else(|| {
        "failed to export redacted preflight: no backend preflight diagnostics were recorded"
            .to_string()
    })?;
    let output_path = PathBuf::from(path);
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create export directory '{}': {}",
                    parent.display(),
                    error
                )
            })?;
        }
    }

    let payload = to_string_pretty(preflight)
        .map_err(|error| format!("failed to serialize redacted preflight: {}", error))?;
    fs::write(&output_path, payload).map_err(|error| {
        format!(
            "failed to write redacted preflight '{}': {}",
            output_path.display(),
            error
        )
    })
}

fn build_compact_diagnostics_summary(
    manifest: &RedactedManifestDiagnostics,
    backend_preflight: Option<&RedactedBackendPreflightDiagnostics>,
    route: &RedactedRouteDiagnostics,
    policy: &RedactedPolicyDiagnostics,
) -> String {
    let selected_endpoint = route
        .selected_endpoint_id
        .clone()
        .unwrap_or_else(|| "none".to_string());
    let selected_backend = route
        .selected_backend_name
        .clone()
        .unwrap_or_else(|| "none".to_string());
    let fallback_state = if route.fallback_triggered {
        "triggered"
    } else {
        "not-triggered"
    };
    let preference = policy.route_preference.to_lowercase();
    let fallback_mode = policy.fallback_mode.to_lowercase();
    let rejected_summary = summarize_rejected_routes(&route.rejected_routes);
    let backend_preflight_summary = backend_preflight
        .map(|preflight| {
            format!(
                "{} ready={} binary_present={}",
                preflight.backend_name,
                preflight.ready_for_dry_run_connect,
                preflight.binary_present
            )
        })
        .unwrap_or_else(|| "none".to_string());

    [
        format!(
            "manifest: schema v{}, endpoints {}, profile {:?}",
            manifest.metadata.schema_version, manifest.endpoint_count, manifest.profile_kind
        ),
        format!("backend preflight: {backend_preflight_summary}"),
        format!("selected endpoint: {selected_endpoint}"),
        format!("selected backend: {selected_backend}"),
        format!("route fallback: {fallback_state} ({fallback_mode})"),
        format!("route preference: {preference}"),
        format!(
            "retry budget: {} candidate(s), cooldown strict: {}",
            policy.retry_budget_max_candidates, policy.forbid_active_cooldown
        ),
        format!("known-good: {} (bonus {})", policy.known_good_enabled, policy.known_good_bonus_points),
        format!("rejected routes: {rejected_summary}"),
    ]
    .join("\n")
}

fn summarize_rejected_routes(rejected_routes: &[RedactedRejectedRoute]) -> String {
    if rejected_routes.is_empty() {
        return "none".to_string();
    }

    rejected_routes
        .iter()
        .map(|route| {
            let first_reason = route
                .reasons
                .first()
                .cloned()
                .unwrap_or_else(|| "no reason recorded".to_string());
            format!("{} ({})", route.endpoint_id, first_reason)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn build_effective_policy(args: &DemoArgs) -> Result<RouteSelectionPolicy, String> {
    let mut policy = RouteSelectionPolicy::default();

    if let Some(path) = &args.policy_file {
        let partial_policy = read_policy_override_file(path)?;
        policy = apply_partial_policy_override(policy, partial_policy);
    }

    Ok(apply_demo_overrides(policy, args))
}

fn read_policy_override_file(path: &str) -> Result<PartialRouteSelectionPolicy, String> {
    let file_path = Path::new(path);
    let contents = fs::read_to_string(file_path)
        .map_err(|error| format!("failed to read policy file '{}': {}", path, error))?;

    serde_json::from_str(&contents)
        .map_err(|error| format!("failed to parse policy file '{}': {}", path, error))
}

fn apply_partial_policy_override(
    mut policy: RouteSelectionPolicy,
    partial: PartialRouteSelectionPolicy,
) -> RouteSelectionPolicy {
    if let Some(route_preference) = partial.route_preference {
        policy.route_preference = route_preference;
    }
    if let Some(fallback) = partial.fallback {
        policy.fallback = fallback;
    }
    if let Some(network_cost) = partial.network_cost {
        if let Some(allow_metered_networks) = network_cost.allow_metered_networks {
            policy.network_cost.allow_metered_networks = allow_metered_networks;
        }
        if let Some(allow_expensive_networks) = network_cost.allow_expensive_networks {
            policy.network_cost.allow_expensive_networks = allow_expensive_networks;
        }
    }
    if let Some(regions) = partial.regions {
        if let Some(preferred) = regions.preferred {
            policy.regions.preferred = preferred;
        }
        if let Some(allow) = regions.allow {
            policy.regions.allow = allow;
        }
        if let Some(deny) = regions.deny {
            policy.regions.deny = deny;
        }
    }
    if let Some(backends) = partial.backends {
        if let Some(allow) = backends.allow {
            policy.backends.allow = allow;
        }
        if let Some(deny) = backends.deny {
            policy.backends.deny = deny;
        }
    }
    if let Some(transports) = partial.transports {
        if let Some(allow) = transports.allow {
            policy.transports.allow = allow;
        }
        if let Some(deny) = transports.deny {
            policy.transports.deny = deny;
        }
    }
    if let Some(retry_budget) = partial.retry_budget {
        if let Some(max_candidates) = retry_budget.max_candidates {
            policy.retry_budget.max_candidates = max_candidates;
        }
    }
    if let Some(cooldown) = partial.cooldown {
        if let Some(forbid_active_cooldown) = cooldown.forbid_active_cooldown {
            policy.cooldown.forbid_active_cooldown = forbid_active_cooldown;
        }
    }
    if let Some(known_good) = partial.known_good {
        if let Some(enabled) = known_good.enabled {
            policy.known_good.enabled = enabled;
        }
        if let Some(bonus_points) = known_good.bonus_points {
            policy.known_good.bonus_points = bonus_points;
        }
    }

    policy
}

fn apply_demo_overrides(mut policy: RouteSelectionPolicy, args: &DemoArgs) -> RouteSelectionPolicy {
    if !args.allow_backends.is_empty() {
        policy.backends.allow = args.allow_backends.clone();
    }
    if !args.deny_backends.is_empty() {
        policy.backends.deny = args.deny_backends.clone();
    }
    if !args.allow_transports.is_empty() {
        policy.transports.allow = args.allow_transports.clone();
    }
    if !args.deny_transports.is_empty() {
        policy.transports.deny = args.deny_transports.clone();
    }
    if !args.preferred_regions.is_empty() {
        policy.regions.preferred = args.preferred_regions.clone();
    }
    if !args.allow_regions.is_empty() {
        policy.regions.allow = args.allow_regions.clone();
    }
    if !args.deny_regions.is_empty() {
        policy.regions.deny = args.deny_regions.clone();
    }

    if let Some(retry_budget) = args.retry_budget {
        policy.retry_budget.max_candidates = retry_budget;
    }
    if args.disable_known_good {
        policy.known_good.enabled = false;
        policy.known_good.bonus_points = 0;
    }
    if args.strict_fallback {
        policy.fallback = FallbackMode::Strict;
    }
    if let Some(route_preference) = args.route_preference.clone() {
        policy.route_preference = route_preference.into();
    }

    policy
}

#[cfg(test)]
mod tests {
    use super::*;
    use veil_diagnostics::{
        RedactedManifestCapability, RedactedManifestDiagnostics, RedactedManifestEndpoint,
        RedactedManifestMetadata, RedactedPolicyDiagnostics, RedactedRejectedRoute,
        RedactedRouteDiagnostics, RedactedStringListPolicyField, RedactedSupportBundle,
    };
    use std::path::Path;

    #[test]
    fn partial_policy_override_updates_selected_fields_only() {
        let policy = apply_partial_policy_override(
            RouteSelectionPolicy::default(),
            PartialRouteSelectionPolicy {
                route_preference: Some(RoutePreference::PreferStability),
                regions: Some(PartialRegionPolicy {
                    preferred: Some(vec!["eu-central".to_string()]),
                    allow: None,
                    deny: Some(vec!["us-west".to_string()]),
                }),
                retry_budget: Some(PartialRetryBudgetPolicy {
                    max_candidates: Some(1),
                }),
                ..PartialRouteSelectionPolicy::default()
            },
        );

        assert_eq!(policy.route_preference, RoutePreference::PreferStability);
        assert_eq!(policy.regions.preferred, vec!["eu-central"]);
        assert_eq!(policy.regions.deny, vec!["us-west"]);
        assert_eq!(policy.retry_budget.max_candidates, 1);
        assert!(policy.backends.allow.is_empty());
    }

    #[test]
    fn cli_flags_override_policy_file_values() {
        let base = apply_partial_policy_override(
            RouteSelectionPolicy::default(),
            PartialRouteSelectionPolicy {
                backends: Some(PartialBackendPolicy {
                    allow: Some(vec!["xray-core".to_string()]),
                    deny: None,
                }),
                known_good: Some(PartialKnownGoodPolicy {
                    enabled: Some(true),
                    bonus_points: Some(7),
                }),
                ..PartialRouteSelectionPolicy::default()
            },
        );
        let args = DemoArgs {
            deny_backends: vec!["xray-core".to_string()],
            disable_known_good: true,
            ..DemoArgs::default()
        };

        let policy = apply_demo_overrides(base, &args);

        assert_eq!(policy.backends.allow, vec!["xray-core"]);
        assert_eq!(policy.backends.deny, vec!["xray-core"]);
        assert!(!policy.known_good.enabled);
        assert_eq!(policy.known_good.bonus_points, 0);
    }

    #[test]
    fn json_policy_file_shape_deserializes() {
        let partial: PartialRouteSelectionPolicy = serde_json::from_str(
            r#"{
                "route_preference": "prefer-latency",
                "backends": { "deny": ["xray-core"] },
                "retry_budget": { "max_candidates": 1 },
                "fallback": "strict"
            }"#,
        )
        .expect("json should deserialize");

        assert_eq!(partial.route_preference, Some(RoutePreference::PreferLatency));
        assert_eq!(partial.fallback, Some(FallbackMode::Strict));
        assert_eq!(
            partial
                .backends
                .expect("backends")
                .deny
                .expect("deny"),
            vec!["xray-core"]
        );
    }

    #[test]
    fn example_policy_files_exist_for_demo_usage() {
        assert!(Path::new("../examples/policies/deny-xray.json").exists());
        assert!(Path::new("../examples/policies/prefer-eu-stable.json").exists());
        assert!(Path::new("../examples/policies/strict-fallback.json").exists());
    }

    #[test]
    fn demo_args_default_to_redacted_report_mode() {
        let args = DemoArgs::default();

        assert!(!args.raw_json);
    }

    #[test]
    fn raw_json_report_mode_can_be_enabled_explicitly() {
        let args = DemoArgs {
            raw_json: true,
            ..DemoArgs::default()
        };

        assert!(args.raw_json);
    }

    #[test]
    fn export_redacted_bundle_writes_json_file() {
        let temp_dir = std::env::temp_dir().join("veil-cli-export-test");
        let output_path = temp_dir.join("bundle.json");
        let bundle = RedactedSupportBundle {
            manifest_valid: true,
            runtime_support_tier: "mvp-supported".to_string(),
            selected_endpoint_id: Some("edge-1".to_string()),
            route_summary: "selected=edge-1".to_string(),
            endpoint_count: 1,
            redacted_manifest_diagnostics: RedactedManifestDiagnostics {
                metadata: RedactedManifestMetadata {
                    schema_version: 1,
                    provider_profile_schema_version: Some(1),
                    generated_at: "2026-04-26 00:00:00 +00:00:00".to_string(),
                    expires_at: "2026-04-27 00:00:00 +00:00:00".to_string(),
                },
                capabilities: vec![],
                endpoint_count: 1,
                endpoints: vec![],
                preferred_transports: vec!["https".to_string()],
                transport_retry_budget: 1,
                profile_kind: Some("ProviderProfile".to_string()),
            },
            redacted_backend_preflight_diagnostics: Some(RedactedBackendPreflightDiagnostics {
                backend_name: "xray-core".to_string(),
                ready_for_dry_run_connect: true,
                binary_path: "xray".to_string(),
                config_path: "runtime/session-1.json".to_string(),
                binary_present: false,
                readiness_note: "dry-run command prepared".to_string(),
                command: veil_diagnostics::BackendPreflightCommandDiagnostics {
                    program: "xray".to_string(),
                    args: vec![
                        "run".to_string(),
                        "-config".to_string(),
                        "runtime/session-1.json".to_string(),
                    ],
                },
                rendered_config: serde_json::json!({
                    "outbounds": [{ "protocol": "vless" }]
                }),
            }),
            redacted_route_diagnostics: RedactedRouteDiagnostics {
                selected_endpoint_id: Some("edge-1".to_string()),
                selected_backend_name: Some("xray-core".to_string()),
                route_summary: "selected=edge-1".to_string(),
                fallback_triggered: false,
                rejected_routes: vec![],
            },
            redacted_policy_diagnostics: RedactedPolicyDiagnostics {
                route_preference: "Balanced".to_string(),
                fallback_mode: "AllowKnownGood".to_string(),
                allow_metered_networks: true,
                allow_expensive_networks: true,
                preferred_regions: RedactedStringListPolicyField {
                    values: vec![],
                    redaction_applied: false,
                },
                allowed_regions: RedactedStringListPolicyField {
                    values: vec![],
                    redaction_applied: false,
                },
                denied_regions: RedactedStringListPolicyField {
                    values: vec![],
                    redaction_applied: false,
                },
                allowed_backends: RedactedStringListPolicyField {
                    values: vec![],
                    redaction_applied: false,
                },
                denied_backends: RedactedStringListPolicyField {
                    values: vec![],
                    redaction_applied: false,
                },
                allowed_transports: RedactedStringListPolicyField {
                    values: vec![],
                    redaction_applied: false,
                },
                denied_transports: RedactedStringListPolicyField {
                    values: vec![],
                    redaction_applied: false,
                },
                retry_budget_max_candidates: 2,
                forbid_active_cooldown: true,
                known_good_enabled: true,
                known_good_bonus_points: 3,
            },
            incident: veil_diagnostics::RedactedIncidentReport {
                severity: "ok".to_string(),
                headline: "headline".to_string(),
                recommended_action: "none".to_string(),
                selected_endpoint_id: Some("edge-1".to_string()),
                route_summary: "selected=edge-1".to_string(),
            },
        };

        export_redacted_bundle(output_path.to_str().expect("utf-8 path"), &bundle)
            .expect("export should succeed");

        let contents = fs::read_to_string(&output_path).expect("file should exist");
        assert!(contents.contains("\"manifest_valid\": true"));
        assert!(contents.contains("\"runtime_support_tier\": \"mvp-supported\""));

        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_dir(&temp_dir);
    }

    #[test]
    fn compact_summary_mentions_selection_and_policy_state() {
        let route = RedactedRouteDiagnostics {
            selected_endpoint_id: Some("edge-1".to_string()),
            selected_backend_name: Some("xray-core".to_string()),
            route_summary: "selected=edge-1".to_string(),
            fallback_triggered: false,
            rejected_routes: Vec::new(),
        };
        let manifest = RedactedManifestDiagnostics {
            metadata: RedactedManifestMetadata {
                schema_version: 1,
                provider_profile_schema_version: Some(1),
                generated_at: "2026-04-26 00:00:00 +00:00:00".to_string(),
                expires_at: "2026-04-27 00:00:00 +00:00:00".to_string(),
            },
            capabilities: vec![RedactedManifestCapability {
                platform: "linux".to_string(),
                supported_dataplanes: vec!["xray-core".to_string()],
                network_adapter: "linux".to_string(),
                status: "mvp-supported".to_string(),
            }],
            endpoint_count: 1,
            endpoints: vec![RedactedManifestEndpoint {
                id: "edge-1".to_string(),
                host: "198.51.100.10".to_string(),
                port: 443,
                transport: "https".to_string(),
                region: "eu-central".to_string(),
                dataplane: Some("xray-core".to_string()),
                supported_client_platforms: vec!["linux".to_string()],
                logical_server: Some("edge".to_string()),
                provider_profile_schema_version: Some(1),
                xray: None,
            }],
            preferred_transports: vec!["https".to_string()],
            transport_retry_budget: 1,
            profile_kind: Some("ProviderProfile".to_string()),
        };
        let policy = RedactedPolicyDiagnostics {
            route_preference: "PreferStability".to_string(),
            fallback_mode: "AllowKnownGood".to_string(),
            allow_metered_networks: true,
            allow_expensive_networks: true,
            preferred_regions: RedactedStringListPolicyField {
                values: vec!["eu-central".to_string()],
                redaction_applied: false,
            },
            allowed_regions: RedactedStringListPolicyField {
                values: vec![],
                redaction_applied: false,
            },
            denied_regions: RedactedStringListPolicyField {
                values: vec![],
                redaction_applied: false,
            },
            allowed_backends: RedactedStringListPolicyField {
                values: vec![],
                redaction_applied: false,
            },
            denied_backends: RedactedStringListPolicyField {
                values: vec![],
                redaction_applied: false,
            },
            allowed_transports: RedactedStringListPolicyField {
                values: vec![],
                redaction_applied: false,
            },
            denied_transports: RedactedStringListPolicyField {
                values: vec![],
                redaction_applied: false,
            },
            retry_budget_max_candidates: 2,
            forbid_active_cooldown: true,
            known_good_enabled: true,
            known_good_bonus_points: 3,
        };
        let backend_preflight = RedactedBackendPreflightDiagnostics {
            backend_name: "xray-core".to_string(),
            ready_for_dry_run_connect: true,
            binary_path: "xray".to_string(),
            config_path: "runtime/session-1.json".to_string(),
            binary_present: false,
            readiness_note: "dry-run command prepared".to_string(),
            command: veil_diagnostics::BackendPreflightCommandDiagnostics {
                program: "xray".to_string(),
                args: vec![
                    "run".to_string(),
                    "-config".to_string(),
                    "runtime/session-1.json".to_string(),
                ],
            },
            rendered_config: serde_json::json!({
                "outbounds": [{ "protocol": "vless" }]
            }),
        };

        let summary =
            build_compact_diagnostics_summary(&manifest, Some(&backend_preflight), &route, &policy);

        assert!(summary.contains("manifest: schema v1, endpoints 1"));
        assert!(summary.contains("backend preflight: xray-core ready=true binary_present=false"));
        assert!(summary.contains("selected endpoint: edge-1"));
        assert!(summary.contains("selected backend: xray-core"));
        assert!(summary.contains("route preference: preferstability"));
        assert!(summary.contains("rejected routes: none"));
    }

    #[test]
    fn export_redacted_preflight_writes_json_file() {
        let temp_dir = std::env::temp_dir().join("veil-cli-preflight-export-test");
        let output_path = temp_dir.join("preflight.json");
        let preflight = RedactedBackendPreflightDiagnostics {
            backend_name: "xray-core".to_string(),
            ready_for_dry_run_connect: true,
            binary_path: "xray".to_string(),
            config_path: "runtime/session-1.json".to_string(),
            binary_present: false,
            readiness_note: "dry-run command prepared".to_string(),
            command: veil_diagnostics::BackendPreflightCommandDiagnostics {
                program: "xray".to_string(),
                args: vec![
                    "run".to_string(),
                    "-config".to_string(),
                    "runtime/session-1.json".to_string(),
                ],
            },
            rendered_config: serde_json::json!({
                "outbounds": [{ "protocol": "vless" }]
            }),
        };

        export_redacted_preflight(output_path.to_str().expect("utf-8 path"), Some(&preflight))
            .expect("export should succeed");

        let contents = fs::read_to_string(&output_path).expect("file should exist");
        assert!(contents.contains("\"backend_name\": \"xray-core\""));
        assert!(contents.contains("\"binary_present\": false"));

        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_dir(&temp_dir);
    }

    #[test]
    fn compact_summary_includes_first_rejection_reason() {
        let rejected = summarize_rejected_routes(&[RedactedRejectedRoute {
            endpoint_id: "edge-2".to_string(),
            reasons: vec!["transport 'https' is denylisted by policy".to_string()],
        }]);

        assert!(rejected.contains("edge-2"));
        assert!(rejected.contains("denylisted"));
    }
}
