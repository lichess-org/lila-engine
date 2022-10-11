use std::{io, net::SocketAddr, time::Duration};

use axum::{
    body::StreamBody,
    extract::{BodyStream, FromRef, Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Router,
};
use axum_extra::routing::{RouterExt, TypedPath};
use clap::Parser;
use futures::stream::Stream;
use futures_util::stream::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr, DurationMilliSeconds};
use shakmaty::{uci::Uci, variant::VariantPosition};
use thiserror::Error;
use tokio::{
    io::AsyncBufReadExt,
    select,
    sync::mpsc::{channel, Sender},
    task,
    time::{error::Elapsed, timeout},
};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::io::StreamReader;

use crate::{
    api::{
        AcquireRequest, AcquireResponse, AnalyseRequest, Engine, EngineId, InvalidWorkError, JobId,
        ProviderSelector, Work,
    },
    hub::{Hub, IsValid},
    ongoing::Ongoing,
    repo::Repo,
    uci::{Eval, UciOut},
};

mod api;
mod hub;
mod ongoing;
mod repo;
mod uci;

#[derive(Parser)]
struct Opt {
    /// Binding address.
    #[arg(long, default_value = "127.0.0.1:9666")]
    pub bind: SocketAddr,
    /// Database.
    #[arg(long, default_value = "mongodb://localhost")]
    pub mongodb: String,
}

#[serde_as]
#[derive(Clone, Debug, Serialize)]
struct EmitPv {
    #[serde_as(as = "Vec<DisplayFromStr>")]
    moves: Vec<Uci>,
    eval: Eval,
    depth: u32,
}

#[serde_as]
#[derive(Clone, Debug, Serialize)]
struct Emit {
    depth: u32,
    nodes: u64,
    eval: Eval,
    pvs: Vec<EmitPv>,
    #[serde_as(as = "DurationMilliSeconds")]
    time: Duration,
}

struct Job {
    tx: Sender<UciOut>,
    pos: VariantPosition,
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
    Io(#[from] io::Error),
    #[error("uci protocol error: {0}")]
    Protocol(#[from] uci::ProtocolError),
    #[error("invalid work: {0}")]
    InvalidWork(#[from] InvalidWorkError),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self {
            Error::MongoDb(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Io(_) | Error::Protocol(_) | Error::InvalidWork(_) => StatusCode::BAD_REQUEST,
            Error::EngineNotFound | Error::WorkNotFound => StatusCode::NOT_FOUND,
        };
        (status, self.to_string()).into_response()
    }
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::new()
            .filter("ENGINE_LOG")
            .write_style("ENGINE_LOG_STYLE"),
    )
    .format_timestamp(None)
    .format_module_path(false)
    .format_target(false)
    .init();

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
        .typed_post(submit)
        .layer(tower_http::cors::CorsLayer::permissive());

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
) -> Result<StreamBody<impl Stream<Item = Result<String, io::Error>>>, Error> {
    let engine = repo
        .find(id, req.client_secret)
        .await?
        .ok_or(Error::EngineNotFound)?;
    let (work, pos) = req.work.sanitize(&engine)?;
    let (tx, rx) = channel(1);
    hub.submit(
        engine.provider_selector.clone(),
        Job {
            tx,
            engine: Engine::from(engine),
            work,
            pos,
        },
    );
    Ok(StreamBody::new(ReceiverStream::new(rx).map(|item| {
        Ok({
            let mut buf = item.to_string();
            buf.push('\n');
            buf
        })
    })))
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
    body: BodyStream,
) -> Result<(), Error> {
    let work = ongoing.remove(&id).ok_or(Error::WorkNotFound)?;
    let stream = body.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
    let read = StreamReader::new(stream);
    let mut lines = read.lines();
    while let Some(line) = select! {
        maybe_line = lines.next_line() => maybe_line?,
        _ = work.tx.closed() => None,
    } {
        if let Some(uci) = UciOut::from_line(&line)? {
            let is_bestmove = matches!(uci, UciOut::Bestmove { .. });
            if work.tx.send(uci).await.is_err() || is_bestmove {
                break;
            }
        }
    }
    Ok(())
}
