#![forbid(unsafe_code)]

use veil_adapter_mock::MockBackend;
use veil_adapter_xray::XrayBackend;
use veil_core::SessionEngine;
use veil_manifest::ProviderManifest;
use veil_policy::RoutePolicy;
use veil_routing::{select_backend, BackendCandidate};

fn main() {
    let scenario = std::env::args().nth(1);
    let manifest = ProviderManifest::demo();
    let candidates = [
        BackendCandidate { backend_name: "xray-core" },
        BackendCandidate { backend_name: "mock-backend" },
    ];

    let (policy, forced_backend_name) = match scenario.as_deref() {
        Some("policy-mismatch") => (RoutePolicy::mismatch_demo(), Some("xray-core")),
        Some("mock-backend") => (RoutePolicy::mismatch_demo(), None),
        _ => (RoutePolicy::demo(), None),
    };

    let selected_backend = forced_backend_name.or_else(|| {
        select_backend(&candidates, &manifest, &policy).map(|selection| selection.backend_name)
    }).unwrap_or("xray-core");

    let report = match selected_backend {
        "mock-backend" => SessionEngine::dry_run(&manifest, &policy, &MockBackend::default()),
        _ => SessionEngine::dry_run(&manifest, &policy, &XrayBackend::default()),
    };
    let diagnostics = report.redacted_diagnostics();

    println!("{}", report.render());
    println!();
    println!("{}", diagnostics.render());
    println!();
    println!("{}", diagnostics.render_json());
}
