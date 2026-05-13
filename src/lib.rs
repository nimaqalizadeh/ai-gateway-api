#[allow(dead_code)] //TODO (phase-5): remove
mod cache;
mod config;
mod errors;
mod middleware;
mod providers;
mod routes;
mod state;

use axum::{routing::get, Router};
pub use config::Settings;
pub use state::AppState;

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/ping", get(routes::health::ping))
        .with_state(state)
}
