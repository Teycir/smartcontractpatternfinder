use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .unwrap()
        .to_path_buf();
    let resources_dir = manifest_dir.join("resources");
    let resources_bin_dir = resources_dir.join("bin");
    let resources_config_dir = resources_dir.join("config");
    let resources_templates_dir = resources_dir.join("templates");

    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    fs::create_dir_all(&resources_bin_dir).expect("failed to create desktop bin resources");
    fs::create_dir_all(&resources_config_dir).expect("failed to create desktop config resources");
    fs::create_dir_all(&resources_templates_dir)
        .expect("failed to create desktop template resources");

    let _ = sync_templates_dir(&workspace_root.join("templates"), &resources_templates_dir);
    let _ = copy_file_if_present(&workspace_root.join(".env.example"), &resources_config_dir);
    let _ = copy_binary_if_present(
        &workspace_root
            .join("target")
            .join(&profile)
            .join(binary_name("scpf-server")),
        &resources_bin_dir,
    );
    let _ = copy_binary_if_present(
        &workspace_root
            .join("target")
            .join(&profile)
            .join(binary_name("scpf")),
        &resources_bin_dir,
    );

    tauri_build::build()
}

fn binary_name(stem: &str) -> String {
    if cfg!(windows) {
        format!("{}.exe", stem)
    } else {
        stem.to_string()
    }
}

fn copy_binary_if_present(source: &Path, destination_dir: &Path) -> io::Result<()> {
    if !source.is_file() {
        return Ok(());
    }

    let destination = destination_dir.join(source.file_name().unwrap());
    fs::copy(source, destination)?;
    Ok(())
}

fn copy_file_if_present(source: &Path, destination_dir: &Path) -> io::Result<()> {
    if !source.is_file() {
        return Ok(());
    }

    let destination = destination_dir.join(source.file_name().unwrap());
    fs::copy(source, destination)?;
    Ok(())
}

fn sync_templates_dir(source: &Path, destination: &Path) -> io::Result<()> {
    if !source.is_dir() {
        return Ok(());
    }

    if destination.exists() {
        fs::remove_dir_all(destination)?;
    }

    copy_dir_recursive(source, destination)
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());

        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &destination_path)?;
        } else {
            fs::copy(&source_path, &destination_path)?;
        }
    }

    Ok(())
}
