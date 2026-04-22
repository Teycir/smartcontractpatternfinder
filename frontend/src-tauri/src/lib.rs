#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use scpf_config::{
    frontend_runtime_config, load_process_env, server_config, FrontendRuntimeConfig, ServerConfig,
    EXPLICIT_ENV_FILE_ENV_VAR, PROJECT_ROOT_ENV_VAR, RUNTIME_DIR_ENV_VAR, SERVER_ADDR_ENV_VAR,
};
use std::fs;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tauri::Manager;
use tokio::sync::Mutex;

#[derive(Clone)]
struct AppState {
    server_process: Arc<Mutex<Option<Child>>>,
    server_config: ServerConfig,
}

async fn start_embedded_server(
    state: Arc<Mutex<Option<Child>>>,
    server: ServerConfig,
    resource_dir: Option<PathBuf>,
    runtime_dir: Option<PathBuf>,
) -> Result<(), String> {
    let mut process_guard = state.lock().await;

    if process_guard.is_some() {
        return Ok(());
    }

    let mut command = if let Some(server_path) = find_server_binary(resource_dir.as_ref()) {
        let mut command = Command::new(server_path);
        command.env(SERVER_ADDR_ENV_VAR, &server.addr);

        if let Some(resource_dir) = resource_dir.as_ref() {
            if let Some(templates_dir) = find_templates_dir(resource_dir) {
                command.env("SCPF_TEMPLATES_DIR", templates_dir);
            }

            if let Some(cli_binary) = find_cli_binary(Some(resource_dir)) {
                command.env("SCPF_CLI_BINARY", cli_binary);
            }
        }

        command
    } else if cfg!(debug_assertions) {
        let workspace_root = find_workspace_root().ok_or_else(|| {
            "Server binary not found and workspace root could not be resolved".to_string()
        })?;

        let mut command = Command::new("cargo");
        command
            .current_dir(&workspace_root)
            .args(["run", "--bin", "scpf-server"])
            .env(SERVER_ADDR_ENV_VAR, &server.addr);
        command
    } else {
        return Err("Server binary not found".to_string());
    };

    if let Some(runtime_dir) = runtime_dir.as_ref() {
        fs::create_dir_all(runtime_dir)
            .map_err(|error| format!("Failed to create runtime directory: {}", error))?;
        fs::create_dir_all(runtime_dir.join("reports"))
            .map_err(|error| format!("Failed to create report directory: {}", error))?;
        seed_env_example(resource_dir.as_ref(), runtime_dir)
            .map_err(|error| format!("Failed to seed desktop env example: {}", error))?;

        command
            .current_dir(runtime_dir)
            .env(RUNTIME_DIR_ENV_VAR, runtime_dir)
            .env("SCPF_REPORTS_DIR", runtime_dir.join("reports"));
    }

    if let Some(workspace_root) = find_workspace_root() {
        command.env(PROJECT_ROOT_ENV_VAR, &workspace_root);

        let repo_env_file = workspace_root.join(".env");
        if repo_env_file.is_file() {
            command.env(EXPLICIT_ENV_FILE_ENV_VAR, repo_env_file);
        }
    }

    let child = command
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("Failed to start server: {}", error))?;

    *process_guard = Some(child);
    Ok(())
}

fn request_stop_scan(server: &ServerConfig) -> Result<(), String> {
    let addr = parse_server_socket_addr(server)?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_millis(250))
        .map_err(|error| format!("Failed to connect to server: {}", error))?;
    stream
        .set_read_timeout(Some(Duration::from_millis(750)))
        .map_err(|error| format!("Failed to set read timeout: {}", error))?;
    stream
        .set_write_timeout(Some(Duration::from_millis(250)))
        .map_err(|error| format!("Failed to set write timeout: {}", error))?;

    let request = format!(
        "POST /api/stop HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n",
        server.addr
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|error| format!("Failed to send stop request: {}", error))?;

    let mut buffer = [0u8; 512];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(_) => continue,
            Err(error)
                if error.kind() == io::ErrorKind::WouldBlock
                    || error.kind() == io::ErrorKind::TimedOut =>
            {
                break;
            }
            Err(error) => {
                return Err(format!("Failed to read stop response: {}", error));
            }
        }
    }

    Ok(())
}

fn parse_server_socket_addr(server: &ServerConfig) -> Result<SocketAddr, String> {
    server
        .addr
        .parse::<SocketAddr>()
        .map_err(|error| format!("Invalid SCPF server address '{}': {}", server.addr, error))
}

fn seed_env_example(resource_dir: Option<&PathBuf>, runtime_dir: &Path) -> io::Result<()> {
    let destination = runtime_dir.join(".env.example");
    if destination.exists() {
        return Ok(());
    }

    let Some(source) = find_env_example(resource_dir) else {
        return Ok(());
    };

    fs::copy(source, destination)?;
    Ok(())
}

