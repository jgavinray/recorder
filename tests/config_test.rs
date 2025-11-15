// Integration test for configuration functionality

use meeting_recorder::Config;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_config_loading() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.yaml");
    let output_dir = temp_dir.path().join("recordings");
    
    // Create config file
    let config_content = format!(
        "output_directory: {}\n",
        output_dir.to_string_lossy()
    );
    fs::write(&config_file, config_content).unwrap();
    
    // Load config
    let config = Config::load_from_path(&config_file).unwrap();
    
    // Verify output directory was created
    assert!(output_dir.exists(), "Output directory should be created");
    assert!(output_dir.is_dir(), "Output directory should be a directory");
    assert_eq!(config.output_directory, output_dir.to_string_lossy());
}

#[test]
fn test_config_with_existing_directory() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.yaml");
    let output_dir = temp_dir.path().join("recordings");
    
    // Create output directory first
    fs::create_dir_all(&output_dir).unwrap();
    
    // Create config file
    let config_content = format!(
        "output_directory: {}\n",
        output_dir.to_string_lossy()
    );
    fs::write(&config_file, config_content).unwrap();
    
    // Load config
    let config = Config::load_from_path(&config_file).unwrap();
    
    // Should work with existing directory
    assert!(output_dir.exists());
    assert_eq!(config.output_directory, output_dir.to_string_lossy());
}

#[test]
fn test_config_missing_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("nonexistent.yaml");
    
    // Should fail when config file doesn't exist
    let result = Config::load_from_path(&config_file);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("not found"), "Error should mention file not found");
}

#[test]
fn test_config_invalid_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.yaml");
    
    // Write invalid YAML
    fs::write(&config_file, "invalid: yaml: content: [unclosed").unwrap();
    
    // Should fail to parse
    let result = Config::load_from_path(&config_file);
    assert!(result.is_err());
}

#[test]
fn test_config_missing_output_directory_field() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.yaml");
    
    // Write YAML without output_directory
    fs::write(&config_file, "some_other_field: value\n").unwrap();
    
    // Should fail to deserialize
    let result = Config::load_from_path(&config_file);
    assert!(result.is_err());
}

#[test]
fn test_recording_path_generation() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.yaml");
    let output_dir = temp_dir.path().join("recordings");
    
    // Create config
    let config_content = format!(
        "output_directory: {}\n",
        output_dir.to_string_lossy()
    );
    fs::write(&config_file, config_content).unwrap();
    
    let config = Config::load_from_path(&config_file).unwrap();
    
    // Test recording path generation
    let path = config.recording_path("recording_1234567890.wav");
    let path_str = path.to_string_lossy();
    let output_dir_str = output_dir.to_string_lossy();
    assert!(path_str.contains("recording_1234567890.wav"));
    assert!(path_str.contains(&*output_dir_str));
    
    // Verify it's a valid path
    assert_eq!(path.parent().unwrap(), &output_dir);
    assert_eq!(path.file_name().unwrap(), "recording_1234567890.wav");
}

#[test]
fn test_config_with_nested_directory_creation() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.yaml");
    let output_dir = temp_dir.path().join("deeply/nested/recordings/path");
    
    // Create config with nested path that doesn't exist
    let config_content = format!(
        "output_directory: {}\n",
        output_dir.to_string_lossy()
    );
    fs::write(&config_file, config_content).unwrap();
    
    // Load config - should create nested directories
    let config = Config::load_from_path(&config_file).unwrap();
    
    // Verify nested directory was created
    assert!(output_dir.exists(), "Nested output directory should be created");
    assert!(output_dir.is_dir(), "Nested output directory should be a directory");
    assert_eq!(config.output_directory, output_dir.to_string_lossy());
}

#[test]
fn test_config_with_file_as_output_directory() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.yaml");
    let output_file = temp_dir.path().join("not_a_directory");
    
    // Create a file (not a directory)
    fs::write(&output_file, "this is a file").unwrap();
    
    // Create config pointing to the file
    let config_content = format!(
        "output_directory: {}\n",
        output_file.to_string_lossy()
    );
    fs::write(&config_file, config_content).unwrap();
    
    // Should fail because output_directory is a file, not a directory
    let result = Config::load_from_path(&config_file);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("not a directory"), "Error should mention it's not a directory");
}

