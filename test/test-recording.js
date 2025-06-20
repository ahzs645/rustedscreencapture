#!/usr/bin/env node

/**
 * ScreenCaptureKit Recording Test
 * 
 * This test demonstrates the complete recording workflow:
 * 1. Enumerate available screens
 * 2. Choose a screen to record
 * 3. Start recording
 * 4. Record for specified duration
 * 5. Stop recording and save video
 * 6. Verify the output file
 */

const { 
    ScreenCaptureKitRecorder, 
    IntegratedRecordingManager,
    checkScreenRecordingPermission 
} = require('../index.js');
const fs = require('fs');
const path = require('path');

// Configuration
const RECORDING_DURATION = 10; // seconds
const OUTPUT_DIR = './recordings';
const DEFAULT_CONFIG = {
    width: 1920,
    height: 1080,
    fps: 30,
    showCursor: true,
    captureAudio: false, // Start with video-only for simplicity
    pixelFormat: 'BGRA',
    colorSpace: 'sRGB'
};

async function ensureOutputDirectory() {
    if (!fs.existsSync(OUTPUT_DIR)) {
        fs.mkdirSync(OUTPUT_DIR, { recursive: true });
        console.log(`📁 Created output directory: ${OUTPUT_DIR}`);
    }
}

async function checkPermissions() {
    console.log('🔐 Checking Permissions');
    console.log('======================');
    
    const hasPermission = checkScreenRecordingPermission();
    console.log(`Screen recording permission: ${hasPermission ? '✅ Granted' : '❌ Denied'}`);
    
    if (!hasPermission) {
        console.log('\n❌ Screen recording permission is required!');
        console.log('📋 To grant permission:');
        console.log('   1. Open System Preferences');
        console.log('   2. Go to Security & Privacy');
        console.log('   3. Click Privacy tab');
        console.log('   4. Select "Screen Recording" from the left');
        console.log('   5. Check the box next to your application');
        console.log('   6. Restart this application');
        return false;
    }
    
    return true;
}

async function getAvailableScreens(recorder) {
    console.log('\n📺 Discovering Available Screens');
    console.log('=================================');
    
    try {
        const screens = await recorder.getAvailableScreens();
        
        if (screens.length === 0) {
            throw new Error('No screens found');
        }
        
        // Separate displays from windows
        const displays = screens.filter(s => s.isDisplay);
        const windows = screens.filter(s => !s.isDisplay);
        
        console.log(`Found ${screens.length} total sources:`);
        console.log(`  📺 ${displays.length} displays`);
        console.log(`  🪟 ${windows.length} windows`);
        
        console.log('\n📺 Available Displays:');
        displays.forEach((display, index) => {
            console.log(`   ${index + 1}. ${display.name}`);
            console.log(`      Resolution: ${display.width}x${display.height}`);
            console.log(`      ID: ${display.id}`);
        });
        
        if (windows.length > 0) {
            console.log('\n🪟 Sample Windows:');
            windows.slice(0, 3).forEach((window, index) => {
                console.log(`   ${index + 1}. ${window.name}`);
                console.log(`      Size: ${window.width}x${window.height}`);
                console.log(`      ID: ${window.id}`);
            });
            if (windows.length > 3) {
                console.log(`   ... and ${windows.length - 3} more windows`);
            }
        }
        
        return { screens, displays, windows };
        
    } catch (error) {
        console.log('❌ Failed to get screens:', error.message);
        throw error;
    }
}

function chooseScreen(displays, windows) {
    console.log('\n🎯 Choosing Screen to Record');
    console.log('============================');
    
    // Priority: Choose the largest display, or main display if available
    let chosenScreen = null;
    
    if (displays.length > 0) {
        // Find main display (usually the first one or largest)
        chosenScreen = displays.reduce((prev, current) => {
            const prevArea = prev.width * prev.height;
            const currentArea = current.width * current.height;
            return currentArea > prevArea ? current : prev;
        });
        
        console.log(`📺 Chose display: ${chosenScreen.name}`);
        console.log(`   Resolution: ${chosenScreen.width}x${chosenScreen.height}`);
        console.log(`   Type: Primary Display`);
    } else if (windows.length > 0) {
        // Fallback to largest window
        chosenScreen = windows.reduce((prev, current) => {
            const prevArea = prev.width * prev.height;
            const currentArea = current.width * current.height;
            return currentArea > prevArea ? current : prev;
        });
        
        console.log(`🪟 Chose window: ${chosenScreen.name}`);
        console.log(`   Size: ${chosenScreen.width}x${chosenScreen.height}`);
        console.log(`   Type: Application Window`);
    }
    
    if (!chosenScreen) {
        throw new Error('No suitable screen found for recording');
    }
    
    return chosenScreen;
}

