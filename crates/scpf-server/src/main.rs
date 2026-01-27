use axum::{
    extract::State,
    http::StatusCode,
    response::{sse::Event, IntoResponse, Sse},
    routing::{get, post},
    Json, Router,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    scan_status: Arc<RwLock<ScanStatus>>,
    scan_config: Arc<RwLock<Option<ScanConfig>>>,
    child_pid: Arc<Mutex<Option<u32>>>,
    progress: Arc<ScanProgress>,
    log_tx: Arc<RwLock<Vec<mpsc::UnboundedSender<String>>>>,
    results: Arc<RwLock<Vec<String>>>,
    report_path: Arc<RwLock<Option<std::path::PathBuf>>>,
}

// Use atomics for progress to avoid lock contention
struct ScanProgress {
    contracts_scanned: AtomicU32,
    contracts_total: AtomicU32,
    contracts_extracted: AtomicU32,
    current_contract: RwLock<Option<String>>,
    current_contract_name: RwLock<Option<String>>,
    eta_seconds: AtomicU32,
    rate: RwLock<Option<f64>>,
    critical_findings: AtomicU32,
}

impl Default for ScanProgress {
    fn default() -> Self {
        Self {
            contracts_scanned: AtomicU32::new(0),
            contracts_total: AtomicU32::new(0),
            contracts_extracted: AtomicU32::new(0),
            current_contract: RwLock::new(None),
            current_contract_name: RwLock::new(None),
            eta_seconds: AtomicU32::new(0),
            rate: RwLock::new(None),
            critical_findings: AtomicU32::new(0),
        }
    }
}

impl ScanProgress {
    fn reset(&self) {
        self.contracts_scanned.store(0, Ordering::SeqCst);
        self.contracts_total.store(0, Ordering::SeqCst);
        self.contracts_extracted.store(0, Ordering::SeqCst);
        self.eta_seconds.store(0, Ordering::SeqCst);
        self.critical_findings.store(0, Ordering::SeqCst);
        if let Ok(mut current) = self.current_contract.try_write() {
            *current = None;
        }
        if let Ok(mut name) = self.current_contract_name.try_write() {
            *name = None;
        }
        if let Ok(mut rate) = self.rate.try_write() {
            *rate = None;
        }
    }
    
    fn to_json(&self) -> serde_json::Value {
        let total = self.contracts_total.load(Ordering::SeqCst);
        let current = self.current_contract.try_read().ok().and_then(|c| c.clone());
        let current_name = self.current_contract_name.try_read().ok().and_then(|c| c.clone());
        let rate = self.rate.try_read().ok().and_then(|r| *r);
        serde_json::json!({
            "contracts_scanned": self.contracts_scanned.load(Ordering::SeqCst),
            "contracts_total": if total > 0 { Some(total) } else { None },
            "current_contract": current,
            "current_contract_name": current_name,
            "contracts_extracted": self.contracts_extracted.load(Ordering::SeqCst),
            "eta_seconds": self.eta_seconds.load(Ordering::SeqCst),
            "rate": rate,
            "critical_findings": self.critical_findings.load(Ordering::SeqCst)
        })
    }
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
    pages: u64,
    concurrency: usize,
    tags: Option<String>,
    contract_type: Option<String>,
    extract_sources: Option<usize>,
    fetch_zero_day: Option<u32>,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            addresses: vec![],
            chain: "ethereum,polygon,arbitrum".to_string(),
            pages: 5,
            concurrency: 2,
            tags: None,
            contract_type: None,
            extract_sources: None,
            fetch_zero_day: None,
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState {
        scan_status: Arc::new(RwLock::new(ScanStatus::Idle)),
        scan_config: Arc::new(RwLock::new(None)),
        child_pid: Arc::new(Mutex::new(None)),
        progress: Arc::new(ScanProgress::default()),
        log_tx: Arc::new(RwLock::new(Vec::new())),
        results: Arc::new(RwLock::new(Vec::new())),
        report_path: Arc::new(RwLock::new(None)),
    };

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/status", get(get_status))
        .route("/api/start", post(start_scan))
        .route("/api/pause", post(pause_scan))
        .route("/api/resume", post(resume_scan))
        .route("/api/stop", post(stop_scan))
        .route("/api/logs", get(stream_logs))
        .route("/api/results", get(get_results))
        .route("/api/export", get(export_results))
        .route("/api/export-logs", post(export_logs))
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
    let status = state.scan_status.read().await.clone();
    let config = state.scan_config.read().await.clone();
    Json(serde_json::json!({
        "status": status,
        "config": config,
        "progress": state.progress.to_json()
    }))
}

