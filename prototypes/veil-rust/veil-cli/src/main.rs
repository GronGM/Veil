use clap::{Parser, Subcommand};
use serde_json::to_string_pretty;
use veil_core::{build_dry_run_plan, build_session_report};
use veil_manifest::demo_provider_manifest;
use veil_policy::{IncidentGuidance, RuntimeSupportAssessment};

#[derive(Debug, Parser)]
#[command(name = "veil")]
#[command(about = "Veil Rust prototype CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Demo,
}

fn main() {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Demo) {
        Commands::Demo => {
            let manifest = demo_provider_manifest();
            let plan = build_dry_run_plan(
                &manifest,
                RuntimeSupportAssessment::mvp_supported(),
                IncidentGuidance::default(),
            );
            let report = build_session_report(&plan);

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
            println!("incident_severity={}", plan.incident_report.severity);
            println!("incident_headline={}", plan.incident_report.headline);
            println!("session_event_count={}", report.event_count);
            println!("adapter_registry_entries={}", plan.adapter_registry.entries.len());
            println!(
                "support_bundle={}",
                to_string_pretty(&plan.support_bundle).unwrap_or_else(|_| "{}".to_string())
            );
        }
    }
}
