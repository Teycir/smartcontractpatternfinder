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
}

// Use atomics for progress to avoid lock contention
struct ScanProgress {
    contracts_scanned: AtomicU32,
    contracts_total: AtomicU32,
    vulnerabilities_found: AtomicU32,
    current_contract: RwLock<Option<String>>,
}

impl Default for ScanProgress {
    fn default() -> Self {
        Self {
            contracts_scanned: AtomicU32::new(0),
            contracts_total: AtomicU32::new(0),
            vulnerabilities_found: AtomicU32::new(0),
            current_contract: RwLock::new(None),
        }
    }
}

impl ScanProgress {
    fn reset(&self) {
        self.contracts_scanned.store(0, Ordering::SeqCst);
        self.contracts_total.store(0, Ordering::SeqCst);
        self.vulnerabilities_found.store(0, Ordering::SeqCst);
        if let Ok(mut current) = self.current_contract.try_write() {
            *current = None;
        }
    }
    
    fn to_json(&self) -> serde_json::Value {
        let total = self.contracts_total.load(Ordering::SeqCst);
        let current = self.current_contract.try_read().ok().and_then(|c| c.clone());
        serde_json::json!({
            "contracts_scanned": self.contracts_scanned.load(Ordering::SeqCst),
            "contracts_total": if total > 0 { Some(total) } else { None },
            "current_contract": current,
            "vulnerabilities_found": self.vulnerabilities_found.load(Ordering::SeqCst)
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
    days: u64,
    concurrency: usize,
    no_cache: bool,
    tags: Option<String>,
    contract_type: Option<String>,
    sort_by_exploitability: bool,
    update_templates: Option<i64>,
    extract_sources: Option<usize>,
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
            extract_sources: None,
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
    };

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/status", get(get_status))
        .route("/api/start", post(start_scan))
        .route("/api/pause", post(pause_scan))
        .route("/api/resume", post(resume_scan))
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
        if !tags.is_empty() {
            cmd.arg("--tags").arg(tags);
        }
    }

    if let Some(contract_type) = &config.contract_type {
        if !contract_type.is_empty() {
            cmd.arg("--contract-type").arg(contract_type);
        }
    }

    if let Some(update_days) = config.update_templates {
        if update_days > 0 {
            cmd.arg("--update-templates").arg(update_days.to_string());
        }
    }

    if let Some(count) = config.extract_sources {
        cmd.arg("--extract-sources").arg(count.to_string());
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
                    tracing::debug!("Progress updated: {}/{}", current, total);
                    
                    // Extract contract address if present (0x followed by hex)
                    if let Some(addr_start) = line.find("0x") {
                        let addr_end = line[addr_start..]
                            .find(|c: char| !c.is_ascii_hexdigit() && c != 'x')
                            .map(|i| addr_start + i)
                            .unwrap_or(line.len());
                        let addr = &line[addr_start..addr_end];
                        if addr.len() >= 10 {
                            if let Ok(mut current_contract) = state.progress.current_contract.try_write() {
                                *current_contract = Some(addr.to_string());
                            }
                        }
                    }
                    return;
                }
            }
        }
    }
    
    // Pattern: "Scanning N contracts" - definitive total count at start
    if line.contains("Scanning") && line.contains("contracts") {
        // Extract number between "Scanning" and "contracts"
        if let Some(scanning_pos) = line.find("Scanning") {
            let after_scanning = &line[scanning_pos + 8..]; // "Scanning" is 8 chars
            let trimmed = after_scanning.trim_start();
            if let Some(space_pos) = trimmed.find(|c: char| !c.is_ascii_digit()) {
                let num_str = &trimmed[..space_pos];
                if let Ok(total) = num_str.parse::<u32>() {
                    state.progress.contracts_total.store(total, Ordering::SeqCst);
                    tracing::debug!("Total contracts set: {}", total);
                    return;
                }
            }
        }
    }
    
    // Pattern: "N findings" - extract findings count
    if line.contains("findings") || line.contains("finding") {
        // Look for pattern like "4 findings" or "1 finding"
        let words: Vec<&str> = line.split_whitespace().collect();
        for (i, word) in words.iter().enumerate() {
            if (*word == "findings" || *word == "finding") && i > 0 {
                if let Ok(n) = words[i - 1].parse::<u32>() {
                    if n > 0 {
                        let new_total = state.progress.vulnerabilities_found.fetch_add(n, Ordering::SeqCst) + n;
                        tracing::debug!("Found {} findings, total now: {}", n, new_total);
                    }
                    return;
                }
            }
        }
    }
    
    // Pattern: "Exploitable: X contracts with Y findings" - final summary
    if let Some(caps) = regex_lite::Regex::new(r"Exploitable:\s*\d+\s+contracts?\s+with\s+(\d+)\s+findings?")
        .ok()
        .and_then(|re| re.captures(line))
    {
        if let Some(findings) = caps.get(1) {
            if let Ok(n) = findings.as_str().parse::<u32>() {
                state.progress.vulnerabilities_found.store(n, Ordering::SeqCst);
            }
        }
        return;
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
