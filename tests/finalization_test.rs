// Test to validate WAV file finalization behavior
// This simulates the Arc unwrapping issue

use std::io::Read;
use std::sync::Arc;
use std::sync::Mutex;
use hound::{WavWriter, WavSpec, SampleFormat};

#[test]
fn test_wav_writer_finalization() {
    let test_file = "test_finalization.wav";
    let spec = WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    
    // Create writer wrapped in Arc<Mutex<>>
    let writer = Arc::new(Mutex::new(WavWriter::create(test_file, spec).unwrap()));
    
    // Write some data
    {
        let mut w = writer.lock().unwrap();
        for i in 0..1000 {
            w.write_sample((i as i16) % 1000).unwrap();
        }
    }
    
    // Simulate the finalization process
    // If we can unwrap, finalize should work
    match Arc::try_unwrap(writer) {
        Ok(w) => {
            w.into_inner().unwrap().finalize().unwrap();
            
            // Verify file is valid
            let metadata = std::fs::metadata(test_file).unwrap();
            assert!(metadata.len() > 44, "File should have data beyond headers");
            
            // Verify it's a valid WAV file
            let mut file = std::fs::File::open(test_file).unwrap();
            let mut buffer = [0u8; 12];
            file.read_exact(&mut buffer).unwrap();
            
            assert_eq!(&buffer[0..4], b"RIFF", "Should have RIFF header");
            assert_eq!(&buffer[8..12], b"WAVE", "Should have WAVE identifier");
            
            std::fs::remove_file(test_file).unwrap();
        }
        Err(_) => {
            panic!("Arc should be unwrappable when no other references exist");
        }
    }
}

#[test]
fn test_wav_writer_with_held_reference() {
    // Test what happens when Arc is held by another reference
    let test_file = "test_held_ref.wav";
    let spec = WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    
    let writer = Arc::new(Mutex::new(WavWriter::create(test_file, spec).unwrap()));
    
    // Create another reference (simulating callback holding it)
    let writer_clone = Arc::clone(&writer);
    
    // Write data
    {
        let mut w = writer.lock().unwrap();
        for i in 0..1000 {
            w.write_sample((i as i16) % 1000).unwrap();
        }
    }
    
    // Try to unwrap - should fail because clone exists
    match Arc::try_unwrap(writer) {
        Ok(_) => {
            panic!("Should not be able to unwrap when clone exists");
        }
        Err(arc) => {
            // This is expected - we need to drop the clone first
            drop(writer_clone);
            
            // Now it should work
            match Arc::try_unwrap(arc) {
                Ok(w) => {
                    w.into_inner().unwrap().finalize().unwrap();
                    std::fs::remove_file(test_file).unwrap();
                }
                Err(_) => {
                    panic!("Should be able to unwrap after clone is dropped");
                }
            }
        }
    }
}

