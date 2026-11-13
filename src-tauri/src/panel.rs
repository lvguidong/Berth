use crate::{config::Panel, AppState};
use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize};

fn ensure_ui(app: &AppHandle) {
    let state = app.state::<AppState>();
    if app.get_webview("main").is_some() || state.rendering.swap(true, Ordering::SeqCst) {
        return;
    }
    let app = app.clone();
    // WebView creation must not block the main event loop on Windows.
    std::thread::spawn(move || {
        let result = (|| -> Result<(), String> {
            let window = app.get_window("main").ok_or("面板窗口不存在")?;
            let size = window.inner_size().map_err(|e| e.to_string())?;
            let builder =
                tauri::WebviewBuilder::new("main", tauri::WebviewUrl::App("index.html".into()))
                    .auto_resize();
            window
                .add_child(builder, PhysicalPosition::new(0, 0), size)
                .map_err(|e| e.to_string())?;
            Ok(())
        })();
        let state = app.state::<AppState>();
        if let Err(e) = result {
            state.data.lock().unwrap().warning = Some(format!("面板恢复失败：{e}"));
        }
        state.rendering.store(false, Ordering::SeqCst);
    });
}
// On macOS a child WKWebView cannot be reliably re-attached after close():
// the native window survives but its content stays blank. Keep the WebView
// alive there; the idle memory recycle remains enabled on Windows.
#[cfg(target_os = "macos")]
fn release_ui(_app: &AppHandle) {}

#[cfg(not(target_os = "macos"))]
fn release_ui(app: &AppHandle) {
    let state = app.state::<AppState>();
    if state.interacting.load(Ordering::SeqCst) || state.rendering.load(Ordering::SeqCst) {
        return;
    }
    if let Some(webview) = app.get_webview("main") {
        let _ = webview.close();
    }
}

