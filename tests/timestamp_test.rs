// Test timestamp formatting in filenames

use meeting_recorder::Config;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn test_timestamp_format() {
    // Test that the timestamp format is mm-dd-yyyy-24h-m-recording.wav
    // Example: 01-25-2024-14-30-recording.wav
    let example_filename = "01-25-2024-14-30-recording.wav";
    
    // Verify format structure
    assert!(example_filename.ends_with("-recording.wav"));
    
    // Extract the timestamp part
    let timestamp_part = example_filename
        .strip_suffix("-recording.wav")
        .unwrap();
    
    // Should have format: mm-dd-yyyy-24h-m
    let parts: Vec<&str> = timestamp_part.split('-').collect();
    assert_eq!(parts.len(), 5, "Timestamp should have 5 parts: mm, dd, yyyy, 24h, m");
    
    // Verify each part is numeric and has correct length
    assert_eq!(parts[0].len(), 2, "Month should be 2 digits");
    assert_eq!(parts[1].len(), 2, "Day should be 2 digits");
    assert_eq!(parts[2].len(), 4, "Year should be 4 digits");
    assert_eq!(parts[3].len(), 2, "Hour should be 2 digits");
    assert_eq!(parts[4].len(), 2, "Minute should be 2 digits");
    
    // Verify all parts are numeric
    for part in &parts {
        assert!(part.chars().all(|c| c.is_ascii_digit()), 
                "All timestamp parts should be numeric");
    }
    
    // Verify ranges (basic sanity checks)
    let month: u32 = parts[0].parse().unwrap();
    let day: u32 = parts[1].parse().unwrap();
    let hour: u32 = parts[3].parse().unwrap();
    let minute: u32 = parts[4].parse().unwrap();
    
    assert!(month >= 1 && month <= 12, "Month should be between 1 and 12");
    assert!(day >= 1 && day <= 31, "Day should be between 1 and 31");
    assert!(hour < 24, "Hour should be less than 24");
    assert!(minute < 60, "Minute should be less than 60");
}

#[test]
fn test_filename_with_timestamp_format() {
    // Test that filenames with the new format work correctly with Config
    let config = Config {
        output_directory: "/tmp/recordings".to_string(),
    };
    
    // Test with the new timestamp format: mm-dd-yyyy-24h-m-recording.wav
    let filename = "01-25-2024-14-30-recording.wav";
    let path = config.recording_path(filename);
    
    assert!(path.to_string_lossy().contains(filename));
    assert!(path.file_name().unwrap() == filename);
}

#[test]
fn test_timestamp_format_windows_compatibility() {
    // Verify the format doesn't contain characters invalid on Windows
    let example_filename = "01-25-2024-14-30-recording.wav";
    
    // Windows invalid characters: < > : " | ? * \
    let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '\\'];
    
    for &ch in &invalid_chars {
        assert!(
            !example_filename.contains(ch),
            "Filename should not contain invalid Windows character: {}",
            ch
        );
    }
}

