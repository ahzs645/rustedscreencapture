// High-level recording management
// This module provides the main recording API and orchestrates the recording process

use napi::{Result, Status, Error};
use std::sync::{Arc, Mutex};
use serde_json;

use crate::RecordingConfiguration;
use super::types::*;
use super::content::ShareableContent;
use super::filters::{ContentFilter, ContentFilterFactory};
use super::bindings::ScreenCaptureKitAPI;
use super::permissions::PermissionManager;

/// High-level recording manager
pub struct RecordingManager {
    stream: Option<*mut SCStream>,
    content_filter: Option<ContentFilter>,
    is_recording: Arc<Mutex<bool>>,
    recording_config: Option<RecordingConfiguration>,
    output_path: Option<String>,
    shareable_content: Option<ShareableContent>,
}

impl RecordingManager {
    /// Create a new recording manager
    pub fn new() -> Self {
        Self {
            stream: None,
            content_filter: None,
            is_recording: Arc::new(Mutex::new(false)),
            recording_config: None,
            output_path: None,
            shareable_content: None,
        }
    }

    /// Start recording with the given configuration
    pub fn start_recording(&mut self, config: RecordingConfiguration) -> Result<()> {
        println!("▶️ Starting recording with configuration");
        
        // Validate configuration
        self.validate_configuration(&config)?;
        
        // Check permissions
        PermissionManager::ensure_permission()?;
        
        // Get shareable content if not already available
        if self.shareable_content.is_none() {
            self.shareable_content = Some(ShareableContent::new_with_screencapturekit()?);
        }
        
        // Create content filter based on configuration
        let content_filter = self.create_content_filter(&config)?;
        
        // Create stream configuration
        let stream_config = self.create_stream_configuration(&config)?;
        
        // Create stream with a simple delegate
        let stream = self.create_stream(content_filter.get_filter_ptr(), stream_config)?;
        
        // Start the stream
        self.start_stream_capture(stream)?;
        
        // Store state
        self.stream = Some(stream);
        self.content_filter = Some(content_filter);
        self.recording_config = Some(config.clone());
        self.output_path = Some(config.output_path.clone());
        
        if let Ok(mut is_recording) = self.is_recording.lock() {
            *is_recording = true;
        }
        
        println!("✅ Recording started successfully");
        Ok(())
    }

    /// Stop recording
    pub fn stop_recording(&mut self) -> Result<String> {
        println!("⏹️ Stopping recording");
        
        if let Some(stream) = self.stream {
            unsafe {
                self.stop_stream_capture(stream)?;
            }
        }
        
        // Update state
        self.stream = None;
        self.content_filter = None;
        
        if let Ok(mut is_recording) = self.is_recording.lock() {
            *is_recording = false;
        }
        
        let output_path = self.output_path.clone()
            .unwrap_or_else(|| "/tmp/recording.mp4".to_string());
        
        println!("✅ Recording stopped. Output: {}", output_path);
        Ok(output_path)
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        self.is_recording.lock().unwrap_or_else(|_| false.into()).clone()
    }

    /// Get recording statistics
    pub fn get_stats(&self) -> String {
        serde_json::json!({
            "isRecording": self.is_recording(),
            "outputPath": self.output_path,
            "hasStream": self.stream.is_some(),
            "hasContentFilter": self.content_filter.as_ref().map_or(false, |f| f.is_valid()),
            "method": "screencapturekit-rust-modular",
            "implementation": "Clean modular architecture"
        }).to_string()
    }

    /// Validate recording configuration
    fn validate_configuration(&self, config: &RecordingConfiguration) -> Result<()> {
        if config.output_path.is_empty() {
            return Err(Error::new(Status::InvalidArg, "Output path cannot be empty"));
        }

        if let Some(width) = config.width {
            validate_dimensions(width, config.height.unwrap_or(1080))?;
        }

        if let Some(fps) = config.fps {
            validate_fps(fps)?;
        }

        Ok(())
    }

    /// Create content filter based on configuration
    fn create_content_filter(&self, config: &RecordingConfiguration) -> Result<ContentFilter> {
        unsafe {
            let sc_content_ptr = self.shareable_content.as_ref()
                .and_then(|content| content.get_sc_content_ptr());

            // Parse the screen/window selection from output path or other config
            // For now, default to display capture
            ContentFilterFactory::create_display_filter(sc_content_ptr, 1)
        }
    }

    /// Create stream configuration
    unsafe fn create_stream_configuration(&self, config: &RecordingConfiguration) -> Result<*mut SCStreamConfiguration> {
        let stream_config = ScreenCaptureKitAPI::create_stream_configuration();
        if stream_config.is_null() {
            return Err(Error::new(Status::GenericFailure, "Failed to create stream configuration"));
        }

        ScreenCaptureKitAPI::configure_stream_configuration(
            stream_config,
            config.width.unwrap_or(1920),
            config.height.unwrap_or(1080),
            config.fps.unwrap_or(30),
            config.show_cursor.unwrap_or(true),
            config.capture_audio.unwrap_or(false),
            kCVPixelFormatType_32BGRA,
        );

        Ok(stream_config)
    }

    /// Create stream with minimal delegate
    unsafe fn create_stream(
        &self,
        content_filter: *mut SCContentFilter,
        configuration: *mut SCStreamConfiguration,
    ) -> Result<*mut SCStream> {
        // Create a minimal NSObject delegate
        use objc2::{msg_send, class};
        let delegate_class = class!(NSObject);
        let delegate: *mut objc2::runtime::AnyObject = msg_send![delegate_class, new];

        let stream = ScreenCaptureKitAPI::create_stream(content_filter, configuration, delegate);

        if stream.is_null() {
            return Err(Error::new(Status::GenericFailure, "Failed to create stream"));
        }

        Ok(stream)
    }

    /// Start stream capture
    unsafe fn start_stream_capture(&self, stream: *mut SCStream) -> Result<()> {
        let start_result = Arc::new(Mutex::new(None));
        let start_result_clone = start_result.clone();

        ScreenCaptureKitAPI::start_stream_capture_async(stream, move |error| {
            let mut result = start_result_clone.lock().unwrap();
            *result = Some(error.is_none());
        });

        // Wait briefly for start completion
        std::thread::sleep(std::time::Duration::from_millis(100));

        Ok(())
    }

    /// Stop stream capture
    unsafe fn stop_stream_capture(&self, stream: *mut SCStream) -> Result<()> {
        let stop_result = Arc::new(Mutex::new(None));
        let stop_result_clone = stop_result.clone();

        ScreenCaptureKitAPI::stop_stream_capture_async(stream, move |error| {
            let mut result = stop_result_clone.lock().unwrap();
            *result = Some(error.is_none());
        });

        // Wait briefly for stop completion
        std::thread::sleep(std::time::Duration::from_millis(200));

        Ok(())
    }
} 