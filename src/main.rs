use meeting_recorder::{DeviceManager, Recorder, Config};
use meeting_recorder::input::{read_index, read_index_optional};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Meeting Recorder - Capturing microphone and system audio");
    println!("========================================================\n");
    
    // Load configuration
    let config = Config::load()?;
    println!("Output directory: {}\n", config.output_directory);

    let device_manager = DeviceManager::new()?;
    device_manager.list_devices()?;

    // Get device selections
    println!("\nSelect microphone device (index):");
    let mic_idx = read_index(device_manager.device_count())?;
    let mic_name = device_manager.device_name(mic_idx)?;
    println!("Selected microphone: {}\n", mic_name);

    println!("Select system audio device (index, or -1 to skip):");
    let sys_idx = read_index_optional(device_manager.device_count())?;
    
    if let Some(idx) = sys_idx {
        let name = device_manager.device_name(idx)?;
        println!("Selected system audio: {}\n", name);
    } else {
        println!("System audio recording skipped.\n");
    }

    // Get device configurations
    let mic_config = device_manager.device_config(mic_idx)?;
    let mic_sample_rate = mic_config.sample_rate().0;
    let mic_channels = mic_config.channels() as u16;

    println!("Microphone config: {} channels, {} Hz", mic_channels, mic_sample_rate);

    let sys_config = sys_idx
        .and_then(|idx| device_manager.device_config(idx).ok());

    if let Some(config) = sys_config.as_ref() {
        let sys_sample_rate = config.sample_rate().0;
        let sys_channels = config.channels() as u16;
        println!("System audio config: {} channels, {} Hz", sys_channels, sys_sample_rate);
    }

    // Create recorder and start recording
    // Take ownership of devices from the manager
    let mut device_manager = device_manager;
    let mic_device = device_manager.take_device(mic_idx)
        .ok_or_else(|| format!("Failed to get microphone device at index {}", mic_idx))?;
    
    let sys_device = if let Some(idx) = sys_idx {
        device_manager.take_device(idx)
    } else {
        None
    };
    
    let recorder = Recorder::new(
        mic_device,
        mic_config,
        sys_device,
        sys_config,
    );
    
    recorder.record(&config)?;

    Ok(())
}
