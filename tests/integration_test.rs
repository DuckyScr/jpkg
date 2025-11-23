use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("jpkg").unwrap();
    cmd.arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains("jpkg v"));
}

#[test]
fn test_init_creates_structure() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("test_project");

    let mut cmd = Command::cargo_bin("jpkg").unwrap();
    cmd.arg("init")
        .arg("test_project")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Verify directory structure
    assert!(project_dir.join("src/main/java").exists());
    assert!(project_dir.join("jpkg.json").exists());
    assert!(project_dir.join(".gitignore").exists());

    // Verify jpkg.json content
    let manifest = fs::read_to_string(project_dir.join("jpkg.json")).unwrap();
    assert!(manifest.contains("test_project"));
}

#[test]
fn test_build_without_init_fails() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("jpkg").unwrap();
    cmd.arg("build")
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("jpkg.json not found"));
}

#[test]
fn test_full_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("workflow_test");

    // Init
    Command::cargo_bin("jpkg")
        .unwrap()
        .arg("init")
        .arg("workflow_test")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Build
    Command::cargo_bin("jpkg")
        .unwrap()
        .arg("build")
        .current_dir(&project_dir)
        .assert()
        .success();

    // Verify bin directory created
    assert!(project_dir.join("bin").exists());

    // Run
    Command::cargo_bin("jpkg")
        .unwrap()
        .arg("run")
        .current_dir(&project_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello"));
}

#[test]
fn test_lock_file_created_on_install() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("lock_test");

    // Init
    Command::cargo_bin("jpkg")
        .unwrap()
        .arg("init")
        .arg("lock_test")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Install (even with no dependencies, should create lock file)
    Command::cargo_bin("jpkg")
        .unwrap()
        .arg("install")
        .current_dir(&project_dir)
        .assert()
        .success();

    // Verify lock file exists
    assert!(project_dir.join("jpkg.lock").exists());

    // Verify lock file content
    let lock_content = fs::read_to_string(project_dir.join("jpkg.lock")).unwrap();
    assert!(lock_content.contains("\"version\""));
}

#[test]
fn test_package_creates_jar() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("package_test");

    // Init and build
    Command::cargo_bin("jpkg")
        .unwrap()
        .arg("init")
        .arg("package_test")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    Command::cargo_bin("jpkg")
        .unwrap()
        .arg("build")
        .current_dir(&project_dir)
        .assert()
        .success();

    // Package
    Command::cargo_bin("jpkg")
        .unwrap()
        .arg("package")
        .current_dir(&project_dir)
        .assert()
        .success();

    // Verify JAR exists
    assert!(project_dir.join("target/app.jar").exists());
}
