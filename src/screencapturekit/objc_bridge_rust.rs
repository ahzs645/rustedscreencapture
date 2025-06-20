use std::ffi::c_void;
use std::sync::{Arc, Weak};
use objc2_core_media::CMSampleBuffer;
use objc2_foundation::NSError;
use objc2::runtime::AnyObject;

use super::delegate::RealStreamDelegate;

// External C functions from the Objective-C bridge
extern "C" {
    fn create_delegate_bridge(
        rust_context: *mut c_void,
        video_callback: extern "C" fn(*mut c_void, *const CMSampleBuffer),
        audio_callback: extern "C" fn(*mut c_void, *const CMSampleBuffer),
        stream_stopped_callback: extern "C" fn(*mut c_void, *const NSError),
    ) -> *mut c_void;
    
    fn release_delegate_bridge(bridge: *mut c_void);
}

// Callback functions that bridge from C to Rust
extern "C" fn video_callback_bridge(context: *mut c_void, sample_buffer: *const CMSampleBuffer) {
    if context.is_null() || sample_buffer.is_null() {
        println!("❌ video_callback_bridge: NULL context or sample buffer");
        return;
    }
    
    unsafe {
        let delegate_ref = &*(context as *const RealStreamDelegate);
        let sample_buffer_ref = &*sample_buffer;
        
        println!("📹 Video callback bridge: forwarding to Rust delegate");
        delegate_ref.handle_video_sample_buffer(sample_buffer_ref);
    }
}

extern "C" fn audio_callback_bridge(context: *mut c_void, sample_buffer: *const CMSampleBuffer) {
    if context.is_null() || sample_buffer.is_null() {
        println!("❌ audio_callback_bridge: NULL context or sample buffer");
        return;
    }
    
    unsafe {
        let delegate_ref = &*(context as *const RealStreamDelegate);
        let sample_buffer_ref = &*sample_buffer;
        
        println!("🔊 Audio callback bridge: forwarding to Rust delegate");
        delegate_ref.handle_audio_sample_buffer(sample_buffer_ref);
    }
}

extern "C" fn stream_stopped_callback_bridge(context: *mut c_void, error: *const NSError) {
    if context.is_null() {
        println!("❌ stream_stopped_callback_bridge: NULL context");
        return;
    }
    
    unsafe {
        let delegate_ref = &*(context as *const RealStreamDelegate);
        let error_ref = if error.is_null() { None } else { Some(&*error) };
        
        println!("🛑 Stream stopped callback bridge: forwarding to Rust delegate");
        delegate_ref.handle_stream_stopped(error_ref);
    }
}

/// Wrapper for the Objective-C delegate bridge
pub struct ObjCDelegateBridge {
    bridge_ptr: *mut c_void,
    _delegate: Arc<RealStreamDelegate>, // Keep delegate alive
}

impl ObjCDelegateBridge {
    /// Create a new Objective-C delegate bridge
    pub fn new(delegate: Arc<RealStreamDelegate>) -> Result<Self, String> {
        println!("🔧 Creating Objective-C delegate bridge");
        
        // Get raw pointer to the delegate for use as context
        let context_ptr = Arc::as_ptr(&delegate) as *mut c_void;
        
        unsafe {
            let bridge_ptr = create_delegate_bridge(
                context_ptr,
                video_callback_bridge,
                audio_callback_bridge,
                stream_stopped_callback_bridge,
            );
            
            if bridge_ptr.is_null() {
                return Err("Failed to create Objective-C delegate bridge".to_string());
            }
            
            println!("✅ Objective-C delegate bridge created successfully");
            
            Ok(Self {
                bridge_ptr,
                _delegate: delegate,
            })
        }
    }
    
    /// Get the raw pointer to the Objective-C delegate object
    pub fn as_objc_delegate(&self) -> *mut AnyObject {
        self.bridge_ptr as *mut AnyObject
    }
    
    /// Check if the bridge is valid
    pub fn is_valid(&self) -> bool {
        !self.bridge_ptr.is_null()
    }
}

impl Drop for ObjCDelegateBridge {
    fn drop(&mut self) {
        if !self.bridge_ptr.is_null() {
            println!("🗑️ Releasing Objective-C delegate bridge");
            unsafe {
                release_delegate_bridge(self.bridge_ptr);
            }
            self.bridge_ptr = std::ptr::null_mut();
        }
    }
}

// Ensure the bridge is Send and Sync for use across threads
unsafe impl Send for ObjCDelegateBridge {}
unsafe impl Sync for ObjCDelegateBridge {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    
    #[test]
    fn test_bridge_creation() {
        // This test can only run on macOS with ScreenCaptureKit
        if cfg!(target_os = "macos") {
            let is_recording = Arc::new(Mutex::new(false));
            let delegate = Arc::new(RealStreamDelegate::new(
                "test_output".to_string(),
                is_recording,
                1920,
                1080,
                30,
            ));
            
            let bridge = ObjCDelegateBridge::new(delegate);
            assert!(bridge.is_ok(), "Bridge creation should succeed");
            
            let bridge = bridge.unwrap();
            assert!(bridge.is_valid(), "Bridge should be valid");
            assert!(!bridge.as_objc_delegate().is_null(), "Delegate pointer should not be null");
        }
    }
} 