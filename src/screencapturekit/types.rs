// Shared types and constants for ScreenCaptureKit
// This module centralizes all type definitions and constants

use objc2::runtime::AnyObject;
use objc2_core_media::{CMSampleBuffer, CMTime};
use objc2_foundation::{NSString, NSError};
use block2::Block;

// ScreenCaptureKit Class Names
pub const SC_SHAREABLE_CONTENT_CLASS: &str = "SCShareableContent";
pub const SC_DISPLAY_CLASS: &str = "SCDisplay";
pub const SC_WINDOW_CLASS: &str = "SCWindow";
pub const SC_CONTENT_FILTER_CLASS: &str = "SCContentFilter";
pub const SC_STREAM_CLASS: &str = "SCStream";
pub const SC_STREAM_CONFIGURATION_CLASS: &str = "SCStreamConfiguration";

// Type aliases for ScreenCaptureKit objects
pub type SCShareableContent = AnyObject;
pub type SCDisplay = AnyObject;
pub type SCWindow = AnyObject;
pub type SCContentFilter = AnyObject;
pub type SCStream = AnyObject;
pub type SCStreamConfiguration = AnyObject;

// Completion handler type aliases
pub type SCShareableContentCompletionHandler = 
    Block<dyn Fn(*mut SCShareableContent, *mut NSError)>;

pub type SCStreamStartCompletionHandler = 
    Block<dyn Fn(*mut NSError)>;

pub type SCStreamStopCompletionHandler = 
    Block<dyn Fn(*mut NSError)>;

// Stream output type enum
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum SCStreamOutputType {
    Screen = 0,
    Audio = 1,
    Microphone = 2,
}

unsafe impl objc2::Encode for SCStreamOutputType {
    const ENCODING: objc2::Encoding = u32::ENCODING;
}

// Stream delegate trait
pub trait SCStreamDelegate {
    fn stream_did_output_sample_buffer(&self, stream: &SCStream, sample_buffer: &CMSampleBuffer, of_type: SCStreamOutputType);
    fn stream_did_stop_with_error(&self, stream: &SCStream, error: Option<&NSError>);
}

// Display information structure
#[derive(Debug, Clone)]
pub struct DisplayInfo {
    pub id: u32,
    pub name: String,
    pub width: u32,
    pub height: u32,
}

// Window information structure
#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub id: u32,
    pub title: String,
    pub width: u32,
    pub height: u32,
}

// Stream configuration structure
#[derive(Debug, Clone)]
pub struct StreamConfiguration {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub shows_cursor: bool,
    pub captures_audio: bool,
    pub pixel_format: u32,
    pub color_space: u32,
}

impl Default for StreamConfiguration {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fps: 30,
            shows_cursor: true,
            captures_audio: false,
            pixel_format: kCVPixelFormatType_32BGRA,
            color_space: kCGColorSpaceSRGB,
        }
    }
}

// Pixel format constants
pub const kCVPixelFormatType_32BGRA: u32 = 0x42475241; // 'BGRA'
pub const kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange: u32 = 0x34323076; // '420v'

// Color space constants
pub const kCGColorSpaceDisplayP3: u32 = 0;
pub const kCGColorSpaceSRGB: u32 = 1;

// Recording state enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecordingState {
    Idle,
    Starting,
    Recording,
    Stopping,
    Error,
}

// Content filter type enum
#[derive(Debug, Clone, Copy)]
pub enum ContentFilterType {
    Display(u32),
    Window(u32),
    Desktop,
    All,
}

// Audio device type enum
#[derive(Debug, Clone, Copy)]
pub enum AudioDeviceType {
    Input,
    Output,
    SystemAudio,
    Microphone,
}

// Permission status enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PermissionStatus {
    Granted,
    Denied,
    NotDetermined,
    Restricted,
}

// Error types specific to ScreenCaptureKit
#[derive(Debug, Clone)]
pub enum SCError {
    PermissionDenied,
    ContentNotFound,
    StreamCreationFailed,
    FilterCreationFailed,
    RecordingFailed,
    InvalidConfiguration,
    SystemError(String),
}

impl std::fmt::Display for SCError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SCError::PermissionDenied => write!(f, "Screen recording permission denied"),
            SCError::ContentNotFound => write!(f, "Screen content not found"),
            SCError::StreamCreationFailed => write!(f, "Failed to create stream"),
            SCError::FilterCreationFailed => write!(f, "Failed to create content filter"),
            SCError::RecordingFailed => write!(f, "Recording failed"),
            SCError::InvalidConfiguration => write!(f, "Invalid configuration"),
            SCError::SystemError(msg) => write!(f, "System error: {}", msg),
        }
    }
}

impl std::error::Error for SCError {}

// Utility functions for type conversions
pub fn create_cmtime_from_fps(fps: u32) -> CMTime {
    CMTime {
        value: 1,
        timescale: fps as i32,
        flags: objc2_core_media::CMTimeFlags(0),
        epoch: 0,
    }
}

pub fn validate_dimensions(width: u32, height: u32) -> Result<(), SCError> {
    if width < 100 || width > 7680 {
        return Err(SCError::InvalidConfiguration);
    }
    if height < 100 || height > 4320 {
        return Err(SCError::InvalidConfiguration);
    }
    Ok(())
}

pub fn validate_fps(fps: u32) -> Result<(), SCError> {
    if fps < 1 || fps > 120 {
        return Err(SCError::InvalidConfiguration);
    }
    Ok(())
} 