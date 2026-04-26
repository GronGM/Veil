//! Dry-run mock backend for Veil adapter API testing.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, to_value};
use std::path::Path;
use veil_adapter_api::{
    AdapterContext, AdapterError, AdapterMetadata, DataplaneBackend, DryRunCommandSpec,
    DryRunPreflight, GeneratedConfig, HealthStatus, RuntimeSnapshot,
};
use veil_manifest::Endpoint;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MockRenderedConfig {
    pub profile: String,
    pub endpoint_id: String,
    pub bind_address: String,
    pub remote_address: String,
    pub remote_port: u16,
}

#[derive(Debug, Default)]
pub struct MockDryRunBackend;

impl MockDryRunBackend {
    fn ensure_supported(endpoint: &Endpoint) -> Result<(), AdapterError> {
        match endpoint.dataplane.as_deref() {
            Some("mock-backend") => Ok(()),
            _ => Err(AdapterError::UnsupportedEndpoint {
                backend: "mock-backend".to_string(),
            }),
        }
    }

    pub fn render_config(endpoint: &Endpoint) -> Result<MockRenderedConfig, AdapterError> {
        Self::ensure_supported(endpoint)?;
        Ok(MockRenderedConfig {
            profile: "mock-dry-run".to_string(),
            endpoint_id: endpoint.id.clone(),
            bind_address: "127.0.0.1:18080".to_string(),
            remote_address: endpoint.host.clone(),
            remote_port: endpoint.port,
        })
    }
}

#[async_trait]
impl DataplaneBackend for MockDryRunBackend {
    fn metadata(&self) -> AdapterMetadata {
        AdapterMetadata {
            backend_name: self.backend_name().to_string(),
            display_name: "Mock Dry Run Backend".to_string(),
            version: "0.1.0".to_string(),
            supports_reload: false,
            dry_run_only: true,
        }
    }

    fn backend_name(&self) -> &'static str {
        "mock-backend"
    }

    fn init(&self, endpoint: &Endpoint, _context: &AdapterContext) -> Result<(), AdapterError> {
        Self::ensure_supported(endpoint)
    }

    fn build_dry_run_preflight(
        &self,
        endpoint: &Endpoint,
        context: &AdapterContext,
    ) -> Result<DryRunPreflight, AdapterError> {
        let binary_path = "veil-mock-backend";
        let config_path = format!("runtime/{}-mock.json", context.session_id);
        Ok(DryRunPreflight {
            backend_name: self.backend_name().to_string(),
            ready_for_dry_run_connect: true,
            binary_path: binary_path.to_string(),
            config_path: config_path.clone(),
            binary_present: Path::new(binary_path).exists(),
            readiness_note: "mock backend dry-run config prepared without launching a process"
                .to_string(),
            command: DryRunCommandSpec {
                program: binary_path.to_string(),
                args: vec!["--config".to_string(), config_path],
            },
            rendered_config: to_value(Self::render_config(endpoint)?)
                .map_err(|error| AdapterError::InvalidConfig(error.to_string()))?,
        })
    }

    fn apply_config(
        &self,
        endpoint: &Endpoint,
        context: &AdapterContext,
    ) -> Result<GeneratedConfig, AdapterError> {
        let preflight = self.build_dry_run_preflight(endpoint, context)?;
        Ok(GeneratedConfig {
            backend_name: self.backend_name().to_string(),
            payload: json!({
                "kind": "mock-dry-run",
                "client_platform": context.client_platform,
                "dry_run": context.dry_run,
                "session_id": context.session_id,
                "endpoint_id": endpoint.id,
                "preflight": preflight,
            }),
        })
    }

    async fn start(
        &self,
        config: &GeneratedConfig,
        _context: &AdapterContext,
    ) -> Result<RuntimeSnapshot, AdapterError> {
        Ok(RuntimeSnapshot {
            backend_name: config.backend_name.clone(),
            active: true,
            detail: "mock backend start is dry-run only".to_string(),
            config_applied: true,
        })
    }

    async fn health_check(&self) -> Result<HealthStatus, AdapterError> {
        Ok(HealthStatus {
            healthy: true,
            detail: "mock backend reports healthy".to_string(),
            reason_code: None,
        })
    }

    async fn reload(
        &self,
        _config: &GeneratedConfig,
        _context: &AdapterContext,
    ) -> Result<RuntimeSnapshot, AdapterError> {
        Err(AdapterError::ReloadUnsupported {
            backend: self.backend_name().to_string(),
        })
    }

    async fn stop(&self) -> Result<RuntimeSnapshot, AdapterError> {
        Ok(RuntimeSnapshot {
            backend_name: self.backend_name().to_string(),
            active: false,
            detail: "mock backend stop is dry-run only".to_string(),
            config_applied: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo_endpoint() -> Endpoint {
        Endpoint {
            id: "edge-mock-1".to_string(),
            host: "203.0.113.50".to_string(),
            port: 8443,
            transport: "https".to_string(),
            region: "us-west".to_string(),
            dataplane: Some("mock-backend".to_string()),
            supported_client_platforms: vec!["linux".to_string()],
            logical_server: Some("mock-edge".to_string()),
            provider_profile_schema_version: Some(1),
            xray: None,
        }
    }

    #[test]
    fn render_config_builds_mock_shape() {
        let rendered = MockDryRunBackend::render_config(&demo_endpoint())
            .expect("rendered config should build");

        assert_eq!(rendered.profile, "mock-dry-run");
        assert_eq!(rendered.endpoint_id, "edge-mock-1");
        assert_eq!(rendered.remote_address, "203.0.113.50");
    }

    #[test]
    fn trait_preflight_builds_without_real_binary() {
        let backend = MockDryRunBackend;
        let context = AdapterContext {
            client_platform: "linux".to_string(),
            dry_run: true,
            session_id: "session-1".to_string(),
        };

        let preflight = backend
            .build_dry_run_preflight(&demo_endpoint(), &context)
            .expect("preflight should build");

        assert_eq!(preflight.backend_name, "mock-backend");
        assert_eq!(preflight.command.program, "veil-mock-backend");
        assert_eq!(preflight.rendered_config["profile"], "mock-dry-run");
    }
}
