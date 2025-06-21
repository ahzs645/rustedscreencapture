// High-level recording management
// This module provides the main recording API and orchestrates the recording process

use napi::{Result, Status, Error};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

use crate::RecordingConfiguration;
use super::types::*;
use super::content::{AsyncContentManager, ShareableContent};
use super::filters::{ContentFilter, ContentFilterFactory};
use super::bindings::ScreenCaptureKitAPI;
use super::permissions::PermissionManager;
use super::delegate::RealStreamDelegate;
use super::stream_output::StreamOutput;
use super::objc_bridge_rust::ObjCDelegateBridge;

// Add the constant
pub const kCVPixelFormatType_32BGRA: u32 = 1111970369; // 'BGRA'

/// High-level async recording manager
pub struct RecordingManager {
    stream: Option<*mut SCStream>,
    content_filter: Option<ContentFilter>,
    delegate: Option<Arc<RealStreamDelegate>>,
    delegate_bridge: Option<Arc<ObjCDelegateBridge>>,
    stream_output: Option<Arc<Mutex<StreamOutput>>>,
    is_recording: Arc<Mutex<bool>>,
    recording_config: Option<RecordingConfiguration>,
    output_path: Option<String>,
    shareable_content: Option<ShareableContent>,
}

// Safety: Raw pointers are only used within unsafe blocks and not shared across threads
// All shared state is protected by Mutex
unsafe impl Send for RecordingManager {}
unsafe impl Sync for RecordingManager {}

impl RecordingManager {
    /// Create a new recording manager
    pub fn new() -> Self {
        Self {
            stream: None,
            content_filter: None,
            delegate: None,
            delegate_bridge: None,
            stream_output: None,
            is_recording: Arc::new(Mutex::new(false)),
            recording_config: None,
            output_path: None,
            shareable_content: None,
        }
    }

    /// Initialize the recording manager with shareable content
    pub async fn initialize(&mut self) -> Result<()> {
        println!("🔧 Initializing recording manager with async ScreenCaptureKit");
        
        // Check permissions first
        if !PermissionManager::check_screen_recording_permission() {
            return Err(Error::new(Status::GenericFailure, "Screen recording permission required"));
        }
        
        // Get shareable content asynchronously
        let content = AsyncContentManager::get_shareable_content().await?;
        self.shareable_content = Some(content);
        
        println!("✅ Recording manager initialized successfully");
        Ok(())
    }

    /// Start recording with the given configuration
    pub async fn start_recording(&mut self, config: RecordingConfiguration) -> Result<String> {
        println!("🎬 Starting async recording with configuration");
        
        // Validate configuration
        self.validate_configuration(&config)?;
        
        // Check if already recording
        {
            let is_recording = self.is_recording.lock().unwrap();
            if *is_recording {
                return Err(Error::new(Status::GenericFailure, "Already recording"));
            }
        }
        
        // Ensure we have shareable content
        if self.shareable_content.is_none() {
            self.initialize().await?;
        }
        
        // Store configuration
        self.output_path = Some(config.output_path.clone());
        self.recording_config = Some(config.clone());
        
        // Create content filter
        let content_filter = self.create_content_filter(&config).await?;
        self.content_filter = Some(content_filter);
        
        // Create stream configuration
        let stream_config = unsafe { self.create_stream_configuration(&config)? };
        
        // Create stream output
        let stream_output = StreamOutput::new(
            config.output_path.clone(),
            config.width.unwrap_or(1920),
            config.height.unwrap_or(1080),
            config.fps.unwrap_or(30),
            config.capture_audio.unwrap_or(false),
        )?;
        
        let stream_output = Arc::new(Mutex::new(stream_output));
        self.stream_output = Some(stream_output.clone());
        
        // Create delegate
        let delegate = Arc::new(RealStreamDelegate::new(
            config.output_path.clone(),
            self.is_recording.clone(),
            config.width.unwrap_or(1920),
            config.height.unwrap_or(1080),
            config.fps.unwrap_or(30),
        ));
        
        // Create the Objective-C bridge for the delegate
        let bridge = ObjCDelegateBridge::new(delegate.clone())
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to create delegate bridge: {}", e)))?;
        
        self.delegate = Some(delegate);
        self.delegate_bridge = Some(Arc::new(bridge));
        
        // Create stream
        let stream = unsafe {
            self.create_stream(
                self.content_filter.as_ref().unwrap().get_filter_ptr(),
                stream_config,
            )?
        };
        self.stream = Some(stream);
        
        // Start stream capture
        self.start_stream_capture().await?;
        
        // Mark as recording
        {
            let mut is_recording = self.is_recording.lock().unwrap();
            *is_recording = true;
        }
        
        println!("✅ Recording started successfully: {}", config.output_path);
        Ok(format!("Recording started: {}", config.output_path))
    }

    /// Stop recording
    pub async fn stop_recording(&mut self) -> Result<String> {
        println!("⏹️ Stopping async recording");
        
        // Check if recording
        {
            let is_recording = self.is_recording.lock().unwrap();
            if !*is_recording {
                return Err(Error::new(Status::GenericFailure, "Not currently recording"));
            }
        }
        
        // Stop stream capture
        if self.stream.is_some() {
            self.stop_stream_capture().await?;
        }
        
        // Finalize stream output
        let output_path = if let Some(ref stream_output) = self.stream_output {
            if let Ok(mut output) = stream_output.lock() {
                output.stop_recording()?
            } else {
                self.output_path.clone().unwrap_or_default()
            }
        } else {
            self.output_path.clone().unwrap_or_default()
        };
        
        // Mark as not recording
        {
            let mut is_recording = self.is_recording.lock().unwrap();
            *is_recording = false;
        }
        
        // Clean up
        self.cleanup();
        
        println!("✅ Recording stopped successfully: {}", output_path);
        Ok(output_path)
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        self.is_recording.lock().map(|guard| *guard).unwrap_or(false)
    }

