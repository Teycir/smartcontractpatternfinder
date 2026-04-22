use scpf_config::{resolve_runtime_env_paths, RuntimeEnvContext};
use std::fs;

#[test]
fn resolves_env_files_in_priority_order_and_dedupes_ancestor_matches() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();

    let explicit_env = root.join("custom.env");
    let runtime_dir = root.join("runtime");
    let project_root = root.join("project");
    let config_dir = root.join("config");
    let data_local_dir = root.join("data-local");
    let current_dir = project_root.join("workspace").join("nested");

    fs::create_dir_all(&runtime_dir).unwrap();
    fs::create_dir_all(project_root.join("workspace")).unwrap();
    fs::create_dir_all(config_dir.join("smartcontractpatternfinder")).unwrap();
    fs::create_dir_all(data_local_dir.join("com.teycir.scpf.desktop")).unwrap();
    fs::create_dir_all(&current_dir).unwrap();

    fs::write(&explicit_env, "EXPLICIT=1\n").unwrap();
    fs::write(runtime_dir.join(".env"), "RUNTIME=1\n").unwrap();
    fs::write(project_root.join(".env"), "PROJECT=1\n").unwrap();
    fs::write(
        config_dir.join("smartcontractpatternfinder").join(".env"),
        "CONFIG=1\n",
    )
    .unwrap();
    fs::write(
        data_local_dir.join("com.teycir.scpf.desktop").join(".env"),
        "DATA_LOCAL=1\n",
    )
    .unwrap();

    let context = RuntimeEnvContext {
        explicit_env_file: Some(explicit_env.clone()),
        project_root: Some(project_root.clone()),
        runtime_dir: Some(runtime_dir.clone()),
        config_dir: Some(config_dir.clone()),
        data_local_dir: Some(data_local_dir.clone()),
        current_dir: Some(current_dir),
        ..RuntimeEnvContext::default()
    };

    let paths = resolve_runtime_env_paths(&context);

    assert_eq!(
        paths.env_files,
        vec![
            explicit_env,
            runtime_dir.join(".env"),
            project_root.join(".env"),
            config_dir.join("smartcontractpatternfinder").join(".env"),
            data_local_dir.join("com.teycir.scpf.desktop").join(".env"),
        ]
    );
}

#[test]
fn preferred_env_file_favors_runtime_then_project_then_current_dir_then_data_local() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let runtime_dir = root.join("runtime");
    let project_root = root.join("project");
    let current_dir = root.join("workspace");
    let data_local_dir = root.join("data-local");

    fs::create_dir_all(&runtime_dir).unwrap();
    fs::create_dir_all(&project_root).unwrap();
    fs::create_dir_all(&current_dir).unwrap();
    fs::create_dir_all(&data_local_dir).unwrap();

    let with_runtime = RuntimeEnvContext {
        runtime_dir: Some(runtime_dir.clone()),
        project_root: Some(project_root.clone()),
        data_local_dir: Some(data_local_dir.clone()),
        ..RuntimeEnvContext::default()
    };
    assert_eq!(
        resolve_runtime_env_paths(&with_runtime).preferred_env_file,
        runtime_dir.join(".env")
    );

    let with_project_only = RuntimeEnvContext {
        project_root: Some(project_root.clone()),
        data_local_dir: Some(data_local_dir.clone()),
        ..RuntimeEnvContext::default()
    };
    assert_eq!(
        resolve_runtime_env_paths(&with_project_only).preferred_env_file,
        project_root.join(".env")
    );

    let with_data_local_only = RuntimeEnvContext {
        current_dir: Some(current_dir.clone()),
        data_local_dir: Some(data_local_dir.clone()),
        ..RuntimeEnvContext::default()
    };
    assert_eq!(
        resolve_runtime_env_paths(&with_data_local_only).preferred_env_file,
        current_dir.join(".env")
    );

    let with_data_local_fallback = RuntimeEnvContext {
        data_local_dir: Some(data_local_dir.clone()),
        ..RuntimeEnvContext::default()
    };
    assert_eq!(
        resolve_runtime_env_paths(&with_data_local_fallback).preferred_env_file,
        data_local_dir.join("com.teycir.scpf.desktop").join(".env")
    );
}
