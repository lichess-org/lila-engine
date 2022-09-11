use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use shakmaty::{fen::Fen, uci::Uci};

#[derive(Deserialize, String)]
pub struct EngineId(String);

#[derive(Deserialize, Debug)]
pub struct ProviderSecret(String);

#[derive(Deserialize, Debug)]
pub struct ClientSecret(String);

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct AnalyseRequest {
    client_secret: ClientSecret,
    threads: u32,
    hash_mib: u32,
    path: String,
    max_depth: u32,
    multi_pv: u32,
    ply: u32,
    #[serde_as(as = "DisplayFromStr")]
    initial_fen: Fen,
    #[serde_as(as = "Vec<DisplayFromStr>")]
    moves: Vec<Uci>,
}

#[derive(Deserialize, Debug)]
pub struct AcquireRequest {
    provider_secret: ProviderSecret,
}
