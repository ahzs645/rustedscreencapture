use std::sync::{Arc, Mutex};
use std::path::Path;
use napi::{Result, Status, Error};
use serde_json;

use objc2_core_media::CMSampleBuffer;

use super::{
    content::{ShareableContent, AsyncContentManager},
    types::{DisplayInfo, WindowInfo},
    filters::{ContentFilter, ContentFilterFactory},
    stream_output::StreamOutput,
    permission_manager::PermissionManager,
    transcription::{TranscriptionManager, TranscriptionConfig, TranscriptionResult},
    types::{SCStream, SCStreamConfiguration, SCStreamOutputType},
    bindings::ScreenCaptureKitAPI,
};
use crate::RecordingConfiguration;

/// Complete recording manager that handles the entire recording pipeline
pub struct RecordingManager {
    // Core components
    stream_output: Option<Arc<Mutex<StreamOutput>>>,
    transcription_manager: Option<TranscriptionManager>,
    
    // ScreenCaptureKit objects
    stream: Option<*mut SCStream>,
    content_filter: Option<ContentFilter>,
    
    // State management
    is_recording: Arc<Mutex<bool>>,
    recording_config: Option<RecordingConfiguration>,
    output_path: Option<String>,
    
    // Content management
    shareable_content: Option<ShareableContent>,
}

impl RecordingManager {
    /// Create a new recording manager
    pub fn new() -> Result<Self> {
        println!("🎬 Creating RecordingManager");
        
        // Validate system requirements and permissions
        PermissionManager::validate_system_requirements()?;
        PermissionManager::ensure_all_permissions()?;
        
        Ok(Self {
            stream_output: None,
            transcription_manager: None,
            stream: None,
            content_filter: None,
            is_recording: Arc::new(Mutex::new(false)),
            recording_config: None,
            output_path: None,
            shareable_content: None,
        })
    }
    
    /// Initialize the recording manager with shareable content
    pub async fn initialize(&mut self) -> Result<()> {
        println!("🔧 Initializing RecordingManager");
        
        // Get shareable content
        self.shareable_content = Some(AsyncContentManager::get_shareable_content().await?);
        
        println!("✅ RecordingManager initialized successfully");
        Ok(())
    }
    
    /// Get available screens for recording
    pub fn get_available_screens(&self) -> Result<Vec<DisplayInfo>> {
        if let Some(ref content) = self.shareable_content {
            content.get_displays()
        } else {
            Err(Error::new(Status::GenericFailure, "RecordingManager not initialized"))
        }
    }
    
    /// Get available windows for recording
    pub fn get_available_windows(&self) -> Result<Vec<WindowInfo>> {
        if let Some(ref content) = self.shareable_content {
            content.get_windows()
        } else {
            Err(Error::new(Status::GenericFailure, "RecordingManager not initialized"))
        }
    }
    
    /// Start recording with the specified configuration
    pub fn start_recording(&mut self, config: RecordingConfiguration) -> Result<()> {
        println!("▶️ Starting recording with configuration");
        
        // Validate configuration
        self.validate_recording_configuration(&config)?;
        
        // Create output directory if needed
        self.ensure_output_directory(&config.output_path)?;
        
        // Initialize stream output with proper settings
        let stream_output = StreamOutput::new(
            config.output_path.clone(),
            config.width.unwrap_or(1920),
            config.height.unwrap_or(1080),
            config.fps.unwrap_or(30),
            config.capture_audio.unwrap_or(false),
        )?;
        
        let stream_output_arc = Arc::new(Mutex::new(stream_output));
        
        // Initialize the asset writer
        if let Ok(mut output) = stream_output_arc.lock() {
            output.initialize_asset_writer()?;
        }
        
        // Create content filter based on screen/window selection
        let content_filter = self.create_content_filter(&config)?;
        
        // Create stream configuration
        let stream_config = self.create_stream_configuration(&config)?;
        
        // Create ScreenCaptureKit stream
        let stream = self.create_screencapturekit_stream(content_filter, stream_config, stream_output_arc.clone())?;
        
        // Start the stream output recording
        if let Ok(mut output) = stream_output_arc.lock() {
            output.start_recording()?;
        }
        
        // Start ScreenCaptureKit capture
        self.start_screencapturekit_capture(stream)?;
        
        // Store state
        self.stream_output = Some(stream_output_arc);
        self.stream = Some(stream);
        self.recording_config = Some(config.clone());
        self.output_path = Some(config.output_path.clone());
        
        if let Ok(mut is_recording) = self.is_recording.lock() {
            *is_recording = true;
        }
        
        println!("✅ Recording started successfully");
        Ok(())
    }
    
