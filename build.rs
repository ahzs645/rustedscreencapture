use std::env;
use std::process::Command;
use std::path::PathBuf;

fn main() {
    // Link ScreenCaptureKit framework
    println!("cargo:rustc-link-lib=framework=ScreenCaptureKit");
    println!("cargo:rustc-link-lib=framework=CoreMedia");
    println!("cargo:rustc-link-lib=framework=CoreVideo");
    println!("cargo:rustc-link-lib=framework=AVFoundation");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=AppKit");
    
    // Set minimum macOS version for ScreenCaptureKit
    println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=12.3");
    
    // Compile Objective-C bridge
    compile_objc_bridge();
    
    napi_build::setup();
}

fn compile_objc_bridge() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    
    // Only compile Objective-C bridge on macOS
    if target_os != "macos" {
        println!("cargo:warning=Skipping Objective-C bridge compilation on non-macOS platform");
        return;
    }
    
    println!("cargo:rerun-if-changed=src/screencapturekit/objc_bridge.h");
    println!("cargo:rerun-if-changed=src/screencapturekit/objc_bridge.m");
    
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let obj_file = out_dir.join("objc_bridge.o");
    
    // Compile the Objective-C file directly using clang
    let output = Command::new("clang")
        .args(&[
            "-c",
            "-fobjc-arc", // Enable ARC
            "-fmodules", // Enable modules
            "-mmacosx-version-min=12.3", // Minimum macOS version
            "-Wno-unused-parameter", // Suppress unused parameter warnings
            "-Wno-deprecated-declarations", // Suppress deprecation warnings
            "-Wno-nullability-completeness", // Suppress nullability warnings
            "-I", "src/screencapturekit", // Include directory
            "-o", obj_file.to_str().unwrap(),
            "src/screencapturekit/objc_bridge.m"
        ])
        .output()
        .expect("Failed to execute clang");
    
    if !output.status.success() {
        panic!("Failed to compile Objective-C bridge: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    // Tell cargo to link the object file directly
    println!("cargo:rustc-link-arg={}", obj_file.display());
} 