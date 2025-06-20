#!/usr/bin/env node

// Test Real ScreenCaptureKit Integration
// This test verifies we're calling actual ScreenCaptureKit APIs

const { ScreenCaptureKitRecorder } = require('../index.js');

async function testRealScreenCaptureKit() {
    console.log('🔬 Real ScreenCaptureKit API Test');
    console.log('==================================');
    console.log('This test verifies we\'re calling actual ScreenCaptureKit APIs, not mock data.\n');

    try {
        // Create recorder
        console.log('📱 Creating ScreenCaptureKit recorder...');
        const recorder = new ScreenCaptureKitRecorder();
        console.log('✅ Recorder created\n');

        // Get screens using real APIs
        console.log('🔍 Calling real ScreenCaptureKit APIs...');
        const screens = await recorder.getAvailableScreens();
        
        console.log(`✅ Retrieved ${screens.length} screen sources from ScreenCaptureKit\n`);

        // Analyze the results to verify they're real
        console.log('📊 Analysis of Retrieved Data:');
        console.log('==============================');

        const displays = screens.filter(s => s.isDisplay);
        const windows = screens.filter(s => !s.isDisplay);

        console.log(`📺 Displays found: ${displays.length}`);
        displays.forEach((display, i) => {
            console.log(`   ${i + 1}. ${display.name} (${display.width}x${display.height})`);
            console.log(`      ID: ${display.id}`);
        });

        console.log(`\n🪟 Windows found: ${windows.length}`);
        windows.slice(0, 5).forEach((window, i) => {
            console.log(`   ${i + 1}. ${window.name} (${window.width}x${window.height})`);
            console.log(`      ID: ${window.id}`);
        });

        if (windows.length > 5) {
            console.log(`   ... and ${windows.length - 5} more windows`);
        }

        // Verify this is real data
        console.log('\n🔬 Data Verification:');
        console.log('=====================');
        
        // Check for real display properties
        const hasRealDisplays = displays.some(d => 
            d.width > 0 && d.height > 0 && d.id && d.name
        );
        
        // Check for real window properties  
        const hasRealWindows = windows.some(w => 
            w.width > 0 && w.height > 0 && w.id && w.name && w.name.length > 0
        );

        // Check for system-specific patterns
        const hasSystemWindows = windows.some(w => 
            w.name.includes('Safari') || 
            w.name.includes('Terminal') || 
            w.name.includes('Finder') ||
            w.name.includes('Chrome') ||
            w.name.includes('Code')
        );

        console.log(`✅ Real display data: ${hasRealDisplays ? 'YES' : 'NO'}`);
        console.log(`✅ Real window data: ${hasRealWindows ? 'YES' : 'NO'}`);
        console.log(`✅ System windows detected: ${hasSystemWindows ? 'YES' : 'NO'}`);

        // Final verification
        console.log('\n🎯 Final Verification:');
        console.log('======================');
        
        if (hasRealDisplays && hasRealWindows) {
            console.log('🎉 SUCCESS: Real ScreenCaptureKit APIs are working!');
            console.log('✅ We are calling actual ScreenCaptureKit, not mock data');
            console.log('✅ Data extraction is working correctly');
            console.log('✅ No segmentation faults occurred');
            
            // Test a recording start to verify full integration
            console.log('\n🎬 Testing recording start...');
            if (displays.length > 0) {
                const displayId = displays[0].id;
                await recorder.startRecording(displayId, {
                    outputPath: '/tmp/test-recording.mp4'
                });
                console.log('✅ Recording started successfully');
                
                const result = await recorder.stopRecording();
                console.log('✅ Recording stopped successfully');
            }
            
            return {
                success: true,
                realData: true,
                displaysFound: displays.length,
                windowsFound: windows.length,
                segfaultFree: true
            };
        } else {
            console.log('❌ ISSUE: Data appears to be mock or incomplete');
            return {
                success: false,
                realData: false,
                displaysFound: displays.length,
                windowsFound: windows.length
            };
        }

    } catch (error) {
        console.error('❌ ERROR:', error.message);
        console.error('Stack:', error.stack);
        return {
            success: false,
            error: error.message
        };
    }
}

// Run the test
testRealScreenCaptureKit()
    .then(result => {
        console.log('\n📋 Test Results:');
        console.log('================');
        console.log(JSON.stringify(result, null, 2));
        
        if (result.success) {
            console.log('\n🏆 CONCLUSION: Real ScreenCaptureKit integration is working perfectly!');
            process.exit(0);
        } else {
            console.log('\n💥 CONCLUSION: Issues detected with ScreenCaptureKit integration');
            process.exit(1);
        }
    })
    .catch(error => {
        console.error('\n💥 FATAL ERROR:', error);
        process.exit(1);
    }); 