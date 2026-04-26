#![forbid(unsafe_code)]

use veil_adapter_xray::XrayBackend;
use veil_core::SessionEngine;
use veil_manifest::ProviderManifest;
use veil_policy::RoutePolicy;

fn main() {
    let scenario = std::env::args().nth(1);
    let manifest = ProviderManifest::demo();
    let policy = match scenario.as_deref() {
        Some("policy-mismatch") => RoutePolicy::mismatch_demo(),
        _ => RoutePolicy::demo(),
    };
    let backend = XrayBackend::default();
    let report = SessionEngine::dry_run(&manifest, &policy, &backend);
    let diagnostics = report.redacted_diagnostics();

    println!("{}", report.render());
    println!();
    println!("{}", diagnostics.render());
}
