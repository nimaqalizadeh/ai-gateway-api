#[allow(dead_code)] //TODO (phase-5): remove
mod cache;
mod errors;
mod middleware;
mod providers;
mod routes;

use axum::{routing::get, Router};

pub fn app() -> Router {
    Router::new().route("/ping", get(routes::health::ping))
}
