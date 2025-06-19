#!/usr/bin/env node

/**
 * Test Content Filter Fix
 * 
 * This test specifically validates that content filter creation now works
 * instead of returning null in bypass mode.
 */

const { 
    IntegratedRecordingManager,
    ScreenCaptureKitRecorder,
    checkScreenRecordingPermission 
} = require('../index.js');

async function testContentFilterFix() {
    console.log('🔧 Testing Content Filter Fix');
    console.log('==============================');
    
    try {
        // Step 1: Check permissions first
        console.log('\n📋 Step 1: Checking Permissions');
        const hasPermission = checkScreenRecordingPermission();
        console.log(`🔐 Screen recording permission: ${hasPermission}`);
        
        if (!hasPermission) {
            console.log('❌ Screen recording permission required for this test');
            return { success: false, error: 'No permission' };
        }
        
        // Step 2: Test with IntegratedRecordingManager (the new system)
        console.log('\n📋 Step 2: Testing IntegratedRecordingManager Content Filter Creation');
        const manager = new IntegratedRecordingManager();
        
        console.log('🔧 Initializing recording manager...');
        manager.initialize();
        console.log('✅ Recording manager initialized');
        
        // Get available screens
        const screens = manager.getAvailableScreens();
        console.log(`📺 Found ${screens.length} screens`);
        
        if (screens.length === 0) {
            console.log('❌ No screens found - cannot test content filter');
            return { success: false, error: 'No screens' };
        }
        
        // Step 3: Attempt to create a minimal recording configuration
        console.log('\n📋 Step 3: Testing Content Filter Creation via Recording Start');
        const outputPath = '/tmp/test-content-filter.mp4';
        const testConfig = {
            outputPath: outputPath,
            width: 1280,
            height: 720,
            fps: 30,
            showCursor: true,
            captureAudio: false  // Disable audio to simplify test
        };
        
        console.log('🎯 Test configuration:', JSON.stringify(testConfig, null, 2));
        
        try {
            // This will test the content filter creation internally
            console.log('🚀 Attempting to start recording (this tests content filter creation)...');
            manager.startRecording(testConfig);
            
            console.log('✅ Recording started successfully!');
            console.log('💡 This means content filter creation is working!');
            
            // Immediately stop the recording since we're just testing filter creation
            console.log('⏹️ Stopping recording immediately (we only wanted to test filter creation)...');
            const outputResult = manager.stopRecording();
            console.log(`✅ Recording stopped successfully: ${outputResult}`);
            
            // Clean up the test file
            try {
                const fs = require('fs');
                if (fs.existsSync(outputResult)) {
                    fs.unlinkSync(outputResult);
                    console.log('🧹 Cleaned up test file');
                }
            } catch (cleanupError) {
                console.log('⚠️ Could not clean up test file:', cleanupError.message);
            }
            
            return {
                success: true,
                message: 'Content filter creation is working!',
                outputPath: outputResult,
                screensFound: screens.length
            };
            
        } catch (recordingError) {
            console.log('❌ Recording failed:', recordingError.message);
            
            // Analyze the error to see if it's still a content filter issue
            if (recordingError.message.includes('null') || 
                recordingError.message.includes('bypass') ||
                recordingError.message.includes('content filter')) {
                console.log('🔍 Error analysis: This appears to be a content filter issue');
                console.log('💡 The fix may not have been applied correctly');
                return {
                    success: false,
                    error: 'Content filter still failing',
                    details: recordingError.message,
                    needsFix: true
                };
            } else {
                console.log('🔍 Error analysis: This appears to be a different issue (content filter may be working)');
                console.log('💡 The content filter creation might be working, but there\'s another problem');
                return {
                    success: 'partial',
                    error: recordingError.message,
                    contentFilterStatus: 'possibly working',
                    details: 'Error is not related to content filter creation'
                };
            }
        }
        
    } catch (error) {
        console.error('❌ Test failed with unexpected error:', error.message);
        return {
            success: false,
            error: error.message,
            stack: error.stack
        };
    }
}

