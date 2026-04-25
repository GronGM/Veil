//! Dry-run Xray adapter prototype for Veil.

use async_trait::async_trait;
use serde_json::json;
use veil_adapter_api::{
    AdapterContext, AdapterError, AdapterMetadata, DataplaneBackend, GeneratedConfig, HealthStatus,
    RuntimeSnapshot,
};
use veil_manifest::{Endpoint, XrayEndpointMetadata};

#[derive(Debug, Default)]
pub struct XrayDryRunBackend;

impl XrayDryRunBackend {
    fn ensure_supported(endpoint: &Endpoint) -> Result<(), AdapterError> {
        match endpoint.dataplane.as_deref() {
            Some("xray-core") | None => Ok(()),
            _ => Err(AdapterError::UnsupportedEndpoint {
                backend: "xray-core".to_string(),
            }),
        }
    }
}

#[async_trait]
impl DataplaneBackend for XrayDryRunBackend {
    fn metadata(&self) -> AdapterMetadata {
        AdapterMetadata {
            backend_name: self.backend_name().to_string(),
            display_name: "Xray Dry Run Backend".to_string(),
            version: "0.1.0".to_string(),
            supports_reload: true,
            dry_run_only: true,
        }
    }

    fn backend_name(&self) -> &'static str {
        "xray-core"
    }

    fn init(&self, endpoint: &Endpoint, _context: &AdapterContext) -> Result<(), AdapterError> {
        Self::ensure_supported(endpoint)
    }

    fn apply_config(
        &self,
        endpoint: &Endpoint,
        context: &AdapterContext,
    ) -> Result<GeneratedConfig, AdapterError> {
        Self::ensure_supported(endpoint)?;
        Ok(GeneratedConfig {
            backend_name: self.backend_name().to_string(),
            payload: json!({
                "kind": "xray-dry-run",
                "client_platform": context.client_platform,
                "dry_run": context.dry_run,
                "session_id": context.session_id,
                "endpoint_id": endpoint.id,
                "host": endpoint.host,
                "port": endpoint.port,
                "transport": endpoint.transport,
                "xray": endpoint.xray.as_ref().map(|xray| {
                    json!({
                        "protocol": xray.protocol,
                        "stream": xray.stream,
                        "security": xray.security,
                        "server_name": xray.server_name,
                    })
                }),
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
            detail: "dry-run start only; no real xray process launched".to_string(),
            config_applied: true,
        })
    }

    async fn health_check(&self) -> Result<HealthStatus, AdapterError> {
        Ok(HealthStatus {
            healthy: true,
            detail: "dry-run backend reports healthy".to_string(),
            reason_code: None,
        })
    }

    async fn reload(
        &self,
        config: &GeneratedConfig,
        _context: &AdapterContext,
    ) -> Result<RuntimeSnapshot, AdapterError> {
        Ok(RuntimeSnapshot {
            backend_name: config.backend_name.clone(),
            active: true,
            detail: "dry-run reload only".to_string(),
            config_applied: true,
        })
    }

    async fn stop(&self) -> Result<RuntimeSnapshot, AdapterError> {
        Ok(RuntimeSnapshot {
            backend_name: self.backend_name().to_string(),
            active: false,
            detail: "dry-run stop only".to_string(),
            config_applied: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use veil_adapter_api::{AdapterContext, StaticAdapterRegistry};

    #[test]
    fn apply_config_builds_dry_run_payload() {
        let endpoint = Endpoint {
            id: "edge-1".to_string(),
            host: "198.51.100.10".to_string(),
            port: 443,
            transport: "https".to_string(),
            region: "eu-central".to_string(),
            dataplane: Some("xray-core".to_string()),
            supported_client_platforms: vec!["linux".to_string()],
            logical_server: Some("edge".to_string()),
            provider_profile_schema_version: Some(1),
            xray: Some(XrayEndpointMetadata {
                protocol: "vless".to_string(),
                stream: "tcp".to_string(),
                security: "tls".to_string(),
                server_name: Some("cdn.example.net".to_string()),
            }),
        };

        let backend = XrayDryRunBackend;
        let context = AdapterContext {
            client_platform: "linux".to_string(),
            dry_run: true,
            session_id: "session-1".to_string(),
        };
        let config = backend
            .apply_config(&endpoint, &context)
            .expect("config should build");

        assert_eq!(config.backend_name, "xray-core");
        assert_eq!(config.payload["endpoint_id"], "edge-1");
        assert_eq!(config.payload["client_platform"], "linux");
    }

    #[test]
    fn metadata_reports_dry_run_only_backend() {
        let backend = XrayDryRunBackend;
        let metadata = backend.metadata();

        assert_eq!(metadata.backend_name, "xray-core");
        assert!(metadata.dry_run_only);
    }

    #[test]
    fn registry_resolves_xray_backend_for_endpoint() {
        let endpoint = Endpoint {
            id: "edge-1".to_string(),
            host: "198.51.100.10".to_string(),
            port: 443,
            transport: "https".to_string(),
            region: "eu-central".to_string(),
            dataplane: Some("xray-core".to_string()),
            supported_client_platforms: vec!["linux".to_string()],
            logical_server: Some("edge".to_string()),
            provider_profile_schema_version: Some(1),
            xray: Some(XrayEndpointMetadata {
                protocol: "vless".to_string(),
                stream: "tcp".to_string(),
                security: "tls".to_string(),
                server_name: Some("cdn.example.net".to_string()),
            }),
        };

        let mut registry = StaticAdapterRegistry::new();
        registry.register(Box::new(XrayDryRunBackend));

        let backend_name = registry
            .resolve_backend_name_for_endpoint(&endpoint)
            .expect("xray backend should resolve");

        assert_eq!(backend_name, "xray-core");
    }
}
