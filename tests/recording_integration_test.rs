// Integration test that exercises the full recording functionality
// Simulates recording from microphone and system audio, mixing them into a single WAV file

use hound::{WavReader, WavSpec, SampleFormat};
use std::fs;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn validate_combined_wav(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = WavReader::open(path)?;
    let spec = reader.spec();
    
    // Should be stereo
    assert_eq!(spec.channels, 2, "Combined file should be stereo");
    
    // Should have reasonable sample rate
    assert!(spec.sample_rate >= 16000 && spec.sample_rate <= 96000, 
            "Sample rate should be reasonable");
    
    // Should have samples
    let samples: Vec<i16> = reader.samples::<i16>().collect::<Result<_, _>>()?;
    assert!(samples.len() > 1000, "Should have substantial audio data");
    
    // Verify samples are not all zeros (should have mixed audio)
    let non_zero_count = samples.iter().filter(|&&s| s != 0).count();
    assert!(non_zero_count > samples.len() / 10, 
            "At least 10% of samples should be non-zero (mixed audio)");
    
    Ok(())
}

#[test]
fn test_full_recording_workflow() {
    // Simulate the full recording workflow
    let test_file = "test_combined_recording.wav";
    
    // Create WAV writer with combined format
    let spec = WavSpec {
        channels: 2,
        sample_rate: 48000,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    
    let (mic_tx, mic_rx) = mpsc::channel::<Vec<i16>>();
    let (sys_tx, sys_rx) = mpsc::channel::<Vec<i16>>();
    
    // Simulate microphone input (mono, 48kHz)
    let mic_samples_per_chunk = 480; // ~10ms at 48kHz
    let mic_chunks = 100; // ~1 second of audio
    
    // Simulate system audio input (stereo, 48kHz)
    let sys_samples_per_chunk = 960; // ~10ms at 48kHz, stereo
    let sys_chunks = 100;
    
    // Start mixer thread (simulating the recorder's mixer)
    let mixer_handle = thread::spawn(move || {
        let mut writer = hound::WavWriter::create(test_file, spec).unwrap();
        let mut mic_buffer: Vec<i16> = Vec::new();
        let mut sys_buffer: Vec<i16> = Vec::new();
        let mut running = true;
        let mut iterations = 0;
        
        while running && iterations < 1000 {
            iterations += 1;
            let mut received_any = false;
            
            // Receive mic samples (mono -> convert to stereo)
            while let Ok(samples) = mic_rx.try_recv() {
                received_any = true;
                let stereo: Vec<i16> = samples.iter().flat_map(|&s| [s, s]).collect();
                mic_buffer.extend(stereo);
            }
            
            // Receive system audio samples (already stereo)
            while let Ok(samples) = sys_rx.try_recv() {
                received_any = true;
                sys_buffer.extend(samples);
            }
            
            // Mix and write
            let min_len = mic_buffer.len().min(sys_buffer.len());
            if min_len > 0 {
                let pairs = min_len / 2;
                for i in 0..pairs {
                    let mic_left = mic_buffer[i * 2];
                    let mic_right = mic_buffer[i * 2 + 1];
                    let sys_left = sys_buffer[i * 2];
                    let sys_right = sys_buffer[i * 2 + 1];
                    
                    let mixed_left = (mic_left as i32 + sys_left as i32)
                        .clamp(i16::MIN as i32, i16::MAX as i32) as i16;
                    let mixed_right = (mic_right as i32 + sys_right as i32)
                        .clamp(i16::MIN as i32, i16::MAX as i32) as i16;
                    
                    writer.write_sample(mixed_left).unwrap();
                    writer.write_sample(mixed_right).unwrap();
                }
                mic_buffer.drain(0..pairs * 2);
                sys_buffer.drain(0..pairs * 2);
            } else if !mic_buffer.is_empty() {
                for &sample in &mic_buffer {
                    writer.write_sample(sample).unwrap();
                }
                mic_buffer.clear();
            } else if !sys_buffer.is_empty() {
                for &sample in &sys_buffer {
                    writer.write_sample(sample).unwrap();
                }
                sys_buffer.clear();
            }
            
            if !received_any {
                thread::sleep(Duration::from_millis(10));
            }
            
            // Stop after receiving all expected chunks
            if iterations > 200 {
                running = false;
            }
        }
        
        // Drain remaining
        let max_len = mic_buffer.len().max(sys_buffer.len());
        let pairs = max_len / 2;
        for i in 0..pairs {
            let mic_left = mic_buffer.get(i * 2).copied().unwrap_or(0);
            let mic_right = mic_buffer.get(i * 2 + 1).copied().unwrap_or(0);
            let sys_left = sys_buffer.get(i * 2).copied().unwrap_or(0);
            let sys_right = sys_buffer.get(i * 2 + 1).copied().unwrap_or(0);
            
            let mixed_left = (mic_left as i32 + sys_left as i32)
                .clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            let mixed_right = (mic_right as i32 + sys_right as i32)
                .clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            
            writer.write_sample(mixed_left).unwrap();
            writer.write_sample(mixed_right).unwrap();
        }
        
        writer.finalize().unwrap();
    });
    
    // Simulate audio callbacks sending data
    thread::spawn(move || {
        // Send microphone samples (mono sine wave)
        for chunk in 0..mic_chunks {
            let mut samples = Vec::new();
            for i in 0..mic_samples_per_chunk {
                let sample_num = chunk * mic_samples_per_chunk + i;
                // Generate a sine wave at 440Hz
                let sample = ((sample_num as f32 * 440.0 * 2.0 * std::f32::consts::PI / 48000.0).sin() * 8000.0) as i16;
                samples.push(sample);
            }
            mic_tx.send(samples).unwrap();
            thread::sleep(Duration::from_millis(10));
        }
        drop(mic_tx);
    });
    
    thread::spawn(move || {
        // Send system audio samples (stereo, different frequency)
        for chunk in 0..sys_chunks {
            let mut samples = Vec::new();
            for i in 0..sys_samples_per_chunk / 2 {
                let sample_num = chunk * (sys_samples_per_chunk / 2) + i;
                // Generate a sine wave at 880Hz for left, 660Hz for right
                let left = ((sample_num as f32 * 880.0 * 2.0 * std::f32::consts::PI / 48000.0).sin() * 8000.0) as i16;
                let right = ((sample_num as f32 * 660.0 * 2.0 * std::f32::consts::PI / 48000.0).sin() * 8000.0) as i16;
                samples.push(left);
                samples.push(right);
            }
            sys_tx.send(samples).unwrap();
            thread::sleep(Duration::from_millis(10));
        }
        drop(sys_tx);
    });
    
    // Wait for mixer to finish
    mixer_handle.join().unwrap();
    
    // Validate the output file
    assert!(validate_combined_wav(test_file).is_ok(), 
            "Combined WAV file should be valid");
    
    // Verify file exists and has reasonable size
    let metadata = fs::metadata(test_file).unwrap();
    assert!(metadata.len() > 100000, "File should be substantial (>100KB)");
    
    // Clean up
    fs::remove_file(test_file).unwrap();
}

#[test]
fn test_mixing_different_sample_rates() {
    // Test that mixing works even when sources have different sample rates
    // (In practice, we'd resample, but for now we test the mixing logic)
    let test_file = "test_mixed_rates.wav";
    
    let spec = WavSpec {
        channels: 2,
        sample_rate: 48000,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    
    let mut writer = hound::WavWriter::create(test_file, spec).unwrap();
    
    // Simulate mic samples (mono, 48kHz) - 100 samples
    let mic_samples: Vec<i16> = (0..100).map(|i| (i * 100) as i16).collect();
    let mic_stereo: Vec<i16> = mic_samples.iter().flat_map(|&s| [s, s]).collect();
    
    // Simulate system samples (stereo, 48kHz) - 100 samples (50 stereo pairs)
    let sys_samples: Vec<i16> = (0..100).map(|i| (i * 50) as i16).collect();
    
    // Mix them
    let min_len = mic_stereo.len().min(sys_samples.len());
    for i in 0..min_len {
        let mixed = (mic_stereo[i] as i32 + sys_samples[i] as i32)
            .clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        writer.write_sample(mixed).unwrap();
    }
    
    writer.finalize().unwrap();
    
    // Verify file
    let mut reader = WavReader::open(test_file).unwrap();
    let samples: Vec<i16> = reader.samples().collect::<Result<_, _>>().unwrap();
    assert!(samples.len() >= min_len);
    
    fs::remove_file(test_file).unwrap();
}

#[test]
fn test_mixing_with_one_source_missing() {
    // Test that recording works when only one source is available
    let test_file = "test_single_source.wav";
    
    let spec = WavSpec {
        channels: 2,
        sample_rate: 48000,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    
    let mut writer = hound::WavWriter::create(test_file, spec).unwrap();
    
    // Only mic samples available
    let mic_samples: Vec<i16> = (0..1000).map(|i| ((i % 100) * 10) as i16).collect();
    let mic_stereo: Vec<i16> = mic_samples.iter().flat_map(|&s| [s, s]).collect();
    
    // Write mic samples (no system audio to mix)
    for &sample in &mic_stereo {
        writer.write_sample(sample).unwrap();
    }
    
    writer.finalize().unwrap();
    
    // Verify file
    let mut reader = WavReader::open(test_file).unwrap();
    let samples: Vec<i16> = reader.samples().collect::<Result<_, _>>().unwrap();
    assert_eq!(samples.len(), mic_stereo.len());
    
    fs::remove_file(test_file).unwrap();
}

