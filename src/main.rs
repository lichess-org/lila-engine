use std::net::SocketAddr;

use axum::{routing::get, Router};
use clap::Parser;

#[derive(Parser)]
struct Opt {
    /// Binding address.
    #[clap(long, default_value = "127.0.0.1:9666")]
    pub bind: SocketAddr,
}

#[tokio::main]
async fn main() {
    let opt = Opt::parse();

    let app = Router::new().route("/", get(root));

    axum::Server::bind(&opt.bind)
        .serve(app.into_make_service())
        .await
        .expect("bind");
}

async fn root() {}
