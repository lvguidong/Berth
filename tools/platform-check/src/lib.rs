//! Type-checks the actual platform adapters without the WebView/Objective-C build toolchain.
//! This is not an application build and does not replace real macOS execution.
#![allow(dead_code, unused_imports)]
#[path = "../../../src-tauri/src/paths.rs"]
mod paths;
#[path = "../../../src-tauri/src/platform/mod.rs"]
mod platform;