function createRecordingConfig(screen, outputPath) {
    console.log('\n⚙️ Creating Recording Configuration');
    console.log('===================================');
    
    // Use screen's native resolution, but cap it for reasonable file sizes
    const maxWidth = 1920;
    const maxHeight = 1080;
    
    let width = Math.min(screen.width, maxWidth);
    let height = Math.min(screen.height, maxHeight);
    
    // Maintain aspect ratio
    const aspectRatio = screen.width / screen.height;
    if (width / height !== aspectRatio) {
        if (width / aspectRatio <= maxHeight) {
            height = Math.round(width / aspectRatio);
        } else {
            width = Math.round(height * aspectRatio);
        }
    }
    
    const config = {
        ...DEFAULT_CONFIG,
        width,
        height,
        outputPath
    };
    
    console.log(`📐 Recording resolution: ${config.width}x${config.height}`);
    console.log(`🎬 Frame rate: ${config.fps} FPS`);
    console.log(`👆 Show cursor: ${config.showCursor}`);
    console.log(`🔊 Capture audio: ${config.captureAudio}`);
    console.log(`📁 Output path: ${config.outputPath}`);
    
    return config;
}

async function performRecording(recorder, screen, config) {
    console.log('\n🎬 Starting Recording');
    console.log('=====================');
    
    try {
        // Start recording
        console.log('▶️ Starting recording...');
        const startResult = await recorder.startRecording(screen.id, config);
        console.log(`✅ Recording started: ${startResult}`);
        
        // Monitor recording progress
        console.log(`⏱️ Recording for ${RECORDING_DURATION} seconds...`);
        console.log('💡 Move windows, open applications, or move the cursor to create visible content');
        
        for (let i = 1; i <= RECORDING_DURATION; i++) {
            await new Promise(resolve => setTimeout(resolve, 1000));
            
            // Check if still recording
            const isRecording = await recorder.isRecording();
            const status = recorder.getStatus();
            
            console.log(`   ${i}/${RECORDING_DURATION}s - Recording: ${isRecording ? '✅' : '❌'}`);
            
            if (!isRecording) {
                console.log('⚠️ Recording stopped unexpectedly');
                break;
            }
            
            // Show some status every 5 seconds
            if (i % 5 === 0) {
                try {
                    const statusObj = JSON.parse(status);
                    console.log(`   📊 Status: ${statusObj.method || 'unknown'}`);
                } catch (e) {
                    // Status might not be JSON
                }
            }
        }
        
        // Stop recording
        console.log('⏹️ Stopping recording...');
        const stopResult = await recorder.stopRecording();
        console.log(`✅ Recording stopped: ${stopResult}`);
        
        return stopResult;
        
    } catch (error) {
        console.log('❌ Recording failed:', error.message);
        
        // Try to stop recording if it's still active
        try {
            const isRecording = await recorder.isRecording();
            if (isRecording) {
                console.log('🛑 Attempting to stop failed recording...');
                await recorder.stopRecording();
            }
        } catch (stopError) {
            console.log('⚠️ Could not stop recording:', stopError.message);
        }
        
        throw error;
    }
}

async function verifyOutput(outputPath) {
    console.log('\n🔍 Verifying Output');
    console.log('===================');
    
    try {
        if (!fs.existsSync(outputPath)) {
            throw new Error(`Output file not found: ${outputPath}`);
        }
        
        const stats = fs.statSync(outputPath);
        const fileSizeMB = (stats.size / 1024 / 1024).toFixed(2);
        
        console.log(`✅ Output file exists: ${outputPath}`);
        console.log(`📊 File size: ${fileSizeMB} MB`);
        console.log(`📅 Created: ${stats.birthtime.toLocaleString()}`);
        console.log(`📅 Modified: ${stats.mtime.toLocaleString()}`);
        
        if (stats.size === 0) {
            throw new Error('Output file is empty');
        }
        
        if (stats.size < 1024) {
            console.log('⚠️ Warning: File is very small, may be corrupt or empty');
        } else if (stats.size < 100 * 1024) {
            console.log('⚠️ Warning: File is smaller than expected');
        } else {
            console.log('✅ File size looks good for video content');
        }
        
        return {
            exists: true,
            size: stats.size,
            sizeMB: fileSizeMB,
            path: outputPath
        };
        
    } catch (error) {
        console.log('❌ Output verification failed:', error.message);
        return {
            exists: false,
            error: error.message,
            path: outputPath
        };
    }
}

