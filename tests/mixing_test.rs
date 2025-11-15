// Test to validate audio mixing logic

#[test]
fn test_stereo_mixing() {
    // Test that mixing two stereo samples works correctly
    let mic_left = 1000i16;
    let mic_right = 2000i16;
    let sys_left = 3000i16;
    let sys_right = 4000i16;
    
    let mixed_left = (mic_left as i32 + sys_left as i32)
        .clamp(i16::MIN as i32, i16::MAX as i32) as i16;
    let mixed_right = (mic_right as i32 + sys_right as i32)
        .clamp(i16::MIN as i32, i16::MAX as i32) as i16;
    
    assert_eq!(mixed_left, 4000);
    assert_eq!(mixed_right, 6000);
}

#[test]
fn test_mixing_with_clipping() {
    // Test that mixing prevents clipping
    let mic = 20000i16;
    let sys = 20000i16;
    
    let mixed = (mic as i32 + sys as i32)
        .clamp(i16::MIN as i32, i16::MAX as i32) as i16;
    
    assert_eq!(mixed, i16::MAX); // Should clamp to max
}

#[test]
fn test_mono_to_stereo_conversion() {
    // Test mono to stereo conversion
    let mono_samples = vec![1000i16, 2000i16, 3000i16];
    let stereo: Vec<i16> = mono_samples.iter().flat_map(|&s| [s, s]).collect();
    
    assert_eq!(stereo.len(), 6);
    assert_eq!(stereo, vec![1000, 1000, 2000, 2000, 3000, 3000]);
}

