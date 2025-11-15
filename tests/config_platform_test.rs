// Test platform-specific config path behavior

use meeting_recorder::Config;
use std::path::PathBuf;

#[test]
fn test_default_config_path_structure() {
    // Test that default_config_path returns a valid path structure
    let config_path = Config::default_config_path().unwrap();
    
    // Verify path contains expected components
    assert!(config_path.to_string_lossy().contains("meeting-recorder"));
    assert!(config_path.to_string_lossy().contains("config.yaml"));
    
    // Platform-specific validation
    #[cfg(target_os = "windows")]
    {
        // On Windows, should be in PROGRAMDATA or C:\ProgramData
        let path_str = config_path.to_string_lossy().to_lowercase();
        assert!(
            path_str.contains("programdata") || path_str.contains("c:\\programdata"),
            "Windows config path should be in PROGRAMDATA"
        );
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // On Unix, should be /opt/meeting-recorder/config.yaml
        assert_eq!(
            config_path,
            PathBuf::from("/opt/meeting-recorder/config.yaml"),
            "Unix config path should be /opt/meeting-recorder/config.yaml"
        );
    }
}

#[test]
fn test_windows_path_handling() {
    // Test that Windows paths are handled correctly
    #[cfg(target_os = "windows")]
    {
        let config = Config {
            output_directory: "C:\\Recordings\\Meetings".to_string(),
        };
        
        let path = config.recording_path("test.wav");
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("test.wav"));
        assert!(path_str.contains("Recordings"));
    }
}

#[test]
fn test_unix_path_handling() {
    // Test that Unix paths are handled correctly
    #[cfg(not(target_os = "windows"))]
    {
        let config = Config {
            output_directory: "/var/recordings/meetings".to_string(),
        };
        
        let path = config.recording_path("test.wav");
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("test.wav"));
        assert!(path_str.contains("recordings"));
    }
}

