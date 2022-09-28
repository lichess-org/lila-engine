use std::io;
use std::{net::SocketAddr, time::Duration};
use futures_util::stream::TryStreamExt;
use tokio::io::AsyncBufReadExt;
use tokio::select;

use axum::{
    extract::{FromRef, Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Router,
};
use axum_extra::routing::{RouterExt, TypedPath};
use clap::Parser;
use serde::Deserialize;
use thiserror::Error;
use tokio::{
    sync::mpsc::{channel, Sender},
    task,
    time::{error::Elapsed, timeout},
};
use tokio_util::io::StreamReader;

use crate::{
    api::{
        AcquireRequest, AcquireResponse, AnalyseRequest, Engine, EngineId, JobId, ProviderSelector,
        Work,
    },
    hub::{Hub, IsValid},
    ongoing::Ongoing,
    repo::Repo,
};
use axum::extract::BodyStream;

mod api;
mod hub;
mod ongoing;
mod repo;

#[derive(Parser)]
struct Opt {
    /// Binding address.
    #[clap(long, default_value = "127.0.0.1:9666")]
    pub bind: SocketAddr,
    /// Database.
    #[clap(long, default_value = "mongodb://localhost")]
    pub mongodb: String,
}

struct Job {
    tx: Sender<String>,
    engine: Engine,
    work: Work,
}

impl IsValid for Job {
    fn is_valid(&self) -> bool {
        !self.tx.is_closed()
    }
}

struct AppState {
    repo: &'static Repo,
    hub: &'static Hub<ProviderSelector, Job>,
    ongoing: &'static Ongoing<JobId, Job>,
}

impl FromRef<AppState> for &'static Repo {
    fn from_ref(state: &AppState) -> &'static Repo {
        state.repo
    }
}

impl FromRef<AppState> for &'static Hub<ProviderSelector, Job> {
    fn from_ref(state: &AppState) -> &'static Hub<ProviderSelector, Job> {
        state.hub
    }
}

impl FromRef<AppState> for &'static Ongoing<JobId, Job> {
    fn from_ref(state: &AppState) -> &'static Ongoing<JobId, Job> {
        state.ongoing
    }
}

#[derive(Error, Debug)]
enum Error {
    #[error("mongodb error: {0}")]
    MongoDb(#[from] mongodb::error::Error),
    #[error("engine not found or invalid clientSecret")]
    EngineNotFound,
    #[error("work not found or cancelled or expired")]
    WorkNotFound,
    #[error("i/o error: {0}")]
    IoError(#[from] io::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self {
            Error::MongoDb(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::IoError(_) => StatusCode::BAD_REQUEST,
            Error::EngineNotFound | Error::WorkNotFound => StatusCode::NOT_FOUND,
        };
        (status, self.to_string()).into_response()
    }
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::new().filter("ENGINE_LOG").write_style("ENGINE_LOG_STYLE"))
        .format_timestamp(None).format_module_path(false).format_target(false).init();

    let opt = Opt::parse();

    let state = AppState {
        repo: Box::leak(Box::new(Repo::new(&opt.mongodb).await)),
        hub: Box::leak(Box::new(Hub::new())),
        ongoing: Box::leak(Box::new(Ongoing::new())),
    };

    task::spawn(state.hub.garbage_collect());
    task::spawn(state.ongoing.garbage_collect());

    let app = Router::with_state(state)
        .typed_post(analyse)
        .typed_post(acquire)
        .typed_post(submit);

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

#[axum_macros::debug_handler(state = AppState)]
async fn analyse(
    AnalysePath { id }: AnalysePath,
    State(hub): State<&'static Hub<ProviderSelector, Job>>,
    State(repo): State<&'static Repo>,
    Json(req): Json<AnalyseRequest>,
) -> Result<(), Error> {
    let engine = repo
        .find(id, req.client_secret)
        .await?
        .ok_or(Error::EngineNotFound)?;
    let (tx, mut rx) = channel(1);
    hub.submit(
        engine.selector(),
        Job {
            tx,
            engine: Engine::from(engine),
            work: req.work,
        },
    );
    while let Some(line) = rx.recv().await {
        log::info!("received: {}", line);
    }
    Ok(())
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/external-engine/work")]
struct AcquirePath;

struct AcquireTimeout;

impl IntoResponse for AcquireTimeout {
    fn into_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}

#[axum_macros::debug_handler(state = AppState)]
async fn acquire(
    _: AcquirePath,
    State(hub): State<&'static Hub<ProviderSelector, Job>>,
    State(ongoing): State<&'static Ongoing<JobId, Job>>,
    Json(req): Json<AcquireRequest>,
) -> Result<Json<AcquireResponse>, AcquireTimeout> {
    let selector = req.provider_secret.selector();
    let job = timeout(Duration::from_secs(10), hub.acquire(selector))
        .await
        .map_err(|_: Elapsed| AcquireTimeout)?;
    let id = JobId::random();
    let response = AcquireResponse {
        id: id.clone(),
        engine: job.engine.clone(),
        work: job.work.clone(),
    };
    ongoing.add(id, job);
    Ok(Json(response))
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/external-engine/work/:id")]
struct SubmitPath {
    id: JobId,
}

#[axum_macros::debug_handler(state = AppState)]
async fn submit(
    SubmitPath { id }: SubmitPath,
    State(ongoing): State<&'static Ongoing<JobId, Job>>,
    body: BodyStream
) -> Result<(), Error> {
    let work = ongoing.remove(&id).ok_or(Error::WorkNotFound)?;
    let stream = body.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
    let read = StreamReader::new(stream);
    let mut lines = read.lines();
    loop {
        select! {
            maybe_line = lines.next_line() => {
                if let Some(line) = maybe_line? {
                    if work.tx.send(line).await.is_err() {
                        // Requester gone away.
                        break;
                    }
                } else {
                    // Provider gone away.
                    break;
                }
            }
            _ = work.tx.closed() => {
                // Requester gone away.
                break;
            }
        }
    }
    Ok(())
}
