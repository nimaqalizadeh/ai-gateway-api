mod cache;
mod errors;
mod middleware;
mod providers;
mod routes;
use axum::{routing::get, Router};

const ADDR: &str = "0.0.0.0:8080";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Router::new().route("/ping", get(routes::health::ping));
    let listener = tokio::net::TcpListener::bind(ADDR).await?;
    println!("listening on http://{ADDR}");
    axum::serve(listener, app).await?;
    Ok(())
}
