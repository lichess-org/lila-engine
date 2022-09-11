use std::fmt;

use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use shakmaty::{fen::Fen, uci::Uci, variant::Variant};

#[derive(Deserialize, Debug)]
pub struct Sri(String);

#[derive(Deserialize, Debug)]
pub struct EngineId(String);

impl fmt::Display for EngineId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Deserialize, Debug)]
pub struct ProviderSecret(String);

impl ProviderSecret {
    pub fn selector(&self) -> ProviderSelector {
        todo!()
    }
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub struct ProviderSelector(String);

#[derive(Deserialize, Debug, Eq)]
pub struct ClientSecret(String);

impl PartialEq for ClientSecret {
    fn eq(&self, other: &ClientSecret) -> bool {
        self.0 == other.0 // TODO
    }
}

#[derive(Debug, Deserialize)]
pub enum LichessVariant {
    #[serde(alias = "antichess")]
    Antichess,
    #[serde(alias = "atomic")]
    Atomic,
    #[serde(alias = "chess960")]
    Chess960,
    #[serde(alias = "crazyhouse")]
    Crazyhouse,
    #[serde(alias = "fromPosition", alias = "From Position")]
    FromPosition,
    #[serde(alias = "horde")]
    Horde,
    #[serde(alias = "kingOfTheHill", alias = "King of the Hill")]
    KingOfTheHill,
    #[serde(alias = "racingKings", alias = "Racing Kings")]
    RacingKings,
    #[serde(alias = "chess", alias = "standard")]
    Standard,
    #[serde(alias = "threeCheck", alias = "Three-check")]
    ThreeCheck,
}

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct AnalyseRequest {
    client_secret: ClientSecret,
    sri: Sri,
    threads: u32,
    hash_mib: u32,
    max_depth: u32,
    multi_pv: u32,
    variant: LichessVariant,
    #[serde_as(as = "DisplayFromStr")]
    initial_fen: Fen,
    #[serde_as(as = "Vec<DisplayFromStr>")]
    moves: Vec<Uci>,
}

#[derive(Deserialize, Debug)]
pub struct AcquireRequest {
    provider_secret: ProviderSecret,
}
