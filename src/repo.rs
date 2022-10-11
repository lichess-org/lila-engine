use std::num::NonZeroU32;

use mongodb::{bson::doc, error::Error, options::ClientOptions, Client, Collection};
use serde::{Deserialize, Serialize};
use tokio::task;

use crate::api::{ClientSecret, EngineId, LichessVariant, ProviderSelector, UserId};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalEngine {
    #[serde(alias = "_id")]
    id: EngineId,
    name: String,
    client_secret: ClientSecret,
    user_id: UserId,
    pub max_threads: NonZeroU32,
    pub max_hash: NonZeroU32,
    shallow_depth: u32,
    deep_depth: u32,
    pub variants: Vec<LichessVariant>,
    #[serde(skip_serializing)]
    pub provider_selector: ProviderSelector,
    provider_data: Option<String>,
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
