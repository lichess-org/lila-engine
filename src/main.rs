use std::net::SocketAddr;

use axum::{
    extract::Json,
    routing::{get, post},
    Router,
};
use clap::Parser;

use serde::Deserialize;

mod api;

#[derive(Parser)]
struct Opt {
    /// Binding address.
    #[clap(long, default_value = "127.0.0.1:9666")]
    pub bind: SocketAddr,
}

#[tokio::main]
async fn main() {
    let opt = Opt::parse();

    let app = Router::new()
        .route("/api/external-engine/:id/analyse", post(analyse));

    axum::Server::bind(&opt.bind)
        .serve(app.into_make_service())
        .await
        .expect("bind");
}

#[derive(Deserialize, Debug)]
struct AnalysisRequest {
    client_secret: String,
}

async fn analyse(Json(req): Json<AnalysisRequest>) {
    dbg!(req);
}
