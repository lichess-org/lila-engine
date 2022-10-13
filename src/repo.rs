use crate::model::Engine;
use mongodb::{bson::doc, error::Error, options::ClientOptions, Client, Collection};
use serde::Deserialize;
use tokio::task;

use crate::model::ClientSecret;
use crate::model::EngineConfig;
use crate::model::EngineId;
use crate::model::ProviderSelector;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalEngine {
    #[serde(rename = "_id")]
    id: EngineId,
    provider_selector: ProviderSelector,
    #[serde(flatten)]
    config: EngineConfig,
}

impl ExternalEngine {
    pub fn into_engine_and_selector(self) -> (Engine, ProviderSelector) {
        (
            Engine {
                id: self.id,
                config: self.config,
            },
            self.provider_selector,
        )
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
                .map(|engine| engine.filter(|e| e.config.client_secret == client_secret))
        })
        .await
        .expect("join mongodb find")
    }
}