pub fn place(app: &AppHandle, expanded: bool) -> Result<(), String> {
    let state = app.state::<AppState>();
    let panel = state.data.lock().unwrap().config.panel.clone();
    let window = app.get_window("main").ok_or("面板窗口不存在")?;
    let monitors = window.available_monitors().map_err(|e| e.to_string())?;
    let chosen = panel
        .monitor
        .as_ref()
        .and_then(|name| monitors.iter().find(|m| m.name() == Some(name)));
    let fallback = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten());
    let pointer_monitor = if panel.mode == "popup" {
        window
            .cursor_position()
            .ok()
            .and_then(|p| window.monitor_from_point(p.x, p.y).ok().flatten())
    } else {
        None
    };
    let Some(monitor) = pointer_monitor.or(chosen.cloned()).or(fallback) else {
        return Ok(());
    };
    let area = monitor.work_area();
    let scale = monitor.scale_factor();
    let width = (panel.width * scale).min(area.size.width as f64) as u32;
    let height = (panel.height * scale).min(area.size.height as f64) as u32;
    let (w, x, y) = if panel.mode == "sidebar" {
        let w = if expanded { width } else { 6 };
        let x = if panel.edge == "left" {
            area.position.x
        } else {
            area.position.x + area.size.width as i32 - w as i32
        };
        (
            w,
            x,
            area.position.y + ((area.size.height - height) / 2) as i32,
        )
    } else {
        let cursor = window.cursor_position().ok();
        let px = if panel.mode == "popup" {
            cursor.map(|p| p.x)
        } else {
            panel.x
        }
        .unwrap_or(area.position.x as f64 + 32.);
        let py = if panel.mode == "popup" {
            cursor.map(|p| p.y)
        } else {
            panel.y
        }
        .unwrap_or(area.position.y as f64 + 32.);
        (
            width,
            (px as i32).clamp(
                area.position.x,
                area.position.x + (area.size.width - width) as i32,
            ),
            (py as i32).clamp(
                area.position.y,
                area.position.y + (area.size.height - height) as i32,
            ),
        )
    };
    state.collapsed.store(!expanded, Ordering::SeqCst);
    let color = if !expanded {
        tauri::window::Color(62, 121, 103, 255)
    } else if window.theme().ok() == Some(tauri::Theme::Dark) {
        tauri::window::Color(27, 35, 32, 255)
    } else {
        tauri::window::Color(249, 250, 247, 255)
    };
    let _ = window.set_background_color(Some(color));
    window
        .set_min_size(Some(PhysicalSize::new(
            if panel.mode == "sidebar" { 6 } else { 280 },
            200,
        )))
        .map_err(|e| e.to_string())?;
    window
        .set_size(PhysicalSize::new(w, height))
        .map_err(|e| e.to_string())?;
    window
        .set_position(PhysicalPosition::new(x, y))
        .map_err(|e| e.to_string())?;
    window
        .set_always_on_top(panel.mode == "sidebar" || panel.always_on_top)
        .map_err(|e| e.to_string())?;
    window
        .set_resizable(panel.mode == "float")
        .map_err(|e| e.to_string())?;
    #[cfg(target_os = "macos")]
    window
        .set_visible_on_all_workspaces(true)
        .map_err(|e| e.to_string())?;
    let _ = app.emit("collapsed", !expanded);
    Ok(())
}
pub fn reveal(app: &AppHandle, focus: bool) {
    let state = app.state::<AppState>();
    state.user_hidden.store(false, Ordering::SeqCst);
    if let Some(w) = app.get_window("main") {
        let _ = place(app, true);
        ensure_ui(app);
        let _ = w.show();
        if focus {
            let _ = w.set_focus();
        }
    }
}
pub fn toggle(app: &AppHandle) {
    let state = app.state::<AppState>();
    if let Some(w) = app.get_window("main") {
        if w.is_visible().unwrap_or(false) && !state.collapsed.load(Ordering::SeqCst) {
            state.user_hidden.store(true, Ordering::SeqCst);
            let _ = w.hide();
        } else {
            reveal(app, true);
        }
    }
}
pub fn watch(app: AppHandle) {
    std::thread::spawn(move || {
        let mut outside = Instant::now();
        let mut fullscreen_hidden = false;
        let mut last_monitor_check = Instant::now();
        let mut monitor_fingerprint = app
            .get_webview_window("main")
            .map(|w| format!("{:?}", w.available_monitors().ok()))
            .unwrap_or_default();
        let mut last_geometry = None;
        let mut geometry_changed = Instant::now();
        let mut last_active = Instant::now();
        let mut fullscreen_checked = Instant::now() - Duration::from_secs(1);
        let mut fullscreen = false;
        loop {
            std::thread::sleep(Duration::from_millis(100));
            let state = app.state::<AppState>();
            let panel: Panel = state.data.lock().unwrap().config.panel.clone();
            let Some(w) = app.get_window("main") else {
                break;
            };
            if fullscreen_checked.elapsed() > Duration::from_millis(500) {
                fullscreen = panel.hide_fullscreen && crate::platform::fullscreen();
                fullscreen_checked = Instant::now();
            }
            if fullscreen {
                if !fullscreen_hidden {
                    let _ = w.hide();
                    fullscreen_hidden = true;
                }
                if last_active.elapsed() > Duration::from_secs(5) {
                    release_ui(&app);
                }
                continue;
            }
            if fullscreen_hidden {
                fullscreen_hidden = false;
                if !state.user_hidden.load(Ordering::SeqCst) {
                    let _ = w.show();
                    if !state.collapsed.load(Ordering::SeqCst) {
                        ensure_ui(&app);
                    }
                }
            }
            if state.user_hidden.load(Ordering::SeqCst) {
                if last_active.elapsed() > Duration::from_secs(5) {
                    release_ui(&app);
                }
                continue;
            }
            if !state.collapsed.load(Ordering::SeqCst) {
                last_active = Instant::now();
            } else if last_active.elapsed() > Duration::from_secs(5) {
                release_ui(&app);
            }
            if last_monitor_check.elapsed() > Duration::from_secs(2) {
                let fingerprint = format!("{:?}", w.available_monitors().ok());
                if fingerprint != monitor_fingerprint {
                    monitor_fingerprint = fingerprint;
                    let _ = place(&app, !state.collapsed.load(Ordering::SeqCst));
                }
                last_monitor_check = Instant::now();
            }
            if panel.mode == "float" {
                if let (Ok(pos), Ok(size), Ok(scale)) =
                    (w.outer_position(), w.inner_size(), w.scale_factor())
                {
                    let geometry = (pos.x, pos.y, size.width, size.height);
                    if last_geometry != Some(geometry) {
                        last_geometry = Some(geometry);
                        geometry_changed = Instant::now();
                    } else if geometry_changed.elapsed() > Duration::from_millis(700) {
                        let width = size.width as f64 / scale;
                        let height = size.height as f64 / scale;
                        if panel.x != Some(pos.x as f64)
                            || panel.y != Some(pos.y as f64)
                            || (panel.width - width).abs() > 1.
                            || (panel.height - height).abs() > 1.
                        {
                            if let Ok(_transaction) = state.transaction.try_lock() {
                                let monitor = w
                                    .current_monitor()
                                    .ok()
                                    .flatten()
                                    .and_then(|m| m.name().cloned());
                                let mut data = state.data.lock().unwrap();
                                let mut next = data.config.clone();
                                next.panel.x = Some(pos.x as f64);
                                next.panel.y = Some(pos.y as f64);
                                next.panel.width = width;
                                next.panel.height = height;
                                next.panel.monitor = monitor;
                                if crate::config::save(&state.dir, &next).is_ok() {
                                    data.config = next;
                                    data.revision += 1;
                                    drop(data);
                                    let _ = app.emit("config-changed", crate::view(&state));
                                }
                            }
                        }
                    }
                }
            }
            if panel.mode != "sidebar" || panel.pinned || state.interacting.load(Ordering::SeqCst) {
                outside = Instant::now();
                continue;
            }
            if let (Ok(cursor), Ok(pos), Ok(size)) =
                (w.cursor_position(), w.outer_position(), w.outer_size())
            {
                let inside = cursor.x >= pos.x as f64
                    && cursor.x < (pos.x + size.width as i32) as f64
                    && cursor.y >= pos.y as f64
                    && cursor.y < (pos.y + size.height as i32) as f64;
                if inside {
                    outside = Instant::now();
                    if state.collapsed.load(Ordering::SeqCst) {
                        let _ = place(&app, true);
                        ensure_ui(&app);
                    }
                } else if outside.elapsed() > Duration::from_millis(800)
                    && !state.collapsed.load(Ordering::SeqCst)
                {
                    let _ = place(&app, false);
                }
            }
        }
    });
}
