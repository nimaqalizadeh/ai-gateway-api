use ai_gateway::{app, AppState, Settings};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = envy::from_env::<Settings>()?;
    let bind = settings.bind;
    let state = AppState { settings };
    let listener = tokio::net::TcpListener::bind(bind).await?;
    println!("listening on http://{bind}");
    axum::serve(listener, app(state)).await?;
    Ok(())
}
