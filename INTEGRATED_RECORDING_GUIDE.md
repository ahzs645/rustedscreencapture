# Integrated Recording Manager Guide

This guide explains how to use the new `IntegratedRecordingManager` that provides complete screen recording functionality with actual audio/video file output, transcription integration, and proper error handling.

## 🎯 What's New

The `IntegratedRecordingManager` is a complete rewrite that addresses all the requirements:

1. ✅ **SCStreamOutput Protocol Implementation** - Properly handles CMSampleBuffer frames
2. ✅ **Complete AVAssetWriter Settings** - Full video/audio encoding configuration
3. ✅ **Transcription Integration** - Connects to transcription APIs after recording
4. ✅ **Error Handling** - Proper recovery from ScreenCaptureKit errors
5. ✅ **Permission Management** - Handles screen recording permissions properly
6. ✅ **Actual File Output** - Records and saves working audio/video files

## 🏗️ Architecture

The new system is modular and well-organized:

```
src/screencapturekit/
├── recording_manager.rs    # Main orchestrator
├── stream_output.rs        # SCStreamOutput protocol implementation
├── permission_manager.rs   # Permission handling and validation
├── transcription.rs        # Transcription service integration
├── encoder.rs             # Video/audio encoding (refactored)
├── delegate.rs            # Stream delegate (refactored)
└── content.rs             # Content management (existing)
```

## 🚀 Quick Start

### Basic Usage

```javascript
const { IntegratedRecordingManager } = require('screencapturekit-rust');

async function recordScreen() {
    // Create and initialize the recording manager
    const recorder = new IntegratedRecordingManager();
    await recorder.initialize();
    
    // Check permissions
    const permissionStatus = recorder.get_permission_status();
    console.log('Permissions:', permissionStatus);
    
    // Get available screens
    const screens = await recorder.get_available_screens();
    console.log(`Found ${screens.length} screens`);
    
    // Configure recording
    const config = {
        output_path: './recordings/my-recording.mp4',
        width: 1920,
        height: 1080,
        fps: 30,
        show_cursor: true,
        capture_audio: true
    };
    
    // Start recording
    await recorder.start_recording(config);
    console.log('Recording started!');
    
    // Record for 10 seconds
    await new Promise(resolve => setTimeout(resolve, 10000));
    
    // Stop recording
    const outputPath = await recorder.stop_recording();
    console.log(`Recording saved to: ${outputPath}`);
}

recordScreen().catch(console.error);
```

### Advanced Configuration

```javascript
const config = {
    output_path: './recordings/high-quality.mp4',
    width: 3840,           // 4K recording
    height: 2160,
    fps: 60,               // High frame rate
    show_cursor: true,
    capture_audio: true,
    pixel_format: 'BGRA',  // High quality pixel format
    color_space: 'sRGB'    // Color space
};
```

## 📹 Recording Features

### Video Encoding

- **Codec**: H.264 (AVC1) with high-quality settings
- **Bitrate**: 8 Mbps for excellent quality
- **Keyframes**: Every 2 seconds for good seeking
- **Pixel Format**: BGRA for maximum compatibility
- **Resolutions**: Support for 100x100 up to 7680x4320 (8K)
- **Frame Rates**: 1-120 FPS

### Audio Encoding

- **Codec**: AAC with 128 kbps bitrate
- **Sample Rate**: 48kHz stereo
- **Quality**: Maximum quality encoding
- **Sources**: System audio and microphone support

### Real-time Monitoring

```javascript
// Monitor recording progress
const interval = setInterval(() => {
    const stats = JSON.parse(recorder.get_recording_stats());
    console.log(`Frames: ${stats.videoFrames}, Audio: ${stats.audioSamples}`);
    console.log(`Duration: ${stats.estimatedDuration.toFixed(1)}s`);
    
    if (!stats.isRecording) {
        clearInterval(interval);
    }
}, 1000);
```

