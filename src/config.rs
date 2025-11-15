use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Directory where recordings are saved
    pub output_directory: String,
}

impl Config {
    /// Load configuration from platform-specific default location
    /// - Windows: %PROGRAMDATA%\meeting-recorder\config.yaml
    /// - macOS/Linux: /opt/meeting-recorder/config.yaml
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::default_config_path()?;
        Self::load_from_path(config_path)
    }
    
    /// Get the default config path for the current platform
    /// This is public for testing purposes
    pub fn default_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        #[cfg(target_os = "windows")]
        {
            use std::env;
            let program_data = env::var("PROGRAMDATA")
                .unwrap_or_else(|_| "C:\\ProgramData".to_string());
            Ok(PathBuf::from(program_data).join("meeting-recorder").join("config.yaml"))
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            Ok(PathBuf::from("/opt/meeting-recorder/config.yaml"))
        }
    }
    
    /// Load configuration from a specific path (useful for testing)
    pub fn load_from_path(config_path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = config_path.as_ref();
        
        if !config_path.exists() {
            return Err(format!(
                "Config file not found at {}. Please create it with an 'output_directory' field.",
                config_path.display()
            ).into());
        }
        
        let contents = fs::read_to_string(config_path)?;
        let config: Config = serde_yaml::from_str(&contents)?;
        
        // Validate that the output directory exists or can be created
        let output_path = Path::new(&config.output_directory);
        if !output_path.exists() {
            fs::create_dir_all(output_path)?;
        }
        
        if !output_path.is_dir() {
            return Err(format!(
                "Output directory '{}' exists but is not a directory",
                config.output_directory
            ).into());
        }
        
        Ok(config)
    }
    
    /// Get the full path for a recording file
    pub fn recording_path(&self, filename: &str) -> PathBuf {
        Path::new(&self.output_directory).join(filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_recording_path() {
        let config = Config {
            output_directory: "/tmp/recordings".to_string(),
        };
        
        let path = config.recording_path("test.wav");
        assert!(path.to_string_lossy().contains("test.wav"));
        assert!(path.to_string_lossy().contains("/tmp/recordings"));
    }
}

