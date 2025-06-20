#import "objc_bridge.h"
#import <os/log.h>

@implementation SCStreamDelegateBridge

- (instancetype)initWithContext:(void*)context
                  videoCallback:(RustVideoCallback)videoCallback
                  audioCallback:(RustAudioCallback)audioCallback
            streamStoppedCallback:(RustStreamStoppedCallback)streamStoppedCallback {
    self = [super init];
    if (self) {
        self.rustContext = context;
        self.videoCallback = videoCallback;
        self.audioCallback = audioCallback;
        self.streamStoppedCallback = streamStoppedCallback;
        
        os_log(OS_LOG_DEFAULT, "🔧 SCStreamDelegateBridge initialized with context: %p", context);
    }
    return self;
}

#pragma mark - SCStreamDelegate Protocol Implementation

- (void)stream:(SCStream *)stream didOutputSampleBuffer:(CMSampleBufferRef)sampleBuffer ofType:(SCStreamOutputType)type {
    // Determine the type of sample buffer and call appropriate Rust callback
    switch (type) {
        case SCStreamOutputTypeScreen:
            if (self.videoCallback && self.rustContext) {
                os_log_debug(OS_LOG_DEFAULT, "📹 Forwarding video sample buffer to Rust");
                self.videoCallback(self.rustContext, sampleBuffer);
            } else {
                os_log_error(OS_LOG_DEFAULT, "❌ Video callback or context is NULL");
            }
            break;
            
        case SCStreamOutputTypeAudio:
        case SCStreamOutputTypeMicrophone:
            if (self.audioCallback && self.rustContext) {
                os_log_debug(OS_LOG_DEFAULT, "🔊 Forwarding audio sample buffer to Rust");
                self.audioCallback(self.rustContext, sampleBuffer);
            } else {
                os_log_error(OS_LOG_DEFAULT, "❌ Audio callback or context is NULL");
            }
            break;
            
        default:
            os_log_error(OS_LOG_DEFAULT, "❌ Unknown sample buffer type: %ld", (long)type);
            break;
    }
}

- (void)stream:(SCStream *)stream didStopWithError:(NSError *)error {
    os_log(OS_LOG_DEFAULT, "🛑 Stream stopped with error: %@", error);
    
    if (self.streamStoppedCallback && self.rustContext) {
        self.streamStoppedCallback(self.rustContext, error);
    } else {
        os_log_error(OS_LOG_DEFAULT, "❌ Stream stopped callback or context is NULL");
    }
}

- (void)dealloc {
    os_log(OS_LOG_DEFAULT, "🗑️ SCStreamDelegateBridge deallocated");
}

@end

#pragma mark - C Interface for Rust

void* create_delegate_bridge(void* rust_context,
                           RustVideoCallback video_callback,
                           RustAudioCallback audio_callback,
                           RustStreamStoppedCallback stream_stopped_callback) {
    
    os_log(OS_LOG_DEFAULT, "🔧 Creating delegate bridge with context: %p", rust_context);
    
    SCStreamDelegateBridge* bridge = [[SCStreamDelegateBridge alloc] 
        initWithContext:rust_context
          videoCallback:video_callback
          audioCallback:audio_callback
    streamStoppedCallback:stream_stopped_callback];
    
    if (bridge) {
        os_log(OS_LOG_DEFAULT, "✅ Delegate bridge created successfully: %p", (__bridge void*)bridge);
        return (__bridge_retained void*)bridge;
    } else {
        os_log_error(OS_LOG_DEFAULT, "❌ Failed to create delegate bridge");
        return NULL;
    }
}

void release_delegate_bridge(void* bridge) {
    if (bridge) {
        os_log(OS_LOG_DEFAULT, "🗑️ Releasing delegate bridge: %p", bridge);
        SCStreamDelegateBridge* objcBridge = (__bridge_transfer SCStreamDelegateBridge*)bridge;
        (void)objcBridge; // Suppress unused variable warning - ARC will handle deallocation
    } else {
        os_log_error(OS_LOG_DEFAULT, "❌ Attempted to release NULL delegate bridge");
    }
} 