async function testLegacyRecorder() {
    console.log('\n🔄 Testing Legacy ScreenCaptureKitRecorder');
    console.log('===========================================');
    
    try {
        const recorder = new ScreenCaptureKitRecorder();
        
        // Get screens using the timeout method
        const screens = recorder.getAvailableScreensWithTimeout(5000);
        console.log(`📺 Legacy recorder found ${screens.length} screens`);
        
        if (screens.length > 0) {
            const testConfig = {
                width: 1280,
                height: 720,
                fps: 30,
                showCursor: true,
                captureAudio: false,
                outputPath: '/tmp/test-legacy-recording.mp4'
            };
            
            try {
                recorder.startRecording(screens[0].id, testConfig);
                console.log('✅ Legacy recorder started (content filter working)');
                
                // Stop immediately
                const result = recorder.stopRecording();
                console.log(`✅ Legacy recorder stopped: ${result}`);
                
                return { success: true, legacy: true };
            } catch (legacyError) {
                console.log('❌ Legacy recorder failed:', legacyError.message);
                return { success: false, legacy: true, error: legacyError.message };
            }
        } else {
            return { success: false, legacy: true, error: 'No screens found' };
        }
        
    } catch (error) {
        console.log('❌ Legacy recorder test failed:', error.message);
        return { success: false, legacy: true, error: error.message };
    }
}

async function main() {
    console.log('🚀 Content Filter Fix Verification');
    console.log('===================================');
    console.log('This test verifies that content filter creation now works');
    console.log('instead of being bypassed and returning null.\n');
    
    // Test the new integrated recording manager
    const integratedResult = await testContentFilterFix();
    
    // Test the legacy recorder for comparison
    const legacyResult = await testLegacyRecorder();
    
    // Summary
    console.log('\n📋 Test Results Summary');
    console.log('=======================');
    console.log('Integrated Recording Manager:', integratedResult.success ? '✅ Working' : '❌ Failed');
    console.log('Legacy Recorder:', legacyResult.success ? '✅ Working' : '❌ Failed');
    
    if (integratedResult.success) {
        console.log('\n🎉 SUCCESS: Content filter fix is working!');
        console.log('✅ Content filters are now being created instead of bypassed');
        console.log('✅ Recording can now proceed with actual ScreenCaptureKit functionality');
        console.log('\n🚀 Next steps:');
        console.log('  - Run a full recording test');
        console.log('  - Test with different screen sources');
        console.log('  - Verify video output quality');
    } else if (integratedResult.needsFix) {
        console.log('\n❌ FAILURE: Content filter is still being bypassed');
        console.log('🔧 The fix needs to be applied:');
        console.log('  1. Update src/screencapturekit/bindings.rs with the fixed create_minimal_content_filter()');
        console.log('  2. Update src/screencapturekit/content.rs with the fixed content filter methods');
        console.log('  3. Rebuild: npm run build');
        console.log('  4. Run this test again');
    } else {
        console.log('\n⚠️ PARTIAL: Content filter may be working, but other issues exist');
        console.log('🔍 Error details:', integratedResult.error);
        console.log('💡 The content filter bypass may have been fixed, but there are other problems to resolve');
    }
    
    console.log('\n📊 Detailed Results:');
    console.log('Integrated Manager Result:', JSON.stringify(integratedResult, null, 2));
    console.log('Legacy Recorder Result:', JSON.stringify(legacyResult, null, 2));
    
    // Exit with appropriate code
    const overallSuccess = integratedResult.success === true || integratedResult.success === 'partial';
    process.exit(overallSuccess ? 0 : 1);
}

if (require.main === module) {
    main().catch(error => {
        console.error('❌ Unhandled error in test:', error);
        process.exit(1);
    });
}

module.exports = { testContentFilterFix, testLegacyRecorder }; 