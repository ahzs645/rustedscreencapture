const { 
    initScreencapturekit, 
    checkScreenRecordingPermission,
    ContentManager,
    ScreenCaptureKitRecorder
} = require('../index.js');

async function testRecordingFunctionality() {
    console.log('🎬 ScreenCaptureKit Recording Functionality Test');
    console.log('='.repeat(50));
    
    try {
        // Step 1: Initialize ScreenCaptureKit
        console.log('\n📋 Step 1: Initializing ScreenCaptureKit...');
        await initScreencapturekit();
        console.log('✅ ScreenCaptureKit initialized');

        // Step 2: Check permissions
        console.log('\n📋 Step 2: Checking permissions...');
        const hasPermission = await checkScreenRecordingPermission();
        console.log(`🔐 Screen recording permission: ${hasPermission}`);
        
        if (!hasPermission) {
            console.log('❌ Recording permission not granted. Please enable screen recording permission in System Preferences.');
            return { success: false, error: 'No screen recording permission' };
        }

        // Step 3: Create recording manager (using ScreenCaptureKitRecorder instead)
        console.log('\n📋 Step 3: Creating ScreenCaptureKit recorder...');
        const recorder = new ScreenCaptureKitRecorder();
        console.log('✅ ScreenCaptureKit recorder created');

        // Step 4: Get available screens
        console.log('\n📋 Step 4: Getting available screens...');
        const screens = await recorder.getAvailableScreens();
        console.log(`📺 Found ${screens.length} screens`);
        
        if (screens.length === 0) {
            console.log('❌ No screens found');
            return { success: false, error: 'No screens available' };
        }

        // Find a display (not a window)
        const displays = screens.filter(s => s.isDisplay);
        if (displays.length === 0) {
            console.log('❌ No displays found');
            return { success: false, error: 'No displays available' };
        }

        // Step 5: Attempt to start recording
        console.log('\n📋 Step 5: Attempting to start recording...');
        const outputPath = '/tmp/test-screen-recording.mp4';
        const primaryDisplay = displays[0];
        
        console.log(`🎯 Target display: ${primaryDisplay.name} (${primaryDisplay.width}x${primaryDisplay.height})`);
        console.log(`📁 Output path: ${outputPath}`);

        // Use the correct configuration format for ScreenCaptureKitRecorder
        const recordingConfig = {
            width: primaryDisplay.width,
            height: primaryDisplay.height,
            fps: 30,
            showCursor: true,
            captureAudio: false, // Start without audio to simplify
            outputPath: outputPath,
            pixelFormat: 'BGRA',
            colorSpace: 'sRGB'
        };

        console.log('🔧 Recording configuration:', JSON.stringify(recordingConfig, null, 2));

        try {
            // ScreenCaptureKitRecorder.startRecording takes (screen_id, config)
            const result = await recorder.startRecording(primaryDisplay.id, recordingConfig);
            console.log('✅ Recording started successfully');
            console.log('📊 Start result:', result);

            // Step 6: Record for a few seconds
            console.log('\n📋 Step 6: Recording for 3 seconds...');
            await new Promise(resolve => setTimeout(resolve, 3000));

            // Step 7: Stop recording
            console.log('\n📋 Step 7: Stopping recording...');
            const stopResult = await recorder.stopRecording();
            console.log('✅ Recording stopped');
            console.log('📊 Stop result:', stopResult);

            // Step 8: Check if file was created
            console.log('\n📋 Step 8: Checking output file...');
            const fs = require('fs');
            
            // Check both the expected path and the returned path
            const pathsToCheck = [outputPath, stopResult].filter(p => p && typeof p === 'string');
            
            for (const pathToCheck of pathsToCheck) {
                if (fs.existsSync(pathToCheck)) {
                    const stats = fs.statSync(pathToCheck);
                    console.log(`✅ Recording file created: ${pathToCheck}`);
                    console.log(`📊 File size: ${stats.size} bytes`);
                    console.log(`📅 Created: ${stats.birthtime}`);
                    
                    if (stats.size > 0) {
                        console.log('🎉 Recording appears to have content!');
                        return { 
                            success: true, 
                            outputPath: pathToCheck, 
                            fileSize: stats.size,
                            duration: '3 seconds'
                        };
                    } else {
                        console.log('⚠️ Recording file is empty');
                        return { 
                            success: false, 
                            error: 'Recording file is empty',
                            outputPath: pathToCheck,
                            fileSize: stats.size
                        };
                    }
                }
            }
            
            console.log('❌ Recording file was not created');
            console.log('🔍 Checked paths:', pathsToCheck);
            return { 
                success: false, 
                error: 'Recording file not created',
                expectedPaths: pathsToCheck
            };

        } catch (recordingError) {
            console.log('❌ Recording failed:', recordingError.message);
            console.log('🔍 Error details:', recordingError);
            
            // Try to diagnose the issue
            console.log('\n🔍 Diagnostic Information:');
            console.log('- Permission status:', hasPermission);
            console.log('- Available screens:', screens.length);
            console.log('- Available displays:', displays.length);
            console.log('- Target display:', primaryDisplay);
            console.log('- Recording config:', recordingConfig);
            
            return { 
                success: false, 
                error: recordingError.message,
                diagnostics: {
                    hasPermission,
                    screenCount: screens.length,
                    displayCount: displays.length,
                    targetDisplay: primaryDisplay,
                    config: recordingConfig
                }
            };
        }

    } catch (error) {
        console.log('❌ Test failed with error:', error.message);
        console.log('🔍 Full error:', error);
        return { success: false, error: error.message, fullError: error };
    }
}