async function testWithScreenCaptureKitRecorder() {
    console.log('🎯 Testing with ScreenCaptureKitRecorder');
    console.log('=========================================');
    
    const recorder = new ScreenCaptureKitRecorder();
    
    // Get available screens
    const { displays, windows } = await getAvailableScreens(recorder);
    
    // Choose a screen
    const chosenScreen = chooseScreen(displays, windows);
    
    // Create output path
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    const outputPath = path.join(OUTPUT_DIR, `screen-recording-${timestamp}.mp4`);
    
    // Create recording configuration
    const config = createRecordingConfig(chosenScreen, outputPath);
    
    // Perform the recording
    const finalOutputPath = await performRecording(recorder, chosenScreen, config);
    
    // Verify the output
    const verification = await verifyOutput(finalOutputPath || outputPath);
    
    return {
        method: 'ScreenCaptureKitRecorder',
        screen: chosenScreen,
        config,
        outputPath: finalOutputPath || outputPath,
        verification
    };
}

async function testWithIntegratedRecordingManager() {
    console.log('\n🎯 Testing with IntegratedRecordingManager');
    console.log('==========================================');
    
    const manager = new IntegratedRecordingManager();
    
    // Initialize
    console.log('🔧 Initializing manager...');
    await manager.initialize();
    console.log('✅ Manager initialized');
    
    // Get available screens
    const displays = await manager.getAvailableScreens();
    const windows = await manager.getAvailableWindows();
    
    console.log(`📺 Manager found ${displays.length} displays and ${windows.length} windows`);
    
    if (displays.length === 0) {
        throw new Error('No displays found by IntegratedRecordingManager');
    }
    
    // Choose the first display
    const chosenScreen = displays[0];
    console.log(`📺 Chose display: ${chosenScreen.name} (${chosenScreen.width}x${chosenScreen.height})`);
    
    // Create output path
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    const outputPath = path.join(OUTPUT_DIR, `integrated-recording-${timestamp}.mp4`);
    
    // Create configuration (IntegratedRecordingManager uses different API)
    const config = {
        outputPath,
        width: Math.min(chosenScreen.width, 1920),
        height: Math.min(chosenScreen.height, 1080),
        fps: 30,
        showCursor: true,
        captureAudio: false
    };
    
    console.log(`📐 Recording configuration: ${config.width}x${config.height} @ ${config.fps}fps`);
    
    // Start recording
    console.log('▶️ Starting recording...');
    await manager.startRecording(config);
    console.log('✅ Recording started');
    
    // Record for specified duration
    console.log(`⏱️ Recording for ${RECORDING_DURATION} seconds...`);
    for (let i = 1; i <= RECORDING_DURATION; i++) {
        await new Promise(resolve => setTimeout(resolve, 1000));
        const isRecording = manager.isRecording();
        console.log(`   ${i}/${RECORDING_DURATION}s - Recording: ${isRecording ? '✅' : '❌'}`);
        
        if (!isRecording) {
            console.log('⚠️ Recording stopped unexpectedly');
            break;
        }
    }
    
    // Stop recording
    console.log('⏹️ Stopping recording...');
    const finalOutputPath = await manager.stopRecording();
    console.log(`✅ Recording stopped: ${finalOutputPath}`);
    
    // Verify output
    const verification = await verifyOutput(finalOutputPath);
    
    return {
        method: 'IntegratedRecordingManager',
        screen: chosenScreen,
        config,
        outputPath: finalOutputPath,
        verification
    };
}