    /// Stop recording and finalize the output
    pub fn stop_recording(&mut self) -> Result<String> {
        println!("⏹️ Stopping recording");
        
        // Mark as not recording
        if let Ok(mut is_recording) = self.is_recording.lock() {
            *is_recording = false;
        }
        
        // Stop ScreenCaptureKit stream
        if let Some(stream) = self.stream.take() {
            self.stop_screencapturekit_capture(stream)?;
        }
        
        // Finalize stream output
        let output_path = if let Some(stream_output) = self.stream_output.take() {
            if let Ok(mut output) = stream_output.lock() {
                output.stop_recording()?
            } else {
                return Err(Error::new(Status::GenericFailure, "Failed to access stream output"));
            }
        } else {
            return Err(Error::new(Status::GenericFailure, "No active recording to stop"));
        };
        
        println!("✅ Recording stopped: {}", output_path);
        
        // Note: Transcription would be started asynchronously in a real implementation
        // For now, we just log that it would happen
        if let Some(ref config) = self.recording_config {
            if config.capture_audio.unwrap_or(false) {
                println!("💡 Transcription would be started for: {}", output_path);
            }
        }
        
        Ok(output_path)
    }
    
    /// Start transcription asynchronously (separate method for async operations)
    pub async fn start_transcription(&self, output_path: &str) -> Result<TranscriptionResult> {
        if let Some(ref transcription_manager) = self.transcription_manager {
            println!("🎤 Starting transcription of recorded file");
            transcription_manager.transcribe_file(output_path).await
        } else {
            Err(Error::new(Status::GenericFailure, "Transcription not configured"))
        }
    }
    
    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        self.is_recording.lock().map(|r| *r).unwrap_or(false)
    }
    
    /// Configure transcription settings
    pub fn configure_transcription(&mut self, config: TranscriptionConfig) -> Result<()> {
        println!("🎤 Configuring transcription with service: {:?}", config.service);
        
        // Check dependencies
        let transcription_manager = TranscriptionManager::new(config);
        
        // Validate dependencies asynchronously (simplified for now)
        println!("💡 Transcription configured. Dependencies will be checked during transcription.");
        
        self.transcription_manager = Some(transcription_manager);
        Ok(())
    }
    
    /// Start transcription if configured
    async fn start_transcription_if_configured(&self, output_path: &str) -> Result<()> {
        if let Some(ref transcription_manager) = self.transcription_manager {
            println!("🎤 Starting transcription of recorded file");
            
            match transcription_manager.transcribe_file(output_path).await {
                Ok(result) => {
                    println!("✅ Transcription completed successfully");
                    println!("📝 Transcribed text length: {} characters", result.text.len());
                    if let Some(duration) = result.duration {
                        println!("⏱️ Audio duration: {:.2} seconds", duration);
                    }
                }
                Err(e) => {
                    println!("⚠️ Transcription failed: {}", e);
                    // Don't fail the entire recording process if transcription fails
                }
            }
        }
        Ok(())
    }
    
    /// Validate recording configuration
    fn validate_recording_configuration(&self, config: &RecordingConfiguration) -> Result<()> {
        // Check output path
        if config.output_path.is_empty() {
            return Err(Error::new(Status::GenericFailure, "Output path cannot be empty"));
        }
        
        // Validate dimensions
        if let Some(width) = config.width {
            if width < 100 || width > 7680 {
                return Err(Error::new(Status::GenericFailure, "Width must be between 100 and 7680"));
            }
        }
        
        if let Some(height) = config.height {
            if height < 100 || height > 4320 {
                return Err(Error::new(Status::GenericFailure, "Height must be between 100 and 4320"));
            }
        }
        
        // Validate FPS
        if let Some(fps) = config.fps {
            if fps < 1 || fps > 120 {
                return Err(Error::new(Status::GenericFailure, "FPS must be between 1 and 120"));
            }
        }
        
        Ok(())
    }
    
    /// Ensure output directory exists
    fn ensure_output_directory(&self, output_path: &str) -> Result<()> {
        if let Some(parent) = Path::new(output_path).parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to create output directory: {}", e)))?;
                println!("📁 Created output directory: {}", parent.display());
            }
        }
        Ok(())
    }
    
    /// Create content filter based on configuration
    fn create_content_filter(&self, _config: &RecordingConfiguration) -> Result<ContentFilter> {
        if let Some(ref content) = self.shareable_content {
            // For now, create a filter for the first display
            // In a real implementation, this would parse the screen_id from the config
            let displays = content.get_displays()?;
            if let Some(display) = displays.first() {
                // Create a basic filter for now
                unsafe {
                    let filter = ContentFilter::new_basic()?;
                    println!("🎯 Created content filter for display: {}", display.name);
                    Ok(filter)
                }
            } else {
                Err(Error::new(Status::GenericFailure, "No displays available for recording"))
            }
        } else {
            Err(Error::new(Status::GenericFailure, "Shareable content not initialized"))
        }
    }
    
    /// Create stream configuration
    fn create_stream_configuration(&self, config: &RecordingConfiguration) -> Result<*mut SCStreamConfiguration> {
        unsafe {
            let stream_config = ScreenCaptureKitAPI::create_stream_configuration();
            
            ScreenCaptureKitAPI::configure_stream_configuration(
                stream_config,
                config.width.unwrap_or(1920),
                config.height.unwrap_or(1080),
                config.fps.unwrap_or(30),
                config.show_cursor.unwrap_or(true),
                config.capture_audio.unwrap_or(false),
                0x42475241, // 'BGRA' pixel format
            );
            
            println!("⚙️ Created stream configuration");
            Ok(stream_config)
        }
    }
    
    /// Create ScreenCaptureKit stream with proper delegate
    fn create_screencapturekit_stream(
        &self,
        content_filter: ContentFilter,
        stream_config: *mut SCStreamConfiguration,
        stream_output: Arc<Mutex<StreamOutput>>,
    ) -> Result<*mut SCStream> {
        unsafe {
            // Create a delegate that will handle the sample buffers
            let delegate = super::stream_output::create_stream_delegate(stream_output);
            
            // Create the SCStream
            let stream = ScreenCaptureKitAPI::create_stream(
                content_filter.get_filter_ptr(),
                stream_config,
                delegate,
            );
            
            if stream.is_null() {
                return Err(Error::new(Status::GenericFailure, "Failed to create ScreenCaptureKit stream"));
            }
            
            println!("🎬 Created ScreenCaptureKit stream");
            Ok(stream)
        }
    }
    
    /// Start ScreenCaptureKit capture
    fn start_screencapturekit_capture(&self, stream: *mut SCStream) -> Result<()> {
        unsafe {
            println!("🚀 Starting ScreenCaptureKit capture");
            
            // Use the async start method with a completion handler
            ScreenCaptureKitAPI::start_stream_capture_async(stream, |error| {
                if let Some(error) = error {
                    println!("❌ Failed to start capture: {:?}", error);
                } else {
                    println!("✅ ScreenCaptureKit capture started successfully");
                }
            });
            
            Ok(())
        }
    }
    
    /// Stop ScreenCaptureKit capture
    fn stop_screencapturekit_capture(&self, stream: *mut SCStream) -> Result<()> {
        unsafe {
            println!("⏹️ Stopping ScreenCaptureKit capture");
            
            ScreenCaptureKitAPI::stop_stream_capture_async(stream, |error| {
                if let Some(error) = error {
                    println!("⚠️ Warning during capture stop: {:?}", error);
                } else {
                    println!("✅ ScreenCaptureKit capture stopped successfully");
                }
            });
            
            Ok(())
        }
    }
    
    /// Handle incoming sample buffers from ScreenCaptureKit
    pub fn handle_sample_buffer(&self, sample_buffer: &CMSampleBuffer, output_type: SCStreamOutputType) -> Result<()> {
        if let Some(ref stream_output) = self.stream_output {
            if let Ok(mut output) = stream_output.lock() {
                match output_type {
                    SCStreamOutputType::Screen => {
                        output.handle_video_sample(sample_buffer)?;
                    }
                    SCStreamOutputType::Audio => {
                        output.handle_audio_sample(sample_buffer)?;
                    }
                    SCStreamOutputType::Microphone => {
                        // Handle microphone input if needed
                        output.handle_audio_sample(sample_buffer)?;
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Get permission status report
    pub fn get_permission_status(&self) -> String {
        PermissionManager::get_permission_status_report()
    }
    
    /// Handle ScreenCaptureKit errors with recovery
    pub fn handle_error(&self, error_description: &str) -> Result<String> {
        PermissionManager::handle_screencapturekit_error(error_description)
    }
}

impl Drop for RecordingManager {
    fn drop(&mut self) {
        // Ensure recording is stopped when the manager is dropped
        if self.is_recording() {
            println!("🧹 Cleaning up recording manager");
            let _ = self.stop_recording();
        }
    }
} 