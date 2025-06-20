const { ScreenCaptureKitRecorder, IntegratedRecordingManager, checkScreenRecordingPermission } = require('../index.js');

async function testObjCBridge() {
    console.log('🧪 Testing Objective-C Bridge Integration...\n');
    
    try {
        // Test 1: Check if the module loads correctly
        console.log('✅ Module loaded successfully');
        console.log('Available exports:', Object.keys(require('../index.js')));
        
        // Test 2: Check permissions first
        console.log('\n📋 Checking screen recording permission...');
        const hasPermission = checkScreenRecordingPermission();
        console.log(`Screen recording permission: ${hasPermission ? '✅ Granted' : '❌ Not granted'}`);
        
        if (!hasPermission) {
            console.log('⚠️  Please grant screen recording permission in System Preferences');
            console.log('   Go to: System Preferences > Security & Privacy > Privacy > Screen Recording');
            return;
        }
        
        // Test 3: Create IntegratedRecordingManager instance
        console.log('\n🎬 Creating IntegratedRecordingManager...');
        const recorder = new IntegratedRecordingManager();
        console.log('✅ IntegratedRecordingManager instance created');
        
        // Test 4: Start a brief recording to test the delegate bridge
        console.log('\n🎥 Testing delegate bridge with brief recording...');
        
        const outputPath = './test/recordings/objc-bridge-test.mp4';
        
        // Ensure recordings directory exists
        const fs = require('fs');
        const path = require('path');
        const recordingsDir = path.dirname(outputPath);
        if (!fs.existsSync(recordingsDir)) {
            fs.mkdirSync(recordingsDir, { recursive: true });
        }
        
        console.log('🔴 Starting recording...');
        console.log(`📁 Output path: ${outputPath}`);
        
        // Start recording (this should test the Objective-C delegate bridge)
        try {
            await recorder.startRecording({
                outputPath: outputPath,
                duration: 3, // 3 seconds
                width: 1280,
                height: 720,
                fps: 30
            });
            
            console.log('✅ Recording method called successfully');
            
            // Check if file was created
            if (fs.existsSync(outputPath)) {
                const stats = fs.statSync(outputPath);
                console.log(`✅ Output file created: ${stats.size} bytes`);
                
                if (stats.size > 0) {
                    console.log('✅ Objective-C delegate bridge appears to be working! Video data was written.');
                } else {
                    console.log('⚠️  Output file is empty - delegate bridge may not be receiving callbacks');
                }
            } else {
                console.log('❌ No output file created - possible issue with recording');
            }
            
        } catch (recordingError) {
            console.log('❌ Recording failed:', recordingError.message);
            console.log('This might indicate an issue with the delegate bridge or ScreenCaptureKit setup');
        }
        
    } catch (error) {
        console.error('❌ Test failed with error:', error);
        console.error('Stack trace:', error.stack);
    }
}

// Run the test
testObjCBridge().then(() => {
    console.log('\n🏁 Test completed');
}).catch(error => {
    console.error('\n💥 Test crashed:', error);
    process.exit(1);
}); 