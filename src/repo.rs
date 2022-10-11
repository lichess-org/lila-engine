use std::num::NonZeroU32;

use mongodb::{bson::doc, error::Error, options::ClientOptions, Client, Collection};
use serde::Deserialize;
use tokio::task;

use crate::api::{ClientSecret, Engine, EngineId, LichessVariant, ProviderSelector, UserId};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalEngine {
    #[serde(rename = "_id")]
    id: EngineId,
    name: String,
    client_secret: ClientSecret,
    user_id: UserId,
    pub max_threads: NonZeroU32,
    pub max_hash: NonZeroU32,
    shallow_depth: u32,
    deep_depth: u32,
    pub variants: Vec<LichessVariant>,
    pub provider_selector: ProviderSelector,
    provider_data: Option<String>,
}

impl From<ExternalEngine> for Engine {
    fn from(engine: ExternalEngine) -> Engine {
        Engine {
            id: engine.id,
            name: engine.name,
            client_secret: engine.client_secret,
            user_id: engine.user_id,
            max_threads: engine.max_threads,
            max_hash: engine.max_hash,
            shallow_depth: engine.shallow_depth,
            deep_depth: engine.deep_depth,
            variants: engine.variants,
            provider_data: engine.provider_data,
        }
    }
}

pub struct Repo {
    coll: Collection<ExternalEngine>,
}

impl Repo {
    pub async fn new(url: &str) -> Repo {
        let client =
            Client::with_options(ClientOptions::parse(url).await.expect("mongodb options"))
                .expect("mongodb client");

        Repo {
            coll: client
                .default_database()
                .unwrap_or_else(|| client.database("lichess"))
                .collection("external_engine"),
        }
    }

    pub async fn find(
        &'static self,
        id: EngineId,
        client_secret: ClientSecret,
    ) -> Result<Option<ExternalEngine>, Error> {
        // MongoDB driver does not support cancellation.
        task::spawn(async move {
            self.coll
                .find_one(doc! { "_id": id.0 }, None)
                .await
                .map(|engine| engine.filter(|e| e.client_secret == client_secret))
        })
        .await
        .expect("join mongodb find")
    }
}
