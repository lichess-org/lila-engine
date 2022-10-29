use std::{fmt, num::NonZeroU32};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, FromInto};
use shakmaty::variant::Variant;

use crate::model::{ClientSecret, UciVariant, UserId};

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

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineConfig {
    pub name: String,
    pub client_secret: ClientSecret,
    pub user_id: UserId,
    pub max_threads: NonZeroU32,
    pub max_hash: NonZeroU32,
    #[serde(default, alias = "shallowDepth")] // alias not worked with flatten
    pub default_depth: Option<u32>,
    #[serde_as(as = "Vec<FromInto<UciVariant>>")]
    pub variants: Vec<Variant>,
    pub provider_data: Option<String>,
}
