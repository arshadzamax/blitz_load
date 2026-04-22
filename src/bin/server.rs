use axum::{
    extract::{Path, State},
    http::{Method, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use blitz_load::engine::{BlitzConfig, HttpMethod, ProgressEvent, ScenarioKind};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tower_http::{cors::CorsLayer, services::ServeDir};
use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────────────────
// State
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct AppState {
    tests: Arc<DashMap<Uuid, TestEntry>>,
}

#[derive(Clone)]
struct TestEntry {
    tx: broadcast::Sender<ProgressEvent>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Request / Response shapes
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RunRequest {
    url: String,
    #[serde(default = "default_method")]
    method: String,
    #[serde(default = "default_requests")]
    requests: usize,
    #[serde(default = "default_concurrency")]
    concurrency: usize,
    #[serde(default = "default_scenario")]
    scenario: String,
    custom_body: Option<String>,
    #[serde(default = "default_timeout")]
    timeout_ms: u64,
    sla_ms: Option<u64>,
}

fn default_method()      -> String { "post".into() }
fn default_requests()    -> usize  { 100 }
fn default_concurrency() -> usize  { 10 }
fn default_scenario()    -> String { "user_registration".into() }
fn default_timeout()     -> u64    { 30000 }

#[derive(Serialize)]
struct RunResponse {
    test_id: Uuid,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Handlers
// ─────────────────────────────────────────────────────────────────────────────

async fn run_handler(
    State(state): State<AppState>,
    Json(body): Json<RunRequest>,
) -> impl IntoResponse {
    // basic URL validation
    if !body.url.starts_with("http://") && !body.url.starts_with("https://") {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "URL must start with http:// or https://" })),
        )
            .into_response();
    }

    let config = BlitzConfig {
        url: body.url,
        requests: body.requests,
        concurrency: body.concurrency,
        timeout_ms: body.timeout_ms,
        sla_ms: body.sla_ms,
        method: if body.method.to_lowercase() == "get" {
            HttpMethod::Get
        } else {
            HttpMethod::Post
        },
        scenario: match body.scenario.as_str() {
            "simple_get"  => ScenarioKind::SimpleGet,
            "custom_json" => ScenarioKind::CustomJson {
                body: body.custom_body.unwrap_or_default(),
            },
            _ => ScenarioKind::UserRegistration,
        },
    };

    // broadcast channel: capacity 512 so late subscribers still get events
    let (tx, _rx) = broadcast::channel::<ProgressEvent>(512);
    let test_id = Uuid::new_v4();

    state.tests.insert(test_id, TestEntry { tx: tx.clone() });

    // clone state ref to clean up after the test finishes
    let state_cleanup = state.clone();
    let id_cleanup    = test_id;

    // spawn the engine – it runs entirely independently
    tokio::spawn(async move {
        blitz_load::engine::run_blitz(config, tx).await;
        // keep entry alive for 5 min so late UI reconnects can read Done event
        tokio::time::sleep(Duration::from_secs(300)).await;
        state_cleanup.tests.remove(&id_cleanup);
    });

    (StatusCode::OK, Json(serde_json::json!({ "test_id": test_id }))).into_response()
}

async fn stream_handler(
    Path(test_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let entry = match state.tests.get(&test_id) {
        Some(e) => e.clone(),
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Test not found" })),
            )
                .into_response()
        }
    };

    // Every SSE connection gets its own fresh receiver – reconnect-safe
    let rx = entry.tx.subscribe();

    let stream = BroadcastStream::new(rx)
        .map(|result| -> Result<Event, Infallible> {
            let data = match result {
                Ok(event) => serde_json::to_string(&event).unwrap_or_default(),
                Err(_)    => r#"{"type":"error","message":"stream lag"}"#.to_owned(),
            };
            Ok(Event::default().data(data))
        });

    Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response()
}

async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

// ─────────────────────────────────────────────────────────────────────────────
// Main
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);

    let state = AppState {
        tests: Arc::new(DashMap::new()),
    };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(tower_http::cors::Any)
        .allow_origin(tower_http::cors::Any);

    let api_router = Router::new()
        .route("/run",        post(run_handler))
        .route("/stream/:id", get(stream_handler))
        .route("/health",     get(health_handler));

    let app = Router::new()
        .nest("/api", api_router)
        // serve the compiled frontend from frontend/dist
        .nest_service("/", ServeDir::new("frontend/dist"))
        .with_state(state)
        .layer(cors);

    let addr = format!("0.0.0.0:{port}");
    println!("🚀 blitz-server listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
