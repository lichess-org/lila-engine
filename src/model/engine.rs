use serde::{Deserialize, Serialize};
use std::fmt;
use std::num::NonZeroU32;

use crate::model::ClientSecret;
use crate::model::LichessVariant;
use crate::model::UserId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EngineId(pub String);

impl fmt::Display for EngineId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Engine {
    pub id: EngineId,
    #[serde(flatten)]
    pub config: EngineConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineConfig {
    pub name: String,
    pub client_secret: ClientSecret,
    pub user_id: UserId,
    pub max_threads: NonZeroU32,
    pub max_hash: NonZeroU32,
    pub shallow_depth: u32,
    pub deep_depth: u32,
    pub variants: Vec<LichessVariant>,
    pub provider_data: Option<String>,
}
