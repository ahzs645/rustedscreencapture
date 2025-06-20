// src/screencapturekit/content.rs - Real Async ScreenCaptureKit Implementation

use crate::ScreenSource;
use napi::bindgen_prelude::*;
use super::types::*;
use super::bindings::ScreenCaptureKitAPI;
use std::time::Duration;
use tokio::sync::oneshot;

/// Async-only content manager that properly handles ScreenCaptureKit's async nature
pub struct AsyncContentManager;

impl AsyncContentManager {
    /// Get shareable content using real ScreenCaptureKit async APIs
    pub async fn get_shareable_content() -> Result<ShareableContent> {
        println!("🔍 Getting shareable content via real ScreenCaptureKit async APIs");
        
        // Use tokio oneshot channel for async communication
        let (sender, receiver) = oneshot::channel();
        
        // Call ScreenCaptureKit's async API
        unsafe {
            ScreenCaptureKitAPI::get_shareable_content_async(move |content, error| {
                if error.is_null() && !content.is_null() {
                    // Success - extract data synchronously in the callback
                    match ShareableContent::from_screencapturekit_content(content) {
                        Ok(shareable_content) => {
                            let _ = sender.send(Ok(shareable_content));
                        }
                        Err(e) => {
                            let _ = sender.send(Err(e));
                        }
                    }
                } else {
                    let error_msg = if !error.is_null() {
                        use objc2::{msg_send};
                        use objc2_foundation::NSString;
                        
                        let description: *mut NSString = msg_send![error, localizedDescription];
                        if !description.is_null() {
                            format!("ScreenCaptureKit error: {}", (*description).to_string())
                        } else {
                            "ScreenCaptureKit error (no description available)".to_string()
                        }
                    } else {
                        "Unknown ScreenCaptureKit error".to_string()
                    };
                    
                    let _ = sender.send(Err(Error::new(Status::GenericFailure, error_msg)));
                }
            });
        }
        
        // Wait for the result with timeout
        let content = tokio::time::timeout(Duration::from_secs(10), receiver)
            .await
            .map_err(|_| Error::new(Status::GenericFailure, "ScreenCaptureKit content retrieval timed out"))?
            .map_err(|_| Error::new(Status::GenericFailure, "Internal channel error"))??;
        
        println!("✅ Retrieved real ScreenCaptureKit content asynchronously");
        Ok(content)
    }
    
    /// Extract screen sources from async content
    pub async fn extract_screen_sources(content: &ShareableContent) -> Result<Vec<ScreenSource>> {
        content.get_all_sources().await
    }
}

/// Async content manager for ScreenCaptureKit
pub struct ShareableContent {
    displays: Vec<DisplayInfo>,
    windows: Vec<WindowInfo>,
    sc_content_ptr: Option<*mut SCShareableContent>,
}

impl ShareableContent {
    /// Create from real ScreenCaptureKit content pointer
    unsafe fn from_screencapturekit_content(sc_content_ptr: *mut SCShareableContent) -> Result<Self> {
        println!("🔍 Processing real ScreenCaptureKit content");
        
        let displays = Self::extract_displays_from_content(sc_content_ptr)?;
        let windows = Self::extract_windows_from_content(sc_content_ptr)?;
        
        Ok(Self {
            displays,
            windows,
            sc_content_ptr: Some(sc_content_ptr),
        })
    }
    
    /// Extract display information from ScreenCaptureKit content
    unsafe fn extract_displays_from_content(sc_content_ptr: *mut SCShareableContent) -> Result<Vec<DisplayInfo>> {
        use objc2::{msg_send};
        use objc2_foundation::NSArray;
        
        let displays_array: *mut NSArray = msg_send![sc_content_ptr, displays];
        if displays_array.is_null() {
            return Ok(Vec::new());
        }
        
        let displays = &*displays_array;
        let count = displays.count();
        let mut result = Vec::new();
        
        for i in 0..count {
            let display: *mut SCDisplay = msg_send![displays, objectAtIndex: i];
            if !display.is_null() {
                let display_id: u32 = msg_send![display, displayID];
                let width: u32 = msg_send![display, width];
                let height: u32 = msg_send![display, height];
                
                result.push(DisplayInfo {
                    id: display_id,
                    name: format!("Display {}", display_id),
                    width,
                    height,
                });
            }
        }
        
        println!("📺 Found {} displays from ScreenCaptureKit", result.len());
        Ok(result)
    }
    
