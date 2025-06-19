// ScreenCaptureKit implementation with objc2 bindings
// Reorganized for better maintainability and separation of concerns

// Core Foundation Layer
pub mod foundation;      // Core Graphics, system APIs, basic types
pub mod bindings;        // Raw ScreenCaptureKit bindings only
pub mod types;          // Shared types and constants

// Content Discovery Layer  
pub mod content;        // Content enumeration and management
pub mod filters;        // Content filter creation and management

// Recording Layer
pub mod recording;      // High-level recording management
pub mod permissions;    // Permission management

// Legacy modules (to be refactored)
pub mod audio;
pub mod delegate;
pub mod encoder;
pub mod permission_manager;
pub mod recording_manager;
pub mod stream;
pub mod stream_output;
pub mod transcription;

// Re-export main types and functions for easy access
pub use content::{ContentManager, ShareableContent};
pub use recording::RecordingManager;
pub use permissions::PermissionManager;
pub use filters::{ContentFilter, ContentFilterFactory};
pub use types::{DisplayInfo, WindowInfo, SCStream, SCStreamConfiguration, SCStreamOutputType, SCContentFilter, SCShareableContent, SCDisplay, SCWindow};
pub use bindings::ScreenCaptureKitAPI;

// Legacy compatibility exports (can be removed later)
// pub use recording_manager::RealStreamManager;  // Disabled - using new RecordingManager
pub use stream_output::StreamOutput;
 