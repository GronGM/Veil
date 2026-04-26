#![forbid(unsafe_code)]

use veil_adapter_mock::MockBackend;
use veil_adapter_xray::XrayBackend;
use veil_core::SessionEngine;
use veil_manifest::ProviderManifest;
use veil_policy::RoutePolicy;
use veil_routing::{select_backend_with_eligibility, BackendCandidate, BackendEligibility};
use veil_transport::TransportProfile;

fn main() {
    let scenario = std::env::args().nth(1);
    let candidates = [
        BackendCandidate { backend_name: "xray-core" },
        BackendCandidate { backend_name: "mock-backend" },
    ];

    let (manifest, policy, forced_backend_name) = match scenario.as_deref() {
        Some("policy-mismatch") => (
            ProviderManifest::demo(),
            RoutePolicy::mismatch_demo(),
            Some("xray-core"),
        ),
        Some("mock-backend") => (
            ProviderManifest::demo(),
            RoutePolicy::mismatch_demo(),
            None,
        ),
        Some("grpc-transport") => (
            ProviderManifest::grpc_demo(),
            RoutePolicy::transport_mismatch_demo(),
            None,
        ),
        Some("planned-windows") => (
            ProviderManifest::planned_windows_demo(),
            RoutePolicy::demo(),
            None,
        ),
        Some("linux-foundation") => (
            ProviderManifest::linux_foundation_demo(),
            RoutePolicy::mismatch_demo(),
            None,
        ),
        Some("contract-mismatch") => (
            ProviderManifest::contract_mismatch_demo(),
            RoutePolicy::demo(),
            None,
        ),
        Some("mock-grpc") => (
            ProviderManifest::grpc_demo(),
            RoutePolicy {
                allow_backend: "mock-backend".to_string(),
                allow_fallback: false,
                allow_transport: TransportProfile::Grpc,
                required_capabilities: veil_policy::RoutePolicy::demo().required_capabilities,
            },
            None,
        ),
        Some("typed-config-required") => (
            ProviderManifest::demo(),
            RoutePolicy::typed_config_demo(),
            None,
        ),
        Some("real-binary-required") => (
            ProviderManifest::demo(),
            RoutePolicy::real_binary_demo(),
            None,
        ),
        Some("transport-mismatch") => (
            ProviderManifest::demo(),
            RoutePolicy::transport_mismatch_demo(),
            None,
        ),
        _ => (
            ProviderManifest::demo(),
            RoutePolicy::demo(),
            None,
        ),
    };

    let xray_report = SessionEngine::dry_run(&manifest, &policy, &XrayBackend::default());
    let mock_report = SessionEngine::dry_run(&manifest, &policy, &MockBackend::default());

    let eligibility = [
        BackendEligibility {
            backend_name: "xray-core",
            outcome: xray_report.outcome,
        },
        BackendEligibility {
            backend_name: "mock-backend",
            outcome: mock_report.outcome,
        },
    ];

    let routing_result =
        select_backend_with_eligibility(&candidates, &manifest, &policy, &eligibility);

    let selected_backend = forced_backend_name
        .or_else(|| routing_result.selection.as_ref().map(|selection| selection.backend_name))
        .unwrap_or(policy.allow_backend.as_str());

    let report = match selected_backend {
        "mock-backend" => mock_report,
        _ => xray_report,
    };
    let mut report = report;
    report.set_routing(routing_result.summary.clone(), routing_result.candidates.clone());
    let diagnostics = report.redacted_diagnostics();

    println!("{}", routing_result.summary);
    for candidate in &routing_result.candidates {
        println!(
            "candidate: {} eligible={} reason={} summary={}",
            candidate.backend_name,
            candidate.eligible,
            candidate.reason.as_str(),
            candidate.summary
        );
    }
    println!();
    println!("{}", report.render());
    println!();
    println!("{}", diagnostics.render());
    println!();
    println!("{}", diagnostics.render_json());
}
