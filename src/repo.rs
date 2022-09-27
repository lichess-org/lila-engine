use mongodb::{bson::doc, error::Error, options::ClientOptions, Client, Collection};
use serde::Deserialize;

use crate::api::{ClientSecret, EngineId, ProviderSecret};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalEngine {
    pub client_secret: ClientSecret,
    pub provider_secret: ProviderSecret,
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
        &self,
        id: EngineId,
        client_secret: ClientSecret,
    ) -> Result<Option<ExternalEngine>, Error> {
        self.coll
            .find_one(doc! { "_id": id.0 }, None)
            .await
            .map(|engine| engine.filter(|e| e.client_secret == client_secret))
    }
}
