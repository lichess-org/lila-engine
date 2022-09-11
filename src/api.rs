use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ProviderSecret(String);

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationRequest {
    provider_secret: ProviderSecret,
    provider_data: String,
    name: String,
    max_threads: u32,
    max_hash: u32,
    variants: Vec<String>,
    official_stockfish: bool,
}

pub struct AcquireRequest {
    provider_secret: ProviderSecret,
}

pub struct AcquireResponse {
    provider_data: String,
    work: Work,
}

pub struct Work {}

pub struct Emit {}
