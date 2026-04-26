#![forbid(unsafe_code)]

use veil_adapter_xray::XrayBackend;
use veil_core::SessionEngine;

fn main() {
    let backend = XrayBackend::default();
    let report = SessionEngine::dry_run(&backend);

    println!("{}", report.render());
}