## 🎤 Transcription Integration

### Setup Transcription

```javascript
// Configure transcription (example with OpenAI Whisper)
const transcriptionConfig = {
    service: 'OpenAIWhisper',
    api_key: 'your-openai-api-key',
    language: 'en',
    output_format: 'SRT',
    include_timestamps: true,
    include_speaker_labels: false
};

recorder.configure_transcription(transcriptionConfig);
```

### Supported Services

1. **Local Whisper** (Recommended for privacy)
   ```bash
   pip install openai-whisper
   ```

2. **OpenAI Whisper API** (Requires API key)
3. **Google Speech-to-Text** (Coming soon)
4. **Azure Speech Service** (Coming soon)
5. **AWS Transcribe** (Coming soon)

### Output Formats

- **Text** (.txt) - Plain text transcription
- **SRT** (.srt) - Subtitle format with timestamps
- **VTT** (.vtt) - WebVTT format for web players
- **JSON** (.json) - Detailed format with segments and metadata

### Transcription Example

```javascript
// After recording, run transcription
const transcriptionResult = await recorder.start_transcription(outputPath);
console.log('Transcription:', transcriptionResult.text);
console.log('Language detected:', transcriptionResult.language);
console.log('Duration:', transcriptionResult.duration);
```

## 🔐 Permission Management

### Automatic Permission Handling

The system automatically:
- Checks macOS version compatibility (10.15+)
- Validates ScreenCaptureKit framework availability
- Requests screen recording permissions
- Provides detailed error messages and recovery instructions

### Manual Permission Check

```javascript
const status = recorder.get_permission_status();
console.log(status);
// Output:
// Permission Status Report:
// 📺 Screen Recording: ✅ Granted
// ♿ Accessibility: ✅ Granted  
// 🍎 macOS Compatible: ✅ Yes
// 🎬 ScreenCaptureKit Available: ✅ Yes
```

## 🛠️ Error Handling

### Automatic Recovery

The system provides intelligent error recovery:

```javascript
try {
    await recorder.start_recording(config);
} catch (error) {
    const recovery = recorder.handle_error(error.message);
    console.log('Recovery suggestion:', recovery);
}
```

### Common Issues and Solutions

| Error | Cause | Solution |
|-------|-------|----------|
| Permission denied | Screen recording not enabled | Enable in System Preferences > Security & Privacy |
| Content filter failed | Invalid screen/window ID | Check available sources |
| Stream creation failed | Invalid configuration | Verify width/height/fps values |
| Asset writer failed | Invalid output path | Check directory permissions |

## 📊 Performance Monitoring

### Real-time Statistics

```javascript
const stats = JSON.parse(recorder.get_recording_stats());
console.log({
    isRecording: stats.isRecording,
    videoFrames: stats.videoFrames,
    audioSamples: stats.audioSamples,
    estimatedDuration: stats.estimatedDuration,
    outputPath: stats.outputPath
});
```

### Memory Management

The system automatically:
- Manages memory for video frames
- Handles buffer overflow situations
- Cleans up resources on completion
- Provides graceful shutdown on errors

## 🧪 Testing

### Run Basic Tests

```bash
# Test basic functionality
node test/test-integrated-recording.js basic

# Test permissions only
node test/test-integrated-recording.js permissions

# Full recording test (requires permissions)
node test/test-integrated-recording.js full
```

### Test Output

```
🎬 Testing Integrated Recording Manager
=====================================

📋 Step 1: Creating Recording Manager
✅ Recording manager initialized successfully

🔐 Step 2: Checking Permissions
Permission Status: [Detailed status report]

📺 Step 3: Getting Available Sources
Found 2 screens:
  1. Built-in Retina Display (3024x1964)
  2. External Display (2560x1440)

⚙️ Step 4: Configuring Recording
Output path: ./test/recordings/test-recording-1234567890.mp4

▶️ Step 5: Starting Recording
✅ Recording started successfully!

📊 Step 6: Monitoring Recording
Recording... 1s - Frames: 30, Audio: 48
Recording... 2s - Frames: 60, Audio: 96
...

⏹️ Step 7: Stopping Recording
✅ Recording stopped. Final output: ./test/recordings/test-recording-1234567890.mp4

🔍 Step 8: Verifying Output
✅ Output file exists
📁 File size: 15.2 MB
```

