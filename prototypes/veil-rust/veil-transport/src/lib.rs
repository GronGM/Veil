#![forbid(unsafe_code)]

//! Shared transport domain types for Veil.

/// Minimal transport intent selected by the control plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportProfile {
    TlsTcp,
    Grpc,
}

impl TransportProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TlsTcp => "tls-tcp",
            Self::Grpc => "grpc",
        }
    }
}
