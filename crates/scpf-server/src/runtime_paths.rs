use std::path::{Path, PathBuf};

const CLI_BINARY_ENV: &str = "SCPF_CLI_BINARY";
const TEMPLATES_DIR_ENV: &str = "SCPF_TEMPLATES_DIR";
const REPORTS_DIR_ENV: &str = "SCPF_REPORTS_DIR";

pub fn find_project_root() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;

    for _ in 0..5 {
        if current.join("templates").is_dir() && current.join("Cargo.toml").is_file() {
            return Some(current);
        }
        current = current.parent()?.to_path_buf();
    }

    None
}

pub fn templates_root() -> PathBuf {
    if let Some(path) = std::env::var_os(TEMPLATES_DIR_ENV)
        .map(PathBuf::from)
        .filter(|path| path.is_dir())
    {
        return path;
    }

    if let Some(project_root) = find_project_root() {
        let templates_dir = project_root.join("templates");
        if templates_dir.is_dir() {
            return templates_dir;
        }
    }

    let bundled_templates = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates");
    if bundled_templates.is_dir() {
        return bundled_templates;
    }

    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("templates")
}

pub fn desktop_cli_binary_path() -> Option<PathBuf> {
    let env_path = std::env::var_os(CLI_BINARY_ENV).map(PathBuf::from);
    let current_exe = std::env::current_exe().ok();
    let current_dir = std::env::current_dir().ok();

    desktop_cli_binary_path_from(
        env_path.as_deref(),
        current_exe.as_deref(),
        current_dir.as_deref(),
    )
}

fn desktop_cli_binary_path_from(
    env_path: Option<&Path>,
    current_exe: Option<&Path>,
    current_dir: Option<&Path>,
) -> Option<PathBuf> {
    let binary_name = if cfg!(windows) { "scpf.exe" } else { "scpf" };

    if let Some(path) = env_path.filter(|path| path.is_file()) {
        return Some(path.to_path_buf());
    }

    for path in cli_candidates_from_exe(binary_name, current_exe) {
        if path.is_file() {
            return Some(path);
        }
    }

    let mut current = current_dir?.to_path_buf();
    for _ in 0..5 {
        let release_bin = current.join("target/release").join(binary_name);
        if release_bin.is_file() {
            return Some(release_bin);
        }

        let debug_bin = current.join("target/debug").join(binary_name);
        if debug_bin.is_file() {
            return Some(debug_bin);
        }

        current = current.parent()?.to_path_buf();
    }

    None
}

pub fn next_report_dir() -> PathBuf {
    let root = reports_root();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    root.join(format!("report_{}", timestamp))
}

fn reports_root() -> PathBuf {
    if let Some(path) = std::env::var_os(REPORTS_DIR_ENV)
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
    {
        return path;
    }

    dirs::home_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("smartcontractpatternfinderReports")
}

fn cli_candidates_from_exe(binary_name: &str, current_exe: Option<&Path>) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let Some(exe_dir) = current_exe.and_then(Path::parent) else {
        return candidates;
    };

    candidates.push(exe_dir.join(binary_name));
    candidates.push(exe_dir.join("target/release").join(binary_name));
    candidates.push(exe_dir.join("target/debug").join(binary_name));

    if let Some(parent) = exe_dir.parent() {
        candidates.push(parent.join("release").join(binary_name));
        candidates.push(parent.join("debug").join(binary_name));

        if cfg!(target_os = "macos") {
            candidates.push(parent.join("Resources").join(binary_name));
        }
    }

    candidates
}
