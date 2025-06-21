#ifndef OBJC_BRIDGE_H
#define OBJC_BRIDGE_H

#import <Foundation/Foundation.h>
#import <ScreenCaptureKit/ScreenCaptureKit.h>
#import <CoreMedia/CoreMedia.h>

NS_ASSUME_NONNULL_BEGIN

// Forward declaration for Rust callback function pointers
typedef void (*RustVideoCallback)(void* _Nonnull context, CMSampleBufferRef _Nonnull sampleBuffer);
typedef void (*RustAudioCallback)(void* _Nonnull context, CMSampleBufferRef _Nonnull sampleBuffer);
typedef void (*RustStreamStoppedCallback)(void* _Nonnull context, NSError* _Nullable error);

// PRODUCTION-READY: Stream capture completion callback
typedef void (*RustStreamStartCallback)(void* context, NSError* _Nullable error);

// Objective-C delegate bridge that implements SCStreamDelegate
@interface SCStreamDelegateBridge : NSObject <SCStreamDelegate>

@property (nonatomic, assign) void* _Nonnull rustContext;
@property (nonatomic, assign) RustVideoCallback _Nonnull videoCallback;
@property (nonatomic, assign) RustAudioCallback _Nonnull audioCallback;
@property (nonatomic, assign) RustStreamStoppedCallback _Nonnull streamStoppedCallback;

- (instancetype _Nonnull)initWithContext:(void* _Nonnull)context
                  videoCallback:(RustVideoCallback _Nonnull)videoCallback
                  audioCallback:(RustAudioCallback _Nonnull)audioCallback
            streamStoppedCallback:(RustStreamStoppedCallback _Nonnull)streamStoppedCallback;

// PRODUCTION-READY: Helper method for proper stream capture with completion handler
+ (void)startStreamCapture:(SCStream*)stream 
            withCompletion:(RustStreamStartCallback)completion
                   context:(void*)context;

@end

// C interface for Rust to create and manage the delegate bridge
#ifdef __cplusplus
extern "C" {
#endif

// Create a new delegate bridge
void* _Nullable create_delegate_bridge(void* _Nonnull rust_context,
                           RustVideoCallback _Nonnull video_callback,
                           RustAudioCallback _Nonnull audio_callback,
                           RustStreamStoppedCallback _Nonnull stream_stopped_callback);

// Release the delegate bridge
void release_delegate_bridge(void* _Nullable bridge);

// PRODUCTION-READY: C interface for proper stream capture
void start_stream_capture_with_handler(void* stream, 
                                     RustStreamStartCallback callback,
                                     void* context);

#ifdef __cplusplus
}
#endif

NS_ASSUME_NONNULL_END

#endif // OBJC_BRIDGE_H 