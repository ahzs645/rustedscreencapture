// ScreenCaptureKit implementation with objc2 bindings

pub mod audio;
pub mod bindings;
pub mod content;
pub mod delegate;
pub mod encoder;  // RE-ENABLED: Fixed AVVideoAverageBitRateKey issue
pub mod stream;
pub mod stream_output;
pub mod permission_manager;
pub mod transcription;
pub mod recording_manager;

// Re-export key types for easier access
pub use content::ShareableContent;
pub use audio::AudioManager;
// Note: These exports are available but may show as unused in some contexts
// They are used by the main lib.rs and external consumers
pub use stream_output::StreamOutput;
pub use permission_manager::PermissionManager;
pub use transcription::{TranscriptionManager, TranscriptionConfig, TranscriptionResult};
pub use recording_manager::RecordingManager;
 