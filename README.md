# Meeting Recorder

A Rust program that records both microphone input and system audio (e.g., from Teams/Zoom meetings) on macOS, Windows, and Linux, saving them to a single combined WAV file.

## Features

- Records microphone input and system audio simultaneously
- Saves recordings to a single combined WAV file with timestamps
- Cross-platform support (macOS, Windows, and Linux)
- Interactive device selection
- Clean shutdown with Ctrl+C (or Ctrl+Break on Windows)
- YAML configuration for output directory
- Minimal dependencies

## Requirements

### macOS

**System Audio Capture:**
macOS does not allow direct capture of system audio output due to security restrictions. You need to install a virtual audio driver that creates a loopback device.

**Option 1: BlackHole (Recommended)**
```bash
brew install --cask blackhole-2ch
```

After installation:
1. Open System Settings > Sound > Output
2. Select "BlackHole 2ch" as your output device
3. Open System Settings > Sound > Input
4. You should see "BlackHole 2ch" in the input devices list

**Option 2: Soundflower**
```bash
brew install --cask soundflower
```

**Option 3: Loopback (Paid)**
- Download from: https://rogueamoeba.com/loopback/

**Why Virtual Audio Drivers are Necessary:**
macOS sandboxing prevents applications from directly accessing system audio output. Virtual audio drivers create a loopback device that routes system audio back into an input channel, making it recordable by standard audio APIs.

### Windows

**System Audio Capture:**
Windows supports system audio capture natively through WASAPI loopback devices. No additional drivers are required.

1. **Enable Stereo Mix (if available):**
   - Right-click the speaker icon in the system tray
   - Select "Sounds" → "Recording" tab
   - Right-click in the empty space → "Show Disabled Devices"
   - Enable "Stereo Mix" if it appears
   - Note: Modern Windows versions may not have Stereo Mix

2. **Use WASAPI Loopback (Recommended):**
   - The recorder will automatically detect WASAPI loopback devices
   - Select a loopback device (e.g., "Speakers (WASAPI Loopback)") as your system audio input
   - These devices capture audio from your default playback device

3. **Alternative: Use VB-Audio Cable (if needed):**
   - Download from: https://vb-audio.com/Cable/
   - Install and set as default playback device
   - Select "CABLE Input" as system audio device in the recorder

### Linux

**System Audio Capture:**
On Linux, you can use PulseAudio's loopback module:

```bash
# Install PulseAudio (if not already installed)
sudo apt-get install pulseaudio pulseaudio-utils  # Debian/Ubuntu
# or
sudo dnf install pulseaudio pulseaudio-utils     # Fedora

# Create a loopback sink
pactl load-module module-loopback
```

Alternatively, you can use JACK or ALSA loopback devices.

**System Dependencies:**
- ALSA development libraries (for cpal):
  ```bash
  sudo apt-get install libasound2-dev  # Debian/Ubuntu
  sudo dnf install alsa-lib-devel      # Fedora
  ```

### Rust

Install Rust from https://rustup.rs/:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Configuration

The application requires a YAML configuration file at a platform-specific location:

### macOS/Linux
```bash
sudo mkdir -p /opt/meeting-recorder
sudo cp config.yaml.example /opt/meeting-recorder/config.yaml
sudo nano /opt/meeting-recorder/config.yaml  # Edit the output_directory
```

Config location: `/opt/meeting-recorder/config.yaml`

### Windows
```powershell
# Run PowerShell as Administrator
New-Item -ItemType Directory -Force -Path "$env:PROGRAMDATA\meeting-recorder"
Copy-Item config.yaml.example "$env:PROGRAMDATA\meeting-recorder\config.yaml"
notepad "$env:PROGRAMDATA\meeting-recorder\config.yaml"  # Edit the output_directory
```

Config location: `%PROGRAMDATA%\meeting-recorder\config.yaml` (typically `C:\ProgramData\meeting-recorder\config.yaml`)

### Example `config.yaml`:
```yaml
output_directory: /var/recordings/meetings  # macOS/Linux
# or
output_directory: C:\Recordings\Meetings    # Windows
```

The `output_directory` will be created automatically if it doesn't exist.

## Building

```bash
cargo build --release
```

The binary will be at `target/release/meeting-recorder`.

## Testing

The project includes a comprehensive test suite that validates WAV file structure and format:

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_test
```

The tests validate:
- WAV file structure (RIFF/WAVE headers)
- Format chunk presence
- Data beyond headers
- Different audio formats (mono/stereo, various sample rates)

## Usage

1. **Run the program:**
   ```bash
   ./target/release/meeting-recorder
   ```

2. **Select your microphone:**
   - The program will list all available input devices
   - Enter the index number of your microphone

3. **Select system audio device:**
   - Enter the index of your virtual audio device (e.g., "BlackHole 2ch")
   - Or enter `-1` to skip system audio recording

4. **Start recording:**
   - The program will begin recording immediately
   - Speak into your microphone and play audio on your system
   - Press `Ctrl+C` to stop recording

5. **Find your recordings:**
   - Microphone: `mic_recording_<timestamp>.wav`
   - System audio: `system_recording_<timestamp>.wav`

## Example Session

```
Meeting Recorder - Capturing microphone and system audio
========================================================

Available input devices:
  0: MacBook Pro Microphone (1 ch, 48000 Hz)
  1: BlackHole 2ch (2 ch, 48000 Hz)

Select microphone device (index):
Enter index: 0
Selected microphone: MacBook Pro Microphone

Select system audio device (index, or -1 to skip):
Enter index: 1
Selected system audio: BlackHole 2ch

Microphone config: 1 channels, 48000 Hz
System audio config: 2 channels, 48000 Hz

