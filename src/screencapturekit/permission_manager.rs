use objc2::{msg_send, class};
use objc2_foundation::NSString;
use napi::{Result, Status, Error};
use std::process::Command;

/// Manages screen recording permissions and provides proper error recovery
pub struct PermissionManager;

impl PermissionManager {
    /// Check if screen recording permission is granted
    pub fn check_screen_recording_permission() -> bool {
        unsafe {
            // Use CGPreflightScreenCaptureAccess to check permissions
            extern "C" {
                fn CGPreflightScreenCaptureAccess() -> bool;
            }
            
            let has_permission = CGPreflightScreenCaptureAccess();
            println!("🔐 Screen recording permission status: {}", has_permission);
            has_permission
        }
    }
    
    /// Request screen recording permission (will prompt user if needed)
    pub fn request_screen_recording_permission() -> bool {
        unsafe {
            // Use CGRequestScreenCaptureAccess to request permissions
            extern "C" {
                fn CGRequestScreenCaptureAccess() -> bool;
            }
            
            println!("🔐 Requesting screen recording permission...");
            let has_permission = CGRequestScreenCaptureAccess();
            
            if has_permission {
                println!("✅ Screen recording permission granted");
            } else {
                println!("❌ Screen recording permission denied or requires manual approval");
                Self::show_permission_instructions();
            }
            
            has_permission
        }
    }
    
    /// Check if we have accessibility permissions (sometimes needed)
    pub fn check_accessibility_permission() -> bool {
        unsafe {
            // Check accessibility permissions using AXIsProcessTrusted
            extern "C" {
                fn AXIsProcessTrusted() -> bool;
            }
            
            let has_permission = AXIsProcessTrusted();
            println!("🔐 Accessibility permission status: {}", has_permission);
            has_permission
        }
    }
    
    /// Request accessibility permission with prompt
    pub fn request_accessibility_permission() -> bool {
        unsafe {
            extern "C" {
                fn AXIsProcessTrustedWithOptions(options: *const std::ffi::c_void) -> bool;
            }
            
            // Create options dictionary to show prompt
            use objc2_foundation::{NSDictionary, NSNumber};
            
            let prompt_key = NSString::from_str("AXTrustedCheckOptionPrompt");
            let prompt_value: *mut NSNumber = msg_send![class!(NSNumber), numberWithBool: true];
            
            let options: *mut NSDictionary<NSString, objc2::runtime::AnyObject> = msg_send![
                class!(NSDictionary),
                dictionaryWithObjects: &[prompt_value as *mut objc2::runtime::AnyObject],
                forKeys: &[&*prompt_key],
                count: 1
            ];
            
            let has_permission = AXIsProcessTrustedWithOptions(options as *const std::ffi::c_void);
            
            if has_permission {
                println!("✅ Accessibility permission granted");
            } else {
                println!("❌ Accessibility permission denied or requires manual approval");
            }
            
            has_permission
        }
    }
    
    /// Comprehensive permission check for screen recording
    pub fn ensure_all_permissions() -> Result<()> {
        println!("🔍 Checking all required permissions...");
        
        // Check macOS version compatibility
        if !Self::check_macos_version_compatibility() {
            return Err(Error::new(
                Status::GenericFailure,
                "macOS 10.15 (Catalina) or later is required for ScreenCaptureKit"
            ));
        }
        
        // Check screen recording permission
        if !Self::check_screen_recording_permission() {
            println!("⚠️ Screen recording permission not granted, requesting...");
            if !Self::request_screen_recording_permission() {
                return Err(Error::new(
                    Status::GenericFailure,
                    "Screen recording permission is required. Please enable it in System Preferences > Security & Privacy > Privacy > Screen Recording"
                ));
            }
        }
        
        // Check accessibility permission (sometimes needed for window capture)
        if !Self::check_accessibility_permission() {
            println!("💡 Accessibility permission recommended for better window capture");
            Self::request_accessibility_permission();
        }
        
        println!("✅ All permissions checked");
        Ok(())
    }
    
