// ScreenCaptureKit implementation with objc2 bindings
// Reorganized for better maintainability and separation of concerns

// Core Foundation Layer
pub mod foundation;      // Core Graphics, system APIs, basic types
pub mod bindings;        // Raw ScreenCaptureKit bindings only
pub mod types;          // Shared types and constants

// Content Discovery Layer  
pub mod content;        // Content enumeration and management
pub mod filters;        // Content filter creation and management

// Recording Layer - temporarily commented out for compilation
// pub mod recording;      // High-level recording management
pub mod permissions;    // Permission management

// Legacy modules (to be refactored) - temporarily commented out
pub mod audio;
pub mod delegate;
pub mod encoder;
// pub mod permission_manager;
// pub mod recording_manager;
pub mod stream;
pub mod stream_output;
pub mod transcription;

// Re-export main types and functions for easy access
pub use content::{AsyncContentManager, ShareableContent};
pub use types::{DisplayInfo, WindowInfo};

// Legacy compatibility exports (can be removed later)
// pub use stream_output::StreamOutput;
 