#![forbid(unsafe_code)]

//! Manifest and provider input skeleton for Veil.

use veil_transport::TransportProfile;

/// Target client runtime modeled by the manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientPlatform {
    Linux,
    Windows,
    Macos,
    Android,
    Ios,
    Simulated,
}

impl ClientPlatform {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Linux => "linux",
            Self::Windows => "windows",
            Self::Macos => "macos",
            Self::Android => "android",
            Self::Ios => "ios",
            Self::Simulated => "simulated",
        }
    }
}

/// Local or modeled network adapter family for the current runtime contour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformAdapterKind {
    Linux,
    Placeholder,
    Simulated,
}

impl PlatformAdapterKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Linux => "linux",
            Self::Placeholder => "placeholder",
            Self::Simulated => "simulated",
        }
    }
}

/// Manifest-declared rollout status for a platform runtime contour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformSupportStatus {
    MvpSupported,
    Planned,
    BridgeOnly,
}

impl PlatformSupportStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MvpSupported => "mvp-supported",
            Self::Planned => "planned",
            Self::BridgeOnly => "bridge-only",
        }
    }
}

/// Minimal platform capability contract carried in the manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformCapability {
    pub platform: ClientPlatform,
    pub supported_backends: Vec<String>,
    pub platform_adapter: PlatformAdapterKind,
    pub status: PlatformSupportStatus,
}

/// Minimal typed manifest input for early dry-run wiring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderManifest {
    pub provider_name: String,
    pub profile_name: String,
    pub preferred_backend: String,
    pub preferred_transport: TransportProfile,
    pub client_platform: ClientPlatform,
    pub platform_adapter: PlatformAdapterKind,
    pub platform_capability: PlatformCapability,
}

impl ProviderManifest {
    /// Build a small demo manifest for the public dry-run path.
    pub fn demo() -> Self {
        Self {
            provider_name: "demo-provider".to_string(),
            profile_name: "stable-eu".to_string(),
            preferred_backend: "xray-core".to_string(),
            preferred_transport: TransportProfile::TlsTcp,
            client_platform: ClientPlatform::Linux,
            platform_adapter: PlatformAdapterKind::Linux,
            platform_capability: PlatformCapability {
                platform: ClientPlatform::Linux,
                supported_backends: vec!["xray-core".to_string()],
                platform_adapter: PlatformAdapterKind::Linux,
                status: PlatformSupportStatus::MvpSupported,
            },
        }
    }

    /// Build a demo manifest that prefers the gRPC transport profile.
    pub fn grpc_demo() -> Self {
        Self {
            provider_name: "demo-provider".to_string(),
            profile_name: "stable-eu".to_string(),
            preferred_backend: "xray-core".to_string(),
            preferred_transport: TransportProfile::Grpc,
            client_platform: ClientPlatform::Linux,
            platform_adapter: PlatformAdapterKind::Linux,
            platform_capability: PlatformCapability {
                platform: ClientPlatform::Linux,
                supported_backends: vec!["xray-core".to_string()],
                platform_adapter: PlatformAdapterKind::Linux,
                status: PlatformSupportStatus::MvpSupported,
            },
        }
    }

    /// Build a demo manifest where the selected contour is outside the MVP support contract.
    pub fn planned_windows_demo() -> Self {
        Self {
            provider_name: "demo-provider".to_string(),
            profile_name: "desktop-preview".to_string(),
            preferred_backend: "xray-core".to_string(),
            preferred_transport: TransportProfile::TlsTcp,
            client_platform: ClientPlatform::Windows,
            platform_adapter: PlatformAdapterKind::Placeholder,
            platform_capability: PlatformCapability {
                platform: ClientPlatform::Windows,
                supported_backends: vec!["xray-core".to_string()],
                platform_adapter: PlatformAdapterKind::Placeholder,
                status: PlatformSupportStatus::Planned,
            },
        }
    }

    /// Build a demo manifest for a valid linux contour that is still outside the first MVP target.
    pub fn linux_foundation_demo() -> Self {
        Self {
            provider_name: "demo-provider".to_string(),
            profile_name: "linux-foundation".to_string(),
            preferred_backend: "mock-backend".to_string(),
            preferred_transport: TransportProfile::TlsTcp,
            client_platform: ClientPlatform::Linux,
            platform_adapter: PlatformAdapterKind::Linux,
            platform_capability: PlatformCapability {
                platform: ClientPlatform::Linux,
                supported_backends: vec!["mock-backend".to_string()],
                platform_adapter: PlatformAdapterKind::Linux,
                status: PlatformSupportStatus::Planned,
            },
        }
    }

    /// Build a demo manifest where the selected contour mismatches the declared contract.
    pub fn contract_mismatch_demo() -> Self {
        Self {
            provider_name: "demo-provider".to_string(),
            profile_name: "desktop-preview".to_string(),
            preferred_backend: "xray-core".to_string(),
            preferred_transport: TransportProfile::TlsTcp,
            client_platform: ClientPlatform::Linux,
            platform_adapter: PlatformAdapterKind::Linux,
            platform_capability: PlatformCapability {
                platform: ClientPlatform::Linux,
                supported_backends: vec!["mock-backend".to_string()],
                platform_adapter: PlatformAdapterKind::Placeholder,
                status: PlatformSupportStatus::Planned,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_manifest_models_linux_mvp_contour() {
        let manifest = ProviderManifest::demo();

        assert_eq!(manifest.client_platform.as_str(), "linux");
        assert_eq!(manifest.platform_adapter.as_str(), "linux");
        assert_eq!(manifest.platform_capability.status.as_str(), "mvp-supported");
        assert_eq!(
            manifest.platform_capability.supported_backends,
            vec!["xray-core".to_string()]
        );
    }

    #[test]
    fn planned_windows_demo_models_non_mvp_contour() {
        let manifest = ProviderManifest::planned_windows_demo();

        assert_eq!(manifest.client_platform.as_str(), "windows");
        assert_eq!(manifest.platform_adapter.as_str(), "placeholder");
        assert_eq!(manifest.platform_capability.status.as_str(), "planned");
    }

    #[test]
    fn linux_foundation_demo_models_valid_non_mvp_linux_contour() {
        let manifest = ProviderManifest::linux_foundation_demo();

        assert_eq!(manifest.client_platform.as_str(), "linux");
        assert_eq!(manifest.preferred_backend, "mock-backend");
        assert_eq!(
            manifest.platform_capability.supported_backends,
            vec!["mock-backend".to_string()]
        );
    }
}
