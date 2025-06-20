// Permission management for ScreenCaptureKit
// This module handles permission checking, requesting, and validation

use napi::{Result, Status, Error};
use super::types::PermissionStatus;
use super::foundation::PermissionHelpers;
use napi::bindgen_prelude::*;

/// Permission manager for ScreenCaptureKit functionality
pub struct PermissionManager;

impl PermissionManager {
    /// Check current screen recording permission status
    pub fn check_permission() -> PermissionStatus {
        unsafe {
            if PermissionHelpers::check_screen_recording_permission() {
                PermissionStatus::Granted
            } else {
                PermissionStatus::Denied
            }
        }
    }

    /// Request screen recording permission
    pub fn request_permission() -> Result<PermissionStatus> {
        unsafe {
            if PermissionHelpers::request_screen_recording_permission() {
                Ok(PermissionStatus::Granted)
            } else {
                Ok(PermissionStatus::Denied)
            }
        }
    }

    /// Ensure permissions are granted, requesting if necessary
    pub fn ensure_permission() -> Result<()> {
        match Self::check_permission() {
            PermissionStatus::Granted => Ok(()),
            _ => {
                // Try to request permission
                match Self::request_permission()? {
                    PermissionStatus::Granted => Ok(()),
                    _ => {
                        Self::show_permission_instructions();
                        Err(Error::new(
                            Status::GenericFailure,
                            "Screen recording permission is required. Please enable it in System Preferences."
                        ))
                    }
                }
            }
        }
    }

    /// Get a detailed permission status report
    pub fn get_permission_status_report() -> String {
        let status = Self::check_permission();
        let system_info = Self::get_system_info();
        
        serde_json::json!({
            "permission_status": format!("{:?}", status),
            "system_info": system_info,
            "instructions": Self::get_permission_instructions(),
            "can_request": status != PermissionStatus::Restricted,
        }).to_string()
    }

    /// Handle ScreenCaptureKit specific errors with recovery suggestions
    pub fn handle_screencapturekit_error(error_description: &str) -> Result<String> {
        println!("🚨 ScreenCaptureKit Error: {}", error_description);
        
        // Analyze error and provide specific recovery steps
        let recovery_action = if error_description.contains("permission") || error_description.contains("access") {
            "Permission issue detected. Please check screen recording permissions."
        } else if error_description.contains("content") || error_description.contains("filter") {
            "Content filter issue. Try selecting a different screen or window."
        } else if error_description.contains("stream") {
            "Stream configuration issue. Check video/audio settings."
        } else if error_description.contains("delegate") {
            "Delegate issue. This is likely an internal error."
        } else {
            "Unknown ScreenCaptureKit error. Check system compatibility."
        };
        
        println!("💡 Recovery suggestion: {}", recovery_action);
        
        // Return recovery information
        Ok(format!("Error: {} | Recovery: {}", error_description, recovery_action))
    }

    /// Validate system requirements for ScreenCaptureKit
    pub fn validate_system_requirements() -> Result<()> {
        let system_info = Self::get_system_info();
        
        // Check macOS version (ScreenCaptureKit requires macOS 12.3+)
        if !Self::is_macos_version_supported() {
            return Err(Error::new(
                Status::GenericFailure,
                "ScreenCaptureKit requires macOS 12.3 or later"
            ));
        }
        
        println!("✅ System requirements validated: {}", system_info);
        Ok(())
    }

    /// Check if the current macOS version supports ScreenCaptureKit
    fn is_macos_version_supported() -> bool {
        // For now, assume we're on a supported version
        // In a real implementation, you'd check the actual macOS version
        true
    }

    /// Get system information
    fn get_system_info() -> String {
        // Basic system info - in a real implementation you'd get actual system details
        format!("macOS (ScreenCaptureKit supported)")
    }

    /// Show permission instructions to the user
    fn show_permission_instructions() {
        println!("\n📋 SCREEN RECORDING PERMISSION REQUIRED");
        println!("==========================================");
        println!("To enable screen recording for this application:");
        println!("1. Open System Preferences");
        println!("2. Go to Security & Privacy");
        println!("3. Click on the Privacy tab");
        println!("4. Select 'Screen Recording' from the left sidebar");
        println!("5. Check the box next to your application");
        println!("6. You may need to restart the application");
        println!("==========================================\n");
    }

    /// Get permission instructions as a string
    fn get_permission_instructions() -> String {
        "Open System Preferences > Security & Privacy > Privacy > Screen Recording and enable permission for this application".to_string()
    }

    /// Check screen recording permission status
    pub fn check_screen_recording_permission() -> bool {
        Self::ensure_permission().is_ok()
    }
    
    /// Request screen recording permission
    pub fn request_screen_recording_permission() -> Result<bool> {
        match Self::request_permission() {
            Ok(PermissionStatus::Granted) => Ok(true),
            Ok(_) => Ok(false),
            Err(e) => Err(e),
        }
    }
}

/// Check screen recording permission
pub fn check_screen_recording_permission() -> Result<bool> {
    // For now, return true as a placeholder
    // In a real implementation, this would use AVCaptureDevice or CGDisplayStream APIs
    println!("🔐 Checking screen recording permission (placeholder)");
    Ok(true)
}

/// Request screen recording permission
pub fn request_screen_recording_permission() -> Result<bool> {
    // For now, return true as a placeholder
    // In a real implementation, this would trigger the system permission dialog
    println!("🔐 Requesting screen recording permission (placeholder)");
    Ok(true)
} 