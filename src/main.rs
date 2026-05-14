use ai_gateway::{app, AppState, Settings};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    ai_gateway::telemetry::init();
    let settings = envy::from_env::<Settings>()?;
    let addr = settings.bind;
    let state = AppState { settings };
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on http://{addr}");
    axum::serve(listener, app(state)).await?;
    Ok(())
}