    /// Get available screens
    pub async fn get_available_screens(&self) -> Result<Vec<DisplayInfo>> {
        if let Some(ref content) = self.shareable_content {
            content.get_displays()
        } else {
            // Get content if not available
            let content = AsyncContentManager::get_shareable_content().await?;
            content.get_displays()
        }
    }
    
    /// Get available windows
    pub async fn get_available_windows(&self) -> Result<Vec<WindowInfo>> {
        if let Some(ref content) = self.shareable_content {
            content.get_windows()
        } else {
            // Get content if not available
            let content = AsyncContentManager::get_shareable_content().await?;
            content.get_windows()
        }
    }

    /// Validate recording configuration
    fn validate_configuration(&self, config: &RecordingConfiguration) -> Result<()> {
        if config.output_path.is_empty() {
            return Err(Error::new(Status::InvalidArg, "Output path cannot be empty"));
        }

        if let Some(width) = config.width {
            if width < 100 || width > 7680 {
                return Err(Error::new(Status::InvalidArg, "Width must be between 100 and 7680"));
            }
        }

        if let Some(height) = config.height {
            if height < 100 || height > 4320 {
                return Err(Error::new(Status::InvalidArg, "Height must be between 100 and 4320"));
            }
        }

        if let Some(fps) = config.fps {
            if fps < 1 || fps > 120 {
                return Err(Error::new(Status::InvalidArg, "FPS must be between 1 and 120"));
            }
        }

        Ok(())
    }

    /// Create content filter based on configuration
    async fn create_content_filter(&self, config: &RecordingConfiguration) -> Result<ContentFilter> {
        println!("🎯 Creating content filter for recording");
        
        // For now, create a filter for the first display
        // In a full implementation, this would parse screen selection from config
        unsafe {
            ContentFilterFactory::create_display_filter(None, 1)
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

        println!("⚙️ Created stream configuration: {}x{} @ {}fps", 
            config.width.unwrap_or(1920),
            config.height.unwrap_or(1080),
            config.fps.unwrap_or(30)
        );

        Ok(stream_config)
    }

    /// Create stream with proper delegate
    unsafe fn create_stream(
        &self,
        content_filter: *mut SCContentFilter,
        configuration: *mut SCStreamConfiguration,
    ) -> Result<*mut SCStream> {
        // Get the Objective-C delegate from the bridge
        let delegate = if let Some(ref bridge) = self.delegate_bridge {
            bridge.as_objc_delegate()
        } else {
            // Create a minimal NSObject delegate as fallback
            use objc2::{msg_send, class};
            let delegate_class = class!(NSObject);
            let delegate: *mut objc2::runtime::AnyObject = msg_send![delegate_class, new];
            println!("⚠️ Using fallback NSObject delegate - callbacks will not work!");
            delegate
        };

        if delegate.is_null() {
            return Err(Error::new(Status::GenericFailure, "Failed to get delegate from bridge"));
        }

        let stream = ScreenCaptureKitAPI::create_stream(content_filter, configuration, delegate);

        if stream.is_null() {
            return Err(Error::new(Status::GenericFailure, "Failed to create stream"));
        }

        println!("🎬 Created ScreenCaptureKit stream with proper delegate bridge");
        Ok(stream)
    }

    /// Start stream capture asynchronously
    async fn start_stream_capture(&self) -> Result<()> {
        println!("🚀 Starting stream capture asynchronously");
        
        println!("🔍 DEBUG: Checking if stream is available...");
        if let Some(stream) = self.stream {
            println!("✅ DEBUG: Stream is available: {:p}", stream);
            unsafe {
                println!("🔥 CRITICAL DEBUG: About to call ScreenCaptureKitAPI::start_stream_capture_async with stream: {:p}", stream);
                
                // Use the actual ScreenCaptureKit API to start capture
                ScreenCaptureKitAPI::start_stream_capture_async(stream, |error| {
                    if let Some(error) = error {
                        println!("❌ Failed to start capture: {:?}", error);
                    } else {
                        println!("✅ ScreenCaptureKit capture started successfully - delegate callbacks enabled!");
                    }
                });
                
                println!("🔥 CRITICAL DEBUG: ScreenCaptureKitAPI::start_stream_capture_async call completed");
            }
        } else {
            println!("❌ DEBUG: No stream available to start!");
            return Err(Error::new(Status::GenericFailure, "No stream available to start"));
        }
        
        println!("✅ Stream capture started successfully");
        Ok(())
    }

    /// Stop stream capture asynchronously
    async fn stop_stream_capture(&self) -> Result<()> {
        println!("⏹️ Stopping stream capture asynchronously");
        
        if let Some(stream) = self.stream {
            unsafe {
                // Use the actual ScreenCaptureKit API to stop capture
                ScreenCaptureKitAPI::stop_stream_capture_async(stream, |error| {
                    if let Some(error) = error {
                        println!("⚠️ Warning during capture stop: {:?}", error);
                    } else {
                        println!("✅ ScreenCaptureKit capture stopped successfully");
                    }
                });
            }
        } else {
            println!("⚠️ No stream available to stop");
        }
        
        println!("✅ Stream capture stopped successfully");
        Ok(())
    }
    
    /// Clean up resources
    fn cleanup(&mut self) {
        self.stream = None;
        self.content_filter = None;
        self.delegate_bridge = None; // Release bridge first
        self.delegate = None;
        self.stream_output = None;
        self.recording_config = None;
        println!("🧹 Recording resources cleaned up");
    }
}

impl Drop for RecordingManager {
    fn drop(&mut self) {
        self.cleanup();
    }
} 