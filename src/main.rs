use std::{convert::Infallible, io, net::SocketAddr, time::Duration};

use axum::{
    extract::{BodyStream, FromRef, Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Router,
};
use axum_extra::{
    json_lines,
    json_lines::JsonLines,
    routing::{RouterExt, TypedPath},
};
use clap::Parser;
use futures::Stream;
use futures_util::stream::{StreamExt, TryStreamExt};
use serde::Deserialize;
use shakmaty::variant::VariantPosition;
use thiserror::Error;
use tokio::{
    io::AsyncBufReadExt,
    select,
    sync::{
        mpsc,
        oneshot::{self, error::RecvError},
    },
    task,
    time::{error::Elapsed, timeout},
};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::io::StreamReader;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    api::{AcquireRequest, AcquireResponse, AnalyseRequest, InvalidWorkError, Work},
    emit::Emit,
    hub::{Hub, IsValid},
    model::{Engine, EngineId, JobId, ProviderSelector},
    ongoing::Ongoing,
    repo::Repo,
    uci::UciOut,
};

mod api;
mod emit;
mod hub;
mod model;
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

struct Job {
    tx: oneshot::Sender<mpsc::Receiver<Emit>>,
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
    #[error("recv error: {0}")]
    RecvError(#[from] RecvError),
    #[error("provider did not pick up work")]
    ProviderTimeout,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self {
            Error::MongoDb(_) | Error::RecvError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Io(_) | Error::Protocol(_) | Error::InvalidWork(_) => StatusCode::BAD_REQUEST,
            Error::EngineNotFound | Error::WorkNotFound => StatusCode::NOT_FOUND,
            Error::ProviderTimeout => StatusCode::SERVICE_UNAVAILABLE,
        };
        (status, self.to_string()).into_response()
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("LILA_ENGINE_LOG")
                .unwrap_or_else(|_| "lila_engine=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer().without_time())
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
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

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
) -> Result<JsonLines<impl Stream<Item = Result<Emit, Infallible>>, json_lines::AsResponse>, Error>
{
    let (engine, provider_selector) = repo
        .find(id, req.client_secret)
        .await?
        .ok_or(Error::EngineNotFound)?
        .into_engine_and_selector();
    let (work, pos) = req.work.sanitize(&engine)?;
    let (tx, rx) = oneshot::channel();
    hub.submit(
        provider_selector,
        Job {
            tx,
            engine,
            work,
            pos,
        },
    );
    let rx = timeout(Duration::from_secs(15), rx)
        .await
        .map_err(|_: Elapsed| Error::ProviderTimeout)??;
    Ok(JsonLines::new(
        ReceiverStream::new(rx).map(|item| Ok::<_, Infallible>(item)),
    ))
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
    let (tx, rx) = mpsc::channel(1);
    let _: Result<(), _> = work.tx.send(rx);

    let stream = body.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
    let read = StreamReader::new(stream);
    let mut lines = read.lines();

    let mut emit = Emit::default();

    while let Some(line) = select! {
        maybe_line = lines.next_line() => maybe_line?,
        _ = tx.closed() => None,
    } {
        if let Some(uci) = UciOut::from_line(&line)? {
            emit.update(&uci, &work.pos);

            if matches!(uci, UciOut::Bestmove { .. }) {
                break;
            }

            if emit.should_emit() && tx.send(emit.clone()).await.is_err() {
                break;
            }
        }
    }
    Ok(())
}
