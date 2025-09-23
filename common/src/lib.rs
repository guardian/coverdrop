//! The `common` crate provides basic components for both clients and servers in the CoverDrop system, primarily
//! cryptographic wrappers, such as [`SecretBox`] and data structures like [`PaddedCompressedString`].
//!
//! [`SecretBox`]: crypto::SecretBox
//! [`PaddedCompressedString`]: PaddedCompressedString

pub mod api;
pub mod argon2_sqlcipher;
pub mod aws;
pub mod backup;
pub mod clap;
pub mod client;
pub mod clients;
mod cover_serializable;
pub mod crypto;
pub mod epoch;
mod error;
mod fixed_buffer;
pub mod form;
pub mod generators;
pub mod healthcheck;
pub mod identity_api;
pub mod metrics;
pub mod monitoring;
#[allow(dead_code)]
mod padded_byte_vector;
mod padded_compressed_string;
pub mod protocol;
mod read_ext;
pub mod service;
pub mod system;
pub mod task;
pub mod throttle;
pub mod time;
pub mod tracing;
pub mod u2j_appender;

pub use argon2::password_hash::SaltString as Argon2Salt;
pub use error::Error;
pub use fixed_buffer::FixedBuffer;
pub use padded_compressed_string::{FixedSizeMessageText, PaddedCompressedString};
