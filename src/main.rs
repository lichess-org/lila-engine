use std::net::SocketAddr;

use axum::{
    extract::Json,
    routing::{get, post},
    Router,
};
use axum_extra::routing::{RouterExt, TypedPath};
use clap::Parser;
use mongodb::{options::ClientOptions, Client};
use serde::Deserialize;

use crate::api::{AcquireRequest, AnalyseRequest, EngineId};

mod api;
mod hub;

#[derive(Parser)]
struct Opt {
    /// Binding address.
    #[clap(long, default_value = "127.0.0.1:9666")]
    pub bind: SocketAddr,
    /// Database.
    #[clap(long, default_value = "mongodb://localhost")]
    pub mongodb: String,
}

#[derive(Deserialize, Debug)]
struct Registration {}

#[tokio::main]
async fn main() {
    let opt = Opt::parse();

    let db = Client::with_options(
        ClientOptions::parse(opt.mongodb)
            .await
            .expect("mongodb options"),
    )
    .expect("mongodb client")
    .database("lichess");

    let registrations = db.collection::<Registration>("external_engine");

    let app = Router::new()
        .typed_post(analyse)
        .route("/api/external-engine/acquire", get(acquire));

    axum::Server::bind(&opt.bind)
        .serve(app.into_make_service())
        .await
        .expect("bind");
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/external-engine/:id/analyse")]
struct AnalysePath {
    id: EngineId,
}

async fn analyse(AnalysePath { id }: AnalysePath, Json(req): Json<AnalyseRequest>) {
    dbg!(id, req);
}

async fn acquire(Json(req): Json<AcquireRequest>) {
    dbg!(req);
}
