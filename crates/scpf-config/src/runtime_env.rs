use crate::server_config;
use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub const APPIMAGE_ENV_VAR: &str = "APPIMAGE";
pub const EXPLICIT_ENV_FILE_ENV_VAR: &str = "SCPF_ENV_FILE";
pub const PROJECT_ROOT_ENV_VAR: &str = "SCPF_PROJECT_ROOT";
pub const RUNTIME_DIR_ENV_VAR: &str = "SCPF_RUNTIME_DIR";
pub const DESKTOP_BUNDLE_DIR: &str = "com.teycir.scpf.desktop";

#[derive(Debug, Clone, Default)]
pub struct RuntimeEnvContext {
    pub explicit_env_file: Option<PathBuf>,
    pub project_root: Option<PathBuf>,
    pub runtime_dir: Option<PathBuf>,
    pub config_dir: Option<PathBuf>,
    pub data_local_dir: Option<PathBuf>,
    pub appimage_path: Option<PathBuf>,
    pub current_exe: Option<PathBuf>,
    pub current_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeEnvPaths {
    pub env_files: Vec<PathBuf>,
    pub preferred_env_file: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendRuntimeConfig {
    pub service: &'static str,
    pub api_base_url: String,
    pub server_addr: String,
    pub preferred_env_file: PathBuf,
    pub env_files: Vec<PathBuf>,
    pub has_explorer_keys: bool,
    pub has_github_token: bool,
}

pub fn load_process_env() -> RuntimeEnvPaths {
    let paths = resolve_runtime_env_paths(&detect_runtime_env_context());

    for env_file in &paths.env_files {
        let _ = dotenvy::from_path(env_file);
    }

    paths
}

pub fn frontend_runtime_config() -> FrontendRuntimeConfig {
    let server = server_config();
    let paths = resolve_runtime_env_paths(&detect_runtime_env_context());

    FrontendRuntimeConfig {
        service: "scpf-runtime",
        api_base_url: server.origin.clone(),
        server_addr: server.addr,
        preferred_env_file: paths.preferred_env_file,
        env_files: paths.env_files,
        has_explorer_keys: !crate::load_api_keys_from_env().is_empty(),
        has_github_token: std::env::var("GITHUB_TOKEN")
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false),
    }
}

pub fn api_key_help_message() -> String {
    format!(
        "Set ETHERSCAN_API_KEY in your environment or create {}.",
        preferred_api_key_env_file().display()
    )
}

pub fn preferred_api_key_env_file() -> PathBuf {
    resolve_runtime_env_paths(&detect_runtime_env_context()).preferred_env_file
}

pub fn detect_runtime_env_context() -> RuntimeEnvContext {
    RuntimeEnvContext {
        explicit_env_file: env_path(EXPLICIT_ENV_FILE_ENV_VAR),
        project_root: env_dir(PROJECT_ROOT_ENV_VAR),
        runtime_dir: env_dir(RUNTIME_DIR_ENV_VAR),
        config_dir: dirs::config_dir(),
        data_local_dir: dirs::data_local_dir(),
        appimage_path: env_path(APPIMAGE_ENV_VAR),
        current_exe: std::env::current_exe().ok(),
        current_dir: std::env::current_dir().ok(),
    }
}

pub fn resolve_runtime_env_paths(context: &RuntimeEnvContext) -> RuntimeEnvPaths {
    let mut candidates = Vec::new();

    push_if_existing_file(&mut candidates, context.explicit_env_file.clone());

    if let Some(runtime_dir) = context.runtime_dir.as_ref() {
        push_if_existing_file(&mut candidates, Some(runtime_dir.join(".env")));
    }

    if let Some(project_root) = context.project_root.as_ref() {
        push_if_existing_file(&mut candidates, Some(project_root.join(".env")));
    }

    if let Some(config_dir) = context.config_dir.as_ref() {
        push_if_existing_file(
            &mut candidates,
            Some(config_dir.join("smartcontractpatternfinder").join(".env")),
        );
        push_if_existing_file(&mut candidates, Some(config_dir.join("scpf").join(".env")));
        push_if_existing_file(
            &mut candidates,
            Some(config_dir.join(DESKTOP_BUNDLE_DIR).join(".env")),
        );
    }

    if let Some(data_local_dir) = context.data_local_dir.as_ref() {
        push_if_existing_file(
            &mut candidates,
            Some(
                data_local_dir
                    .join("smartcontractpatternfinder")
                    .join(".env"),
            ),
        );
        push_if_existing_file(
            &mut candidates,
            Some(data_local_dir.join("scpf").join(".env")),
        );
        push_if_existing_file(
            &mut candidates,
            Some(data_local_dir.join(DESKTOP_BUNDLE_DIR).join(".env")),
        );
    }

    push_if_existing_file(
        &mut candidates,
        find_first_ancestor_env_file(context.appimage_path.as_deref().and_then(Path::parent), 8),
    );
    push_if_existing_file(
        &mut candidates,
        find_first_ancestor_env_file(context.current_exe.as_deref().and_then(Path::parent), 8),
    );
    push_if_existing_file(
        &mut candidates,
        find_first_ancestor_env_file(context.current_dir.as_deref(), 8),
    );

    RuntimeEnvPaths {
        env_files: dedupe_paths(candidates),
        preferred_env_file: preferred_env_file_for(context),
    }
}

fn preferred_env_file_for(context: &RuntimeEnvContext) -> PathBuf {
    if let Some(path) = context.explicit_env_file.as_ref() {
        return path.clone();
    }

    if let Some(runtime_dir) = context.runtime_dir.as_ref() {
        return runtime_dir.join(".env");
    }

    if let Some(project_root) = context.project_root.as_ref() {
        return project_root.join(".env");
    }

    if let Some(current_dir) = context.current_dir.as_ref() {
        return current_dir.join(".env");
    }

    if let Some(data_local_dir) = context.data_local_dir.as_ref() {
        return data_local_dir.join(DESKTOP_BUNDLE_DIR).join(".env");
    }

    PathBuf::from(".env")
}

fn find_first_ancestor_env_file(start: Option<&Path>, depth: usize) -> Option<PathBuf> {
    let mut current = start?.to_path_buf();

    for _ in 0..depth {
        let candidate = current.join(".env");
        if candidate.is_file() {
            return Some(candidate);
        }

        let parent = current.parent()?.to_path_buf();
        current = parent;
    }

    None
}

fn push_if_existing_file(paths: &mut Vec<PathBuf>, path: Option<PathBuf>) {
    if let Some(path) = path.filter(|path| path.is_file()) {
        paths.push(path);
    }
}

fn dedupe_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = HashSet::new();

    paths
        .into_iter()
        .filter(|path| seen.insert(path.clone()))
        .collect()
}

fn env_dir(name: &str) -> Option<PathBuf> {
    env_path(name).filter(|path| path.is_dir())
}

fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name).map(PathBuf::from)
}
