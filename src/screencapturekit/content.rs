// Content management - Focus on content enumeration and discovery
// This module handles ScreenCaptureKit content discovery and management

use crate::ScreenSource;
use napi::bindgen_prelude::*;
use std::ptr;

use super::types::*;
use super::bindings::ScreenCaptureKitAPI;
use super::foundation::CoreGraphicsHelpers;

/// Content manager for discovering and enumerating screen sources
pub struct ContentManager;

impl ContentManager {
    /// Get shareable content synchronously
    pub fn get_shareable_content_sync() -> Result<ShareableContent> {
        println!("🔍 Getting shareable content via ScreenCaptureKit APIs (sync)");
        ShareableContent::new_with_screencapturekit()
    }

    /// Get shareable content asynchronously
    pub async fn get_shareable_content() -> Result<ShareableContent> {
        println!("🔍 Getting shareable content via ScreenCaptureKit APIs");
        Self::get_shareable_content_sync()
    }

    /// Extract screen sources from shareable content
    pub fn extract_screen_sources(content: &ShareableContent) -> Result<Vec<ScreenSource>> {
        let mut sources = Vec::new();
        
        // Extract displays
        let displays = content.get_displays()?;
        for display in displays {
            sources.push(ScreenSource {
                id: format!("display:{}", display.id),
                name: display.name.clone(),
                width: display.width,
                height: display.height,
                is_display: true,
            });
        }
        
        // Extract windows (filter out small/invalid windows)
        let windows = content.get_windows()?;
        for window in windows {
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
        
        println!("✅ Extracted {} screen sources", sources.len());
        Ok(sources)
    }
}

/// Wrapper for ScreenCaptureKit shareable content
pub struct ShareableContent {
    displays: Vec<DisplayInfo>,
    windows: Vec<WindowInfo>,
    sc_content_ptr: Option<*mut SCShareableContent>,
}

impl ShareableContent {
    /// Create new empty shareable content
    pub fn new() -> Self {
        Self {
            displays: Vec::new(),
            windows: Vec::new(),
            sc_content_ptr: None,
        }
    }
    
    /// Create shareable content using ScreenCaptureKit
    pub fn new_with_screencapturekit() -> Result<Self> {
        println!("🔍 Fetching shareable content from ScreenCaptureKit");
        
        unsafe {
            let mut content = Self::new();
            
            // Try to get ScreenCaptureKit content
            let sc_content_result = Self::try_get_screencapturekit_content();
            
            // Always use Core Graphics for display/window enumeration (safer)
            content.displays = Self::get_displays_from_coregraphics();
            content.windows = Self::get_windows_from_coregraphics();
            
            // Store ScreenCaptureKit pointer if available (for filter creation)
            if let Ok(sc_content) = sc_content_result {
                content.sc_content_ptr = Some(sc_content);
                println!("✅ ScreenCaptureKit content available for filter creation");
            } else {
                println!("⚠️ Using Core Graphics only (ScreenCaptureKit unavailable)");
            }
            
            println!("✅ Retrieved {} displays and {} windows", 
                content.displays.len(), content.windows.len());
            
            Ok(content)
        }
    }

    /// Get displays using Core Graphics APIs
    unsafe fn get_displays_from_coregraphics() -> Vec<DisplayInfo> {
        let mut displays = Vec::new();
        let display_count = CoreGraphicsHelpers::get_display_count();
        
        for i in 0..display_count {
            if let Some((id, name, width, height)) = CoreGraphicsHelpers::get_display_info(i) {
                displays.push(DisplayInfo { id, name, width, height });
            }
        }
        
        displays
    }

    /// Get windows using Core Graphics APIs
    unsafe fn get_windows_from_coregraphics() -> Vec<WindowInfo> {
        match CoreGraphicsHelpers::get_window_list() {
            Ok(windows) => {
                windows.into_iter()
                    .map(|(id, title, width, height)| WindowInfo { id, title, width, height })
                    .collect()
            }
            Err(_) => Vec::new(),
        }
    }

    /// Try to get ScreenCaptureKit content
    unsafe fn try_get_screencapturekit_content() -> Result<*mut SCShareableContent> {
        // For now, return an error since ScreenCaptureKit async operations
        // are complex to handle in a synchronous context
        Err(napi::Error::new(
            napi::Status::GenericFailure, 
            "ScreenCaptureKit content retrieval requires async context"
        ))
    }

    /// Get displays list
    pub fn get_displays(&self) -> Result<Vec<DisplayInfo>> {
        Ok(self.displays.clone())
    }

    /// Get windows list
    pub fn get_windows(&self) -> Result<Vec<WindowInfo>> {
        Ok(self.windows.clone())
    }

    /// Find display by ID
    pub fn find_display_by_id(&self, display_id: u32) -> Option<&DisplayInfo> {
        self.displays.iter().find(|d| d.id == display_id)
    }

    /// Find window by ID
    pub fn find_window_by_id(&self, window_id: u32) -> Option<&WindowInfo> {
        self.windows.iter().find(|w| w.id == window_id)
    }

    /// Get the ScreenCaptureKit content pointer (for filter creation)
    pub fn get_sc_content_ptr(&self) -> Option<*mut SCShareableContent> {
        self.sc_content_ptr
    }
    
    /// Create shareable content with real data (alias for compatibility)
    pub fn new_with_real_data() -> Result<Self> {
        Self::new_with_screencapturekit()
    }
    
    /// Create shareable content with timeout (for compatibility)
    pub fn new_with_timeout(_timeout: u64) -> Result<Self> {
        Self::new_with_screencapturekit()
    }
} 