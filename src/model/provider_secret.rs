use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Deserialize, Debug)]
pub struct ProviderSecret(String);

impl ProviderSecret {
    pub fn selector(&self) -> ProviderSelector {
        let mut hasher = Sha256::new();
        hasher.update("providerSecret:");
        hasher.update(self.0.as_bytes());
        ProviderSelector(hex::encode(hasher.finalize()))
    }
}

#[derive(Deserialize, Eq, PartialEq, Hash, Debug, Clone)]
pub struct ProviderSelector(String);
