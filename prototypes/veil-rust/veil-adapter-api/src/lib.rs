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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
