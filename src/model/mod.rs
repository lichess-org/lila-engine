use serde::{Deserialize, Serialize};

mod client_secret;
mod engine;
mod job_id;
mod lichess_variant;
mod multi_pv;
mod provider_secret;

pub use client_secret::ClientSecret;
pub use engine::{Engine, EngineConfig, EngineId};
pub use job_id::JobId;
pub use lichess_variant::LichessVariant;
pub use multi_pv::{InvalidMultiPvError, MultiPv};
pub use provider_secret::{ProviderSecret, ProviderSelector};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UserId(String);

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SessionId(String);
