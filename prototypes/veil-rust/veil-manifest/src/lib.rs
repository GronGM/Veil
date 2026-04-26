//! Typed manifest skeleton for Veil.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;

pub const MANIFEST_SCHEMA_VERSION: u32 = 1;
pub const PROVIDER_PROFILE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestMetadata {
    pub schema_version: u32,
    pub provider_profile_schema_version: Option<u32>,
    pub generated_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProfileKind {
    ProviderProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformCapability {
    pub platform: String,
    pub supported_dataplanes: Vec<String>,
    pub network_adapter: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct XrayEndpointMetadata {
    pub protocol: String,
    pub stream: String,
    pub security: String,
    pub server_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Endpoint {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub transport: String,
    pub region: String,
    pub dataplane: Option<String>,
    #[serde(default)]
    pub supported_client_platforms: Vec<String>,
    pub logical_server: Option<String>,
    pub provider_profile_schema_version: Option<u32>,
    pub xray: Option<XrayEndpointMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportPolicy {
    pub preferred_order: Vec<String>,
    pub retry_budget: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FeatureFlags {
    pub profile_kind: Option<ProfileKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderManifest {
    pub metadata: ManifestMetadata,
    pub capabilities: Vec<PlatformCapability>,
    pub transport_policy: TransportPolicy,
    #[serde(default)]
    pub features: FeatureFlags,
    pub endpoints: Vec<Endpoint>,
}

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("provider manifest must contain at least one endpoint")]
    EmptyEndpoints,
    #[error("manifest schema_version must be positive")]
    InvalidSchemaVersion,
    #[error("unsupported manifest schema_version '{0}'")]
    UnsupportedSchemaVersion(u32),
    #[error("unsupported provider_profile_schema_version '{0}'")]
    UnsupportedProviderProfileSchemaVersion(u32),
    #[error("provider-profile endpoint '{endpoint_id}' is missing logical_server")]
    MissingLogicalServer { endpoint_id: String },
    #[error("provider-profile endpoint '{endpoint_id}' has mismatched provider_profile_schema_version '{found}'")]
    ProviderProfileSchemaMismatch { endpoint_id: String, found: u32 },
    #[error("platform capability '{platform}' must declare at least one dataplane")]
    EmptySupportedDataplanes { platform: String },
    #[error("transport policy retry_budget must be positive")]
    InvalidRetryBudget,
    #[error("endpoint '{endpoint_id}' uses xray-core without typed xray metadata")]
    MissingXrayMetadata { endpoint_id: String },
}

pub fn validate_manifest(manifest: &ProviderManifest) -> Result<(), ManifestError> {
    if manifest.metadata.schema_version == 0 {
        return Err(ManifestError::InvalidSchemaVersion);
    }
    if manifest.metadata.schema_version != MANIFEST_SCHEMA_VERSION {
        return Err(ManifestError::UnsupportedSchemaVersion(
            manifest.metadata.schema_version,
        ));
    }
    if let Some(provider_profile_schema_version) = manifest.metadata.provider_profile_schema_version {
        if provider_profile_schema_version != PROVIDER_PROFILE_SCHEMA_VERSION {
            return Err(ManifestError::UnsupportedProviderProfileSchemaVersion(
                provider_profile_schema_version,
            ));
        }
    }
    if manifest.transport_policy.retry_budget == 0 {
        return Err(ManifestError::InvalidRetryBudget);
    }
    for capability in &manifest.capabilities {
        if capability.supported_dataplanes.is_empty() {
            return Err(ManifestError::EmptySupportedDataplanes {
                platform: capability.platform.clone(),
            });
        }
    }
    if manifest.endpoints.is_empty() {
        return Err(ManifestError::EmptyEndpoints);
    }
    let provider_profile_schema_version = manifest
        .metadata
        .provider_profile_schema_version
        .unwrap_or(PROVIDER_PROFILE_SCHEMA_VERSION);
    let is_provider_profile = matches!(
        manifest.features.profile_kind,
        Some(ProfileKind::ProviderProfile)
    );
    for endpoint in &manifest.endpoints {
        if is_provider_profile && endpoint.logical_server.is_none() {
            return Err(ManifestError::MissingLogicalServer {
                endpoint_id: endpoint.id.clone(),
            });
        }
        if let Some(found) = endpoint.provider_profile_schema_version {
            if found != provider_profile_schema_version {
                return Err(ManifestError::ProviderProfileSchemaMismatch {
                    endpoint_id: endpoint.id.clone(),
                    found,
                });
            }
        }
        if endpoint.dataplane.as_deref() == Some("xray-core") && endpoint.xray.is_none() {
            return Err(ManifestError::MissingXrayMetadata {
                endpoint_id: endpoint.id.clone(),
            });
        }
    }
    Ok(())
}

pub fn demo_provider_manifest() -> ProviderManifest {
    ProviderManifest {
        metadata: ManifestMetadata {
            schema_version: MANIFEST_SCHEMA_VERSION,
            provider_profile_schema_version: Some(PROVIDER_PROFILE_SCHEMA_VERSION),
            generated_at: OffsetDateTime::now_utc(),
            expires_at: OffsetDateTime::now_utc(),
        },
        capabilities: vec![PlatformCapability {
            platform: "linux".to_string(),
            supported_dataplanes: vec!["xray-core".to_string(), "mock-backend".to_string()],
            network_adapter: "linux".to_string(),
            status: "mvp-supported".to_string(),
        }],
        transport_policy: TransportPolicy {
            preferred_order: vec!["https".to_string()],
            retry_budget: 1,
        },
        features: FeatureFlags {
            profile_kind: Some(ProfileKind::ProviderProfile),
        },
        endpoints: vec![
            Endpoint {
                id: "edge-1".to_string(),
                host: "198.51.100.10".to_string(),
                port: 443,
                transport: "https".to_string(),
                region: "eu-central".to_string(),
                dataplane: Some("xray-core".to_string()),
                supported_client_platforms: vec!["linux".to_string()],
                logical_server: Some("edge".to_string()),
                provider_profile_schema_version: Some(PROVIDER_PROFILE_SCHEMA_VERSION),
                xray: Some(XrayEndpointMetadata {
                    protocol: "vless".to_string(),
                    stream: "tcp".to_string(),
                    security: "tls".to_string(),
                    server_name: Some("cdn.example.net".to_string()),
                }),
            },
            Endpoint {
                id: "edge-mock-1".to_string(),
                host: "203.0.113.50".to_string(),
                port: 8443,
                transport: "https".to_string(),
                region: "us-west".to_string(),
                dataplane: Some("mock-backend".to_string()),
                supported_client_platforms: vec!["linux".to_string()],
                logical_server: Some("mock-edge".to_string()),
                provider_profile_schema_version: Some(PROVIDER_PROFILE_SCHEMA_VERSION),
                xray: None,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_manifest_requires_endpoints() {
        let manifest = ProviderManifest {
            metadata: ManifestMetadata {
                schema_version: MANIFEST_SCHEMA_VERSION,
                provider_profile_schema_version: Some(PROVIDER_PROFILE_SCHEMA_VERSION),
                generated_at: OffsetDateTime::now_utc(),
                expires_at: OffsetDateTime::now_utc(),
            },
            capabilities: vec![],
            transport_policy: TransportPolicy {
                preferred_order: vec!["https".to_string()],
                retry_budget: 1,
            },
            features: FeatureFlags::default(),
            endpoints: vec![],
        };

        let error = validate_manifest(&manifest).expect_err("validation must fail");
        assert!(matches!(error, ManifestError::EmptyEndpoints));
    }

    #[test]
    fn validate_provider_profile_requires_logical_server() {
        let mut manifest = demo_provider_manifest();
        manifest.endpoints[0].logical_server = None;

        let error = validate_manifest(&manifest).expect_err("validation must fail");
        assert!(matches!(error, ManifestError::MissingLogicalServer { .. }));
    }

    #[test]
    fn validate_xray_endpoint_requires_typed_xray_metadata() {
        let mut manifest = demo_provider_manifest();
        manifest.endpoints[0].xray = None;

        let error = validate_manifest(&manifest).expect_err("validation must fail");
        assert!(matches!(error, ManifestError::MissingXrayMetadata { .. }));
    }

    #[test]
    fn demo_manifest_includes_second_backend_for_adapter_api_exercises() {
        let manifest = demo_provider_manifest();

        assert_eq!(manifest.endpoints.len(), 2);
        assert!(
            manifest.capabilities[0]
                .supported_dataplanes
                .iter()
                .any(|backend| backend == "mock-backend")
        );
        assert_eq!(
            manifest.endpoints[1].dataplane.as_deref(),
            Some("mock-backend")
        );
    }
}
