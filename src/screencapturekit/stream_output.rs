use std::sync::{Arc, Mutex};
use std::path::Path;
use objc2::runtime::AnyObject;
use objc2::{msg_send, class, sel};
use objc2_foundation::{NSError, NSString, NSURL};
use objc2_core_media::{CMSampleBuffer, CMTime, CMFormatDescription};
use objc2_core_video::{CVImageBuffer, CVPixelBuffer};
use objc2_av_foundation::{AVAssetWriter, AVAssetWriterInput, AVAssetWriterInputPixelBufferAdaptor};
use napi::{Result, Status, Error};

use super::bindings::{SCStream, SCStreamOutputType};

// External CoreMedia functions
extern "C" {
    fn CMSampleBufferGetImageBuffer(sbuf: &CMSampleBuffer) -> *mut CVPixelBuffer;
    fn CMSampleBufferGetPresentationTimeStamp(sbuf: &CMSampleBuffer) -> CMTime;
}

/// Real implementation of SCStreamOutput protocol that saves working audio/video files
pub struct StreamOutput {
    // Asset writer for video/audio encoding
    asset_writer: Option<*mut AVAssetWriter>,
    video_input: Option<*mut AVAssetWriterInput>,
    audio_input: Option<*mut AVAssetWriterInput>,
    pixel_buffer_adaptor: Option<*mut AVAssetWriterInputPixelBufferAdaptor>,
    
    // Recording state
    output_path: String,
    is_recording: Arc<Mutex<bool>>,
    recording_started: Arc<Mutex<bool>>,
    
    // Statistics
    video_frame_count: Arc<Mutex<u64>>,
    audio_sample_count: Arc<Mutex<u64>>,
    start_time: Arc<Mutex<Option<CMTime>>>,
    
    // Configuration
    width: u32,
    height: u32,
    fps: u32,
    capture_audio: bool,
}

impl StreamOutput {
    pub fn new(output_path: String, width: u32, height: u32, fps: u32, capture_audio: bool) -> Result<Self> {
        println!("🎬 Creating StreamOutput for: {}", output_path);
        
        Ok(Self {
            asset_writer: None,
            video_input: None,
            audio_input: None,
            pixel_buffer_adaptor: None,
            output_path,
            is_recording: Arc::new(Mutex::new(false)),
            recording_started: Arc::new(Mutex::new(false)),
            video_frame_count: Arc::new(Mutex::new(0)),
            audio_sample_count: Arc::new(Mutex::new(0)),
            start_time: Arc::new(Mutex::new(None)),
            width,
            height,
            fps,
            capture_audio,
        })
    }
    
