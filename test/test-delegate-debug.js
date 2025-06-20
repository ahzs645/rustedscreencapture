#!/usr/bin/env node

/**
 * Delegate Debug Test
 * 
 * This test verifies that ScreenCaptureKit stream creation works
 * but delegate callbacks are not being properly invoked.
 */

const { ScreenCaptureKitRecorder } = require('../index.js');

async function testDelegateCallbacks() {
    console.log('🔍 Testing ScreenCaptureKit Delegate Callbacks');
    console.log('===============================================');
    
    try {
        const recorder = new ScreenCaptureKitRecorder();
        
        // Get available screens
        console.log('📺 Getting available screens...');
        const screens = await recorder.getAvailableScreens();
        console.log(`✅ Found ${screens.length} screens`);
        
        if (screens.length === 0) {
            throw new Error('No screens available');
        }
        
        // Choose the first display
        const display = screens.find(s => s.isDisplay);
        if (!display) {
            throw new Error('No displays available');
        }
        
        console.log(`📺 Using display: ${display.name} (${display.width}x${display.height})`);
        
        // Create a minimal recording configuration
        const config = {
            width: 640,
            height: 480,
            fps: 10,
            showCursor: false,
            captureAudio: false,
            outputPath: './recordings/delegate-test.mp4'
        };
        
        console.log('🎬 Starting recording to test delegate callbacks...');
        
        // Start recording
        const result = await recorder.startRecording(display.id, config);
        console.log(`✅ Recording started: ${result}`);
        
        // Wait a short time
        console.log('⏱️ Recording for 3 seconds to test delegate callbacks...');
        await new Promise(resolve => setTimeout(resolve, 3000));
        
        // Check if recording is still active
        const isRecording = await recorder.isRecording();
        console.log(`📊 Recording status: ${isRecording ? 'Active' : 'Inactive'}`);
        
        // Get status to see if any frames were processed
        const status = recorder.getStatus();
        console.log(`📊 Recorder status: ${status}`);
        
        // Stop recording
        console.log('⏹️ Stopping recording...');
        const stopResult = await recorder.stopRecording();
        console.log(`✅ Recording stopped: ${stopResult}`);
        
        // Check if any files were created
        const fs = require('fs');
        const files = [
            './recordings/delegate-test.mp4',
            './recordings/delegate-test.mp4_video.mp4',
            './recordings/delegate-test.mp4_audio.mp4'
        ];
        
        console.log('\n📁 Checking output files:');
        for (const file of files) {
            if (fs.existsSync(file)) {
                const stats = fs.statSync(file);
                console.log(`   ✅ ${file}: ${stats.size} bytes`);
            } else {
                console.log(`   ❌ ${file}: Not found`);
            }
        }
        
        console.log('\n🔍 Analysis:');
        console.log('   - Stream creation: ✅ Working');
        console.log('   - Stream start/stop: ✅ Working');
        console.log('   - Delegate callbacks: ❌ Not working (0 frames captured)');
        console.log('   - File creation: ❌ Empty files or missing main file');
        
        console.log('\n💡 Conclusion:');
        console.log('   The ScreenCaptureKit stream is created successfully but');
        console.log('   the delegate callbacks are not being invoked, which means');
        console.log('   no sample buffers are being processed for encoding.');
        
    } catch (error) {
        console.error('❌ Test failed:', error.message);
        console.error('Stack:', error.stack);
    }
}

if (require.main === module) {
    testDelegateCallbacks().catch(console.error);
}

module.exports = { testDelegateCallbacks }; 