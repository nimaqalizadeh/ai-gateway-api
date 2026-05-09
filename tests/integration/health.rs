use ai_gateway::app;
use axum_test::TestServer;

#[tokio::test]
async fn ping_returns_pong() {
    let server = TestServer::new(app());
    let response = server.get("/ping").await;
    response.assert_status_ok();
    response.assert_text("pong");
}
