use serde::{Deserialize, Serialize};

mod client_secret;
mod engine;
mod job_id;
mod multi_pv;
mod provider_secret;
mod uci_variant;

pub use client_secret::ClientSecret;
pub use engine::{Engine, EngineConfig, EngineId};
pub use job_id::JobId;
pub use multi_pv::{InvalidMultiPvError, MultiPv};
pub use provider_secret::{ProviderSecret, ProviderSelector};
pub use uci_variant::UciVariant;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UserId(String);

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SessionId(String);
