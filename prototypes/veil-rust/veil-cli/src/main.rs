#![forbid(unsafe_code)]

use veil_adapter_xray::XrayBackend;
use veil_core::SessionEngine;
use veil_manifest::ProviderManifest;
use veil_policy::RoutePolicy;

fn main() {
    let manifest = ProviderManifest::demo();
    let policy = RoutePolicy::demo();
    let backend = XrayBackend::default();
    let report = SessionEngine::dry_run(&manifest, &policy, &backend);

    println!("{}", report.render());
}