async fn start_scan(
    State(state): State<AppState>,
    Json(config): Json<ScanConfig>,
) -> impl IntoResponse {
    {
        let status = state.scan_status.read().await;
        if *status == ScanStatus::Running || *status == ScanStatus::Paused {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Scan already running or paused"})),
            );
        }
    }

    // Reset progress for new scan
    state.progress.reset();
    
    // Clear previous results
    {
        let mut results = state.results.write().await;
        results.clear();
    }
    {
        let mut report_path = state.report_path.write().await;
        *report_path = None;
    }
    
    {
        let mut status = state.scan_status.write().await;
        *status = ScanStatus::Running;
    }
    
    {
        let mut cfg = state.scan_config.write().await;
        *cfg = Some(config.clone());
    }

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
    {
        let status = state.scan_status.read().await;
        if *status != ScanStatus::Running {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "No scan running to pause"})),
            );
        }
    }

    let pid = state.child_pid.lock().await.clone();
    
    if let Some(pid) = pid {
        #[cfg(unix)]
        {
            // Kill entire process group (negative PID)
            let result = unsafe { libc::kill(-(pid as i32), libc::SIGSTOP) };
            if result != 0 {
                // Fallback: try killing just the process
                let result = unsafe { libc::kill(pid as i32, libc::SIGSTOP) };
                if result != 0 {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": "Failed to pause process"})),
                    );
                }
            }
        }
        
        {
            let mut status = state.scan_status.write().await;
            *status = ScanStatus::Paused;
        }
        
        send_log(&state, "⏸️ Scan paused").await;
        
        (
            StatusCode::OK,
            Json(serde_json::json!({"message": "Scan paused"})),
        )
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "No process to pause"})),
        )
    }
}

async fn resume_scan(State(state): State<AppState>) -> impl IntoResponse {
    {
        let status = state.scan_status.read().await;
        if *status != ScanStatus::Paused {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "No paused scan to resume"})),
            );
        }
    }

    let pid = state.child_pid.lock().await.clone();
    
    if let Some(pid) = pid {
        #[cfg(unix)]
        {
            // Resume entire process group
            let result = unsafe { libc::kill(-(pid as i32), libc::SIGCONT) };
            if result != 0 {
                // Fallback: try just the process
                unsafe { libc::kill(pid as i32, libc::SIGCONT) };
            }
        }
        
        {
            let mut status = state.scan_status.write().await;
            *status = ScanStatus::Running;
        }
        
        send_log(&state, "▶️ Scan resumed").await;
        
        (
            StatusCode::OK,
            Json(serde_json::json!({"message": "Scan resumed"})),
        )
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "No process to resume"})),
        )
    }
}

async fn stop_scan(State(state): State<AppState>) -> impl IntoResponse {
    {
        let status = state.scan_status.read().await;
        if *status == ScanStatus::Idle {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "No scan to stop"})),
            );
        }
    }

    let pid = state.child_pid.lock().await.take();
    
    if let Some(pid) = pid {
        #[cfg(unix)]
        {
            // First resume if paused, then terminate entire process group
            unsafe { 
                libc::kill(-(pid as i32), libc::SIGCONT);
                libc::kill(-(pid as i32), libc::SIGTERM);
            }
            
            // Give it a moment, then force kill if needed
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            unsafe {
                libc::kill(-(pid as i32), libc::SIGKILL);
            }
        }
    }

    {
        let mut status = state.scan_status.write().await;
        *status = ScanStatus::Idle;
    }
    
    {
        let mut cfg = state.scan_config.write().await;
        *cfg = None;
    }
    
    send_log(&state, "🛑 Scan stopped by user").await;
    
    (
        StatusCode::OK,
        Json(serde_json::json!({"message": "Scan stopped"})),
    )
}

