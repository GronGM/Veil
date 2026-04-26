//! In-process adapter contracts for the Veil prototype.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use thiserror::Error;
use veil_manifest::Endpoint;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedConfig {
    pub backend_name: String,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterMetadata {
    pub backend_name: String,
    pub display_name: String,
    pub version: String,
    pub supports_reload: bool,
    pub dry_run_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterContext {
    pub client_platform: String,
    pub dry_run: bool,
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterRegistryEntry {
    pub metadata: AdapterMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AdapterRegistrySnapshot {
    pub entries: Vec<AdapterRegistryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub detail: String,
    pub reason_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSnapshot {
    pub backend_name: String,
    pub active: bool,
    pub detail: String,
    pub config_applied: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DryRunCommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DryRunPreflight {
    pub backend_name: String,
    pub ready_for_dry_run_connect: bool,
    pub binary_path: String,
    pub config_path: String,
    pub binary_present: bool,
    pub readiness_note: String,
    pub command: DryRunCommandSpec,
    pub rendered_config: Value,
}

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("unsupported endpoint for backend '{backend}'")]
    UnsupportedEndpoint { backend: String },
    #[error("invalid generated config: {0}")]
    InvalidConfig(String),
    #[error("backend '{backend}' does not support reload")]
    ReloadUnsupported { backend: String },
    #[error("backend '{backend}' is not registered")]
    BackendNotRegistered { backend: String },
}

#[async_trait]
pub trait DataplaneBackend: Send + Sync {
    fn metadata(&self) -> AdapterMetadata;
    fn backend_name(&self) -> &'static str;
    fn init(&self, endpoint: &Endpoint, context: &AdapterContext) -> Result<(), AdapterError>;
    fn build_dry_run_preflight(
        &self,
        endpoint: &Endpoint,
        context: &AdapterContext,
    ) -> Result<DryRunPreflight, AdapterError>;
    fn apply_config(
        &self,
        endpoint: &Endpoint,
        context: &AdapterContext,
    ) -> Result<GeneratedConfig, AdapterError>;
    async fn start(
        &self,
        config: &GeneratedConfig,
        context: &AdapterContext,
    ) -> Result<RuntimeSnapshot, AdapterError>;
    async fn health_check(&self) -> Result<HealthStatus, AdapterError>;
    async fn reload(
        &self,
        config: &GeneratedConfig,
        context: &AdapterContext,
    ) -> Result<RuntimeSnapshot, AdapterError>;
    async fn stop(&self) -> Result<RuntimeSnapshot, AdapterError>;
}

#[derive(Default)]
pub struct StaticAdapterRegistry {
    backends: BTreeMap<String, Box<dyn DataplaneBackend>>,
}

impl StaticAdapterRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, backend: Box<dyn DataplaneBackend>) {
        let name = backend.backend_name().to_string();
        self.backends.insert(name, backend);
    }

    pub fn metadata_snapshot(&self) -> AdapterRegistrySnapshot {
        let mut entries = self
            .backends
            .values()
            .map(|backend| AdapterRegistryEntry {
                metadata: backend.metadata(),
            })
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| left.metadata.backend_name.cmp(&right.metadata.backend_name));
        AdapterRegistrySnapshot { entries }
    }

    pub fn resolve_backend_name_for_endpoint(&self, endpoint: &Endpoint) -> Result<String, AdapterError> {
        let backend_name = endpoint
            .dataplane
            .clone()
            .unwrap_or_else(|| "xray-core".to_string());
        if self.backends.contains_key(&backend_name) {
            Ok(backend_name)
        } else {
            Err(AdapterError::BackendNotRegistered {
                backend: backend_name,
            })
        }
    }

    pub fn resolve_backend_for_endpoint(
        &self,
        endpoint: &Endpoint,
    ) -> Result<&dyn DataplaneBackend, AdapterError> {
        let backend_name = self.resolve_backend_name_for_endpoint(endpoint)?;
        self.backends
            .get(&backend_name)
            .map(|backend| backend.as_ref())
            .ok_or(AdapterError::BackendNotRegistered {
                backend: backend_name,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockBackend;

    #[async_trait]
    impl DataplaneBackend for MockBackend {
        fn metadata(&self) -> AdapterMetadata {
            AdapterMetadata {
                backend_name: self.backend_name().to_string(),
                display_name: "Mock Backend".to_string(),
                version: "0.1.0".to_string(),
                supports_reload: false,
                dry_run_only: true,
            }
        }

        fn backend_name(&self) -> &'static str {
            "mock-backend"
        }

        fn init(&self, _endpoint: &Endpoint, _context: &AdapterContext) -> Result<(), AdapterError> {
            Ok(())
        }

        fn build_dry_run_preflight(
            &self,
            _endpoint: &Endpoint,
            _context: &AdapterContext,
        ) -> Result<DryRunPreflight, AdapterError> {
            Ok(DryRunPreflight {
                backend_name: self.backend_name().to_string(),
                ready_for_dry_run_connect: true,
                binary_path: "mock".to_string(),
                config_path: "runtime/mock.json".to_string(),
                binary_present: false,
                readiness_note: "mock preflight".to_string(),
                command: DryRunCommandSpec {
                    program: "mock".to_string(),
                    args: vec!["run".to_string()],
                },
                rendered_config: serde_json::json!({
                    "kind": "mock"
                }),
            })
        }

        fn apply_config(
            &self,
            _endpoint: &Endpoint,
            _context: &AdapterContext,
        ) -> Result<GeneratedConfig, AdapterError> {
            Ok(GeneratedConfig {
                backend_name: self.backend_name().to_string(),
                payload: serde_json::json!({ "kind": "mock" }),
            })
        }

        async fn start(
            &self,
            _config: &GeneratedConfig,
            _context: &AdapterContext,
        ) -> Result<RuntimeSnapshot, AdapterError> {
            Ok(RuntimeSnapshot {
                backend_name: self.backend_name().to_string(),
                active: true,
                detail: "mock start".to_string(),
                config_applied: true,
            })
        }

        async fn health_check(&self) -> Result<HealthStatus, AdapterError> {
            Ok(HealthStatus {
                healthy: true,
                detail: "mock healthy".to_string(),
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
                detail: "mock stop".to_string(),
                config_applied: true,
            })
        }
    }

    #[test]
    fn adapter_context_tracks_dry_run_mode() {
        let context = AdapterContext {
            client_platform: "linux".to_string(),
            dry_run: true,
            session_id: "session-1".to_string(),
        };

        assert!(context.dry_run);
        assert_eq!(context.client_platform, "linux");
    }

    #[test]
    fn registry_snapshot_is_empty_by_default() {
        let registry = StaticAdapterRegistry::new();
        let snapshot = registry.metadata_snapshot();

        assert!(snapshot.entries.is_empty());
    }

    #[test]
    fn registry_resolves_backend_instance_for_endpoint() {
        let mut registry = StaticAdapterRegistry::new();
        registry.register(Box::new(MockBackend));
        let endpoint = Endpoint {
            id: "edge-1".to_string(),
            host: "198.51.100.10".to_string(),
            port: 443,
            transport: "https".to_string(),
            region: "eu-central".to_string(),
            dataplane: Some("mock-backend".to_string()),
            supported_client_platforms: vec!["linux".to_string()],
            logical_server: None,
            provider_profile_schema_version: None,
            xray: None,
        };
        let context = AdapterContext {
            client_platform: "linux".to_string(),
            dry_run: true,
            session_id: "session-1".to_string(),
        };

        let backend = registry
            .resolve_backend_for_endpoint(&endpoint)
            .expect("backend should resolve");
        let preflight = backend
            .build_dry_run_preflight(&endpoint, &context)
            .expect("preflight should build");

        assert_eq!(backend.backend_name(), "mock-backend");
        assert_eq!(preflight.backend_name, "mock-backend");
        assert_eq!(preflight.rendered_config["kind"], "mock");
    }
}