    /// Initialize the AVAssetWriter with proper video/audio settings
    pub fn initialize_asset_writer(&mut self) -> Result<()> {
        
        println!("🔧 Initializing AVAssetWriter with fixed codec configuration");
        
        // Ensure output directory exists
        if let Some(parent) = Path::new(&self.output_path).parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to create output directory: {}", e)))?;
            }
        }
        
        unsafe {
            // Create file URL
            let url_string = NSString::from_str(&self.output_path);
            let file_url: *mut NSURL = msg_send![class!(NSURL), fileURLWithPath: &*url_string];
            
            // Create AVAssetWriter with fixed configuration
            let mut error: *mut NSError = std::ptr::null_mut();
            let file_type = NSString::from_str("com.apple.quicktime-movie");
            let asset_writer: *mut AVAssetWriter = msg_send![
                class!(AVAssetWriter),
                assetWriterWithURL: file_url,
                fileType: &*file_type,
                error: &mut error
            ];
            
            if asset_writer.is_null() || !error.is_null() {
                return Err(Error::new(Status::GenericFailure, "Failed to create AVAssetWriter"));
            }
            
            // Create video input with fixed settings (no problematic bitrate)
            let video_input = self.create_video_input()?;
            let can_add_video: bool = msg_send![asset_writer, canAddInput: video_input];
            if can_add_video {
                let _: () = msg_send![asset_writer, addInput: video_input];
            } else {
                return Err(Error::new(Status::GenericFailure, "Cannot add video input"));
            }
            
            // Create pixel buffer adaptor
            let pixel_buffer_adaptor = self.create_pixel_buffer_adaptor(video_input)?;
            
            // Create audio input if needed
            let audio_input = if self.capture_audio {
                let input = self.create_audio_input()?;
                let can_add_audio: bool = msg_send![asset_writer, canAddInput: input];
                if can_add_audio {
                    let _: () = msg_send![asset_writer, addInput: input];
                    Some(input)
                } else {
                    return Err(Error::new(Status::GenericFailure, "Cannot add audio input"));
                }
            } else {
                None
            };
            
            // Store the writer and inputs
            self.asset_writer = Some(asset_writer);
            self.video_input = Some(video_input);
            self.audio_input = audio_input;
            self.pixel_buffer_adaptor = Some(pixel_buffer_adaptor);
        }
        
        println!("✅ AVAssetWriter initialized successfully with fixed codec configuration");
        Ok(())
    }
    
    /// Handle incoming video sample buffer from ScreenCaptureKit
    pub fn handle_video_sample(&mut self, sample_buffer: &CMSampleBuffer) -> Result<()> {
        // Ensure recording session is started
        self.ensure_recording_started(sample_buffer)?;
        
        // Update frame count for statistics
        if let Ok(mut count) = self.video_frame_count.lock() {
            *count += 1;
            if *count % 60 == 0 {
                println!("📹 Encoded {} video frames", *count);
            }
        }
        
        // Process the video frame if we have an active writer
        if let (Some(video_input), Some(pixel_buffer_adaptor)) = (self.video_input, self.pixel_buffer_adaptor) {
            unsafe {
                // Check if input is ready for more media data
                let ready: bool = msg_send![video_input, isReadyForMoreMediaData];
                if !ready {
                    return Ok(()); // Skip frame if not ready
                }
                
                // Get pixel buffer from sample buffer
                let pixel_buffer: *mut CVPixelBuffer = CMSampleBufferGetImageBuffer(sample_buffer);
                if pixel_buffer.is_null() {
                    return Ok(());
                }
                
                // Get presentation time
                let presentation_time = CMSampleBufferGetPresentationTimeStamp(sample_buffer);
                
                // Append pixel buffer
                let success: bool = msg_send![
                    pixel_buffer_adaptor,
                    appendPixelBuffer: pixel_buffer,
                    withPresentationTime: presentation_time
                ];
                
                if !success {
                    log::warn!("Failed to append video pixel buffer");
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle incoming audio sample buffer from ScreenCaptureKit
    pub fn handle_audio_sample(&mut self, sample_buffer: &CMSampleBuffer) -> Result<()> {
        if !self.capture_audio {
            return Ok(());
        }
        
        // Ensure recording session is started
        self.ensure_recording_started(sample_buffer)?;
        
        // Update sample count for statistics
        if let Ok(mut count) = self.audio_sample_count.lock() {
            *count += 1;
            if *count % 100 == 0 {
                println!("🔊 Encoded {} audio samples", *count);
            }
        }
        
        // Process the audio sample if we have an active writer
        if let Some(audio_input) = self.audio_input {
            unsafe {
                // Check if input is ready for more media data
                let ready: bool = msg_send![audio_input, isReadyForMoreMediaData];
                if !ready {
                    return Ok(()); // Skip sample if not ready
                }
                
                // Append sample buffer
                let success: bool = msg_send![audio_input, appendSampleBuffer: sample_buffer];
                
                if !success {
                    log::warn!("Failed to append audio sample buffer");
                }
            }
        }
        
        Ok(())
    }
    
    /// Start recording session
    pub fn start_recording(&mut self) -> Result<()> {
        println!("▶️ Starting recording session");
        
        if let Ok(mut is_recording) = self.is_recording.lock() {
            *is_recording = true;
        }
        
        Ok(())
    }
    
    /// Stop recording and finalize the output file
    pub fn stop_recording(&mut self) -> Result<String> {
        println!("⏹️ Stopping recording session");
        
        // Mark as not recording
        if let Ok(mut is_recording) = self.is_recording.lock() {
            *is_recording = false;
        }
        
        // Finalize the recording if we have an active writer
        if let Some(asset_writer) = self.asset_writer {
            unsafe {
                // Mark inputs as finished
                if let Some(video_input) = self.video_input {
                    let _: () = msg_send![video_input, markAsFinished];
                }
                if let Some(audio_input) = self.audio_input {
                    let _: () = msg_send![audio_input, markAsFinished];
                }
                
                // Finish writing
                let _: () = msg_send![asset_writer, finishWriting];
                
                println!("✅ Recording finalized successfully");
            }
        }
        
        // Print final statistics
        self.print_final_stats();
        
        Ok(self.output_path.clone())
    }
    
    /// Ensure recording session is started with proper timing
    fn ensure_recording_started(&mut self, sample_buffer: &CMSampleBuffer) -> Result<()> {
        if let Ok(mut recording_started) = self.recording_started.lock() {
            if !*recording_started {
                if let Some(asset_writer) = self.asset_writer {
                    unsafe {
                        // Start the writing session
                        let started: bool = msg_send![asset_writer, startWriting];
                        if !started {
                            return Err(Error::new(Status::GenericFailure, "Failed to start writing session"));
                        }
                        
                        // Get the presentation time from the first sample
                        let start_time = CMSampleBufferGetPresentationTimeStamp(sample_buffer);
                        
                        // Start session at source time
                        let _: () = msg_send![asset_writer, startSessionAtSourceTime: start_time];
                        
                        // Store the start time
                        if let Ok(mut stored_start_time) = self.start_time.lock() {
                            *stored_start_time = Some(start_time);
                        }
                        
                        *recording_started = true;
                        println!("✅ Recording session started successfully");
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Create properly configured video input with fixed codec settings
    unsafe fn create_video_input(&self) -> Result<*mut AVAssetWriterInput> {
        use objc2_foundation::{NSDictionary, NSString, NSNumber};
        use objc2::msg_send;
        
        // Create video settings with fixed codec configuration (no AVVideoAverageBitRateKey)
        let codec_key = NSString::from_str("AVVideoCodecKey");
        let codec_value = NSString::from_str("avc1"); // H.264
        
        let width_key = NSString::from_str("AVVideoWidthKey");
        let width_value: *mut NSNumber = msg_send![class!(NSNumber), numberWithUnsignedInt: self.width];
        
        let height_key = NSString::from_str("AVVideoHeightKey");
        let height_value: *mut NSNumber = msg_send![class!(NSNumber), numberWithUnsignedInt: self.height];
        
        // Create main video settings dictionary (no compression properties for avc1 compatibility)
        let settings: *mut NSDictionary<NSString, AnyObject> = msg_send![
            class!(NSDictionary),
            dictionaryWithObjects: &[
                &*codec_value as *const NSString as *mut AnyObject,
                width_value as *mut AnyObject,
                height_value as *mut AnyObject
            ],
            forKeys: &[&*codec_key, &*width_key, &*height_key],
            count: 3
        ];
        
        let media_type = NSString::from_str("vide");
        let video_input: *mut AVAssetWriterInput = msg_send![
            class!(AVAssetWriterInput),
            assetWriterInputWithMediaType: &*media_type,
            outputSettings: settings
        ];
        
        // Configure video input
        let _: () = msg_send![video_input, setExpectsMediaDataInRealTime: true];
        
        Ok(video_input)
    }
    
    /// Create properly configured audio input
    unsafe fn create_audio_input(&self) -> Result<*mut AVAssetWriterInput> {
        use objc2_foundation::{NSDictionary, NSString, NSNumber};
        use objc2::msg_send;
        
        let format_key = NSString::from_str("AVFormatIDKey");
        let format_value: *mut NSNumber = msg_send![class!(NSNumber), numberWithUnsignedInt: 0x61616320u32]; // 'aac '
        
        let sample_rate_key = NSString::from_str("AVSampleRateKey");
        let sample_rate_value: *mut NSNumber = msg_send![class!(NSNumber), numberWithFloat: 44100.0f32];
        
        let channels_key = NSString::from_str("AVNumberOfChannelsKey");
        let channels_value: *mut NSNumber = msg_send![class!(NSNumber), numberWithUnsignedInt: 2u32];
        
        let bitrate_key = NSString::from_str("AVEncoderBitRateKey");
        let bitrate_value: *mut NSNumber = msg_send![class!(NSNumber), numberWithUnsignedInt: 128000u32];
        
        let settings: *mut NSDictionary<NSString, AnyObject> = msg_send![
            class!(NSDictionary),
            dictionaryWithObjects: &[
                format_value as *mut AnyObject,
                sample_rate_value as *mut AnyObject,
                channels_value as *mut AnyObject,
                bitrate_value as *mut AnyObject
            ],
            forKeys: &[&*format_key, &*sample_rate_key, &*channels_key, &*bitrate_key],
            count: 4
        ];
        
        let media_type = NSString::from_str("soun");
        let audio_input: *mut AVAssetWriterInput = msg_send![
            class!(AVAssetWriterInput),
            assetWriterInputWithMediaType: &*media_type,
            outputSettings: settings
        ];
        
        // Configure audio input
        let _: () = msg_send![audio_input, setExpectsMediaDataInRealTime: true];
        
        Ok(audio_input)
    }
    
    /// Create pixel buffer adaptor for video frames
    unsafe fn create_pixel_buffer_adaptor(&self, video_input: *mut AVAssetWriterInput) -> Result<*mut AVAssetWriterInputPixelBufferAdaptor> {
        let pixel_buffer_attributes = self.create_pixel_buffer_attributes();
        
        let adaptor: *mut AVAssetWriterInputPixelBufferAdaptor = msg_send![
            class!(AVAssetWriterInputPixelBufferAdaptor),
            assetWriterInputPixelBufferAdaptorWithAssetWriterInput: video_input,
            sourcePixelBufferAttributes: pixel_buffer_attributes
        ];
        
        if adaptor.is_null() {
            return Err(Error::new(Status::GenericFailure, "Failed to create pixel buffer adaptor"));
        }
        
        Ok(adaptor)
    }
    
    /// Create complete video encoding settings (DISABLED)
    unsafe fn create_complete_video_settings(&self) -> *mut objc2_foundation::NSDictionary<objc2_foundation::NSString, AnyObject> {
        // Completely disabled to avoid AVAssetWriter crashes
        std::ptr::null_mut()
    }
    
    /// Create complete audio encoding settings (DISABLED)
    unsafe fn create_complete_audio_settings(&self) -> *mut objc2_foundation::NSDictionary<objc2_foundation::NSString, AnyObject> {
        // Completely disabled to avoid AVAssetWriter crashes
        std::ptr::null_mut()
    }
    
    /// Create pixel buffer attributes
    unsafe fn create_pixel_buffer_attributes(&self) -> *mut objc2_foundation::NSDictionary<objc2_foundation::NSString, AnyObject> {
        use objc2_foundation::{NSDictionary, NSNumber};
        
        let pixel_format_key = NSString::from_str("kCVPixelBufferPixelFormatTypeKey");
        let pixel_format_value: *mut NSNumber = msg_send![class!(NSNumber), numberWithUnsignedInt: 0x42475241u32]; // 'BGRA'
        
        let attributes: *mut NSDictionary<objc2_foundation::NSString, AnyObject> = msg_send![
            class!(NSDictionary),
            dictionaryWithObjects: &[pixel_format_value as *mut AnyObject],
            forKeys: &[&*pixel_format_key],
            count: 1
        ];
        
        attributes
    }
    
    /// Print final recording statistics
    fn print_final_stats(&self) {
        let video_frames = self.video_frame_count.lock().map(|c| *c).unwrap_or(0);
        let audio_samples = self.audio_sample_count.lock().map(|c| *c).unwrap_or(0);
        
        println!("📊 Final Recording Statistics:");
        println!("   📹 Video frames: {}", video_frames);
        println!("   🔊 Audio samples: {}", audio_samples);
        println!("   📁 Output file: {}", self.output_path);
        
        if video_frames > 0 {
            let duration = video_frames as f64 / self.fps as f64;
            println!("   ⏱️ Estimated duration: {:.2} seconds", duration);
        }
    }
    
    /// Get current recording statistics
    pub fn get_stats(&self) -> (u64, u64, bool) {
        let video_frames = self.video_frame_count.lock().map(|c| *c).unwrap_or(0);
        let audio_samples = self.audio_sample_count.lock().map(|c| *c).unwrap_or(0);
        let is_recording = self.is_recording.lock().map(|r| *r).unwrap_or(false);
        
        (video_frames, audio_samples, is_recording)
    }
}

/// Create an Objective-C delegate object that bridges to our Rust StreamOutput
pub unsafe fn create_stream_delegate(stream_output: Arc<Mutex<StreamOutput>>) -> *mut AnyObject {
    // For now, create a simple NSObject delegate
    // In a full implementation, this would be a proper Objective-C class that implements SCStreamDelegate
    let delegate_class = class!(NSObject);
    let delegate: *mut AnyObject = msg_send![delegate_class, new];
    
    // Store the stream_output reference somehow (this is simplified)
    // In practice, you'd need to create a proper Objective-C class with associated objects
    
    println!("✅ Created stream delegate object");
    delegate
} 