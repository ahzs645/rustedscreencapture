// Foundation layer - Core Graphics types and system APIs
// This module provides the basic building blocks for screen capture

use objc2::{msg_send, class};
use objc2_foundation::{NSString, NSError, NSArray, NSDictionary, NSNumber};
use objc2::runtime::AnyObject;
use napi::{Result, Status, Error};
use std::ptr;

// Core Graphics structures for frame handling
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CGRect {
    pub origin: CGPoint,
    pub size: CGSize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CGPoint {
    pub x: f64,
    pub y: f64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CGSize {
    pub width: f64,
    pub height: f64,
}

// Implement encoding for Objective-C interop
unsafe impl objc2::Encode for CGRect {
    const ENCODING: objc2::Encoding = objc2::Encoding::Struct("CGRect", &[CGPoint::ENCODING, CGSize::ENCODING]);
}

unsafe impl objc2::Encode for CGPoint {
    const ENCODING: objc2::Encoding = objc2::Encoding::Struct("CGPoint", &[f64::ENCODING, f64::ENCODING]);
}

unsafe impl objc2::Encode for CGSize {
    const ENCODING: objc2::Encoding = objc2::Encoding::Struct("CGSize", &[f64::ENCODING, f64::ENCODING]);
}

/// Core Graphics helper functions for display and window management
pub struct CoreGraphicsHelpers;

impl CoreGraphicsHelpers {
    /// Get the number of active displays
    pub unsafe fn get_display_count() -> u32 {
        extern "C" {
            fn CGGetActiveDisplayList(maxDisplays: u32, activeDisplays: *mut u32, displayCount: *mut u32) -> i32;
        }
        
        let mut display_count: u32 = 0;
        let result = CGGetActiveDisplayList(0, ptr::null_mut(), &mut display_count);
        
        if result == 0 {
            display_count
        } else {
            1 // Fallback to at least one display
        }
    }

    /// Get display information by index
    pub unsafe fn get_display_info(index: u32) -> Option<(u32, String, u32, u32)> {
        extern "C" {
            fn CGGetActiveDisplayList(maxDisplays: u32, activeDisplays: *mut u32, displayCount: *mut u32) -> i32;
            fn CGDisplayPixelsWide(display: u32) -> usize;
            fn CGDisplayPixelsHigh(display: u32) -> usize;
        }
        
        const MAX_DISPLAYS: u32 = 32;
        let mut displays: [u32; MAX_DISPLAYS as usize] = [0; MAX_DISPLAYS as usize];
        let mut display_count: u32 = 0;
        
        let result = CGGetActiveDisplayList(MAX_DISPLAYS, displays.as_mut_ptr(), &mut display_count);
        
        if result == 0 && index < display_count {
            let display_id = displays[index as usize];
            let width = CGDisplayPixelsWide(display_id) as u32;
            let height = CGDisplayPixelsHigh(display_id) as u32;
            
            let name = if index == 0 {
                "Built-in Display".to_string()
            } else {
                format!("Display {}", index + 1)
            };
            
            Some((display_id, name, width, height))
        } else {
            None
        }
    }

    /// Get the main display ID
    pub unsafe fn get_main_display_id() -> u32 {
        extern "C" {
            fn CGMainDisplayID() -> u32;
        }
        CGMainDisplayID()
    }

    /// Get window information using Core Graphics
    pub unsafe fn get_window_list() -> Result<Vec<(u32, String, u32, u32)>> {
        extern "C" {
            fn CGWindowListCopyWindowInfo(option: u32, relativeToWindow: u32) -> *mut NSArray;
        }
        
        const kCGWindowListOptionOnScreenOnly: u32 = 1 << 0;
        const kCGWindowListExcludeDesktopElements: u32 = 1 << 4;
        
        let mut windows = Vec::new();
        
        let window_list_raw = CGWindowListCopyWindowInfo(
            kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
            0
        );
        
        if window_list_raw.is_null() {
            return Ok(Self::get_fallback_windows());
        }
        
        let window_list: &NSArray = &*window_list_raw;
        let count = window_list.count();
        
        for i in 0..count {
            let window_dict_obj = window_list.objectAtIndex(i);
            if let Ok(window_dict) = window_dict_obj.downcast::<NSDictionary>() {
                if let Some(window_info) = Self::extract_window_from_dict(&window_dict, i as u32) {
                    windows.push(window_info);
                }
            }
        }
        
        // Clean up
        objc2::rc::autoreleasepool(|_| {
            std::ptr::drop_in_place(window_list_raw);
        });
        
        if windows.is_empty() {
            Ok(Self::get_fallback_windows())
        } else {
            Ok(windows)
        }
    }

    unsafe fn extract_window_from_dict(window_dict: &NSDictionary, fallback_id: u32) -> Option<(u32, String, u32, u32)> {
        let window_number_key = NSString::from_str("kCGWindowNumber");
        let window_name_key = NSString::from_str("kCGWindowName");
        let window_owner_name_key = NSString::from_str("kCGWindowOwnerName");
        let window_bounds_key = NSString::from_str("kCGWindowBounds");
        
        let window_id = if let Some(number_obj) = window_dict.objectForKey(&window_number_key) {
            if let Ok(number) = number_obj.downcast::<NSNumber>() {
                number.intValue() as u32
            } else {
                fallback_id
            }
        } else {
            fallback_id
        };
        
        let title = if let Some(name_obj) = window_dict.objectForKey(&window_name_key) {
            if let Ok(name_str) = name_obj.downcast::<NSString>() {
                let title_str = name_str.to_string();
                if !title_str.is_empty() {
                    title_str
                } else {
                    if let Some(owner_obj) = window_dict.objectForKey(&window_owner_name_key) {
                        if let Ok(owner_str) = owner_obj.downcast::<NSString>() {
                            owner_str.to_string()
                        } else {
                            "Unknown Window".to_string()
                        }
                    } else {
                        "Unknown Window".to_string()
                    }
                }
            } else {
                "Unknown Window".to_string()
            }
        } else {
            if let Some(owner_obj) = window_dict.objectForKey(&window_owner_name_key) {
                if let Ok(owner_str) = owner_obj.downcast::<NSString>() {
                    owner_str.to_string()
                } else {
                    "Unknown Window".to_string()
                }
            } else {
                "Unknown Window".to_string()
            }
        };
        
        let (width, height) = if let Some(bounds_obj) = window_dict.objectForKey(&window_bounds_key) {
            if let Ok(bounds_dict) = bounds_obj.downcast::<NSDictionary>() {
                let width_key = NSString::from_str("Width");
                let height_key = NSString::from_str("Height");
                
                let width = if let Some(width_obj) = bounds_dict.objectForKey(&width_key) {
                    if let Ok(width_num) = width_obj.downcast::<NSNumber>() {
                        width_num.intValue() as u32
                    } else {
                        800
                    }
                } else {
                    800
                };
                
                let height = if let Some(height_obj) = bounds_dict.objectForKey(&height_key) {
                    if let Ok(height_num) = height_obj.downcast::<NSNumber>() {
                        height_num.intValue() as u32
                    } else {
                        600
                    }
                } else {
                    600
                };
                
                (width, height)
            } else {
                (800, 600)
            }
        } else {
            (800, 600)
        };
        
        // Filter out invalid windows
        if title.is_empty() || width < 100 || height < 100 {
            return None;
        }
        
        Some((window_id, title, width, height))
    }

    fn get_fallback_windows() -> Vec<(u32, String, u32, u32)> {
        vec![
            (1, "Desktop".to_string(), 1920, 1080),
            (2, "Finder".to_string(), 800, 600),
        ]
    }
}

/// Permission management for screen recording
pub struct PermissionHelpers;

impl PermissionHelpers {
    /// Check if screen recording permissions are granted
    pub unsafe fn check_screen_recording_permission() -> bool {
        extern "C" {
            fn CGPreflightScreenCaptureAccess() -> bool;
        }
        
        let has_permission = CGPreflightScreenCaptureAccess();
        println!("🔐 Screen recording permission status: {}", has_permission);
        has_permission
    }
    
    /// Request screen recording permissions
    pub unsafe fn request_screen_recording_permission() -> bool {
        extern "C" {
            fn CGRequestScreenCaptureAccess() -> bool;
        }
        
        let has_permission = CGRequestScreenCaptureAccess();
        println!("🔐 Screen recording permission after request: {}", has_permission);
        has_permission
    }
} 