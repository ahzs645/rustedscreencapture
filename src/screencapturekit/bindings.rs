// Raw ScreenCaptureKit API bindings
// This module provides direct access to ScreenCaptureKit APIs without complex logic

use objc2::runtime::AnyObject;
use objc2::{msg_send, class};
use objc2_foundation::{NSString, NSError, NSArray};
use objc2_core_media::{CMSampleBuffer, CMTime};
use block2::{Block, StackBlock};
use std::ptr;

pub use super::types::*;
use super::foundation::CGRect;

/// Raw ScreenCaptureKit API bindings
/// This provides direct access to ScreenCaptureKit APIs without complex logic
pub struct ScreenCaptureKitAPI;

impl ScreenCaptureKitAPI {
    /// Get shareable content asynchronously
    pub unsafe fn get_shareable_content_async<F>(completion: F) 
    where
        F: Fn(Option<*mut SCShareableContent>, Option<&NSError>) + Send + Sync + Clone + 'static,
    {
        let block = StackBlock::new(move |content: *mut SCShareableContent, error: *mut NSError| {
            let error_ref = if error.is_null() { None } else { Some(&*error) };
            let content_opt = if content.is_null() { None } else { Some(content) };
            completion(content_opt, error_ref);
        });
        let block = block.copy();
        
        let class = class!(SCShareableContent);
        let _: () = msg_send![
            class,
            getShareableContentWithCompletionHandler: &*block
        ];
    }

    /// Create a content filter with display
    pub unsafe fn create_content_filter_with_display(display: *mut SCDisplay) -> *mut SCContentFilter {
        let class = class!(SCContentFilter);
        let alloc: *mut AnyObject = msg_send![class, alloc];
        msg_send![alloc, initWithDisplay: display]
    }
    
    /// Create a content filter with window
    pub unsafe fn create_content_filter_with_window(window: *mut SCWindow) -> *mut SCContentFilter {
        let class = class!(SCContentFilter);
        let alloc: *mut AnyObject = msg_send![class, alloc];
        msg_send![alloc, initWithDesktopIndependentWindow: window]
    }

    /// Create stream configuration
    pub unsafe fn create_stream_configuration() -> *mut SCStreamConfiguration {
        let class = class!(SCStreamConfiguration);
        let alloc: *mut AnyObject = msg_send![class, alloc];
        msg_send![alloc, init]
    }
    
    /// Configure stream configuration
    pub unsafe fn configure_stream_configuration(
        config: *mut SCStreamConfiguration,
        width: u32,
        height: u32,
        fps: u32,
        shows_cursor: bool,
        captures_audio: bool,
        pixel_format: u32,
    ) {
        let _: () = msg_send![config, setWidth: width];
        let _: () = msg_send![config, setHeight: height];
        
        let frame_interval = CMTime {
            value: 1,
            timescale: fps as i32,
            flags: objc2_core_media::CMTimeFlags(0),
            epoch: 0,
        };
        let _: () = msg_send![config, setMinimumFrameInterval: frame_interval];
        
        let _: () = msg_send![config, setShowsCursor: shows_cursor];
        let _: () = msg_send![config, setCapturesAudio: captures_audio];
        let _: () = msg_send![config, setPixelFormat: pixel_format];
    }

    /// Create SCStream
    pub unsafe fn create_stream(
        filter: *mut SCContentFilter,
        configuration: *mut SCStreamConfiguration,
        delegate: *mut AnyObject,
    ) -> *mut SCStream {
        let class = class!(SCStream);
        let alloc: *mut AnyObject = msg_send![class, alloc];
        msg_send![
            alloc,
            initWithFilter: filter,
            configuration: configuration,
            delegate: delegate
        ]
    }

    /// Start stream capture
    pub unsafe fn start_stream_capture_async<F>(stream: *mut SCStream, completion: F)
    where
        F: Fn(Option<&NSError>) + Send + Sync + Clone + 'static,
    {
        let block = StackBlock::new(move |error: *mut NSError| {
            let error_ref = if error.is_null() { None } else { Some(&*error) };
            completion(error_ref);
        });
        let block = block.copy();
        
        let _: () = msg_send![
            stream,
            startCaptureWithCompletionHandler: &*block
        ];
    }
    
    /// Stop stream capture
    pub unsafe fn stop_stream_capture_async<F>(stream: *mut SCStream, completion: F)
    where
        F: Fn(Option<&NSError>) + Send + Sync + Clone + 'static,
    {
        let block = StackBlock::new(move |error: *mut NSError| {
            let error_ref = if error.is_null() { None } else { Some(&*error) };
            completion(error_ref);
        });
        let block = block.copy();
        
        let _: () = msg_send![
            stream,
            stopCaptureWithCompletionHandler: &*block
        ];
    }

    /// Get display information from SCDisplay
    pub unsafe fn get_display_info(display: *mut SCDisplay) -> (u32, String, u32, u32) {
        if display.is_null() {
            return (0, "Unknown Display".to_string(), 0, 0);
        }
        
        let display_id: u32 = msg_send![display, displayID];
        let width: u32 = msg_send![display, width];
        let height: u32 = msg_send![display, height];
        let name = format!("Display {}", display_id);
        
        (display_id, name, width, height)
    }
    
    /// Get window information from SCWindow
    pub unsafe fn get_window_info(window: *mut SCWindow) -> (u32, String, u32, u32) {
        if window.is_null() {
            return (0, "Unknown Window".to_string(), 0, 0);
        }
        
        let window_id: u32 = msg_send![window, windowID];
        let frame: CGRect = msg_send![window, frame];
        let title = format!("Window {}", window_id);
        
        (window_id, title, frame.size.width as u32, frame.size.height as u32)
    }

    /// Extract displays from shareable content
    pub unsafe fn extract_displays(content: *mut SCShareableContent) -> Result<Vec<*mut SCDisplay>, String> {
        if content.is_null() {
            return Err("Content is null".to_string());
        }
        
        let displays_array: *mut NSArray = msg_send![content, displays];
        if displays_array.is_null() {
            return Err("No displays array".to_string());
        }
        
        let displays = &*displays_array;
        let count = displays.count();
        let mut result = Vec::new();
        
        for i in 0..count {
            let display: *mut SCDisplay = msg_send![displays, objectAtIndex: i];
            if !display.is_null() {
                result.push(display);
            }
        }
        
        Ok(result)
    }

    /// Extract windows from shareable content
    pub unsafe fn extract_windows(content: *mut SCShareableContent) -> Result<Vec<*mut SCWindow>, String> {
        if content.is_null() {
            return Err("Content is null".to_string());
        }
        
        let windows_array: *mut NSArray = msg_send![content, windows];
        if windows_array.is_null() {
            return Err("No windows array".to_string());
        }
        
        let windows = &*windows_array;
        let count = windows.count();
        let mut result = Vec::new();
        
        for i in 0..count {
            let window: *mut SCWindow = msg_send![windows, objectAtIndex: i];
            if !window.is_null() {
                result.push(window);
            }
        }
        
        Ok(result)
    }
}

// Pixel format constants for ScreenCaptureKit
pub const kCVPixelFormatType_32BGRA: u32 = 0x42475241; // 'BGRA'
pub const kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange: u32 = 0x34323076; // '420v' as hex

// Color space constants
pub const kCGColorSpaceDisplayP3: u32 = 0;
pub const kCGColorSpaceSRGB: u32 = 1;