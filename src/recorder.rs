use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::SupportedStreamConfig;
use hound::{WavSpec, WavWriter, SampleFormat};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::SystemTime;
use crate::config::Config;

/// Main recorder that handles audio recording from devices
pub struct Recorder {
    mic_device: cpal::Device,
    mic_config: SupportedStreamConfig,
    sys_device: Option<cpal::Device>,
    sys_config: Option<SupportedStreamConfig>,
    running: Arc<AtomicBool>,
}

impl Recorder {
    /// Create a new Recorder
    pub fn new(
        mic_device: cpal::Device,
        mic_config: SupportedStreamConfig,
        sys_device: Option<cpal::Device>,
        sys_config: Option<SupportedStreamConfig>,
    ) -> Self {
        Self {
            mic_device,
            mic_config,
            sys_device,
            sys_config,
            running: Arc::new(AtomicBool::new(true)),
        }
    }
    
    /// Record audio to a single combined WAV file
    pub fn record(&self, config: &Config) -> Result<RecordingResult, Box<dyn std::error::Error>> {
        // Format timestamp as dd-mm-yyyy-hh-mm
        let now = SystemTime::now();
        let datetime = now.duration_since(std::time::UNIX_EPOCH)?;
        let secs = datetime.as_secs();
        
        // Convert to local time components
        // Note: This uses UTC. For local time, we'd need chrono crate.
        // Using UTC for simplicity and consistency across platforms.
        let days = secs / 86400;
        let secs_in_day = secs % 86400;
        
        // Calculate date (simplified - doesn't account for leap years perfectly)
        let mut year = 1970;
        let mut day_of_year = days as i64;
        while day_of_year >= 365 {
            let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
            let days_in_year = if is_leap { 366 } else { 365 };
            if day_of_year >= days_in_year {
                day_of_year -= days_in_year;
                year += 1;
            } else {
                break;
            }
        }
        
        // Calculate month and day
        let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let days_in_months = if is_leap {
            [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        } else {
            [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        };
        
        let mut month = 1;
        let mut day = day_of_year as u32 + 1;
        for &days_in_month in &days_in_months {
            if day > days_in_month {
                day -= days_in_month;
                month += 1;
            } else {
                break;
            }
        }
        
        // Calculate hours and minutes
        let hours = (secs_in_day / 3600) as u32;
        let minutes = ((secs_in_day % 3600) / 60) as u32;
        
        // Format as mm-dd-yyyy-24h-m-recording.wav
        let filename = format!("{:02}-{:02}-{}-{:02}-{:02}-recording.wav", month, day, year, hours, minutes);
        let combined_path = config.recording_path(&filename);
        let combined_filename = combined_path.to_string_lossy().to_string();
        
        let mic_sample_rate = self.mic_config.sample_rate().0;
        let mic_channels = self.mic_config.channels() as u16;
        
        // Determine output format - use higher sample rate, stereo
        let (sys_sample_rate, sys_channels) = if let Some(config) = self.sys_config.as_ref() {
            (config.sample_rate().0, config.channels() as u16)
        } else {
            (mic_sample_rate, 1)
        };
        
        let output_sample_rate = mic_sample_rate.max(sys_sample_rate);
        let output_channels = 2u16; // Always stereo for combined output
        
        let combined_spec = WavSpec {
            channels: output_channels,
            sample_rate: output_sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        
        // Create channels for sample data (callback doesn't hold WavWriter Arc)
        let (mic_tx, mic_rx) = mpsc::channel::<Vec<i16>>();
        let (sys_tx, sys_rx) = if self.sys_device.is_some() {
            let (tx, rx) = mpsc::channel::<Vec<i16>>();
            (Some(tx), Some(rx))
        } else {
            (None, None)
        };
        
        // Create single combined WAV writer
        let combined_writer = WavWriter::create(&combined_filename, combined_spec)?;
        
        // Setup signal handler for Ctrl+C
        let r = self.running.clone();
        ctrlc::set_handler(move || {
            println!("\n\nStopping recording...");
            r.store(false, Ordering::SeqCst);
        })?;
        
        // Start mixer thread - mixes samples from both sources into single file
        let mic_running = self.running.clone();
        let mic_ch = mic_channels;
        let sys_ch = sys_channels;
        
        let mixer_handle = thread::spawn(move || {
            let mut writer = combined_writer;
            let mut mic_buffer: Vec<i16> = Vec::new();
            let mut sys_buffer: Vec<i16> = Vec::new();
            let mut mic_samples_received = 0u64;
            let mut sys_samples_received = 0u64;
            let mut samples_written = 0u64;
            
            loop {
                // Receive samples from both sources
                let mut received_any = false;
                
                // Try to get mic samples
                while let Ok(samples) = mic_rx.try_recv() {
                    received_any = true;
                    mic_samples_received += samples.len() as u64;
                    // Convert to stereo if needed
                    let stereo_samples: Vec<i16> = if mic_ch == 1 {
                        samples.iter().flat_map(|&s| [s, s]).collect()
                    } else {
                        samples
                    };
                    mic_buffer.extend(stereo_samples);
                }
                
                // Try to get system audio samples
                if let Some(ref rx) = sys_rx {
                    while let Ok(samples) = rx.try_recv() {
                        received_any = true;
                        sys_samples_received += samples.len() as u64;
                        // Convert to stereo if needed
                        let stereo_samples: Vec<i16> = if sys_ch == 1 {
                            samples.iter().flat_map(|&s| [s, s]).collect()
                        } else {
                            samples
                        };
                        sys_buffer.extend(stereo_samples);
                    }
                }
                
                // Mix and write samples - mix corresponding samples together
                // For stereo: mix left with left, right with right
                // Write as many samples as we can from both buffers
                let min_len = mic_buffer.len().min(sys_buffer.len());
                if min_len >= 2 {
                    // Ensure we mix in stereo pairs (left, right)
                    let pairs = min_len / 2;
                    for i in 0..pairs {
                        let mic_left = mic_buffer[i * 2];
                        let mic_right = mic_buffer[i * 2 + 1];
                        let sys_left = sys_buffer[i * 2];
                        let sys_right = sys_buffer[i * 2 + 1];
                        
                        // Mix left channels
                        let mixed_left = (mic_left as i32 + sys_left as i32)
                            .clamp(i16::MIN as i32, i16::MAX as i32) as i16;
                        // Mix right channels
                        let mixed_right = (mic_right as i32 + sys_right as i32)
                            .clamp(i16::MIN as i32, i16::MAX as i32) as i16;
                        
                        writer.write_sample(mixed_left).unwrap();
                        writer.write_sample(mixed_right).unwrap();
                        samples_written += 2;
                    }
                    mic_buffer.drain(0..pairs * 2);
                    sys_buffer.drain(0..pairs * 2);
                }
                
                // If one buffer has more data than the other, write what we can
                // This handles cases where one source is faster than the other
                if mic_buffer.len() >= 2 && sys_buffer.is_empty() {
                    // Only mic data available - write it
                    let pairs = mic_buffer.len() / 2;
                    for i in 0..pairs {
                        writer.write_sample(mic_buffer[i * 2]).unwrap();
                        writer.write_sample(mic_buffer[i * 2 + 1]).unwrap();
                        samples_written += 2;
                    }
                    mic_buffer.drain(0..pairs * 2);
                } else if sys_buffer.len() >= 2 && mic_buffer.is_empty() {
                    // Only system data available - write it
                    let pairs = sys_buffer.len() / 2;
                    for i in 0..pairs {
                        writer.write_sample(sys_buffer[i * 2]).unwrap();
                        writer.write_sample(sys_buffer[i * 2 + 1]).unwrap();
                        samples_written += 2;
                    }
                    sys_buffer.drain(0..pairs * 2);
                }
                
                // Check if we should exit
                if !mic_running.load(Ordering::SeqCst) && !received_any {
                    // Drain remaining buffers - mix any remaining samples
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
                    // Write any remaining unpaired samples
                    if mic_buffer.len() > pairs * 2 {
                        for &sample in mic_buffer.iter().skip(pairs * 2) {
                            writer.write_sample(sample).unwrap();
                        }
                    }
                    if sys_buffer.len() > pairs * 2 {
                        for &sample in sys_buffer.iter().skip(pairs * 2) {
                            writer.write_sample(sample).unwrap();
                        }
                    }
                    break;
                }
                
                if !received_any {
                    thread::sleep(std::time::Duration::from_millis(10));
                }
            }
            
            writer.finalize().unwrap();
            eprintln!("Mixer stats: mic_samples={}, sys_samples={}, written={}", 
                     mic_samples_received, sys_samples_received, samples_written);
        });
        
        // Build microphone stream - callback sends to channel
        let mic_tx_clone = mic_tx.clone();
        let mic_running = self.running.clone();
        
        let mic_stream = self.mic_device.build_input_stream(
            &self.mic_config.clone().into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if !mic_running.load(Ordering::SeqCst) {
                    return;
                }
                
                let samples: Vec<i16> = data.iter()
                    .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
                    .collect();
                
                if let Err(e) = mic_tx_clone.send(samples) {
                    eprintln!("Error sending mic samples: {}", e);
                }
            },
            |err| eprintln!("Microphone stream error: {}", err),
            None,
        )?;
        
        // Build system audio stream if selected  
        let sys_stream = if let (Some(dev), Some(config), Some(tx)) = 
            (self.sys_device.as_ref(), self.sys_config.as_ref(), sys_tx.as_ref()) {
            let sys_tx_clone = tx.clone();
            let sys_running = self.running.clone();
            
            let stream = dev.build_input_stream(
                &config.clone().into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if !sys_running.load(Ordering::SeqCst) {
                        return;
                    }
                    
                    let samples: Vec<i16> = data.iter()
                        .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
                        .collect();
                    
                    if let Err(e) = sys_tx_clone.send(samples) {
                        eprintln!("Error sending system audio samples: {}", e);
                    }
                },
                |err| eprintln!("System audio stream error: {}", err),
                None,
            )?;
            
            Some(stream)
        } else {
            None
        };
        
        // Start recording
        println!("\n=== Recording Started ===");
        println!("Recording to: {}", combined_filename);
        println!("Format: {} channels, {} Hz", output_channels, output_sample_rate);
        println!("Microphone: {} channels, {} Hz", mic_channels, mic_sample_rate);
        if let Some(config) = self.sys_config.as_ref() {
            println!("System audio: {} channels, {} Hz", config.channels(), config.sample_rate().0);
        }
        println!("\nPress Ctrl+C to stop recording...\n");
        
        mic_stream.play()?;
        match &sys_stream {
            Some(stream) => stream.play()?,
            None => {}
        }
        
        // Wait until Ctrl+C
        while self.running.load(Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        
        // Stop streams
        mic_stream.pause()?;
        match &sys_stream {
            Some(stream) => stream.pause()?,
            None => {}
        }
        
        // Drop streams and channels to signal completion
        drop(mic_stream);
        drop(mic_tx);
        drop(sys_stream);
        if let Some(tx) = sys_tx {
            drop(tx);
        }
        
        // Wait for mixer thread to finish and finalize
        mixer_handle.join()
            .map_err(|_| "Failed to join mixer thread")?;
        
        println!("\n=== Recording Complete ===");
        println!("Saved recording: {}", combined_filename);
        
        // Check file size
        let file_size = std::fs::metadata(&combined_filename)?.len();
        println!("\nFile size: {} bytes ({:.2} KB)", file_size, file_size as f64 / 1024.0);
        
        Ok(RecordingResult {
            filename: combined_filename,
        })
    }
    
    /// Stop the recording
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

/// Result of a recording session
#[derive(Debug)]
pub struct RecordingResult {
    pub filename: String,
}