async function main() {
    console.log('🚀 ScreenCaptureKit Recording Test');
    console.log('===================================');
    console.log(`Recording duration: ${RECORDING_DURATION} seconds`);
    console.log(`Output directory: ${OUTPUT_DIR}`);
    console.log('');
    
    try {
        // Ensure output directory exists
        await ensureOutputDirectory();
        
        // Check permissions
        const hasPermission = await checkPermissions();
        if (!hasPermission) {
            process.exit(1);
        }
        
        // Test with ScreenCaptureKitRecorder
        console.log('\n' + '='.repeat(60));
        let recorderResult;
        try {
            recorderResult = await testWithScreenCaptureKitRecorder();
            console.log('✅ ScreenCaptureKitRecorder test completed successfully');
        } catch (error) {
            console.log('❌ ScreenCaptureKitRecorder test failed:', error.message);
            recorderResult = { method: 'ScreenCaptureKitRecorder', error: error.message };
        }
        
        // Test with IntegratedRecordingManager
        console.log('\n' + '='.repeat(60));
        let managerResult;
        try {
            managerResult = await testWithIntegratedRecordingManager();
            console.log('✅ IntegratedRecordingManager test completed successfully');
        } catch (error) {
            console.log('❌ IntegratedRecordingManager test failed:', error.message);
            managerResult = { method: 'IntegratedRecordingManager', error: error.message };
        }
        
        // Final summary
        console.log('\n' + '='.repeat(60));
        console.log('📋 Final Test Results');
        console.log('=====================');
        
        console.log('\n🎬 ScreenCaptureKitRecorder:');
        if (recorderResult.verification?.exists) {
            console.log(`   ✅ SUCCESS: ${recorderResult.verification.sizeMB} MB recorded`);
            console.log(`   📁 File: ${recorderResult.outputPath}`);
        } else {
            console.log(`   ❌ FAILED: ${recorderResult.error || recorderResult.verification?.error}`);
        }
        
        console.log('\n🎬 IntegratedRecordingManager:');
        if (managerResult.verification?.exists) {
            console.log(`   ✅ SUCCESS: ${managerResult.verification.sizeMB} MB recorded`);
            console.log(`   📁 File: ${managerResult.outputPath}`);
        } else {
            console.log(`   ❌ FAILED: ${managerResult.error || managerResult.verification?.error}`);
        }
        
        // Open the output directory
        console.log(`\n📁 Check your recordings in: ${path.resolve(OUTPUT_DIR)}`);
        
        // Success if at least one method worked
        const anySuccess = (recorderResult.verification?.exists) || (managerResult.verification?.exists);
        
        if (anySuccess) {
            console.log('\n🎉 SUCCESS: At least one recording method is working!');
            console.log('🎯 Next steps:');
            console.log('   - Open the video files to verify screen content was captured');
            console.log('   - Try enabling audio recording');
            console.log('   - Test with different screen sources');
            process.exit(0);
        } else {
            console.log('\n❌ FAILURE: Both recording methods failed');
            console.log('🔧 Troubleshooting:');
            console.log('   - Verify screen recording permissions');
            console.log('   - Check if ScreenCaptureKit is available on your system');
            console.log('   - Try running on macOS 12.3+ with ScreenCaptureKit support');
            process.exit(1);
        }
        
    } catch (error) {
        console.error('\n💥 Test failed with error:', error.message);
        console.error('Stack trace:', error.stack);
        process.exit(1);
    }
}

// Handle command line arguments
const args = process.argv.slice(2);
if (args.includes('--help') || args.includes('-h')) {
    console.log('ScreenCaptureKit Recording Test');
    console.log('===============================');
    console.log('');
    console.log('This test will:');
    console.log('1. Check screen recording permissions');
    console.log('2. Discover available screens and windows');
    console.log('3. Choose the best screen to record');
    console.log('4. Record screen content for 10 seconds');
    console.log('5. Save the video to ./recordings/');
    console.log('6. Verify the output file');
    console.log('');
    console.log('Requirements:');
    console.log('- macOS 12.3+ with ScreenCaptureKit');
    console.log('- Screen recording permission granted');
    console.log('- Sufficient disk space for video files');
    console.log('');
    console.log('Usage: node test-recording.js');
    process.exit(0);
}

// Run the test
if (require.main === module) {
    main().catch(error => {
        console.error('💥 Unhandled error:', error);
        process.exit(1);
    });
}

module.exports = { 
    testWithScreenCaptureKitRecorder, 
    testWithIntegratedRecordingManager,
    main
}; 




