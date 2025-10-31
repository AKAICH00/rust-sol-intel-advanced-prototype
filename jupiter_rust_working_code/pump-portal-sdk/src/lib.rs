//! PumpPortal Trading API SDK for Rust
//!
//! A Rust client for interacting with the PumpPortal Lightning Transaction API.
//! This SDK provides a simple interface for executing buy and sell trades on Solana.

use thiserror::Error;

pub mod types;
pub mod client;

pub use types::*;
pub use client::PumpPortalClient;

/// Result type for PumpPortal SDK operations
pub type Result<T> = std::result::Result<T, PumpPortalError>;

/// Errors that can occur when using the PumpPortal SDK
#[derive(Error, Debug)]
pub enum PumpPortalError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    /// API returned an error
    #[error("API error: {0}")]
    ApiError(String),

    /// Invalid parameter provided
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