async fn get_results(State(state): State<AppState>) -> impl IntoResponse {
    let results = state.results.read().await;
    Json(serde_json::json!({
        "count": results.len(),
        "findings": results.clone()
    }))
}

async fn export_results(State(state): State<AppState>) -> impl IntoResponse {
    let report_path = state.report_path.read().await;
    
    if let Some(path) = report_path.as_ref() {
        if path.exists() {
            match tokio::fs::read_to_string(path).await {
                Ok(content) => {
                    return (
                        StatusCode::OK,
                        [("Content-Type", "text/markdown")],
                        content
                    ).into_response();
                }
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": format!("Failed to read report: {}", e)}))
                    ).into_response();
                }
            }
        }
    }
    
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"error": "No report available"}))
    ).into_response()
}

#[derive(Deserialize)]
struct ExportLogsRequest {
    logs: String,
}

async fn export_logs(
    State(state): State<AppState>,
    Json(payload): Json<ExportLogsRequest>,
) -> impl IntoResponse {
    let report_path = state.report_path.read().await;
    
    let log_path = if let Some(path) = report_path.as_ref() {
        path.parent().map(|p| p.join("console.log"))
    } else {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let default_dir = std::path::PathBuf::from(format!(
            "/home/teycir/smartcontractpatternfinderReports/report_{}",
            timestamp
        ));
        Some(default_dir.join("console.log"))
    };
    
    if let Some(path) = log_path {
        if let Some(parent) = path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("Failed to create directory: {}", e)}))
                ).into_response();
            }
        }
        
        match tokio::fs::write(&path, payload.logs).await {
            Ok(_) => {
                (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "message": "Logs exported successfully",
                        "path": path.display().to_string()
                    }))
                ).into_response()
            }
            Err(e) => {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("Failed to write logs: {}", e)}))
                ).into_response()
            }
        }
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Could not determine export path"}))
        ).into_response()
    }
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
    send_log(&state, &format!("Pages: {}", config.pages)).await;
    send_log(&state, &format!("Concurrency: {}", config.concurrency)).await;

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
        .arg("--pages")
        .arg(config.pages.to_string())
        .arg("--concurrency")
        .arg(config.concurrency.to_string())
        .arg("--min-severity")
        .arg("high");

    if let Some(tags) = &config.tags {
        if !tags.is_empty() {
            cmd.arg("--tags").arg(tags);
        }
    }

    if let Some(contract_type) = &config.contract_type {
        if !contract_type.is_empty() {
            cmd.arg("--contract-type").arg(contract_type);
        }
    }

    if let Some(count) = config.extract_sources {
        cmd.arg("--extract-sources").arg(count.to_string());
    }

    if let Some(days) = config.fetch_zero_day {
        cmd.arg("--fetch-zero-day").arg(days.to_string());
    }

    for addr in &config.addresses {
        if !addr.is_empty() {
            cmd.arg(addr);
        }
    }

    // Create new process group for better signal handling
    #[cfg(unix)]
    {
        #[allow(unused_imports)]
        use std::os::unix::process::CommandExt;
        // SAFETY: setsid() is safe to call in pre_exec
        unsafe {
            cmd.pre_exec(|| {
                // Create new session and process group
                libc::setsid();
                Ok(())
            });
        }
    }

    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    match cmd.spawn() {
        Ok(mut child) => {
            // Store the child PID for pause/resume/stop
            if let Some(pid) = child.id() {
                {
                    let mut child_pid = state.child_pid.lock().await;
                    *child_pid = Some(pid);
                }
                send_log(&state, &format!("📋 Process started with PID: {}", pid)).await;
            }

            let stdout = child.stdout.take();
            let stderr = child.stderr.take();

            // Spawn stdout reader
            let state_stdout = state.clone();
            let stdout_handle = if let Some(stdout) = stdout {
                Some(tokio::spawn(async move {
                    use tokio::io::{AsyncBufReadExt, BufReader};
                    let reader = BufReader::new(stdout);
                    let mut lines = reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        parse_and_update_progress(&state_stdout, &line);
                        send_log(&state_stdout, &line).await;
                    }
                }))
            } else {
                None
            };

            // Spawn stderr reader
            let state_stderr = state.clone();
            let stderr_handle = if let Some(stderr) = stderr {
                Some(tokio::spawn(async move {
                    use tokio::io::{AsyncBufReadExt, BufReader};
                    let reader = BufReader::new(stderr);
                    let mut lines = reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        // Debug: log raw line for troubleshooting
                        tracing::debug!("STDERR raw: {}", line);
                        parse_and_update_progress(&state_stderr, &line);
                        send_log(&state_stderr, &format!("⚠️ {}", line)).await;
                    }
                }))
            } else {
                None
            };

            // Wait for process to complete
            match child.wait().await {
                Ok(status) => {
                    let current_status = state.scan_status.read().await.clone();
                    
                    if current_status == ScanStatus::Idle {
                        // Was stopped by user - message already sent
                    } else if status.success() {
                        send_log(&state, "✅ Scan completed successfully").await;
                    } else {
                        send_log(&state, &format!("❌ Scan exited with status: {}", status)).await;
                    }
                }
                Err(e) => {
                    send_log(&state, &format!("❌ Error waiting for scan: {}", e)).await;
                }
            }

            // Wait for readers to finish
            if let Some(h) = stdout_handle {
                let _ = h.await;
            }
            if let Some(h) = stderr_handle {
                let _ = h.await;
            }
        }
        Err(e) => {
            send_log(&state, &format!("❌ Failed to start scan: {}", e)).await;
        }
    }

    // Clean up
    {
        let mut status = state.scan_status.write().await;
        if *status != ScanStatus::Idle {
            *status = ScanStatus::Idle;
        }
    }
    {
        let mut child_pid = state.child_pid.lock().await;
        *child_pid = None;
    }
}

