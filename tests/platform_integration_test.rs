// Integration tests for platform-specific functionality

use meeting_recorder::Config;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_platform_specific_config_loading() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.yaml");
    let output_dir = temp_dir.path().join("recordings");
    
    // Create config file
    let config_content = format!(
        "output_directory: {}\n",
        output_dir.to_string_lossy()
    );
    fs::write(&config_file, config_content).unwrap();
    
    // Test loading from platform-specific path structure
    let config = Config::load_from_path(&config_file).unwrap();
    
    // Verify it works on all platforms
    assert_eq!(config.output_directory, output_dir.to_string_lossy());
    
    // Test recording path generation works cross-platform
    let recording_path = config.recording_path("test_recording.wav");
    assert!(recording_path.to_string_lossy().contains("test_recording.wav"));
    assert!(recording_path.parent().unwrap() == &output_dir);
}

#[test]
fn test_windows_path_handling_in_config() {
    #[cfg(target_os = "windows")]
    {
        // Test Windows-style paths in config
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yaml");
        let output_dir = temp_dir.path().join("Recordings\\Meetings");
        
        let config_content = format!(
            "output_directory: {}\n",
            output_dir.to_string_lossy().replace('/', "\\")
        );
        fs::write(&config_file, config_content).unwrap();
        
        let config = Config::load_from_path(&config_file).unwrap();
        let path = config.recording_path("test.wav");
        
        // Path should be valid and contain the filename
        assert!(path.to_string_lossy().contains("test.wav"));
        assert!(path.to_string_lossy().contains("Recordings"));
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // On non-Windows, test that Unix paths work
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.yaml");
        let output_dir = temp_dir.path().join("recordings/meetings");
        
        let config_content = format!(
            "output_directory: {}\n",
            output_dir.to_string_lossy()
        );
        fs::write(&config_file, config_content).unwrap();
        
        let config = Config::load_from_path(&config_file).unwrap();
        let path = config.recording_path("test.wav");
        
        assert!(path.to_string_lossy().contains("test.wav"));
        assert!(path.to_string_lossy().contains("recordings"));
    }
}

#[test]
fn test_default_config_path_platform_detection() {
    let path = Config::default_config_path().unwrap();
    
    // Verify platform-specific path structure
    #[cfg(target_os = "windows")]
    {
        let path_str = path.to_string_lossy().to_lowercase();
        // Should contain programdata and meeting-recorder
        assert!(path_str.contains("programdata") || path_str.contains("c:\\programdata"));
        assert!(path_str.contains("meeting-recorder"));
        assert!(path_str.ends_with("config.yaml"));
        // Should use backslashes on Windows
        assert!(path_str.contains("\\"));
    }
    
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        // Should be exactly /opt/meeting-recorder/config.yaml
        assert_eq!(path, PathBuf::from("/opt/meeting-recorder/config.yaml"));
        // Should use forward slashes
        assert!(path.to_string_lossy().contains("/"));
    }
}

#[test]
fn test_cross_platform_path_join() {
    // Test that PathBuf.join works correctly on all platforms
    let config = Config {
        output_directory: "/tmp/test".to_string(),
    };
    
    let path = config.recording_path("file.wav");
    
    // Should work on all platforms (PathBuf handles platform differences)
    assert!(path.to_string_lossy().contains("file.wav"));
    assert!(path.file_name().unwrap() == "file.wav");
    
    // Parent should be the output directory
    #[cfg(not(target_os = "windows"))]
    {
        assert_eq!(path.parent().unwrap(), PathBuf::from("/tmp/test"));
    }
}

#[test]
fn test_config_with_absolute_paths() {
    // Test that absolute paths work on all platforms
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.yaml");
    let output_dir = temp_dir.path().join("recordings").join("meetings");
    
    // Use absolute path from temp directory (which we can create)
    let output_dir_abs = fs::canonicalize(temp_dir.path())
        .unwrap()
        .join("recordings")
        .join("meetings");
    
    #[cfg(target_os = "windows")]
    {
        // On Windows, test with Windows-style path format
        let output_dir_str = output_dir_abs.to_string_lossy().replace('/', "\\");
        let config_content = format!("output_directory: {}\n", output_dir_str);
        fs::write(&config_file, config_content).unwrap();
        
        let config = Config::load_from_path(&config_file).unwrap();
        let path = config.recording_path("test.wav");
        assert!(path.to_string_lossy().to_lowercase().contains("recordings"));
        assert!(path.to_string_lossy().contains("test.wav"));
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // On Unix, test with Unix-style path
        let output_dir_str = output_dir_abs.to_string_lossy();
        let config_content = format!("output_directory: {}\n", output_dir_str);
        fs::write(&config_file, config_content).unwrap();
        
        let config = Config::load_from_path(&config_file).unwrap();
        let path = config.recording_path("test.wav");
        assert!(path.to_string_lossy().contains("recordings"));
        assert!(path.to_string_lossy().contains("test.wav"));
    }
}

