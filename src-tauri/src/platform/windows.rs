use super::FolderWindow;
use windows::{
    core::Interface,
    Win32::{
        Foundation::{HWND, LPARAM},
        System::{
            Com::{
                CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL,
                COINIT_APARTMENTTHREADED,
            },
            Threading::{AttachThreadInput, GetCurrentThreadId},
            Variant::VARIANT,
        },
        UI::{
            Shell::{
                Folder2, IShellFolderViewDual, IShellWindows, IWebBrowser2,
                SHQueryUserNotificationState, ShellWindows, QUNS_BUSY, QUNS_PRESENTATION_MODE,
                QUNS_RUNNING_D3D_FULL_SCREEN,
            },
            WindowsAndMessaging::*,
        },
    },
};

use std::{
    cell::RefCell,
    time::{Duration, Instant},
};
type BrowserRow = (String, IWebBrowser2, FolderWindow);
pub struct Explorer {
    shell: RefCell<Option<IShellWindows>>,
    cache: RefCell<Vec<BrowserRow>>,
    updated: RefCell<Option<Instant>>,
}
unsafe extern "system" fn collect_window(hwnd: HWND, param: LPARAM) -> windows::core::BOOL {
    let windows = &mut *(param.0 as *mut Vec<isize>);
    windows.push(hwnd.0 as isize);
    true.into()
}
impl Explorer {
    pub fn new() -> Result<Self, String> {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED)
                .ok()
                .map_err(|e| e.to_string())?;
        }
        Ok(Self {
            shell: RefCell::new(None),
            cache: RefCell::new(vec![]),
            updated: RefCell::new(None),
        })
    }
    fn browsers(&self) -> Result<Vec<(String, IWebBrowser2, FolderWindow)>, String> {
        unsafe {
            if self.shell.borrow().is_none() {
                *self.shell.borrow_mut() = Some(
                    CoCreateInstance(&ShellWindows, None, CLSCTX_ALL).map_err(|e| e.to_string())?,
                );
            }
            let shell = self.shell.borrow().as_ref().unwrap().clone();
            let count = match shell.Count() {
                Ok(c) => c,
                Err(e) => {
                    self.shell.borrow_mut().take();
                    self.cache.borrow_mut().clear();
                    return Err(e.to_string());
                }
            };
            let mut rows = vec![];
            for i in 0..count {
                let Ok(dispatch) = shell.Item(&VARIANT::from(i)) else {
                    continue;
                };
                let Ok(browser) = dispatch.cast::<IWebBrowser2>() else {
                    continue;
                };
                let Ok(handle) = browser.HWND() else { continue };
                let hwnd = HWND(handle.0 as *mut _);
                let mut class = [0u16; 256];
                let len = GetClassNameW(hwnd, &mut class);
                let class = String::from_utf16_lossy(&class[..len as usize]);
                if class != "CabinetWClass" && class != "ExploreWClass" {
                    continue;
                }
                let path = browser
                    .LocationURL()
                    .ok()
                    .and_then(|u| url::Url::parse(&u.to_string()).ok())
                    .and_then(|u| u.to_file_path().ok())
                    .map(|p| p.to_string_lossy().into_owned())
                    .or_else(|| {
                        browser
                            .Document()
                            .ok()?
                            .cast::<IShellFolderViewDual>()
                            .ok()?
                            .Folder()
                            .ok()?
                            .cast::<Folder2>()
                            .ok()?
                            .Self_()
                            .ok()?
                            .Path()
                            .ok()
                            .map(|s| s.to_string())
                    });
                let Some(path) = path else { continue };
                if !std::path::Path::new(&path).is_absolute() {
                    continue;
                }
                let id = format!("{}:{}", handle.0, crate::paths::key(&path));
                let title = browser
                    .LocationName()
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                let row = FolderWindow {
                    id: id.clone(),
                    path,
                    title,
                    minimized: IsIconic(hwnd).as_bool(),
                };
                rows.push((id, browser, row));
            }
            // Z-order reflects the most recently used Explorer window, including windows not clicked via Berth.
            let mut zorder = Vec::<isize>::new();
            let _ = EnumWindows(Some(collect_window), LPARAM(&mut zorder as *mut _ as isize));
            rows.sort_by_key(|(_, b, _)| {
                b.HWND()
                    .ok()
                    .and_then(|h| zorder.iter().position(|p| *p == h.0))
                    .unwrap_or(usize::MAX)
            });
            Ok(rows)
        }
    }
    pub fn list(&self) -> Result<Vec<FolderWindow>, String> {
        let rows = self.browsers()?;
        let result = rows.iter().map(|(_, _, w)| w.clone()).collect();
        *self.cache.borrow_mut() = rows;
        *self.updated.borrow_mut() = Some(Instant::now());
        Ok(result)
    }
    pub fn recent(&self) -> Result<Vec<FolderWindow>, String> {
        if self
            .updated
            .borrow()
            .is_some_and(|t| t.elapsed() < Duration::from_secs(1))
        {
            Ok(self
                .cache
                .borrow()
                .iter()
                .map(|(_, _, w)| w.clone())
                .collect())
        } else {
            self.list()
        }
    }
    pub fn activate(&self, id: &str) -> Result<(), String> {
        if !self.cache.borrow().iter().any(|(key, _, _)| key == id) {
            self.list()?;
        }
        let rows = self.cache.borrow();
        let (_, b, _) = rows
            .iter()
            .find(|(key, _, _)| key == id)
            .ok_or("窗口已关闭，请重试")?;
        unsafe {
            let hwnd = HWND(b.HWND().map_err(|e| e.to_string())?.0 as *mut _);
            if !IsWindow(Some(hwnd)).as_bool() {
                return Err("窗口已关闭，请重试".into());
            }
            let expected = &rows.iter().find(|(key, _, _)| key == id).unwrap().2.path;
            let current = b
                .LocationURL()
                .ok()
                .and_then(|u| url::Url::parse(&u.to_string()).ok())
                .and_then(|u| u.to_file_path().ok());
            if current.is_some_and(|p| {
                crate::paths::key(&p.to_string_lossy()) != crate::paths::key(expected)
            }) {
                return Err("窗口目录已变化，请重试".into());
            }
            // Refuse ambiguous tab activation instead of claiming to have switched to the requested folder.
            let siblings = rows
                .iter()
                .filter(|(_, other, _)| other.HWND().ok() == b.HWND().ok())
                .count();
            if siblings > 1 {
                return Err("该窗口包含多个资源管理器标签页；请在窗口中选择对应标签，或按 Shift 点击新开独立窗口".into());
            }
            if IsIconic(hwnd).as_bool() {
                let _ = ShowWindowAsync(hwnd, SW_RESTORE);
            }
            if SetForegroundWindow(hwnd).as_bool() || GetForegroundWindow() == hwnd {
                return Ok(());
            }
            let fg = GetForegroundWindow();
            let current = GetCurrentThreadId();
            let foreground_thread = GetWindowThreadProcessId(fg, None);
            let target_thread = GetWindowThreadProcessId(hwnd, None);
            let attached = current != foreground_thread
                && AttachThreadInput(current, foreground_thread, true).as_bool();
            let target_attached = current != target_thread
                && target_thread != foreground_thread
                && AttachThreadInput(current, target_thread, true).as_bool();
            let _ = BringWindowToTop(hwnd);
            let activated = SetForegroundWindow(hwnd).as_bool() || GetForegroundWindow() == hwnd;
            if target_attached {
                let _ = AttachThreadInput(current, target_thread, false);
            }
            if attached {
                let _ = AttachThreadInput(current, foreground_thread, false);
            }
            if activated {
                Ok(())
            } else {
                let _ = FlashWindowEx(&FLASHWINFO {
                    cbSize: std::mem::size_of::<FLASHWINFO>() as u32,
                    hwnd,
                    dwFlags: FLASHW_TRAY,
                    uCount: 3,
                    dwTimeout: 0,
                });
                Err("系统限制了窗口置前，已闪烁提示；请点击任务栏中的资源管理器".into())
            }
        }
    }
    pub fn close(&self, id: &str) -> Result<(), String> {
        let rows = self.browsers()?;
        let (_, b, _) = rows
            .iter()
            .find(|(key, _, _)| key == id)
            .ok_or("窗口已关闭")?;
        unsafe {
            if rows
                .iter()
                .filter(|(_, other, _)| other.HWND().ok() == b.HWND().ok())
                .count()
                > 1
            {
                return Err("包含多个标签页的窗口不会自动关闭".into());
            }
            b.Quit().map_err(|e| e.to_string())
        }
    }
    pub fn foreground_path(&self) -> Result<String, String> {
        let rows = self.browsers()?;
        let fg = unsafe { GetForegroundWindow() };
        rows.into_iter()
            .find(|(_, b, _)| unsafe { b.HWND().map(|h| h.0 == fg.0 as isize).unwrap_or(false) })
            .map(|(_, _, w)| w.path)
            .ok_or("请先切换到一个资源管理器文件夹窗口".into())
    }
}
impl Drop for Explorer {
    fn drop(&mut self) {
        self.cache.get_mut().clear();
        self.shell.get_mut().take();
        unsafe {
            CoUninitialize();
        }
    }
}
pub fn fullscreen() -> bool {
    unsafe {
        SHQueryUserNotificationState()
            .map(|s| {
                s == QUNS_RUNNING_D3D_FULL_SCREEN || s == QUNS_PRESENTATION_MODE || s == QUNS_BUSY
            })
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    #[test]
    #[ignore = "Opens and closes only newly-created temporary Explorer windows; requires an interactive Windows desktop"]
    fn explorer_lifecycle() {
        let dir = tempfile::Builder::new()
            .prefix("Berth 测试 # percent% ")
            .tempdir()
            .unwrap();
        let path = crate::paths::resolve(dir.path().to_str().unwrap()).unwrap();
        let explorer = Explorer::new().unwrap();
        let mut tracker = crate::service::OpenTracker::default();
        struct Cleanup<'a>(&'a Explorer, String);
        impl Drop for Cleanup<'_> {
            fn drop(&mut self) {
                if let Ok(rows) = self.0.list() {
                    for row in rows {
                        if crate::paths::key(&row.path) == crate::paths::key(&self.1) {
                            let _ = self.0.close(&row.id);
                        }
                    }
                }
            }
        }
        let _cleanup = Cleanup(&explorer, path.clone());
        assert_eq!(tracker.open(&explorer, &path, false).unwrap(), "opened");
        // Rapid clicks must not spawn more than one window while Explorer starts.
        let second = tracker.open(&explorer, &path, false).unwrap();
        assert!(second == "opening" || second == "activated");
        let deadline = Instant::now() + Duration::from_secs(10);
        let row = loop {
            let rows = explorer.list().unwrap();
            if let Some(row) = rows
                .iter()
                .find(|w| crate::paths::key(&w.path) == crate::paths::key(&path))
            {
                break row.clone();
            }
            assert!(
                Instant::now() < deadline,
                "new Explorer window was not discovered: expected {path}; got {rows:?}"
            );
            std::thread::sleep(Duration::from_millis(100));
        };
        let start = Instant::now();
        assert_eq!(tracker.open(&explorer, &path, false).unwrap(), "activated");
        let elapsed = start.elapsed();
        println!("existing-window switch: {} ms", elapsed.as_millis());
        assert!(
            elapsed < Duration::from_millis(300),
            "switch exceeded 300ms"
        );
        assert_eq!(
            explorer
                .list()
                .unwrap()
                .iter()
                .filter(|w| crate::paths::key(&w.path) == crate::paths::key(&path))
                .count(),
            1
        );
        let rows = explorer.browsers().unwrap();
        let (_, browser, _) = rows.iter().find(|(id, _, _)| id == &row.id).unwrap();
        unsafe {
            let h = HWND(browser.HWND().unwrap().0 as *mut _);
            let _ = ShowWindowAsync(h, SW_MINIMIZE);
        }
        std::thread::sleep(Duration::from_millis(300));
        assert!(
            explorer
                .list()
                .unwrap()
                .iter()
                .find(|w| w.id == row.id)
                .unwrap()
                .minimized
        );
        explorer.activate(&row.id).unwrap();
        std::thread::sleep(Duration::from_millis(200));
        assert!(
            !explorer
                .list()
                .unwrap()
                .iter()
                .find(|w| w.id == row.id)
                .unwrap()
                .minimized
        );
        tracker.open(&explorer, &path, true).unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if explorer
                .list()
                .unwrap()
                .iter()
                .filter(|w| crate::paths::key(&w.path) == crate::paths::key(&path))
                .count()
                == 2
            {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "force-new did not create a second window"
            );
            std::thread::sleep(Duration::from_millis(100));
        }
        for row in explorer.list().unwrap() {
            if crate::paths::key(&row.path) == crate::paths::key(&path) {
                explorer.close(&row.id).unwrap();
            }
        }
        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            if !explorer
                .list()
                .unwrap()
                .iter()
                .any(|w| crate::paths::key(&w.path) == crate::paths::key(&path))
            {
                break;
            }
            assert!(Instant::now() < deadline, "close not discovered within 1s");
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}
