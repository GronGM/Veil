#![forbid(unsafe_code)]

use veil_adapter_mock::MockBackend;
use veil_adapter_xray::XrayBackend;
use veil_core::SessionEngine;
use veil_manifest::ProviderManifest;
use veil_policy::RoutePolicy;

fn main() {
    let scenario = std::env::args().nth(1);
    let manifest = ProviderManifest::demo();
    let (policy, backend_name) = match scenario.as_deref() {
        Some("policy-mismatch") => (RoutePolicy::mismatch_demo(), "xray"),
        Some("mock-backend") => (RoutePolicy::mismatch_demo(), "mock"),
        _ => (RoutePolicy::demo(), "xray"),
    };

    let report = match backend_name {
        "mock" => SessionEngine::dry_run(&manifest, &policy, &MockBackend::default()),
        _ => SessionEngine::dry_run(&manifest, &policy, &XrayBackend::default()),
    };
    let diagnostics = report.redacted_diagnostics();

    println!("{}", report.render());
    println!();
    println!("{}", diagnostics.render());
    println!();
    println!("{}", diagnostics.render_json());
}
