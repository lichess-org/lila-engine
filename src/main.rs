use std::net::SocketAddr;

use api::Registration;
use axum::{
    extract::Json,
    routing::{get, post},
    Router,
};
use clap::Parser;

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
        .route("/api/engine/registration", post(register))
        .route("/api/engine/socket", get(socket));

    axum::Server::bind(&opt.bind)
        .serve(app.into_make_service())
        .await
        .expect("bind");
}

async fn register(Json(registration): Json<Registration>) {}

async fn socket() {}
