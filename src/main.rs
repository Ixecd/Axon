use axum::{response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use tracing_subscriber::fmt::format::FmtSpan;

use axon::router;
use axon::transfer;

#[derive(Serialize)]
struct Status {
    status: String,
}
#[derive(Serialize)]
struct Home {
    service: String,
    version: String,
}

async fn healthz() -> impl IntoResponse {
    Json(Status {
        status: "ok".into(),
    })
}
async fn home() -> impl IntoResponse {
    Json(Home {
        service: "axon".into(),
        version: "v0.1.0".into(),
    })
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_target(false)
        .init();

    let app_state = router::Router::new();

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/", get(home))
        .route("/api/v1/transfer", axum::routing::post(transfer::transfer))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::info!("Axon v0.1.0 — listening on :8080");
    axum::serve(listener, app).await.unwrap();
}
