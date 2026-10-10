#![forbid(unsafe_code)]

pub mod error;
pub mod signing;
pub mod types;

#[cfg(feature = "datastore")]
pub mod datastore;
#[cfg(feature = "friends")]
pub mod friends;
#[cfg(feature = "scores")]
pub mod scores;
#[cfg(feature = "sessions")]
pub mod sessions;
#[cfg(feature = "time")]
pub mod time;
#[cfg(feature = "trophies")]
pub mod trophies;
#[cfg(feature = "users")]
pub mod users;

pub mod client;
pub use client::{GameJoltClient, GameJoltClientBuilder, HashAlgo};
pub use error::{GameJoltError, Result};
pub use signing::{HashAlgorithm, Signer};
pub use types::*;

#[cfg(feature = "batch")]
pub mod batch;
#[cfg(feature = "bevy")]
pub mod bevy_plugin;
#[cfg(feature = "offline-queue")]
pub mod offline_queue;
#[cfg(feature = "rate-limiter")]
pub mod rate_limiter;
#[cfg(feature = "reactive")]
pub mod reactive;
