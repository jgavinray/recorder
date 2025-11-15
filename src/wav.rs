use std::fs;
use std::io::Read;

/// Validates that a file is a proper WAV file with valid structure
pub fn validate_wav_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::open(path)?;
    let mut buffer = [0u8; 44]; // Read at least the header
    
    let bytes_read = file.read(&mut buffer)?;
    
    if bytes_read < 12 {
        return Err("File too small to be a valid WAV file".into());
    }
    
    // Check RIFF header (bytes 0-3)
    if &buffer[0..4] != b"RIFF" {
        return Err(format!("Invalid RIFF header: expected 'RIFF', got '{:?}'", &buffer[0..4]).into());
    }
    
    // Check WAVE identifier (bytes 8-11)
    if &buffer[8..12] != b"WAVE" {
        return Err(format!("Invalid WAVE identifier: expected 'WAVE', got '{:?}'", &buffer[8..12]).into());
    }
    
    // Check format chunk (bytes 12-15 should be "fmt ")
    if bytes_read >= 16 && &buffer[12..16] != b"fmt " {
        return Err("Format chunk identifier not found".into());
    }
    
    // Verify file has data beyond headers
    let metadata = fs::metadata(path)?;
    if metadata.len() <= 44 {
        return Err("File contains only headers, no audio data".into());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hound::{WavWriter, WavSpec, SampleFormat};

    #[test]
    fn test_wav_file_validation() {
        let test_file = "test_validation.wav";
        let spec = WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        
        {
            let mut writer = WavWriter::create(test_file, spec).unwrap();
            for i in 0..1000 {
                writer.write_sample((i as i16) % 1000).unwrap();
            }
            writer.finalize().unwrap();
        }
        
        assert!(validate_wav_file(test_file).is_ok());
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_wav_file_invalid_riff() {
        let invalid_data = b"XXXX\x24\x00\x00\x00WAVE";
        let test_file = "test_invalid_riff.wav";
        fs::write(test_file, invalid_data).unwrap();
        
        let result = validate_wav_file(test_file);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("RIFF"));
        
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_wav_file_invalid_wave() {
        let invalid_data = b"RIFF\x24\x00\x00\x00XXXX";
        let test_file = "test_invalid_wave.wav";
        fs::write(test_file, invalid_data).unwrap();
        
        let result = validate_wav_file(test_file);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("WAVE"));
        
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_wav_file_too_small() {
        let small_data = b"RIFF";
        let test_file = "test_too_small.wav";
        fs::write(test_file, small_data).unwrap();
        
        let result = validate_wav_file(test_file);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too small"));
        
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_wav_file_headers_only() {
        let header_only = b"RIFF\x28\x00\x00\x00WAVEfmt \x10\x00\x00\x00\x01\x00\x01\x00\x44\xac\x00\x00\x88\x58\x01\x00\x02\x00\x10\x00data\x00\x00\x00\x00";
        let test_file = "test_headers_only.wav";
        fs::write(test_file, header_only).unwrap();
        
        let result = validate_wav_file(test_file);
        assert!(result.is_err());
        
        fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_create_minimal_wav() {
        let test_file = "test_minimal.wav";
        let spec = WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        
        {
            let mut writer = WavWriter::create(test_file, spec).unwrap();
            for i in 0..100 {
                writer.write_sample((i as i16) % 1000).unwrap();
            }
            writer.finalize().unwrap();
        }
        
        assert!(validate_wav_file(test_file).is_ok());
        
        let metadata = fs::metadata(test_file).unwrap();
        assert!(metadata.len() > 44, "WAV file should have data beyond headers");
        
        fs::remove_file(test_file).unwrap();
    }
}

