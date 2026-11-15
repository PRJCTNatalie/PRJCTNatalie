//! HTTP API server (axum).

use crate::agent::Agent;
use crate::config::Config;
use axum::extract::{Json, State};
use axum::routing::{get, post};
use axum::{Router, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
struct AppState {
    config: Config,
    agent: Arc<Mutex<Agent>>,
}

#[derive(Deserialize)]
struct ChatRequest {
    message: String,
}

#[derive(Serialize)]
struct ChatResponse {
    reply: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    model: String,
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        model: state.config.model.clone(),
    })
}

async fn chat(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, String)> {
    let mut agent = state.agent.lock().await;
    match agent.chat(&req.message).await {
        Ok(reply) => Ok(Json(ChatResponse { reply })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn run(config: Config, host: String, port: u16) -> anyhow::Result<()> {
    let agent = Agent::new(config.clone());
    let state = AppState {
        config: config.clone(),
        agent: Arc::new(Mutex::new(agent)),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/chat", post(chat))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    tracing::info!(%addr, "Natalie server listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