/// Parse log lines to extract progress information (non-async, uses atomics)
fn parse_and_update_progress(state: &AppState, line: &str) {
    // Store findings incrementally
    if line.contains("findings") || line.contains("Exploitable") || line.contains("✓") {
        if let Ok(mut results) = state.results.try_write() {
            results.push(line.to_string());
        }
    }
    
    // Capture report path
    if line.contains("Vulnerability summary:") || line.contains("vuln_summary.md") {
        if let Some(path_start) = line.rfind('/') {
            if let Some(path_end) = line[path_start..].find(char::is_whitespace) {
                let path_str = &line[..path_start + path_end];
                if let Ok(mut report_path) = state.report_path.try_write() {
                    *report_path = Some(std::path::PathBuf::from(path_str));
                }
            } else {
                let path_str = line[..].trim();
                if let Ok(mut report_path) = state.report_path.try_write() {
                    *report_path = Some(std::path::PathBuf::from(path_str));
                }
            }
        }
    }
    // Pattern: "[851/10553]" - extract progress from contract scanning lines
    // Using simple string parsing for reliability
    if let Some(bracket_start) = line.find('[') {
        if let Some(bracket_end) = line[bracket_start..].find(']') {
            let bracket_content = &line[bracket_start + 1..bracket_start + bracket_end];
            if let Some(slash_pos) = bracket_content.find('/') {
                let current_str = &bracket_content[..slash_pos];
                let total_str = &bracket_content[slash_pos + 1..];
                
                // Only parse if both parts are numeric
                if let (Ok(current), Ok(total)) = (current_str.parse::<u32>(), total_str.parse::<u32>()) {
                    state.progress.contracts_scanned.store(current, Ordering::SeqCst);
                    state.progress.contracts_total.store(total, Ordering::SeqCst);
                    
                    // Parse enhanced metrics: "[50/200] 25.0% | ETA: 5m30s | Critical: 42 | 2.5/s"
                    if let Some(eta_pos) = line.find("ETA:") {
                        let after_eta = &line[eta_pos + 4..].trim_start();
                        // Parse "5m30s" format
                        if let Some(m_pos) = after_eta.find('m') {
                            if let Ok(mins) = after_eta[..m_pos].parse::<u32>() {
                                let after_m = &after_eta[m_pos + 1..];
                                if let Some(s_pos) = after_m.find('s') {
                                    if let Ok(secs) = after_m[..s_pos].parse::<u32>() {
                                        state.progress.eta_seconds.store(mins * 60 + secs, Ordering::SeqCst);
                                    }
                                }
                            }
                        }
                    }
                    
                    // Parse critical findings: "Critical: 42"
                    if let Some(critical_pos) = line.find("Critical:") {
                        let after_critical = &line[critical_pos + 9..].trim_start();
                        if let Some(space_pos) = after_critical.find(|c: char| !c.is_ascii_digit()) {
                            if let Ok(count) = after_critical[..space_pos].parse::<u32>() {
                                state.progress.critical_findings.store(count, Ordering::SeqCst);
                            }
                        }
                    }
                    
                    // Parse rate: "2.5/s"
                    if let Some(rate_match) = line.rfind(|c: char| c.is_ascii_digit() || c == '.') {
                        let before_rate = &line[..=rate_match];
                        if let Some(space_pos) = before_rate.rfind(|c: char| c.is_whitespace()) {
                            let rate_str = &before_rate[space_pos + 1..];
                            if let Ok(rate_val) = rate_str.parse::<f64>() {
                                if let Ok(mut rate) = state.progress.rate.try_write() {
                                    *rate = Some(rate_val);
                                }
                            }
                        }
                    }
                    
                    tracing::debug!("Progress updated: {}/{}", current, total);
                    return;
                }
            }
        }
    }
    
    // Pattern: "Scanning N contracts" - definitive total count at start
    // Only update if new value is higher (to handle multiple scanning phases)
    if line.contains("Scanning") && line.contains("contracts") {
        // Extract number between "Scanning" and "contracts"
        if let Some(scanning_pos) = line.find("Scanning") {
            let after_scanning = &line[scanning_pos + 8..]; // "Scanning" is 8 chars
            let trimmed = after_scanning.trim_start();
            if let Some(space_pos) = trimmed.find(|c: char| !c.is_ascii_digit()) {
                let num_str = &trimmed[..space_pos];
                if let Ok(total) = num_str.parse::<u32>() {
                    let current_total = state.progress.contracts_total.load(Ordering::SeqCst);
                    // Only update if this is a higher value (prevents overwriting during extraction phase)
                    if total > current_total {
                        state.progress.contracts_total.store(total, Ordering::SeqCst);
                        tracing::debug!("Total contracts set: {}", total);
                    } else {
                        tracing::debug!("Ignoring lower total {} (current: {})", total, current_total);
                    }
                    return;
                }
            }
        }
    }

    // Pattern: "Extracted N contract sources" - track extraction count
    // This should NOT affect contracts_scanned or contracts_total
    if line.contains("Extracted") && line.contains("contract sources") {
        if let Some(extracted_pos) = line.find("Extracted") {
            let after_extracted = &line[extracted_pos + 9..].trim_start();
            if let Some(space_pos) = after_extracted.find(|c: char| !c.is_ascii_digit()) {
                let num_str = &after_extracted[..space_pos];
                if let Ok(count) = num_str.parse::<u32>() {
                    state.progress.contracts_extracted.store(count, Ordering::SeqCst);
                    tracing::debug!("Extracted contracts: {} (scanned count unchanged)", count);
                    return;
                }
            }
        }
    }
    
    // Pattern: "✅ Scanning complete" - mark as done
    if line.contains("Scanning complete") {
        let total = state.progress.contracts_total.load(Ordering::SeqCst);
        if total > 0 {
            state.progress.contracts_scanned.store(total, Ordering::SeqCst);
        }
    }
}

async fn send_log(state: &AppState, message: &str) {
    let log_tx = state.log_tx.read().await;
    for tx in log_tx.iter() {
        let _ = tx.send(message.to_string());
    }
}

fn find_project_root() -> Option<std::path::PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    
    for _ in 0..5 {
        if current.join("templates").is_dir() && current.join("Cargo.toml").is_file() {
            return Some(current);
        }
        current = current.parent()?.to_path_buf();
    }
    
    None
}
