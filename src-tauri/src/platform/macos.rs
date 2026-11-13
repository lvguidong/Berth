use super::FolderWindow;
use std::{
    io::Read,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

// JXA serializes records as JSON: tabs, quotes, newlines and Unicode paths stay lossless.
fn script(source: &str, args: &[&str]) -> Result<String, String> {
    let mut child = Command::new("/usr/bin/osascript")
        .args(["-l", "JavaScript", "-e", source])
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut out = child.stdout.take().unwrap();
    let mut err = child.stderr.take().unwrap();
    let stdout = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = out.read_to_string(&mut s);
        s
    });
    let stderr = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = err.read_to_string(&mut s);
        s
    });
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            let output = stdout.join().unwrap_or_default();
            let error = stderr.join().unwrap_or_default();
            return if status.success() {
                Ok(output.trim().into())
            } else {
                Err(format!("无法读取访达窗口。请在 系统设置 → 隐私与安全性 → 自动化 中允许泊岸控制访达。{error}"))
            };
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout.join();
            let _ = stderr.join();
            return Err("访达响应超时，请检查自动化权限或网络卷连接".into());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}
pub struct Explorer;
pub fn new_window(path: &str) -> Result<(), String> {
    match script("function run(a){const f=Application('Finder');f.FinderWindow({target:Path(a[0])}).make();f.activate();}",&[path]) {
        Ok(_)=>Ok(()),
        // Opening folders remains available even when Automation permission was denied.
        Err(_)=>Command::new("/usr/bin/open").arg(path).spawn().map(|_|()).map_err(|e|e.to_string())
    }
}
impl Explorer {
    pub fn new() -> Result<Self, String> {
        Ok(Self)
    }
    pub fn list(&self) -> Result<Vec<FolderWindow>, String> {
        let s = script("const f=Application('Finder'); JSON.stringify(f.finderWindows().map(w=>{try{return {id:String(w.id()),path:decodeURIComponent(w.target().url().replace(/^file:\\/\\//,'')),title:w.name(),minimized:w.collapsed()}}catch(e){return null}}).filter(Boolean))", &[])?;
        serde_json::from_str(&s).map_err(|e| e.to_string())
    }
    pub fn recent(&self) -> Result<Vec<FolderWindow>, String> {
        self.list()
    }
    pub fn activate(&self, id: &str) -> Result<(), String> {
        script("function run(a){const f=Application('Finder');const w=f.finderWindows.byId(Number(a[0]));w.collapsed=false;w.index=1;f.activate();}", &[id]).map(|_| ())
    }
    pub fn close(&self, id: &str) -> Result<(), String> {
        script(
            "function run(a){Application('Finder').finderWindows.byId(Number(a[0])).close();}",
            &[id],
        )
        .map(|_| ())
    }
    pub fn foreground_path(&self) -> Result<String, String> {
        script("const f=Application('Finder');if(!f.frontmost())throw Error('请先切换到访达');decodeURIComponent(f.finderWindows[0].target().url().replace(/^file:\\/\\//,''))", &[])
    }
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct CGPoint {
    x: f64,
    y: f64,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct CGSize {
    width: f64,
    height: f64,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct CGRect {
    origin: CGPoint,
    size: CGSize,
}
type CFRef = *const std::ffi::c_void;
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGWindowListCopyWindowInfo(options: u32, relative_to: u32) -> CFRef;
    fn CGRectMakeWithDictionaryRepresentation(dict: CFRef, rect: *mut CGRect) -> bool;
    fn CGGetActiveDisplayList(max: u32, displays: *mut u32, count: *mut u32) -> i32;
    fn CGDisplayBounds(display: u32) -> CGRect;
    static kCGWindowLayer: CFRef;
    static kCGWindowBounds: CFRef;
    static kCGWindowOwnerPID: CFRef;
}
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFArrayGetCount(array: CFRef) -> isize;
    fn CFArrayGetValueAtIndex(array: CFRef, index: isize) -> CFRef;
    fn CFDictionaryGetValue(dict: CFRef, key: CFRef) -> CFRef;
    fn CFNumberGetValue(number: CFRef, kind: isize, value: *mut i32) -> bool;
    fn CFRelease(value: CFRef);
}
pub fn fullscreen() -> bool {
    // Only public window bounds are inspected; no titles, pixels or screen recording permission.
    unsafe {
        let array = CGWindowListCopyWindowInfo(1 | 16, 0);
        if array.is_null() {
            return false;
        }
        let mut displays = [0u32; 32];
        let mut count = 0;
        if CGGetActiveDisplayList(32, displays.as_mut_ptr(), &mut count) != 0 {
            CFRelease(array);
            return false;
        }
        let mut result = false;
        for i in 0..CFArrayGetCount(array) {
            let dict = CFArrayGetValueAtIndex(array, i);
            let layer = CFDictionaryGetValue(dict, kCGWindowLayer);
            let pid = CFDictionaryGetValue(dict, kCGWindowOwnerPID);
            let (mut l, mut p) = (0, 0);
            if layer.is_null() || pid.is_null() {
                continue;
            }
            if !CFNumberGetValue(layer, 3, &mut l)
                || !CFNumberGetValue(pid, 3, &mut p)
                || l != 0
                || p == std::process::id() as i32
            {
                continue;
            }
            let bounds = CFDictionaryGetValue(dict, kCGWindowBounds);
            let mut rect = CGRect::default();
            if !bounds.is_null() && CGRectMakeWithDictionaryRepresentation(bounds, &mut rect) {
                result = displays[..count as usize].iter().any(|d| {
                    let screen = CGDisplayBounds(*d);
                    (rect.origin.x - screen.origin.x).abs() < 2.
                        && (rect.origin.y - screen.origin.y).abs() < 2.
                        && (rect.size.width - screen.size.width).abs() < 2.
                        && (rect.size.height - screen.size.height).abs() < 2.
                });
            }
            break; // first normal window is the foreground application
        }
        CFRelease(array);
        result
    }
}