// Additional diagnostic test
async function testRecordingDiagnostics() {
    console.log('\n🔍 Recording Diagnostics');
    console.log('='.repeat(30));
    
    try {
        // Test 1: Check ScreenCaptureKit availability
        console.log('\n🧪 Test 1: ScreenCaptureKit Framework');
        const os = require('os');
        console.log(`- macOS version: ${os.release()}`);
        console.log(`- Architecture: ${os.arch()}`);
        console.log(`- Platform: ${os.platform()}`);
        
        // Test 2: Check content enumeration in detail using ScreenCaptureKitRecorder
        console.log('\n🧪 Test 2: Content Enumeration via ScreenCaptureKitRecorder');
        const recorder = new ScreenCaptureKitRecorder();
        const screens = await recorder.getAvailableScreens();
        
        const displays = screens.filter(s => s.isDisplay);
        const windows = screens.filter(s => !s.isDisplay);
        
        console.log('📺 Displays:');
        displays.forEach((display, index) => {
            console.log(`  ${index + 1}. ${display.name} (ID: ${display.id}, ${display.width}x${display.height})`);
        });
        
        console.log('🪟 Windows (first 5):');
        windows.slice(0, 5).forEach((window, index) => {
            console.log(`  ${index + 1}. ${window.name} (ID: ${window.id}, ${window.width}x${window.height})`);
        });

        // Test 3: Check recorder status
        console.log('\n🧪 Test 3: Recorder Status');
        const isRecording = recorder.isRecording();
        const status = recorder.getStatus();
        console.log('🔧 Is Recording:', isRecording);
        console.log('📊 Status:', status);

        return {
            system: {
                macOS: os.release(),
                arch: os.arch(),
                platform: os.platform()
            },
            content: {
                displays: displays.length,
                windows: windows.length,
                totalScreens: screens.length
            },
            recorder: {
                isRecording,
                status
            }
        };

    } catch (error) {
        console.log('❌ Diagnostics failed:', error.message);
        return { error: error.message };
    }
}

// Run the tests
async function main() {
    console.log('🚀 Starting Comprehensive Recording Test\n');
    
    // Run diagnostics first
    const diagnostics = await testRecordingDiagnostics();
    
    // Then run the actual recording test
    const recordingResult = await testRecordingFunctionality();
    
    console.log('\n📋 Final Test Results');
    console.log('='.repeat(30));
    console.log('🔍 Diagnostics:', JSON.stringify(diagnostics, null, 2));
    console.log('🎬 Recording Test:', JSON.stringify(recordingResult, null, 2));
    
    if (recordingResult.success) {
        console.log('\n🎉 SUCCESS: Recording functionality is working!');
        console.log(`📁 Check your recording at: ${recordingResult.outputPath}`);
    } else {
        console.log('\n❌ FAILURE: Recording functionality has issues');
        console.log(`🔍 Issue: ${recordingResult.error}`);
        
        // Provide troubleshooting suggestions
        console.log('\n🛠️ Troubleshooting Suggestions:');
        if (recordingResult.error?.includes('permission')) {
            console.log('- Enable screen recording permission in System Preferences > Security & Privacy > Privacy > Screen Recording');
        }
        if (recordingResult.error?.includes('display')) {
            console.log('- Check if displays are properly detected');
        }
        if (recordingResult.error?.includes('file')) {
            console.log('- Check if output directory exists and is writable');
            console.log('- Try a different output path');
        }
    }
    
    process.exit(recordingResult.success ? 0 : 1);
}

if (require.main === module) {
    main().catch(console.error);
}

module.exports = { testRecordingFunctionality, testRecordingDiagnostics }; 