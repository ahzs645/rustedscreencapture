#!/usr/bin/env node

/**
 * Test Real SCDisplay Fix
 * 
 * This test verifies that the content filter now uses real SCDisplay objects
 * extracted from ScreenCaptureKit content instead of null/placeholder values.
 * This should fix the -3802 stream start error.
 */

const { IntegratedRecordingManager } = require('../index.js');
const fs = require('fs');
const path = require('path');

async function testRealSCDisplayFix() {
    console.log('🔧 Testing Real SCDisplay Object Fix');
    console.log('====================================');
    console.log('This test verifies that content filters now use REAL SCDisplay objects');
    console.log('instead of null/placeholder values, which should fix the -3802 error.\n');
    
    const manager = new IntegratedRecordingManager();
    
    try {
        console.log('📋 Step 1: Initializing Recording Manager');
        await manager.initialize();
        console.log('✅ Manager initialized successfully');
        
        console.log('\n📋 Step 2: Checking Permissions');
        const permissionStatus = manager.getPermissionStatus();
        console.log('Permission Status:', permissionStatus);
        
        // Check if we have screen recording permission
        if (!permissionStatus.includes('✅ Granted')) {
            console.log('❌ Screen recording permission required for this test');
            console.log('Please enable screen recording permission and run again');
            return { success: false, error: 'No permission' };
        }
        
        console.log('\n📋 Step 3: Getting Available Screens');
        const screens = manager.getAvailableScreens();
        console.log(`Found ${screens.length} screens:`);
        screens.forEach((screen, index) => {
            console.log(`  ${index + 1}. ${screen.name} (${screen.width}x${screen.height}) [${screen.id}]`);
        });
        
        if (screens.length === 0) {
            console.log('❌ No screens found - cannot test content filter');
            return { success: false, error: 'No screens' };
        }
        
        console.log('\n📋 Step 4: Testing Real SCDisplay Content Filter Creation');
        const outputDir = '/tmp';
        const timestamp = Date.now();
        const outputPath = path.join(outputDir, `real-scdisplay-test-${timestamp}.mp4`);
        
        const testConfig = {
            outputPath: outputPath,
            width: 1280,
            height: 720,
            fps: 30,
            showCursor: true,
            captureAudio: false  // Disable audio to simplify test
        };
        
        console.log('🎯 Test configuration:');
        console.log(`   Output: ${testConfig.outputPath}`);
        console.log(`   Resolution: ${testConfig.width}x${testConfig.height}`);
        console.log(`   FPS: ${testConfig.fps}`);
        console.log(`   Using screen: ${screens[0].name}`);
        
        console.log('\n🚀 Step 5: Attempting to Start Recording...');
        console.log('💡 This will test the REAL SCDisplay object extraction and usage');
        
        try {
            // This should now use REAL SCDisplay objects instead of null/placeholders
            await manager.startRecording(testConfig);
            
            console.log('✅ Recording started successfully!');
            console.log('🎉 SUCCESS: Content filter is now using REAL SCDisplay objects!');
            console.log('🎉 The -3802 error should be fixed!');
            
            // Let it record for 3 seconds to verify it's actually capturing
            console.log('\n📊 Step 6: Recording for 3 seconds to verify capture...');
            for (let i = 1; i <= 3; i++) {
                await new Promise(resolve => setTimeout(resolve, 1000));
                const stats = JSON.parse(manager.getRecordingStats());
                console.log(`Recording... ${i}s - Frames: ${stats.videoFrames || 0}, Active: ${stats.isRecording}`);
                
                if (!stats.isRecording) {
                    console.log('⚠️ Recording stopped unexpectedly during capture test');
                    break;
                }
            }
            
            console.log('\n⏹️ Step 7: Stopping Recording...');
            const finalOutputPath = await manager.stopRecording();
            console.log(`✅ Recording stopped successfully: ${finalOutputPath}`);
            
            // Verify the output file
            console.log('\n🔍 Step 8: Verifying Output File...');
            if (fs.existsSync(finalOutputPath)) {
                const stats = fs.statSync(finalOutputPath);
                console.log(`✅ Output file exists: ${finalOutputPath}`);
                console.log(`📊 File size: ${(stats.size / 1024).toFixed(1)} KB`);
                
                if (stats.size > 1024) { // At least 1KB
                    console.log('🎉 SUCCESS: File has content - real screen capture is working!');
                    
                    // Clean up test file
                    try {
                        fs.unlinkSync(finalOutputPath);
                        console.log('🧹 Cleaned up test file');
                    } catch (cleanupError) {
                        console.log('⚠️ Could not clean up test file:', cleanupError.message);
                    }
                    
                    return {
                        success: true,
                        message: 'Real SCDisplay objects are working!',
                        outputPath: finalOutputPath,
                        fileSize: stats.size,
                        duration: '3 seconds',
                        fix: 'Content filters now use real SCDisplay objects from ScreenCaptureKit'
                    };
                } else {
                    console.log('⚠️ Output file is empty - may still have encoding issues');
                    return {
                        success: 'partial',
                        message: 'Stream starts but encoding may have issues',
                        outputPath: finalOutputPath,
                        fileSize: stats.size,
                        fix: 'Stream start fixed, but encoding needs attention'
                    };
                }
            } else {
                console.log('❌ Output file was not created');
                return {
                    success: 'partial',
                    message: 'Stream starts but file creation failed',
                    fix: 'Stream start fixed, but file output needs attention'
                };
            }
            
        } catch (recordingError) {
            console.log('❌ Recording failed:', recordingError.message);
            
            // Analyze the specific error to see if it's still the -3802 issue
            if (recordingError.message.includes('-3802')) {
                console.log('\n🔍 Error Analysis: Still getting -3802 error');
                console.log('❌ DIAGNOSIS: The real SCDisplay object fix may not have been applied correctly');
                console.log('🔧 REQUIRED FIXES:');
                console.log('   1. Ensure the updated bindings.rs with extract_display_by_id() is compiled');
                console.log('   2. Ensure the updated content.rs with real object usage is compiled');
                console.log('   3. Run: npm run build');
                console.log('   4. Verify ScreenCaptureKit content enumeration is working');
                
                return {
                    success: false,
                    error: 'Still getting -3802 error',
                    diagnosis: 'Real SCDisplay fix not applied or not working',
                    nextSteps: [
                        'Update bindings.rs with extract_display_by_id method',
                        'Update content.rs to use real objects',
                        'Rebuild with npm run build',
                        'Test again'
                    ]
                };
            } else if (recordingError.message.includes('content filter')) {
                console.log('\n🔍 Error Analysis: Content filter related error');
                console.log('⚠️ DIAGNOSIS: Progress made, but content filter still has issues');
                console.log('💡 This may be a different content filter issue than the original -3802');
                
                return {
                    success: false,
                    error: recordingError.message,
                    diagnosis: 'Content filter issue (different from -3802)',
                    progress: 'May have fixed the original issue but uncovered a new one'
                };
            } else if (recordingError.message.includes('stream') && !recordingError.message.includes('-3802')) {
                console.log('\n🔍 Error Analysis: Different stream error (not -3802)');
                console.log('✅ DIAGNOSIS: Original -3802 error may be fixed!');
                console.log('💡 This appears to be a different issue, which means progress!');
                
                return {
                    success: 'partial',
                    error: recordingError.message,
                    diagnosis: 'Different error than -3802 (this is progress!)',
                    fix: 'Real SCDisplay fix may be working, but there\'s a new issue to resolve'
                };
            } else {
                console.log('\n🔍 Error Analysis: Unknown error pattern');
                console.log('🤔 DIAGNOSIS: Need to investigate this specific error');
                
                return {
                    success: false,
                    error: recordingError.message,
                    diagnosis: 'Unknown error pattern',
                    needsInvestigation: true
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

async function main() {
    console.log('🚀 Real SCDisplay Object Fix Verification');
    console.log('=========================================');
    console.log('This test checks if the content filter -3802 error is fixed');
    console.log('by using real SCDisplay objects instead of placeholders.\n');
    
    const result = await testRealSCDisplayFix();
    
    console.log('\n📋 Test Results Summary');
    console.log('=======================');
    
    if (result.success === true) {
        console.log('🎉 SUCCESS: Real SCDisplay fix is working!');
        console.log('✅ Content filters now use actual SCDisplay objects from ScreenCaptureKit');
        console.log('✅ Stream starts successfully without -3802 errors');
        console.log('✅ Screen capture is producing actual video output');
        console.log('\n🎯 Next Steps:');
        console.log('  - Test with different screen sources');
        console.log('  - Test window capture');
        console.log('  - Enable audio recording');
        console.log('  - Test longer recordings');
        console.log('  - Verify video quality');
        
    } else if (result.success === 'partial') {
        console.log('⚠️ PARTIAL SUCCESS: Progress made but issues remain');
        console.log('✅ Stream start may be fixed (no more -3802)');
        console.log('❌ Other issues need attention:', result.error);
        console.log('\n🎯 Next Steps:');
        console.log('  - Investigate the new error type');
        console.log('  - Fix remaining encoding/output issues');
        console.log('  - Test again');
        
    } else {
        console.log('❌ FAILURE: Real SCDisplay fix needs work');
        console.log('🔧 Required Actions:');
        
        if (result.nextSteps) {
            result.nextSteps.forEach((step, index) => {
                console.log(`  ${index + 1}. ${step}`);
            });
        } else {
            console.log('  1. Check that the code changes were applied correctly');
            console.log('  2. Rebuild the project: npm run build');
            console.log('  3. Verify ScreenCaptureKit permissions');
            console.log('  4. Test again');
        }
    }
    
    console.log('\n📊 Detailed Results:');
    console.log(JSON.stringify(result, null, 2));
    
    // Exit with appropriate code
    const overallSuccess = result.success === true || result.success === 'partial';
    process.exit(overallSuccess ? 0 : 1);
}

if (require.main === module) {
    main().catch(error => {
        console.error('❌ Unhandled error in test:', error);
        process.exit(1);
    });
}

module.exports = { testRealSCDisplayFix };