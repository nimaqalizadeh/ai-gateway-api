use ai_gateway::{app, AppState, Settings};
use axum_test::TestServer;

#[tokio::test]
async fn ping_returns_pong() {
    let settings = Settings {
        bind: "0.0.0.0:0".parse().unwrap(),
    };
    let state = AppState { settings };
    let server = TestServer::new(app(state));
    let response = server.get("/ping").await;
    response.assert_status_ok();
    response.assert_text("pong");
}