## 🔧 Configuration Options

### Recording Configuration

```typescript
interface RecordingConfiguration {
    output_path: string;           // Required: Output file path
    width?: number;               // Video width (default: 1920)
    height?: number;              // Video height (default: 1080)
    fps?: number;                 // Frame rate (default: 30)
    show_cursor?: boolean;        // Show cursor (default: true)
    capture_audio?: boolean;      // Capture audio (default: false)
    audio_device_id?: string;     // Specific audio device
    pixel_format?: string;        // Pixel format (default: 'BGRA')
    color_space?: string;         // Color space (default: 'sRGB')
}
```

### Quality Presets

```javascript
// High Quality (4K, 60fps)
const highQuality = {
    width: 3840, height: 2160, fps: 60,
    capture_audio: true
};

// Standard Quality (1080p, 30fps)
const standard = {
    width: 1920, height: 1080, fps: 30,
    capture_audio: true
};

// Low Quality (720p, 15fps)
const lowQuality = {
    width: 1280, height: 720, fps: 15,
    capture_audio: false
};
```

## 🚨 Troubleshooting

### Common Issues

1. **"Screen recording permission not granted"**
   - Go to System Preferences > Security & Privacy > Privacy > Screen Recording
   - Add your application and restart

2. **"Failed to create AVAssetWriter"**
   - Check output directory exists and is writable
   - Verify file path doesn't contain invalid characters

3. **"No displays available for recording"**
   - Ensure displays are connected and active
   - Try reinitializing the recording manager

4. **"Transcription failed"**
   - Check FFmpeg is installed: `brew install ffmpeg`
   - For local Whisper: `pip install openai-whisper`
   - For API services: verify API keys

### Debug Mode

Enable detailed logging:

```javascript
process.env.RUST_LOG = 'debug';
const recorder = new IntegratedRecordingManager();
```

## 🔄 Migration from Old API

### Before (Old API)

```javascript
const recorder = new ScreenCaptureKitRecorder();
recorder.start_recording(screenId, config); // No actual file output
```

### After (New API)

```javascript
const recorder = new IntegratedRecordingManager();
await recorder.initialize();
await recorder.start_recording(config); // Saves real files!
```

## 📚 API Reference

### IntegratedRecordingManager Methods

| Method | Description | Returns |
|--------|-------------|---------|
| `new()` | Create new recording manager | `IntegratedRecordingManager` |
| `initialize()` | Initialize with system content | `Promise<void>` |
| `start_recording(config)` | Start recording with config | `Promise<void>` |
| `stop_recording()` | Stop and finalize recording | `Promise<string>` |
| `is_recording()` | Check if currently recording | `boolean` |
| `get_recording_stats()` | Get real-time statistics | `string` (JSON) |
| `get_permission_status()` | Get permission status report | `string` |
| `get_available_screens()` | Get available displays | `Promise<ScreenSource[]>` |
| `get_available_windows()` | Get available windows | `Promise<ScreenSource[]>` |

## 🎯 Next Steps

1. **Try the basic test**: `node test/test-integrated-recording.js basic`
2. **Check permissions**: `node test/test-integrated-recording.js permissions`
3. **Record your first video**: `node test/test-integrated-recording.js full`
4. **Set up transcription**: Configure Whisper or API keys
5. **Integrate into your app**: Use the API in your application

The new `IntegratedRecordingManager` provides everything you need for professional screen recording with actual file output, transcription, and robust error handling! 