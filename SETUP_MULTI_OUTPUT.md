# Setting Up Multi-Output Device on macOS

This allows you to hear audio while recording it through BlackHole.

## Steps:

1. **Open Audio MIDI Setup**
   - Press `Cmd + Space` and search for "Audio MIDI Setup"
   - Or: Applications → Utilities → Audio MIDI Setup

2. **Create Multi-Output Device**
   - Click the `+` button at the bottom left
   - Select "Create Multi-Output Device"

3. **Add Your Output Devices**
   - Check the box for your actual speakers/headphones (e.g., "MacBook Pro Speakers", "Bose NC 700 HP")
   - Check the box for "BlackHole 2ch"
   - Make sure both are enabled

4. **Set as System Output**
   - In System Settings → Sound → Output
   - Select your new "Multi-Output Device"
   - Audio will now play through BOTH your speakers AND BlackHole

5. **Set Teams/Zoom Output**
   - In Teams/Zoom audio settings
   - Set output to your "Multi-Output Device" (or just leave it as system default)

6. **Run the Recorder**
   - Select your microphone as the mic input
   - Select "BlackHole 2ch" as the system audio input
   - You'll hear everything AND it will be recorded!

## Alternative: Use BlackHole 16ch

If you have BlackHole 16ch installed, you can use channels 1-2 for your speakers and channels 3-4 for recording, but the Multi-Output Device approach above is simpler.

