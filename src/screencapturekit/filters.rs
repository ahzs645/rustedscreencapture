// Content filter management
// This module handles creation and management of ScreenCaptureKit content filters

use napi::{Result, Status, Error};
use std::ptr;

use super::types::*;
use super::bindings::ScreenCaptureKitAPI;
use super::foundation::PermissionHelpers;

/// Content filter wrapper that provides safe access to SCContentFilter
pub struct ContentFilter {
    filter_ptr: *mut SCContentFilter,
    filter_type: ContentFilterType,
    is_valid: bool,
}

impl ContentFilter {
    /// Create a new content filter for a display
    pub unsafe fn new_for_display(
        shareable_content: *mut SCShareableContent,
        display_id: u32,
    ) -> Result<Self> {
        // Check permissions first
        if !PermissionHelpers::check_screen_recording_permission() {
            return Err(Error::new(Status::GenericFailure, "Screen recording permission required"));
        }

        // Extract the display from shareable content
        let displays = ScreenCaptureKitAPI::extract_displays(shareable_content)
            .map_err(|e| Error::new(Status::GenericFailure, e))?;

        // Find the requested display
        let target_display = displays
            .into_iter()
            .find(|&display| {
                let (id, _, _, _) = ScreenCaptureKitAPI::get_display_info(display);
                id == display_id
            })
            .ok_or_else(|| Error::new(Status::InvalidArg, format!("Display {} not found", display_id)))?;

        // Create content filter
        let filter_ptr = ScreenCaptureKitAPI::create_content_filter_with_display(target_display);
        
        if filter_ptr.is_null() {
            return Err(Error::new(Status::GenericFailure, "Failed to create display content filter"));
        }

        Ok(Self {
            filter_ptr,
            filter_type: ContentFilterType::Display(display_id),
            is_valid: true,
        })
    }

    /// Create a new content filter for a window
    pub unsafe fn new_for_window(
        shareable_content: *mut SCShareableContent,
        window_id: u32,
    ) -> Result<Self> {
        // Check permissions first
        if !PermissionHelpers::check_screen_recording_permission() {
            return Err(Error::new(Status::GenericFailure, "Screen recording permission required"));
        }

        // Extract the window from shareable content
        let windows = ScreenCaptureKitAPI::extract_windows(shareable_content)
            .map_err(|e| Error::new(Status::GenericFailure, e))?;

        // Find the requested window
        let target_window = windows
            .into_iter()
            .find(|&window| {
                let (id, _, _, _) = ScreenCaptureKitAPI::get_window_info(window);
                id == window_id
            })
            .ok_or_else(|| Error::new(Status::InvalidArg, format!("Window {} not found", window_id)))?;

        // Create content filter
        let filter_ptr = ScreenCaptureKitAPI::create_content_filter_with_window(target_window);
        
        if filter_ptr.is_null() {
            return Err(Error::new(Status::GenericFailure, "Failed to create window content filter"));
        }

        Ok(Self {
            filter_ptr,
            filter_type: ContentFilterType::Window(window_id),
            is_valid: true,
        })
    }

    /// Create a basic content filter (fallback)
    pub unsafe fn new_basic() -> Result<Self> {
        // This creates a minimal filter that should work in most cases
        let filter_ptr = ScreenCaptureKitAPI::create_content_filter_with_display(ptr::null_mut());
        
        if filter_ptr.is_null() {
            return Err(Error::new(Status::GenericFailure, "Failed to create basic content filter"));
        }

        Ok(Self {
            filter_ptr,
            filter_type: ContentFilterType::Desktop,
            is_valid: true,
        })
    }

    /// Get the raw filter pointer
    pub fn get_filter_ptr(&self) -> *mut SCContentFilter {
        if self.is_valid {
            self.filter_ptr
        } else {
            ptr::null_mut()
        }
    }

    /// Check if the filter is valid
    pub fn is_valid(&self) -> bool {
        self.is_valid && !self.filter_ptr.is_null()
    }

    /// Get the filter type
    pub fn get_filter_type(&self) -> ContentFilterType {
        self.filter_type
    }

    /// Invalidate the filter
    pub fn invalidate(&mut self) {
        self.is_valid = false;
        // Note: We don't deallocate the filter_ptr as it's managed by the Objective-C runtime
    }
}

impl Drop for ContentFilter {
    fn drop(&mut self) {
        self.invalidate();
    }
}

/// Content filter factory for creating different types of filters
pub struct ContentFilterFactory;

impl ContentFilterFactory {
    /// Create the best available content filter for a display
    pub unsafe fn create_display_filter(
        shareable_content: Option<*mut SCShareableContent>,
        display_id: u32,
    ) -> Result<ContentFilter> {
        if let Some(content) = shareable_content {
            // Try to create with real shareable content
            match ContentFilter::new_for_display(content, display_id) {
                Ok(filter) => return Ok(filter),
                Err(e) => {
                    println!("⚠️ Failed to create display filter with shareable content: {}", e);
                }
            }
        }

        // Fallback to basic filter
        println!("💡 Using basic content filter as fallback");
        ContentFilter::new_basic()
    }

    /// Create the best available content filter for a window
    pub unsafe fn create_window_filter(
        shareable_content: Option<*mut SCShareableContent>,
        window_id: u32,
    ) -> Result<ContentFilter> {
        if let Some(content) = shareable_content {
            // Try to create with real shareable content
            match ContentFilter::new_for_window(content, window_id) {
                Ok(filter) => return Ok(filter),
                Err(e) => {
                    println!("⚠️ Failed to create window filter with shareable content: {}", e);
                }
            }
        }

        // Fallback to basic filter
        println!("💡 Using basic content filter as fallback");
        ContentFilter::new_basic()
    }

    /// Create a basic desktop capture filter
    pub unsafe fn create_desktop_filter() -> Result<ContentFilter> {
        ContentFilter::new_basic()
    }
} 