#[allow(dead_code)] //TODO (phase-5): remove
mod cache;
mod config;
mod errors;
mod middleware;
mod providers;
mod routes;
mod state;
pub mod telemetry;
use axum::{routing::get, Router};
pub use config::Settings;
pub use state::AppState;
use tower_http::trace::TraceLayer;

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/ping", get(routes::health::ping))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
