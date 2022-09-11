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
use tokio::sync::mpsc::Sender;
use axum::extract::{State, FromRef};
use tokio::sync::mpsc::channel;

use crate::api::{AcquireRequest, AnalyseRequest, EngineId, ProviderSelector};
use crate::hub::{Hub, IsValid};

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

#[derive(Clone, Hash, Eq, PartialEq)]
struct WorkId(String);

struct Work {
    tx: Sender<()>,
}

impl IsValid for Work {
    fn is_valid(&self) -> bool {
        !self.tx.is_closed()
    }
}

struct AppState {
    pending: &'static Hub<ProviderSelector, Work>,
    ongoing: &'static Hub<WorkId, Work>,
}

impl FromRef<AppState> for &'static Hub<ProviderSelector, Work> {
    fn from_ref(state: &AppState) -> &'static Hub<ProviderSelector, Work> {
        state.pending
    }
}

impl FromRef<AppState> for &'static Hub<WorkId, Work> {
    fn from_ref(state: &AppState) -> &'static Hub<WorkId, Work> {
        state.ongoing
    }
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

    let state = AppState {
        pending: Box::leak(Box::new(Hub::new())),
        ongoing: Box::leak(Box::new(Hub::new())),
    };

    let app = Router::with_state(state)
        .typed_post(analyse)
        .route("/api/external-engine/acquire", post(acquire))
        .route("/api/external-engine/submit", post(submit));

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

async fn analyse(AnalysePath { id }: AnalysePath, State(pending): State<&'static Hub<ProviderSelector, Work>>, Json(req): Json<AnalyseRequest>) {
    let selector = todo!();
    let (tx, rx) = channel(4);
    pending.submit(selector, Work { tx });
}

async fn acquire(State(pending): State<&'static Hub<ProviderSelector, Work>>, State(ongoing): State<&'static Hub<WorkId, Work>>, Json(req): Json<AcquireRequest>) {
    let selector = todo!();
    let work = pending.acquire(selector).await;
    ongoing.submit(todo!(), work);
}

async fn submit(State(ongoing): State<&'static Hub<WorkId, Work>>) {
}