    /// Extract window information from ScreenCaptureKit content
    unsafe fn extract_windows_from_content(sc_content_ptr: *mut SCShareableContent) -> Result<Vec<WindowInfo>> {
        use objc2::{msg_send};
        use objc2_foundation::{NSArray, NSString};
        
        let windows_array: *mut NSArray = msg_send![sc_content_ptr, windows];
        if windows_array.is_null() {
            return Ok(Vec::new());
        }
        
        let windows = &*windows_array;
        let count = windows.count();
        let mut result = Vec::new();
        
        // Limit to first 50 windows to avoid overwhelming the system
        for i in 0..count.min(50) {
            let window: *mut SCWindow = msg_send![windows, objectAtIndex: i];
            if !window.is_null() {
                let window_id: u32 = msg_send![window, windowID];
                let title_ptr: *mut NSString = msg_send![window, title];
                let title = if !title_ptr.is_null() {
                    (*title_ptr).to_string()
                } else {
                    format!("Window {}", window_id)
                };
                
                // Get frame information
                let frame: super::foundation::CGRect = msg_send![window, frame];
                
                // Only include windows with reasonable titles and sizes
                if !title.is_empty() && frame.size.width > 50.0 && frame.size.height > 50.0 {
                    result.push(WindowInfo {
                        id: window_id,
                        title,
                        width: frame.size.width as u32,
                        height: frame.size.height as u32,
                    });
                }
            }
        }
        
        println!("🪟 Found {} windows from ScreenCaptureKit", result.len());
        Ok(result)
    }
    
    /// Get all screen sources asynchronously
    pub async fn get_all_sources(&self) -> Result<Vec<ScreenSource>> {
        let mut sources = Vec::new();
        
        // Add displays
        for display in &self.displays {
            sources.push(ScreenSource {
                id: format!("display:{}", display.id),
                name: display.name.clone(),
                width: display.width,
                height: display.height,
                is_display: true,
            });
        }
        
        // Add windows (filter out small windows)
        for window in &self.windows {
            if !window.title.is_empty() && window.width > 100 && window.height > 100 {
                sources.push(ScreenSource {
                    id: format!("window:{}", window.id),
                    name: window.title.clone(),
                    width: window.width,
                    height: window.height,
                    is_display: false,
                });
            }
        }
        
        Ok(sources)
    }
    
    /// Create a display filter asynchronously using real ScreenCaptureKit
    pub async fn create_display_filter(&self, display_id: u32) -> Result<*mut SCContentFilter> {
        println!("🖥️ Creating real display filter for display ID: {}", display_id);
        
        // Find the display in our list
        let display_info = self.displays.iter()
            .find(|d| d.id == display_id)
            .ok_or_else(|| Error::new(Status::InvalidArg, format!("Display {} not found", display_id)))?;
        
        // Create a real content filter using ScreenCaptureKit
        unsafe {
            // Use the display info we already have instead of getting fresh content
            // This avoids the Send issue with raw pointers
            let filter = super::bindings::ScreenCaptureKitAPI::create_content_filter_with_display_id(display_info.id);
            
            if filter.is_null() {
                return Err(Error::new(Status::GenericFailure, "Failed to create content filter"));
            }
            
            println!("✅ Created real SCContentFilter for display: {}", display_info.name);
            Ok(filter)
        }
    }
    
    /// Get the raw ScreenCaptureKit content pointer (not needed for async-only approach)
    pub fn get_sc_content_ptr(&self) -> *mut SCShareableContent {
        // In the async-only approach, we don't store raw pointers
        std::ptr::null_mut()
    }
    
    /// Get displays
    pub fn get_displays(&self) -> Result<Vec<DisplayInfo>> {
        Ok(self.displays.clone())
    }
    
    /// Get windows
    pub fn get_windows(&self) -> Result<Vec<WindowInfo>> {
        Ok(self.windows.clone())
    }
}

// Safety: Raw pointers are only used within unsafe blocks and data is extracted immediately
unsafe impl Send for ShareableContent {}
unsafe impl Sync for ShareableContent {}