fn find_env_example(resource_dir: Option<&PathBuf>) -> Option<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(resource_dir) = resource_dir {
        candidates.push(resource_dir.join(".env.example"));
        candidates.push(resource_dir.join("config").join(".env.example"));
        candidates.push(
            resource_dir
                .join("resources")
                .join("config")
                .join(".env.example"),
        );
    }

    if let Some(workspace_root) = find_workspace_root() {
        candidates.push(workspace_root.join(".env.example"));
    }

    candidates.into_iter().find(|path| path.is_file())
}

fn find_workspace_root() -> Option<PathBuf> {
    let mut candidate_roots = Vec::new();

    if let Ok(current_dir) = std::env::current_dir() {
        candidate_roots.push(current_dir);
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            candidate_roots.push(parent.to_path_buf());
        }
    }

    for mut candidate in candidate_roots {
        loop {
            let cargo_toml = candidate.join("Cargo.toml");
            if cargo_toml.is_file() {
                if let Ok(content) = fs::read_to_string(&cargo_toml) {
                    if content.contains("[workspace]") {
                        return Some(candidate);
                    }
                }
            }

            match candidate.parent() {
                Some(parent) => candidate = parent.to_path_buf(),
                None => break,
            }
        }
    }

    None
}

fn find_server_binary(resource_dir: Option<&PathBuf>) -> Option<PathBuf> {
    find_binary("scpf-server", resource_dir)
}

fn find_cli_binary(resource_dir: Option<&PathBuf>) -> Option<PathBuf> {
    find_binary("scpf", resource_dir)
}

fn find_templates_dir(resource_dir: &Path) -> Option<PathBuf> {
    let candidates = [
        resource_dir.join("templates"),
        resource_dir.join("resources").join("templates"),
    ];

    candidates.into_iter().find(|path| path.is_dir())
}

fn find_binary(binary_stem: &str, resource_dir: Option<&PathBuf>) -> Option<PathBuf> {
    let binary_name = if cfg!(windows) {
        format!("{}.exe", binary_stem)
    } else {
        binary_stem.to_string()
    };

    if let Some(resource_dir) = resource_dir {
        for candidate in bundled_binary_candidates(resource_dir, &binary_name) {
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    for candidate in bundled_binary_candidates(&exe_dir, &binary_name) {
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    let mut current = std::env::current_dir().ok()?;
    for _ in 0..5 {
        let release_bin = current.join("target/release").join(&binary_name);
        if release_bin.is_file() {
            return Some(release_bin);
        }

        let debug_bin = current.join("target/debug").join(&binary_name);
        if debug_bin.is_file() {
            return Some(debug_bin);
        }

        current = current.parent()?.to_path_buf();
    }

    None
}

fn bundled_binary_candidates(root: &Path, binary_name: &str) -> Vec<PathBuf> {
    let mut candidates = vec![
        root.join(binary_name),
        root.join("bin").join(binary_name),
        root.join("resources").join("bin").join(binary_name),
        root.join("release").join(binary_name),
        root.join("target/release").join(binary_name),
        root.join("target/debug").join(binary_name),
    ];

    if let Some(parent) = root.parent() {
        candidates.push(parent.join(binary_name));
    }

    if cfg!(target_os = "macos") {
        if let Some(parent) = root.parent() {
            candidates.push(parent.join("Resources").join(binary_name));
        }
    }

    candidates
}

#[tauri::command]
fn runtime_config() -> FrontendRuntimeConfig {
    frontend_runtime_config()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    load_process_env();
    let server = server_config();

    let app_state = AppState {
        server_process: Arc::new(Mutex::new(None)),
        server_config: server.clone(),
    };

    tauri::Builder::default()
        .manage(app_state.clone())
        .invoke_handler(tauri::generate_handler![runtime_config])
        .setup(move |app| {
            let state = app_state.server_process.clone();
            let server = app_state.server_config.clone();
            let resource_dir = app.path().resource_dir().ok();
            let runtime_dir = app.path().app_local_data_dir().ok();

            tauri::async_runtime::spawn(async move {
                if let Err(error) =
                    start_embedded_server(state, server, resource_dir, runtime_dir).await
                {
                    eprintln!("Failed to start server: {}", error);
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let state = window.state::<AppState>();
                let server = state.server_config.clone();
                let process = tauri::async_runtime::block_on(async {
                    let mut process = state.server_process.lock().await;
                    process.take()
                });

                let _ = request_stop_scan(&server);

                if let Some(mut child) = process {
                    let _ = child.kill();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
