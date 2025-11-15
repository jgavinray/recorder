use cpal::traits::{DeviceTrait, HostTrait};
use cpal::SupportedStreamConfig;

/// Manages audio device enumeration and selection
pub struct DeviceManager {
    devices: Vec<cpal::Device>,
}

impl DeviceManager {
    /// Create a new DeviceManager
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let devices: Vec<_> = host.input_devices()?.collect();
        
        if devices.is_empty() {
            return Err("No input devices found".into());
        }
        
        Ok(Self { devices })
    }
    
    /// List all available input devices
    pub fn list_devices(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Available input devices:");
        for (i, device) in self.devices.iter().enumerate() {
            let name = device.name()?;
            let config = device.default_input_config().ok();
            let info = if let Some(cfg) = config {
                format!(" ({} ch, {} Hz)", cfg.channels(), cfg.sample_rate().0)
            } else {
                String::new()
            };
            println!("  {}: {}{}", i, name, info);
        }
        Ok(())
    }
    
    /// Get a device by index (takes ownership)
    pub fn take_device(&mut self, index: usize) -> Option<cpal::Device> {
        if index < self.devices.len() {
            Some(self.devices.remove(index))
        } else {
            None
        }
    }
    
    /// Get a device reference by index
    pub fn get_device(&self, index: usize) -> Option<&cpal::Device> {
        self.devices.get(index)
    }
    
    /// Get the number of available devices
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }
    
    /// Get device name
    pub fn device_name(&self, index: usize) -> Result<String, Box<dyn std::error::Error>> {
        self.devices
            .get(index)
            .ok_or_else(|| format!("Device index {} out of range", index).into())
            .and_then(|d| d.name().map_err(|e| e.into()))
    }
    
    /// Get device configuration
    pub fn device_config(&self, index: usize) -> Result<SupportedStreamConfig, Box<dyn std::error::Error>> {
        self.devices
            .get(index)
            .ok_or_else(|| format!("Device index {} out of range", index).into())
            .and_then(|d| d.default_input_config().map_err(|e| e.into()))
    }
}

