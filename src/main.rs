use ai_gateway::app;

const ADDR: &str = "0.0.0.0:8080";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(ADDR).await?;
    println!("listening on http://{ADDR}");
    axum::serve(listener, app()).await?;
    Ok(())
}