=== Recording Started ===
Recording microphone to: mic_recording_1700000000.wav
Recording system audio to: system_recording_1700000000.wav

Press Ctrl+C to stop recording...

^C

Stopping recording...

=== Recording Complete ===
Saved microphone recording: mic_recording_1700000000.wav
Saved system audio recording: system_recording_1700000000.wav

File sizes:
  mic_recording_1700000000.wav: 960000 bytes (937.50 KB)
  system_recording_1700000000.wav: 1920000 bytes (1875.00 KB)
```

## Setting Up for Teams/Zoom Meetings

### Windows with WASAPI Loopback:

1. **Start the recorder:**
   - Run `meeting-recorder.exe`
   - Select your microphone device
   - Select a WASAPI loopback device (e.g., "Speakers (WASAPI Loopback)") as system audio
   - The loopback device will capture audio from your default playback device

2. **In Teams/Zoom:**
   - Use your normal audio output (no special configuration needed)
   - The loopback device automatically captures system audio

3. **Alternative with VB-Audio Cable:**
   - Install VB-Audio Cable from https://vb-audio.com/Cable/
   - Set Teams/Zoom output to "CABLE Input"
   - Select "CABLE Input" as system audio device in recorder

### macOS with BlackHole:

**Option 1: Multi-Output Device (RECOMMENDED - You can hear audio while recording)**

1. **Create Multi-Output Device:**
   - Open Audio MIDI Setup (Applications → Utilities → Audio MIDI Setup)
   - Click `+` button → "Create Multi-Output Device"
   - Check boxes for: Your speakers/headphones AND "BlackHole 2ch"
   - Both should be enabled

2. **Set as System Output:**
   - System Settings → Sound → Output → Select "Multi-Output Device"
   - Audio will play through BOTH your speakers AND BlackHole

3. **Run the recorder:**
   - Select your microphone as mic input
   - Select "BlackHole 2ch" as system audio input
   - You'll hear everything AND it will be recorded!

**Option 2: BlackHole Only (You WON'T hear audio)**

1. **Before starting the meeting:**
   - Set your system output to "BlackHole 2ch"
   - In Teams/Zoom, set the output to "BlackHole 2ch" as well
   - Start the meeting recorder and select "BlackHole 2ch" as the system audio device

2. **Alternative (if you want to hear audio):**
   - Use a multi-output device or audio routing software
   - Or use BlackHole 16ch and create an aggregate device in Audio MIDI Setup

### Linux with PulseAudio:

1. **Create a null sink and loopback:**
   ```bash
   pactl load-module module-null-sink sink_name=meeting_output
   pactl load-module module-loopback source=meeting_output.monitor
   ```

2. **Set Teams/Zoom output to "meeting_output"**

3. **Select the loopback device in the recorder**

## How It Works

1. **Device Enumeration:** Uses the `cpal` crate to list all available audio input devices on the system.

2. **Stream Setup:** Creates separate audio input streams for the microphone and system audio device using:
   - CoreAudio on macOS
   - WASAPI on Windows
   - ALSA/PulseAudio on Linux

3. **Simultaneous Recording:** Both streams run concurrently, mixing audio samples in real-time and writing to a single combined WAV file using the `hound` crate.

4. **Sample Conversion:** Converts floating-point samples from the audio API to 16-bit integers for WAV file format.

5. **Clean Shutdown:** Uses signal handling to gracefully stop recording when Ctrl+C is pressed, ensuring WAV files are properly finalized.

## Dependencies

### Rust Crates (in Cargo.toml):

- **cpal (0.15)**: Cross-platform audio I/O library
  - Provides device enumeration and audio stream creation
  - Uses CoreAudio on macOS, ALSA/PulseAudio on Linux
  - Pure Rust implementation

- **hound (3.5)**: WAV file writer
  - Handles WAV file format encoding
  - Writes 16-bit PCM audio data

- **ctrlc (3.4)**: Signal handling for Ctrl+C
  - Allows graceful program shutdown
  - Cross-platform signal handling

### External System Dependencies:

**macOS:**
- Virtual audio driver (BlackHole, Soundflower, or Loopback) - **REQUIRED** for system audio capture
- CoreAudio framework (built into macOS)

**Windows:**
- No additional drivers required (WASAPI loopback is built-in)
- Optional: VB-Audio Cable for alternative loopback method

**Linux:**
- ALSA development libraries (`libasound2-dev`)
- PulseAudio (optional, for easier loopback setup)

## Troubleshooting

### "No input devices found"
- Check that your microphone is connected and recognized by the system
- On macOS, grant microphone permissions in System Settings > Privacy & Security > Microphone

### System audio not recording
- Ensure your virtual audio driver is installed and active
- Set your system output to the virtual audio device
- Verify the device appears in the input device list
- On macOS, you may need to grant microphone permissions to the virtual device

### WAV files are empty or very small
- Check that audio is actually playing through the selected output device
- Verify the selected devices are correct
- Ensure you're recording for at least a few seconds

### Permission errors on macOS
- Grant microphone access in System Settings > Privacy & Security > Microphone
- You may need to restart the application after granting permissions

## Combining Recordings

The program saves microphone and system audio to separate files. To combine them:

**Using ffmpeg:**
```bash
ffmpeg -i mic_recording_<timestamp>.wav -i system_recording_<timestamp>.wav \
  -filter_complex "[0:a][1:a]amerge=inputs=2[a]" -map "[a]" \
  combined_recording.wav
```

**Using Audacity:**
1. Import both WAV files
2. Use Tracks > Mix and Render to combine them

## License

This project is provided as-is for personal use.