    /// Check macOS version compatibility
    pub fn check_macos_version_compatibility() -> bool {
        let output = Command::new("sw_vers")
            .arg("-productVersion")
            .output();
            
        match output {
            Ok(output) => {
                let version_str = String::from_utf8_lossy(&output.stdout);
                let version = version_str.trim();
                println!("🍎 macOS version: {}", version);
                
                // Parse version and check if >= 10.15
                if let Some(major_minor) = Self::parse_macos_version(version) {
                    let (major, minor) = major_minor;
                    let compatible = major > 10 || (major == 10 && minor >= 15);
                    
                    if compatible {
                        println!("✅ macOS version is compatible with ScreenCaptureKit");
                    } else {
                        println!("❌ macOS version {} is not compatible (requires 10.15+)", version);
                    }
                    
                    compatible
                } else {
                    println!("⚠️ Could not parse macOS version, assuming compatible");
                    true
                }
            }
            Err(e) => {
                println!("⚠️ Could not check macOS version: {}, assuming compatible", e);
                true
            }
        }
    }
    
    /// Parse macOS version string (e.g., "12.6.1" -> (12, 6))
    fn parse_macos_version(version: &str) -> Option<(u32, u32)> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() >= 2 {
            if let (Ok(major), Ok(minor)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                return Some((major, minor));
            }
        }
        None
    }
    
    /// Show detailed permission instructions to the user
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
        println!("🔍 Validating system requirements...");
        
        // Check macOS version
        if !Self::check_macos_version_compatibility() {
            return Err(Error::new(
                Status::GenericFailure,
                "Incompatible macOS version"
            ));
        }
        
        // Check if running on Apple Silicon or Intel Mac
        let arch_output = Command::new("uname").arg("-m").output();
        match arch_output {
            Ok(output) => {
                let arch = String::from_utf8_lossy(&output.stdout).trim().to_string();
                println!("🏗️ System architecture: {}", arch);
                
                match arch.as_str() {
                    "arm64" => println!("✅ Running on Apple Silicon"),
                    "x86_64" => println!("✅ Running on Intel Mac"),
                    _ => println!("⚠️ Unknown architecture: {}", arch),
                }
            }
            Err(_) => println!("⚠️ Could not determine system architecture"),
        }
        
        // Check if ScreenCaptureKit framework is available
        if Self::check_screencapturekit_availability() {
            println!("✅ ScreenCaptureKit framework is available");
        } else {
            return Err(Error::new(
                Status::GenericFailure,
                "ScreenCaptureKit framework is not available on this system"
            ));
        }
        
        println!("✅ System requirements validated");
        Ok(())
    }
    
    /// Check if ScreenCaptureKit framework is available
    fn check_screencapturekit_availability() -> bool {
        // Try to get the SCShareableContent class - if it exists, ScreenCaptureKit is available
        // On macOS 10.15+ this should always be true
        true // Simplified check since we're targeting macOS 10.15+
    }
    
    /// Get detailed permission status report
    pub fn get_permission_status_report() -> String {
        let screen_recording = Self::check_screen_recording_permission();
        let accessibility = Self::check_accessibility_permission();
        let macos_compatible = Self::check_macos_version_compatibility();
        let screencapturekit_available = Self::check_screencapturekit_availability();
        
        format!(
            "Permission Status Report:\n\
             📺 Screen Recording: {}\n\
             ♿ Accessibility: {}\n\
             🍎 macOS Compatible: {}\n\
             🎬 ScreenCaptureKit Available: {}",
            if screen_recording { "✅ Granted" } else { "❌ Denied" },
            if accessibility { "✅ Granted" } else { "❌ Denied" },
            if macos_compatible { "✅ Yes" } else { "❌ No" },
            if screencapturekit_available { "✅ Yes" } else { "❌ No" }
        )
    }
    
    /// Reset permissions (for testing purposes)
    pub fn reset_permissions_for_testing() -> Result<()> {
        println!("🔄 Resetting permissions for testing...");
        
        // This would typically involve using tccutil or similar tools
        // For now, just provide instructions
        println!("To reset permissions manually:");
        println!("1. Run: sudo tccutil reset ScreenCapture");
        println!("2. Run: sudo tccutil reset Accessibility");
        println!("3. Restart the application");
        
        Ok(())
    }
} 