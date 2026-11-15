//! HTTP API server (axum).

use crate::agent::Agent;
use crate::config::Config;
use crate::hardware::HardwareInfo;
use crate::localai::deploy;
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
    local_deployment: Arc<Mutex<Option<crate::LocalDeployment>>>,
    local_server: Arc<Mutex<Option<crate::LocalServer>>>,
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

#[derive(Serialize)]
struct HardwareResponse {
    cpu_cores: usize,
    total_ram_mb: u64,
    gpu_name: Option<String>,
    gpu_vram_mb: Option<u64>,
    model_budget_mb: u64,
    recommended: Option<String>,
}

#[derive(Deserialize, Default)]
struct DeployRequest {
    #[serde(default)]
    model: Option<String>,
}

#[derive(Serialize)]
struct DeployResponse {
    status: String,
    model: Option<String>,
    endpoint: Option<String>,
    message: Option<String>,
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

async fn hardware() -> Json<HardwareResponse> {
    let hw = HardwareInfo::detect();
    let rec = crate::localai::catalog::recommend(&hw).map(|m| m.name.to_string());
    Json(HardwareResponse {
        cpu_cores: hw.cpu_cores,
        total_ram_mb: hw.total_ram_mb,
        gpu_name: hw.gpu.as_ref().map(|g| g.name.clone()),
        gpu_vram_mb: hw.gpu.as_ref().map(|g| g.vram_mb),
        model_budget_mb: hw.model_budget_mb(),
        recommended: rec,
    })
}

async fn deploy_local_ai(
    State(state): State<AppState>,
    Json(req): Json<DeployRequest>,
) -> Result<Json<DeployResponse>, (StatusCode, String)> {
    // Already deployed?
    {
        let dep = state.local_deployment.lock().await;
        if let Some(d) = dep.as_ref() {
            return Ok(Json(DeployResponse {
                status: "already_deployed".into(),
                model: Some(d.model.clone()),
                endpoint: Some(d.endpoint.clone()),
                message: None,
            }));
        }
    }

    match deploy::deploy(req.model.as_deref()).await {
        Ok((deployment, server)) => {
            let endpoint = deployment.endpoint.clone();
            let model = deployment.model.clone();
            *state.local_deployment.lock().await = Some(deployment);
            *state.local_server.lock().await = Some(server);
            Ok(Json(DeployResponse {
                status: "deployed".into(),
                model: Some(model),
                endpoint: Some(endpoint),
                message: Some("Set OPENAI_BASE_URL=<endpoint>/v1 to use it.".into()),
            }))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn run(config: Config, host: String, port: u16) -> anyhow::Result<()> {
    let agent = Agent::new(config.clone());
    let state = AppState {
        config: config.clone(),
        agent: Arc::new(Mutex::new(agent)),
        local_deployment: Arc::new(Mutex::new(None)),
        local_server: Arc::new(Mutex::new(None)),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/chat", post(chat))
        .route("/hardware", get(hardware))
        .route("/deploy-local-ai", post(deploy_local_ai))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    tracing::info!(%addr, "Natalie server listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
