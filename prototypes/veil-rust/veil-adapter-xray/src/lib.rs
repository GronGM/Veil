//! Dry-run Xray adapter prototype for Veil.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, to_value};
use std::path::Path;
use veil_adapter_api::{
    AdapterContext, AdapterError, AdapterMetadata, DataplaneBackend, GeneratedConfig, HealthStatus,
    RuntimeSnapshot,
};
use veil_manifest::{Endpoint, XrayEndpointMetadata};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XrayRenderedConfig {
    pub log: XrayLogConfig,
    pub inbounds: Vec<XrayInboundConfig>,
    pub outbounds: Vec<XrayOutboundConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XrayLogConfig {
    pub loglevel: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XrayInboundConfig {
    pub tag: String,
    pub listen: String,
    pub port: u16,
    pub protocol: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XrayOutboundConfig {
    pub tag: String,
    pub protocol: String,
    pub settings: XrayOutboundSettings,
    pub stream_settings: XrayStreamSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XrayOutboundSettings {
    pub address: String,
    pub port: u16,
    pub server_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XrayStreamSettings {
    pub network: String,
    pub security: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XrayCommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XrayDryRunPreflight {
    pub binary_path: String,
    pub config_path: String,
    pub binary_present: bool,
    pub command: XrayCommandSpec,
    pub rendered_config: XrayRenderedConfig,
}

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

    fn require_xray_metadata(endpoint: &Endpoint) -> Result<&XrayEndpointMetadata, AdapterError> {
        endpoint.xray.as_ref().ok_or_else(|| {
            AdapterError::InvalidConfig(format!(
                "endpoint '{}' is missing typed xray metadata",
                endpoint.id
            ))
        })
    }

    pub fn render_config(endpoint: &Endpoint) -> Result<XrayRenderedConfig, AdapterError> {
        Self::ensure_supported(endpoint)?;
        let xray = Self::require_xray_metadata(endpoint)?;

        Ok(XrayRenderedConfig {
            log: XrayLogConfig {
                loglevel: "warning".to_string(),
            },
            inbounds: vec![XrayInboundConfig {
                tag: "veil-socks-in".to_string(),
                listen: "127.0.0.1".to_string(),
                port: 10808,
                protocol: "socks".to_string(),
            }],
            outbounds: vec![XrayOutboundConfig {
                tag: endpoint.id.clone(),
                protocol: xray.protocol.clone(),
                settings: XrayOutboundSettings {
                    address: endpoint.host.clone(),
                    port: endpoint.port,
                    server_name: xray.server_name.clone(),
                },
                stream_settings: XrayStreamSettings {
                    network: xray.stream.clone(),
                    security: xray.security.clone(),
                },
            }],
        })
    }

    pub fn build_command(binary_path: &str, config_path: &str) -> XrayCommandSpec {
        XrayCommandSpec {
            program: binary_path.to_string(),
            args: vec!["run".to_string(), "-config".to_string(), config_path.to_string()],
        }
    }

    pub fn build_dry_run_preflight(
        endpoint: &Endpoint,
        binary_path: &str,
        config_path: &str,
    ) -> Result<XrayDryRunPreflight, AdapterError> {
        let rendered_config = Self::render_config(endpoint)?;
        Ok(XrayDryRunPreflight {
            binary_path: binary_path.to_string(),
            config_path: config_path.to_string(),
            binary_present: Path::new(binary_path).exists(),
            command: Self::build_command(binary_path, config_path),
            rendered_config,
        })
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
        Self::ensure_supported(endpoint)?;
        let _ = Self::require_xray_metadata(endpoint)?;
        Ok(())
    }

    fn apply_config(
        &self,
        endpoint: &Endpoint,
        context: &AdapterContext,
    ) -> Result<GeneratedConfig, AdapterError> {
        let rendered_config = Self::render_config(endpoint)?;
        let preflight = Self::build_dry_run_preflight(
            endpoint,
            "xray",
            &format!("runtime/{}.json", context.session_id),
        )?;

        Ok(GeneratedConfig {
            backend_name: self.backend_name().to_string(),
            payload: json!({
                "kind": "xray-dry-run",
                "client_platform": context.client_platform,
                "dry_run": context.dry_run,
                "session_id": context.session_id,
                "endpoint_id": endpoint.id,
                "rendered_config": to_value(rendered_config)
                    .map_err(|error| AdapterError::InvalidConfig(error.to_string()))?,
                "preflight": to_value(preflight)
                    .map_err(|error| AdapterError::InvalidConfig(error.to_string()))?,
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
            detail: "dry-run start only; xray command prepared but no real process launched"
                .to_string(),
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
            detail: "dry-run reload only; command lifecycle remains mocked".to_string(),
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

    fn demo_endpoint() -> Endpoint {
        Endpoint {
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
        }
    }

    #[test]
    fn render_config_builds_xray_like_shape() {
        let rendered = XrayDryRunBackend::render_config(&demo_endpoint())
            .expect("rendered config should build");

        assert_eq!(rendered.outbounds.len(), 1);
        assert_eq!(rendered.outbounds[0].protocol, "vless");
        assert_eq!(rendered.outbounds[0].settings.address, "198.51.100.10");
        assert_eq!(rendered.outbounds[0].stream_settings.network, "tcp");
        assert_eq!(rendered.inbounds[0].protocol, "socks");
    }

    #[test]
    fn build_command_keeps_execution_details_separate() {
        let command = XrayDryRunBackend::build_command("/usr/bin/xray", "/tmp/demo.json");

        assert_eq!(command.program, "/usr/bin/xray");
        assert_eq!(command.args, vec!["run", "-config", "/tmp/demo.json"]);
    }

    #[test]
    fn build_preflight_does_not_require_real_binary() {
        let preflight = XrayDryRunBackend::build_dry_run_preflight(
            &demo_endpoint(),
            "/definitely/not/xray",
            "/tmp/demo.json",
        )
        .expect("preflight should still build");

        assert!(!preflight.binary_present);
        assert_eq!(preflight.command.program, "/definitely/not/xray");
        assert_eq!(preflight.rendered_config.outbounds[0].tag, "edge-1");
    }

    #[test]
    fn apply_config_builds_dry_run_payload() {
        let backend = XrayDryRunBackend;
        let context = AdapterContext {
            client_platform: "linux".to_string(),
            dry_run: true,
            session_id: "session-1".to_string(),
        };
        let config = backend
            .apply_config(&demo_endpoint(), &context)
            .expect("config should build");

        assert_eq!(config.backend_name, "xray-core");
        assert_eq!(config.payload["endpoint_id"], "edge-1");
        assert_eq!(config.payload["client_platform"], "linux");
        assert_eq!(
            config.payload["rendered_config"]["outbounds"][0]["protocol"],
            "vless"
        );
        assert_eq!(config.payload["preflight"]["command"]["program"], "xray");
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
        let mut registry = StaticAdapterRegistry::new();
        registry.register(Box::new(XrayDryRunBackend));

        let backend_name = registry
            .resolve_backend_name_for_endpoint(&demo_endpoint())
            .expect("xray backend should resolve");

        assert_eq!(backend_name, "xray-core");
    }

    #[test]
    fn init_rejects_missing_xray_metadata() {
        let backend = XrayDryRunBackend;
        let mut endpoint = demo_endpoint();
        endpoint.xray = None;
        let context = AdapterContext {
            client_platform: "linux".to_string(),
            dry_run: true,
            session_id: "session-1".to_string(),
        };

        let error = backend
            .init(&endpoint, &context)
            .expect_err("missing xray metadata should fail");

        assert!(matches!(error, AdapterError::InvalidConfig(_)));
    }
}
