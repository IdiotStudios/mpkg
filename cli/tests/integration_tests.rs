use std::fs;
use tempfile::TempDir;
use predicates::prelude::*;
use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn test_init_creates_pkg_jsonc() {
    let temp = TempDir::new().unwrap();
    
    cargo_bin_cmd!("mpkg")
        .arg("init")
        .arg("test-project")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created pkg.jsonc"));
    
    // Verify file exists
    let pkg_path = temp.path().join("pkg.jsonc");
    assert!(pkg_path.exists(), "pkg.jsonc should be created");
    
    // Verify content
    let content = fs::read_to_string(&pkg_path).unwrap();
    assert!(content.contains("test-project"));
    assert!(content.contains("0.1.0"));
}

#[test]
fn test_init_creates_gitignore() {
    let temp = TempDir::new().unwrap();
    
    cargo_bin_cmd!("mpkg")
        .arg("init")
        .arg("my-project")
        .current_dir(temp.path())
        .assert()
        .success();
    
    let gitignore_path = temp.path().join(".gitignore");
    assert!(gitignore_path.exists(), ".gitignore should be created");
    
    let content = fs::read_to_string(&gitignore_path).unwrap();
    assert!(content.contains("/packages"));
}

#[test]
fn test_init_skips_existing_files() {
    let temp = TempDir::new().unwrap();
    
    // Create pkg.jsonc first
    fs::write(temp.path().join("pkg.jsonc"), "{}").unwrap();
    
    cargo_bin_cmd!("mpkg")
        .arg("init")
        .arg("test-project")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("already exists"));
}

#[test]
fn test_package_command_creates_zip() {
    let temp = TempDir::new().unwrap();
    
    // Create a test directory with files
    let test_dir = temp.path().join("test_project");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("file1.txt"), "content1").unwrap();
    fs::write(test_dir.join("file2.txt"), "content2").unwrap();
    
    cargo_bin_cmd!("mpkg")
        .arg("package")
        .arg(test_dir.to_str().unwrap())
        .current_dir(temp.path())
        .assert()
        .success();
    
    // Check zip was created
    let zip_path = temp.path().join("test_project.zip");
    assert!(zip_path.exists(), "ZIP file should be created");
    
    // Verify it's not empty
    let metadata = fs::metadata(&zip_path).unwrap();
    assert!(metadata.len() > 0, "ZIP should not be empty");
}

#[test]
fn test_package_with_custom_output() {
    let temp = TempDir::new().unwrap();
    
    let test_dir = temp.path().join("myapp");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("index.js"), "console.log('hi')").unwrap();
    
    cargo_bin_cmd!("mpkg")
        .arg("package")
        .arg(test_dir.to_str().unwrap())
        .arg("custom-name.zip")
        .current_dir(temp.path())
        .assert()
        .success();
    
    let zip_path = temp.path().join("custom-name.zip");
    assert!(zip_path.exists(), "Custom named ZIP should exist");
}

#[test]
fn test_run_without_loader_fails() {
    let temp = TempDir::new().unwrap();
    
    // Try to run without initializing
    cargo_bin_cmd!("mpkg")
        .arg("run")
        .arg("test.js")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Loader not found"));
}

#[test]
fn test_package_nonexistent_directory() {
    cargo_bin_cmd!("mpkg")
        .arg("package")
        .arg("/nonexistent/path/to/nowhere")
        .assert()
        .failure();
}

#[test]
fn test_zip_contains_correct_files() {
    use std::io::Read;
    use zip::ZipArchive;
    
    let temp = TempDir::new().unwrap();
    let source_dir = temp.path().join("source");
    fs::create_dir(&source_dir).unwrap();
    
    // Create test files
    fs::write(source_dir.join("file1.txt"), "content1").unwrap();
    fs::write(source_dir.join("file2.txt"), "content2").unwrap();
    
    let zip_path = temp.path().join("test.zip");
    
    // Create zip using the CLI
    cargo_bin_cmd!("mpkg")
        .arg("package")
        .arg(source_dir.to_str().unwrap())
        .arg(zip_path.to_str().unwrap())
        .assert()
        .success();
    
    // Verify zip contents
    let file = fs::File::open(&zip_path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    
    assert_eq!(archive.len(), 2, "Should have 2 files");
    
    // Check first file content
    let mut file1 = archive.by_name("file1.txt").unwrap();
    let mut content1 = String::new();
    file1.read_to_string(&mut content1).unwrap();
    assert_eq!(content1, "content1");
}

#[test]
fn test_zip_nested_directories() {
    use zip::ZipArchive;
    
    let temp = TempDir::new().unwrap();
    let source_dir = temp.path().join("source");
    
    // Create nested structure
    fs::create_dir_all(source_dir.join("subdir/nested")).unwrap();
    fs::write(source_dir.join("root.txt"), "root").unwrap();
    fs::write(source_dir.join("subdir/sub.txt"), "sub").unwrap();
    fs::write(source_dir.join("subdir/nested/deep.txt"), "deep").unwrap();
    
    let zip_path = temp.path().join("nested.zip");
    
    cargo_bin_cmd!("mpkg")
        .arg("package")
        .arg(source_dir.to_str().unwrap())
        .arg(zip_path.to_str().unwrap())
        .assert()
        .success();
    
    // Verify nested structure preserved
    let file = fs::File::open(&zip_path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    
    assert!(archive.by_name("root.txt").is_ok());
    assert!(archive.by_name("subdir/sub.txt").is_ok());
    assert!(archive.by_name("subdir/nested/deep.txt").is_ok());
}