// ScreenCaptureKit implementation with objc2 bindings
// Reorganized for better maintainability and separation of concerns

// Core Foundation Layer
pub mod foundation;      // Core Graphics, system APIs, basic types
pub mod bindings;        // Raw ScreenCaptureKit bindings only
pub mod types;          // Shared types and constants

// Content Discovery Layer  
pub mod content;        // Content enumeration and management
pub mod filters;        // Content filter creation and management

// Recording Layer - re-enabled for full functionality
pub mod recording;      // High-level recording management
pub mod permissions;    // Permission management

// Stream Management Layer
pub mod audio;
pub mod delegate;
pub mod encoder;
pub mod stream;
pub mod stream_output;
pub mod transcription;
pub mod objc_bridge_rust;

// Permission management (legacy compatibility)
pub mod permission_manager;
pub mod recording_manager;

// Re-export main types and functions for easy access
pub use content::{AsyncContentManager, ShareableContent};
pub use types::{DisplayInfo, WindowInfo, RecordingState, SCError};
pub use recording::RecordingManager;
pub use filters::{ContentFilter, ContentFilterFactory};
pub use permissions::PermissionManager;

// Stream output for recording
pub use stream_output::StreamOutput;
 