use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use napi::{Result, Status, Error};
use tokio::time::{timeout, Duration};

/// Configuration for transcription services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionConfig {
    pub service: TranscriptionService,
    pub api_key: Option<String>,
    pub language: Option<String>,
    pub output_format: TranscriptionFormat,
    pub include_timestamps: bool,
    pub include_speaker_labels: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TranscriptionService {
    OpenAIWhisper,
    GoogleSpeechToText,
    AzureSpeechService,
    AWSTranscribe,
    Local, // For local Whisper models
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TranscriptionFormat {
    Text,
    SRT,
    VTT,
    JSON,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub confidence: Option<f32>,
    pub segments: Vec<TranscriptionSegment>,
    pub language: Option<String>,
    pub duration: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    pub start_time: f32,
    pub end_time: f32,
    pub text: String,
    pub confidence: Option<f32>,
    pub speaker: Option<String>,
}

/// Handles transcription of recorded audio/video files
pub struct TranscriptionManager {
    config: TranscriptionConfig,
}

impl TranscriptionManager {
    pub fn new(config: TranscriptionConfig) -> Self {
        Self { config }
    }
    
    /// Create a default transcription configuration
    pub fn default_config() -> TranscriptionConfig {
        TranscriptionConfig {
            service: TranscriptionService::Local,
            api_key: None,
            language: Some("en".to_string()),
            output_format: TranscriptionFormat::Text,
            include_timestamps: true,
            include_speaker_labels: false,
        }
    }
    
    /// Transcribe audio from a recorded file
    pub async fn transcribe_file(&self, file_path: &str) -> Result<TranscriptionResult> {
        println!("🎤 Starting transcription of: {}", file_path);
        
        // Validate input file
        if !Path::new(file_path).exists() {
            return Err(Error::new(
                Status::GenericFailure,
                format!("Input file does not exist: {}", file_path)
            ));
        }
        
        // Extract audio if needed (for video files)
        let audio_path = self.extract_audio_if_needed(file_path).await?;
        
        // Perform transcription based on service
        let result = match self.config.service {
            TranscriptionService::OpenAIWhisper => {
                self.transcribe_with_openai_whisper(&audio_path).await?
            }
            TranscriptionService::GoogleSpeechToText => {
                self.transcribe_with_google(&audio_path).await?
            }
            TranscriptionService::AzureSpeechService => {
                self.transcribe_with_azure(&audio_path).await?
            }
            TranscriptionService::AWSTranscribe => {
                self.transcribe_with_aws(&audio_path).await?
            }
            TranscriptionService::Local => {
                self.transcribe_with_local_whisper(&audio_path).await?
            }
        };
        
        // Save transcription result
        self.save_transcription_result(&result, file_path).await?;
        
        println!("✅ Transcription completed successfully");
        Ok(result)
    }
    
    /// Extract audio from video file if needed
    async fn extract_audio_if_needed(&self, file_path: &str) -> Result<String> {
        let path = Path::new(file_path);
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        // If it's already an audio file, return as-is
        if matches!(extension.as_str(), "mp3" | "wav" | "m4a" | "aac" | "flac") {
            return Ok(file_path.to_string());
        }
        
        // Extract audio using FFmpeg
        let audio_path = format!("{}_audio.wav", 
            path.file_stem().unwrap().to_str().unwrap()
        );
        
        println!("🎵 Extracting audio to: {}", audio_path);
        
        let output = tokio::process::Command::new("ffmpeg")
            .args(&[
                "-i", file_path,
                "-vn", // No video
                "-acodec", "pcm_s16le", // PCM 16-bit
                "-ar", "16000", // 16kHz sample rate (good for speech)
                "-ac", "1", // Mono
                "-y", // Overwrite output file
                &audio_path
            ])
            .output()
            .await;
        
        match output {
            Ok(output) => {
                if output.status.success() {
                    println!("✅ Audio extracted successfully");
                    Ok(audio_path)
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    Err(Error::new(
                        Status::GenericFailure,
                        format!("FFmpeg audio extraction failed: {}", error)
                    ))
                }
            }
            Err(e) => {
                Err(Error::new(
                    Status::GenericFailure,
                    format!("Failed to run FFmpeg: {}. Please ensure FFmpeg is installed.", e)
                ))
            }
        }
    }
    
    /// Transcribe using OpenAI Whisper API
    async fn transcribe_with_openai_whisper(&self, audio_path: &str) -> Result<TranscriptionResult> {
        println!("🤖 Transcribing with OpenAI Whisper API");
        
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| Error::new(Status::GenericFailure, "OpenAI API key required"))?;
        
        // Read audio file
        let audio_data = fs::read(audio_path)
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to read audio file: {}", e)))?;
        
        // Create multipart form data
        let client = reqwest::Client::new();
        let form = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::bytes(audio_data)
                .file_name("audio.wav")
                .mime_str("audio/wav").unwrap())
            .text("model", "whisper-1")
            .text("response_format", "verbose_json");
        
        let form = if let Some(ref language) = self.config.language {
            form.text("language", language.clone())
        } else {
            form
        };
        
        // Make API request with timeout
        let response = timeout(Duration::from_secs(300), // 5 minute timeout
            client.post("https://api.openai.com/v1/audio/transcriptions")
                .header("Authorization", format!("Bearer {}", api_key))
                .multipart(form)
                .send()
        ).await
        .map_err(|_| Error::new(Status::GenericFailure, "Transcription request timed out"))?
        .map_err(|e| Error::new(Status::GenericFailure, format!("API request failed: {}", e)))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::new(
                Status::GenericFailure,
                format!("OpenAI API error: {}", error_text)
            ));
        }
        
        // Parse response
        let whisper_response: serde_json::Value = response.json().await
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to parse response: {}", e)))?;
        
        self.parse_whisper_response(whisper_response)
    }
    
    /// Transcribe using local Whisper model
    async fn transcribe_with_local_whisper(&self, audio_path: &str) -> Result<TranscriptionResult> {
        println!("🏠 Transcribing with local Whisper model");
        
        // Use whisper command-line tool
        let mut cmd = tokio::process::Command::new("whisper");
        cmd.args(&[
            audio_path,
            "--output_format", "json",
            "--output_dir", "/tmp"
        ]);
        
        if let Some(ref language) = self.config.language {
            cmd.args(&["--language", language]);
        }
        
        let output = timeout(Duration::from_secs(600), cmd.output()).await // 10 minute timeout
            .map_err(|_| Error::new(Status::GenericFailure, "Local Whisper transcription timed out"))?
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to run Whisper: {}. Please ensure Whisper is installed (pip install openai-whisper).", e)))?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(Error::new(
                Status::GenericFailure,
                format!("Whisper transcription failed: {}", error)
            ));
        }
        
        // Read the output JSON file
        let audio_filename = Path::new(audio_path).file_stem().unwrap().to_str().unwrap();
        let json_path = format!("/tmp/{}.json", audio_filename);
        
        let json_content = fs::read_to_string(&json_path)
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to read Whisper output: {}", e)))?;
        
        let whisper_response: serde_json::Value = serde_json::from_str(&json_content)
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to parse Whisper JSON: {}", e)))?;
        
        // Clean up temporary file
        let _ = fs::remove_file(&json_path);
        
        self.parse_whisper_response(whisper_response)
    }
    
    /// Parse Whisper API/CLI response
    fn parse_whisper_response(&self, response: serde_json::Value) -> Result<TranscriptionResult> {
        let text = response["text"].as_str()
            .ok_or_else(|| Error::new(Status::GenericFailure, "No text in Whisper response"))?
            .to_string();
        
        let language = response["language"].as_str().map(|s| s.to_string());
        let duration = response["duration"].as_f64().map(|d| d as f32);
        
        let mut segments = Vec::new();
        
        if let Some(segments_array) = response["segments"].as_array() {
            for segment in segments_array {
                if let (Some(start), Some(end), Some(text)) = (
                    segment["start"].as_f64(),
                    segment["end"].as_f64(),
                    segment["text"].as_str()
                ) {
                    segments.push(TranscriptionSegment {
                        start_time: start as f32,
                        end_time: end as f32,
                        text: text.to_string(),
                        confidence: segment["confidence"].as_f64().map(|c| c as f32),
                        speaker: None, // Whisper doesn't provide speaker labels
                    });
                }
            }
        }
        
        Ok(TranscriptionResult {
            text,
            confidence: None, // Overall confidence not provided by Whisper
            segments,
            language,
            duration,
        })
    }
    
    /// Placeholder for Google Speech-to-Text
    async fn transcribe_with_google(&self, _audio_path: &str) -> Result<TranscriptionResult> {
        Err(Error::new(
            Status::GenericFailure,
            "Google Speech-to-Text integration not implemented yet"
        ))
    }
    
    /// Placeholder for Azure Speech Service
    async fn transcribe_with_azure(&self, _audio_path: &str) -> Result<TranscriptionResult> {
        Err(Error::new(
            Status::GenericFailure,
            "Azure Speech Service integration not implemented yet"
        ))
    }
    
    /// Placeholder for AWS Transcribe
    async fn transcribe_with_aws(&self, _audio_path: &str) -> Result<TranscriptionResult> {
        Err(Error::new(
            Status::GenericFailure,
            "AWS Transcribe integration not implemented yet"
        ))
    }
    
    /// Save transcription result to file
    async fn save_transcription_result(&self, result: &TranscriptionResult, original_file: &str) -> Result<()> {
        let base_path = Path::new(original_file).with_extension("");
        
        match self.config.output_format {
            TranscriptionFormat::Text => {
                let output_path = format!("{}.txt", base_path.to_str().unwrap());
                fs::write(&output_path, &result.text)
                    .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to write text file: {}", e)))?;
                println!("💾 Transcription saved as text: {}", output_path);
            }
            TranscriptionFormat::SRT => {
                let output_path = format!("{}.srt", base_path.to_str().unwrap());
                let srt_content = self.format_as_srt(result);
                fs::write(&output_path, srt_content)
                    .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to write SRT file: {}", e)))?;
                println!("💾 Transcription saved as SRT: {}", output_path);
            }
            TranscriptionFormat::VTT => {
                let output_path = format!("{}.vtt", base_path.to_str().unwrap());
                let vtt_content = self.format_as_vtt(result);
                fs::write(&output_path, vtt_content)
                    .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to write VTT file: {}", e)))?;
                println!("💾 Transcription saved as VTT: {}", output_path);
            }
            TranscriptionFormat::JSON => {
                let output_path = format!("{}.json", base_path.to_str().unwrap());
                let json_content = serde_json::to_string_pretty(result)
                    .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to serialize JSON: {}", e)))?;
                fs::write(&output_path, json_content)
                    .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to write JSON file: {}", e)))?;
                println!("💾 Transcription saved as JSON: {}", output_path);
            }
        }
        
        Ok(())
    }
    
    /// Format transcription as SRT subtitles
    fn format_as_srt(&self, result: &TranscriptionResult) -> String {
        let mut srt = String::new();
        
        for (index, segment) in result.segments.iter().enumerate() {
            srt.push_str(&format!("{}\n", index + 1));
            srt.push_str(&format!("{} --> {}\n", 
                self.format_time_srt(segment.start_time),
                self.format_time_srt(segment.end_time)
            ));
            srt.push_str(&format!("{}\n\n", segment.text.trim()));
        }
        
        srt
    }
    
    /// Format transcription as VTT subtitles
    fn format_as_vtt(&self, result: &TranscriptionResult) -> String {
        let mut vtt = String::from("WEBVTT\n\n");
        
        for segment in &result.segments {
            vtt.push_str(&format!("{} --> {}\n", 
                self.format_time_vtt(segment.start_time),
                self.format_time_vtt(segment.end_time)
            ));
            vtt.push_str(&format!("{}\n\n", segment.text.trim()));
        }
        
        vtt
    }
    
    /// Format time for SRT (HH:MM:SS,mmm)
    fn format_time_srt(&self, seconds: f32) -> String {
        let hours = (seconds / 3600.0) as u32;
        let minutes = ((seconds % 3600.0) / 60.0) as u32;
        let secs = (seconds % 60.0) as u32;
        let millis = ((seconds % 1.0) * 1000.0) as u32;
        
        format!("{:02}:{:02}:{:02},{:03}", hours, minutes, secs, millis)
    }
    
    /// Format time for VTT (HH:MM:SS.mmm)
    fn format_time_vtt(&self, seconds: f32) -> String {
        let hours = (seconds / 3600.0) as u32;
        let minutes = ((seconds % 3600.0) / 60.0) as u32;
        let secs = (seconds % 60.0) as u32;
        let millis = ((seconds % 1.0) * 1000.0) as u32;
        
        format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, secs, millis)
    }
    
    /// Get available transcription services
    pub fn get_available_services() -> Vec<TranscriptionService> {
        vec![
            TranscriptionService::Local,
            TranscriptionService::OpenAIWhisper,
            TranscriptionService::GoogleSpeechToText,
            TranscriptionService::AzureSpeechService,
            TranscriptionService::AWSTranscribe,
        ]
    }
    
    /// Check if required dependencies are installed
    pub async fn check_dependencies(&self) -> Result<Vec<String>> {
        let mut missing = Vec::new();
        
        // Check FFmpeg
        if tokio::process::Command::new("ffmpeg").arg("-version").output().await.is_err() {
            missing.push("FFmpeg (required for audio extraction)".to_string());
        }
        
        // Check service-specific dependencies
        match self.config.service {
            TranscriptionService::Local => {
                if tokio::process::Command::new("whisper").arg("--help").output().await.is_err() {
                    missing.push("Whisper CLI (install with: pip install openai-whisper)".to_string());
                }
            }
            TranscriptionService::OpenAIWhisper => {
                if self.config.api_key.is_none() {
                    missing.push("OpenAI API key".to_string());
                }
            }
            _ => {
                // Other services would have their own dependency checks
            }
        }
        
        Ok(missing)
    }
} 