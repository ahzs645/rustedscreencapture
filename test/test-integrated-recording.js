const { IntegratedRecordingManager } = require('../index.js');
const path = require('path');
const fs = require('fs');

async function testIntegratedRecording() {
    console.log('🎬 Testing Integrated Recording Manager');
    console.log('=====================================');
    
    let recordingManager;
    
    try {
        // Step 1: Create and initialize the recording manager
        console.log('\n📋 Step 1: Creating Recording Manager');
        recordingManager = new IntegratedRecordingManager();
        
        console.log('🔧 Initializing...');
        recordingManager.initialize();
        console.log('✅ Recording manager initialized successfully');
        
        // Step 2: Check system requirements and permissions
        console.log('\n🔐 Step 2: Checking Permissions');
        const permissionStatus = recordingManager.getPermissionStatus();
        console.log('Permission Status:', permissionStatus);
        
        // Step 3: Get available screens and windows
        console.log('\n📺 Step 3: Getting Available Sources');
        const screens = recordingManager.getAvailableScreens();
        console.log(`Found ${screens.length} screens:`);
        screens.forEach((screen, index) => {
            console.log(`  ${index + 1}. ${screen.name} (${screen.width}x${screen.height})`);
        });
        
        const windows = recordingManager.getAvailableWindows();
        console.log(`Found ${windows.length} windows:`);
        windows.slice(0, 5).forEach((window, index) => {
            console.log(`  ${index + 1}. ${window.name} (${window.width}x${window.height})`);
        });
        
        // Step 4: Configure recording
        console.log('\n⚙️ Step 4: Configuring Recording');
        const outputDir = path.join(__dirname, 'recordings');
        if (!fs.existsSync(outputDir)) {
            fs.mkdirSync(outputDir, { recursive: true });
        }
        
        const outputPath = path.join(outputDir, `test-recording-${Date.now()}.mp4`);
        console.log(`Output path: ${outputPath}`);
        
        const recordingConfig = {
            outputPath: outputPath,
            width: 1920,
            height: 1080,
            fps: 30,
            showCursor: true,
            captureAudio: true,
            pixelFormat: 'BGRA',
            colorSpace: 'sRGB'
        };
        
        // Step 5: Start recording
        console.log('\n▶️ Step 5: Starting Recording');
        recordingManager.startRecording(recordingConfig);
        console.log('✅ Recording started successfully!');
        
        // Step 6: Monitor recording for a few seconds
        console.log('\n📊 Step 6: Monitoring Recording');
        for (let i = 0; i < 10; i++) {
            await new Promise(resolve => setTimeout(resolve, 1000));
            
            const stats = JSON.parse(recordingManager.getRecordingStats());
            console.log(`Recording... ${i + 1}s - Frames: ${stats.videoFrames}, Audio: ${stats.audioSamples}`);
            
            if (!stats.isRecording) {
                console.log('⚠️ Recording stopped unexpectedly');
                break;
            }
        }
        
        // Step 7: Stop recording
        console.log('\n⏹️ Step 7: Stopping Recording');
        const finalOutputPath = recordingManager.stopRecording();
        console.log(`✅ Recording stopped. Final output: ${finalOutputPath}`);
        
        // Step 8: Verify output file
        console.log('\n🔍 Step 8: Verifying Output');
        if (fs.existsSync(finalOutputPath)) {
            const stats = fs.statSync(finalOutputPath);
            console.log(`✅ Output file exists: ${finalOutputPath}`);
            console.log(`📁 File size: ${(stats.size / 1024 / 1024).toFixed(2)} MB`);
            console.log(`📅 Created: ${stats.birthtime}`);
        } else {
            console.log('❌ Output file not found');
        }
        
        // Step 9: Test transcription (if available)
        console.log('\n🎤 Step 9: Testing Transcription (Optional)');
        console.log('💡 Transcription would be configured and run here');
        console.log('   - Configure transcription service (OpenAI Whisper, local, etc.)');
        console.log('   - Extract audio from video');
        console.log('   - Generate transcription in various formats (SRT, VTT, JSON)');
        
        console.log('\n🎉 Integrated Recording Test Completed Successfully!');
        console.log('=====================================');
        
        return {
            success: true,
            outputPath: finalOutputPath,
            screens: screens.length,
            windows: windows.length,
            recordingDuration: 10
        };
        
    } catch (error) {
        console.error('❌ Test failed:', error.message);
        console.error('Stack trace:', error.stack);
        
        // Try to stop recording if it's still active
        if (recordingManager && recordingManager.isRecording()) {
            try {
                recordingManager.stopRecording();
                console.log('🧹 Cleaned up active recording');
            } catch (cleanupError) {
                console.error('⚠️ Failed to cleanup recording:', cleanupError.message);
            }
        }
        
        return {
            success: false,
            error: error.message
        };
    }
}

async function testPermissionsOnly() {
    console.log('🔐 Testing Permissions Only');
    console.log('============================');
    
    try {
        const recordingManager = new IntegratedRecordingManager();
        const status = recordingManager.getPermissionStatus();
        console.log('Permission Status Report:');
        console.log(status);
        
        return { success: true, status };
    } catch (error) {
        console.error('❌ Permission test failed:', error.message);
        return { success: false, error: error.message };
    }
}

async function testBasicFunctionality() {
    console.log('🧪 Testing Basic Functionality');
    console.log('===============================');
    
    try {
        const recordingManager = new IntegratedRecordingManager();
        
        // Test initialization
        console.log('Testing initialization...');
        recordingManager.initialize();
        console.log('✅ Initialization successful');
        
        // Test getting sources
        console.log('Testing source enumeration...');
        const screens = recordingManager.getAvailableScreens();
        const windows = recordingManager.getAvailableWindows();
        console.log(`✅ Found ${screens.length} screens and ${windows.length} windows`);
        
        // Test recording state
        console.log('Testing recording state...');
        const isRecording = recordingManager.isRecording();
        console.log(`✅ Recording state: ${isRecording}`);
        
        // Test stats
        console.log('Testing stats...');
        const stats = recordingManager.getRecordingStats();
        const parsedStats = JSON.parse(stats);
        console.log(`✅ Stats retrieved: ${parsedStats.isRecording ? 'Recording' : 'Not recording'}`);
        
        return { success: true };
    } catch (error) {
        console.error('❌ Basic functionality test failed:', error.message);
        return { success: false, error: error.message };
    }
}

// Main test runner
async function main() {
    console.log('🚀 Starting Integrated Recording Manager Tests');
    console.log('===============================================');
    
    const args = process.argv.slice(2);
    const testType = args[0] || 'basic';
    
    let result;
    
    switch (testType) {
        case 'full':
            result = await testIntegratedRecording();
            break;
        case 'permissions':
            result = await testPermissionsOnly();
            break;
        case 'basic':
        default:
            result = await testBasicFunctionality();
            break;
    }
    
    console.log('\n📋 Test Results:');
    console.log(JSON.stringify(result, null, 2));
    
    process.exit(result.success ? 0 : 1);
}

// Usage information
if (require.main === module) {
    console.log('Usage: node test-integrated-recording.js [test-type]');
    console.log('Test types:');
    console.log('  basic       - Test basic functionality (default)');
    console.log('  permissions - Test permissions only');
    console.log('  full        - Full recording test (requires permissions)');
    console.log('');
    
    main().catch(error => {
        console.error('❌ Unhandled error:', error);
        process.exit(1);
    });
}

module.exports = {
    testIntegratedRecording,
    testPermissionsOnly,
    testBasicFunctionality
}; 