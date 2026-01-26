use axum::{
    extract::State,
    http::StatusCode,
    response::{sse::Event, IntoResponse, Sse},
    routing::{get, post},
    Json, Router,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    scan_control: Arc<RwLock<ScanControl>>,
    log_tx: Arc<RwLock<Vec<mpsc::UnboundedSender<String>>>>,
}

#[derive(Clone)]
struct ScanControl {
    status: ScanStatus,
    config: Option<ScanConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum ScanStatus {
    Idle,
    Running,
    Paused,
    Stopped,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ScanConfig {
    addresses: Vec<String>,
    chain: String,
    days: u64,
    concurrency: usize,
    no_cache: bool,
    tags: Option<String>,
    contract_type: Option<String>,
    sort_by_exploitability: bool,
    update_templates: Option<i64>,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            addresses: vec![],
            chain: "ethereum".to_string(),
            days: 100,
            concurrency: 3,
            no_cache: false,
            tags: None,
            contract_type: None,
            sort_by_exploitability: false,
            update_templates: None,
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState {
        scan_control: Arc::new(RwLock::new(ScanControl {
            status: ScanStatus::Idle,
            config: None,
        })),
        log_tx: Arc::new(RwLock::new(Vec::new())),
    };

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/status", get(get_status))
        .route("/api/start", post(start_scan))
        .route("/api/pause", post(pause_scan))
        .route("/api/stop", post(stop_scan))
        .route("/api/logs", get(stream_logs))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    
    tracing::info!("Server listening on http://127.0.0.1:8080");
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "message": "Server is running"
    }))
}

async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    let control = state.scan_control.read().await;
    Json(serde_json::json!({
        "status": control.status,
        "config": control.config
    }))
}

async fn start_scan(
    State(state): State<AppState>,
    Json(config): Json<ScanConfig>,
) -> impl IntoResponse {
    let mut control = state.scan_control.write().await;
    
    if control.status == ScanStatus::Running {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Scan already running"})),
        );
    }

    control.status = ScanStatus::Running;
    control.config = Some(config.clone());
    drop(control);

    let state_clone = state.clone();
    tokio::spawn(async move {
        run_scan(state_clone, config).await;
    });

    (
        StatusCode::OK,
        Json(serde_json::json!({"message": "Scan started"})),
    )
}

async fn pause_scan(State(state): State<AppState>) -> impl IntoResponse {
    let mut control = state.scan_control.write().await;
    
    if control.status != ScanStatus::Running {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "No scan running"})),
        );
    }

    control.status = ScanStatus::Paused;
    (
        StatusCode::OK,
        Json(serde_json::json!({"message": "Scan paused"})),
    )
}

async fn stop_scan(State(state): State<AppState>) -> impl IntoResponse {
    let mut control = state.scan_control.write().await;
    
    if control.status == ScanStatus::Idle {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "No scan to stop"})),
        );
    }

    control.status = ScanStatus::Stopped;
    (
        StatusCode::OK,
        Json(serde_json::json!({"message": "Scan stopped"})),
    )
}

async fn stream_logs(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    
    {
        let mut log_tx = state.log_tx.write().await;
        log_tx.push(tx);
    }

    let stream = async_stream::stream! {
        yield Ok(Event::default().data("Connected"));
        
        while let Some(log) = rx.recv().await {
            yield Ok(Event::default().data(log));
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("keep-alive")
    )
}

async fn run_scan(state: AppState, config: ScanConfig) {
    send_log(&state, "🔍 Starting scan...").await;
    send_log(&state, &format!("Chain: {}", config.chain)).await;
    send_log(&state, &format!("Days: {}", config.days)).await;
    send_log(&state, &format!("Concurrency: {}", config.concurrency)).await;

    // Find the project root by looking for the templates directory
    let project_root = find_project_root().unwrap_or_else(|| std::env::current_dir().unwrap());
    
    let mut cmd = tokio::process::Command::new("cargo");
    cmd.current_dir(&project_root);
    cmd.arg("run")
        .arg("--release")
        .arg("-p")
        .arg("scpf-cli")
        .arg("--bin")
        .arg("scpf")
        .arg("--")
        .arg("scan")
        .arg("--chains")
        .arg(&config.chain)
        .arg("--days")
        .arg(config.days.to_string())
        .arg("--concurrency")
        .arg(config.concurrency.to_string())
        .arg("--min-severity")
        .arg("high");

    if config.no_cache {
        cmd.arg("--no-cache");
    }

    if config.sort_by_exploitability {
        cmd.arg("--sort-by-exploitability");
    }

    if let Some(tags) = &config.tags {
        cmd.arg("--tags").arg(tags);
    }

    if let Some(contract_type) = &config.contract_type {
        cmd.arg("--contract-type").arg(contract_type);
    }

    if let Some(update_days) = config.update_templates {
        cmd.arg("--update-templates").arg(update_days.to_string());
    }

    for addr in &config.addresses {
        cmd.arg(addr);
    }

    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    match cmd.spawn() {
        Ok(mut child) => {
            let stdout = child.stdout.take();
            let stderr = child.stderr.take();

            let state_clone = state.clone();
            if let Some(stdout) = stdout {
                tokio::spawn(async move {
                    use tokio::io::{AsyncBufReadExt, BufReader};
                    let reader = BufReader::new(stdout);
                    let mut lines = reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        send_log(&state_clone, &line).await;
                    }
                });
            }

            let state_clone = state.clone();
            if let Some(stderr) = stderr {
                tokio::spawn(async move {
                    use tokio::io::{AsyncBufReadExt, BufReader};
                    let reader = BufReader::new(stderr);
                    let mut lines = reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        send_log(&state_clone, &format!("⚠️ {}", line)).await;
                    }
                });
            }

            match child.wait().await {
                Ok(status) => {
                    if status.success() {
                        send_log(&state, "✅ Scan completed successfully").await;
                    } else {
                        send_log(&state, &format!("❌ Scan failed with status: {}", status)).await;
                    }
                }
                Err(e) => {
                    send_log(&state, &format!("❌ Error waiting for scan: {}", e)).await;
                }
            }
        }
        Err(e) => {
            send_log(&state, &format!("❌ Failed to start scan: {}", e)).await;
        }
    }

    let mut control = state.scan_control.write().await;
    control.status = ScanStatus::Idle;
}

async fn send_log(state: &AppState, message: &str) {
    let mut log_tx = state.log_tx.write().await;
    log_tx.retain(|tx| tx.send(message.to_string()).is_ok());
}

/// Find the project root by looking for the templates directory
fn find_project_root() -> Option<std::path::PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    
    // Try up to 5 levels up
    for _ in 0..5 {
        if current.join("templates").is_dir() && current.join("Cargo.toml").is_file() {
            return Some(current);
        }
        current = current.parent()?.to_path_buf();
    }
    
    None
}
