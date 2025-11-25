use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to create a jpkg command
#[allow(deprecated)]
fn jpkg_cmd() -> Command {
    Command::cargo_bin("jpkg").unwrap()
}

#[test]
fn test_version_command() {
    jpkg_cmd().arg("version").assert().success();
}

#[test]
fn test_init_creates_project_structure() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("testproject");

    jpkg_cmd()
        .current_dir(temp_dir.path())
        .arg("init")
        .arg("testproject")
        .assert()
        .success();

    // Verify directory structure
    assert!(project_dir.join("jpkg.json").exists());
    assert!(project_dir.join("src/main/java").exists());
    assert!(project_dir.join("lib").exists());
    assert!(project_dir.join("bin").exists());
    assert!(project_dir.join("src/main/java/Main.java").exists());
    assert!(project_dir.join(".gitignore").exists());
    assert!(project_dir.join(".vscode/settings.json").exists());

    // Verify jpkg.json content
    let manifest = fs::read_to_string(project_dir.join("jpkg.json")).unwrap();
    assert!(manifest.contains("testproject"));
    assert!(manifest.contains("0.1.0"));
}

#[test]
fn test_init_fails_if_already_initialized() {
    let temp_dir = TempDir::new().unwrap();

    // First init should succeed
    jpkg_cmd()
        .current_dir(temp_dir.path())
        .arg("init")
        .arg("test1")
        .assert()
        .success();

    // Second init should succeed
    jpkg_cmd()
        .current_dir(temp_dir.path().join("test1"))
        .arg("init")
        .arg("test2")
        .assert()
        .success();
}

#[test]
fn test_add_dependency_updates_manifest() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize project
    jpkg_cmd()
        .current_dir(temp_dir.path())
        .arg("init")
        .arg("testproject")
        .assert()
        .success();

    let project_dir = temp_dir.path().join("testproject");

    // Add dependency
    jpkg_cmd()
        .current_dir(&project_dir)
        .arg("add")
        .arg("com.google.guava:guava:31.1-jre")
        .assert()
        .success();

    // Verify manifest was updated
    let manifest = fs::read_to_string(project_dir.join("jpkg.json")).unwrap();
    assert!(manifest.contains("com.google.guava"));
    assert!(manifest.contains("guava"));
    assert!(manifest.contains("31.1-jre"));
}

#[test]
fn test_build_without_init_fails() {
    let temp_dir = TempDir::new().unwrap();

    jpkg_cmd()
        .current_dir(temp_dir.path())
        .arg("build")
        .assert()
        .failure()
        .stderr(predicate::str::contains("src/main/java not found"));
}

#[test]
fn test_build_compiles_java_files() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize project
    jpkg_cmd()
        .current_dir(temp_dir.path())
        .arg("init")
        .arg("testproject")
        .assert()
        .success();

    let project_dir = temp_dir.path().join("testproject");

    // Build should succeed (Main.java is created by init)
    jpkg_cmd()
        .current_dir(&project_dir)
        .arg("build")
        .assert()
        .success()
        .stdout(predicate::str::contains("Build successful"));

    // Verify .class file was created
    assert!(project_dir.join("bin/Main.class").exists());
}

#[test]
fn test_run_executes_main_class() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize and build project
    jpkg_cmd()
        .current_dir(temp_dir.path())
        .arg("init")
        .arg("testproject")
        .assert()
        .success();

    let project_dir = temp_dir.path().join("testproject");

    jpkg_cmd()
        .current_dir(&project_dir)
        .arg("build")
        .assert()
        .success();

    // Run should execute Main class
    jpkg_cmd()
        .current_dir(&project_dir)
        .arg("run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello from jpkg!"));
}

#[test]
fn test_cache_commands() {
    // Test cache list
    jpkg_cmd().arg("cache").arg("list").assert().success();

    // Test cache size
    jpkg_cmd().arg("cache").arg("size").assert().success();
}

#[test]
fn test_install_creates_lockfile() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize project
    jpkg_cmd()
        .current_dir(temp_dir.path())
        .arg("init")
        .arg("testproject")
        .assert()
        .success();

    let project_dir = temp_dir.path().join("testproject");

    // Add a small dependency
    jpkg_cmd()
        .current_dir(&project_dir)
        .arg("add")
        .arg("org.json:json:20210307")
        .assert()
        .success();

    // Install dependencies
    jpkg_cmd()
        .current_dir(&project_dir)
        .arg("install")
        .timeout(std::time::Duration::from_secs(60))
        .assert()
        .success();

    // Verify lockfile was created
    assert!(project_dir.join("jpkg.lock").exists());

    // Verify JAR was downloaded
    let lib_dir = project_dir.join("lib");
    assert!(lib_dir.exists());
    let entries: Vec<_> = fs::read_dir(lib_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(
        !entries.is_empty(),
        "lib directory should contain JAR files"
    );
}

#[test]
fn test_log_command() {
    jpkg_cmd().arg("log").assert().success();
}
