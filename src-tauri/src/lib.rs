mod config;
mod panel;
mod paths;
mod platform;
mod service;

use config::{Config, Hotkeys};
use serde::Serialize;
use service::{Request, Service, Snapshot};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex, OnceLock,
    },
    time::Duration,
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State,
};
use tauri_plugin_autostart::ManagerExt as AutostartExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub struct Data {
    pub config: Config,
    revision: u64,
    undo: Vec<Config>,
    warning: Option<String>,
}
/// Native tray menu items, kept so their labels can follow the UI language.
struct TrayItems {
    show: MenuItem<tauri::Wry>,
    settings: MenuItem<tauri::Wry>,
    quit: MenuItem<tauri::Wry>,
}
static TRAY_ITEMS: OnceLock<TrayItems> = OnceLock::new();

fn tray_labels(language: &str) -> (&'static str, &'static str, &'static str, &'static str) {
    if language == "en" {
        ("Show / Hide Berth", "Settings", "Quit Berth", "Berth")
    } else {
        ("显示 / 隐藏泊岸", "设置", "退出泊岸", "泊岸 · Berth")
    }
}

fn apply_tray_language(app: &AppHandle, language: &str) {
    let (show, settings, quit, tooltip) = tray_labels(language);
    if let Some(items) = TRAY_ITEMS.get() {
        let _ = items.show.set_text(show);
        let _ = items.settings.set_text(settings);
        let _ = items.quit.set_text(quit);
    }
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_tooltip(Some(tooltip));
    }
}

pub struct AppState {
    pub data: Mutex<Data>,
    pub service: Service,
    dir: PathBuf,
    transaction: tokio::sync::Mutex<()>,
    path_slots: Arc<tokio::sync::Semaphore>,
    pub collapsed: AtomicBool,
    pub interacting: AtomicBool,
    pub user_hidden: AtomicBool,
    pub rendering: AtomicBool,
    intents: Mutex<Vec<Intent>>,
    ui_mounts: AtomicUsize,
}
#[derive(Clone, Serialize)]
struct Intent {
    kind: String,
    value: String,
}
fn queue_intent(app: &AppHandle, kind: &str, value: String) {
    let state = app.state::<AppState>();
    let mut intents = state.intents.lock().unwrap();
    if intents.len() >= 32 {
        intents.remove(0);
    }
    intents.push(Intent {
        kind: kind.into(),
        value,
    });
    drop(intents);
    let _ = app.emit("intent", ());
}
#[tauri::command]
fn take_intents(state: State<AppState>) -> Vec<Intent> {
    std::mem::take(&mut *state.intents.lock().unwrap())
}
#[tauri::command]
fn ui_ready(state: State<AppState>) {
    state.ui_mounts.fetch_add(1, Ordering::SeqCst);
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct View {
    config: Config,
    revision: u64,
    can_undo: bool,
    warning: Option<String>,
    snapshot: Snapshot,
    config_path: String,
    collapsed: bool,
}
fn view(state: &AppState) -> View {
    let d = state.data.lock().unwrap();
    View {
        config: d.config.clone(),
        revision: d.revision,
        can_undo: !d.undo.is_empty(),
        warning: d.warning.clone(),
        snapshot: state.service.snapshot.lock().unwrap().clone(),
        config_path: state.dir.join("config.json").display().to_string(),
        collapsed: state.collapsed.load(Ordering::SeqCst),
    }
}
#[tauri::command]
fn get_state(state: State<AppState>) -> View {
    view(&state)
}
#[tauri::command]
fn list_monitors(app: AppHandle) -> Result<Vec<String>, String> {
    app.available_monitors()
        .map(|ms| ms.into_iter().filter_map(|m| m.name().cloned()).collect())
        .map_err(|e| e.to_string())
}

async fn resolve_path(state: &AppState, path: String) -> Result<String, String> {
    let permit = state
        .path_slots
        .clone()
        .try_acquire_owned()
        .map_err(|_| "正在检查其他路径，请稍后再试".to_string())?;
    let task = tauri::async_runtime::spawn_blocking(move || {
        let _permit = permit;
        paths::resolve(&path)
    });
    tokio::time::timeout(Duration::from_secs(3), task)
        .await
        .map_err(|_| "路径检查超时，网络目录可能离线".to_string())?
        .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn validate_path(state: State<'_, AppState>, path: String) -> Result<String, String> {
    resolve_path(&state, path).await
}

fn bind_hotkeys(app: &AppHandle, keys: &Hotkeys) -> Result<(), String> {
    let global = app.global_shortcut();
    global.unregister_all().map_err(|e| e.to_string())?;
    if !keys.toggle.trim().is_empty() {
        global
            .on_shortcut(keys.toggle.as_str(), |app, _, event| {
                if event.state == ShortcutState::Pressed {
                    panel::toggle(app);
                }
            })
            .map_err(|e| format!("呼出快捷键不可用：{e}"))?;
    }
    if !keys.capture.trim().is_empty() {
        global
            .on_shortcut(keys.capture.as_str(), |app, _, event| {
                if event.state != ShortcutState::Pressed {
                    return;
                }
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    let state = app.state::<AppState>();
                    match state.service.request(Request::Capture).await {
                        Ok(path) => {
                            panel::reveal(&app, true);
                            queue_intent(&app, "capture", path);
                        }
                        Err(e) => {
                            panel::reveal(&app, true);
                            queue_intent(&app, "notice", e);
                        }
                    }
                });
            })
            .map_err(|e| format!("固定当前目录的快捷键不可用：{e}"))?;
    }
    if keys.direct {
        for index in 0..9 {
            global
                .on_shortcut(
                    format!("Ctrl+Alt+{}", index + 1).as_str(),
                    move |app, _, event| {
                        if event.state != ShortcutState::Pressed {
                            return;
                        }
                        let app = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let state = app.state::<AppState>();
                            let pin = state.data.lock().unwrap().config.pins.get(index).cloned();
                            if let Some(pin) = pin {
                                if let Err(e) = open_path(&app, &state, pin.path, false).await {
                                    panel::reveal(&app, true);
                                    queue_intent(&app, "notice", e);
                                }
                            }
                        });
                    },
                )
                .map_err(|e| format!("目录直达快捷键被占用：{e}"))?;
        }
    }
    Ok(())
}
fn commit(app: &AppHandle, state: &AppState, next: Config, remember: bool) -> Result<View, String> {
    let old = state.data.lock().unwrap().config.clone();
    if old.hotkeys != next.hotkeys {
        if let Err(e) = bind_hotkeys(app, &next.hotkeys) {
            let _ = bind_hotkeys(app, &old.hotkeys);
            return Err(e);
        }
    }
    if old.autostart != next.autostart {
        let result = if next.autostart {
            app.autolaunch().enable()
        } else {
            app.autolaunch().disable()
        };
        if let Err(e) = result {
            let _ = bind_hotkeys(app, &old.hotkeys);
            return Err(format!("无法更改开机自启：{e}"));
        }
    }
    if let Err(e) = config::save(&state.dir, &next) {
        let _ = bind_hotkeys(app, &old.hotkeys);
        if old.autostart != next.autostart {
            let _ = if old.autostart {
                app.autolaunch().enable()
            } else {
                app.autolaunch().disable()
            };
        }
        return Err(format!("保存失败，修改未生效：{e}"));
    }
    let changed_panel = old.panel != next.panel;
    let language_changed = old.language != next.language;
    let next_language = next.language.clone();
    {
        let mut d = state.data.lock().unwrap();
        if remember {
            d.undo.push(old);
            if d.undo.len() > 50 {
                d.undo.remove(0);
            }
        }
        d.config = next;
        d.revision += 1;
    }
    if changed_panel {
        panel::reveal(app, false);
    }
    if language_changed {
        apply_tray_language(app, &next_language);
    }
    let result = view(state);
    let _ = app.emit("config-changed", &result);
    Ok(result)
}
#[tauri::command]
async fn save_config(
    app: AppHandle,
    state: State<'_, AppState>,
    mut config: Config,
    revision: u64,
) -> Result<View, String> {
    let _guard = state.transaction.lock().await;
    let old = {
        let d = state.data.lock().unwrap();
        if d.revision != revision {
            return Err("配置刚刚发生变化，请重试".into());
        }
        d.config.clone()
    };
    for p in &mut config.pins {
        if !old.pins.iter().any(|o| o.id == p.id && o.path == p.path) {
            p.path = resolve_path(&state, p.path.clone()).await?;
        }
    }
    config.validate()?;
    commit(&app, &state, config, true)
}
#[tauri::command]
async fn undo(app: AppHandle, state: State<'_, AppState>) -> Result<View, String> {
    let _guard = state.transaction.lock().await;
    let previous = state
        .data
        .lock()
        .unwrap()
        .undo
        .last()
        .cloned()
        .ok_or("没有可撤销的操作")?;
    commit(&app, &state, previous, false)?;
    state.data.lock().unwrap().undo.pop();
    let result = view(&state);
    let _ = app.emit("config-changed", &result);
    Ok(result)
}
async fn open_path(
    app: &AppHandle,
    state: &AppState,
    path: String,
    force: bool,
) -> Result<String, String> {
    // An existing Explorer window proves the target is usable; avoid a slow disk roundtrip on the fast switch path.
    let known = state
        .service
        .snapshot
        .lock()
        .unwrap()
        .windows
        .iter()
        .any(|w| paths::key(&w.path) == paths::key(&path));
    let path = if known && !force {
        path
    } else {
        resolve_path(state, path).await?
    };
    let result = state.service.request(Request::Open(path, force)).await?;
    if state.data.lock().unwrap().config.panel.mode == "popup" {
        if let Some(w) = app.get_window("main") {
            state.user_hidden.store(true, Ordering::SeqCst);
            let _ = w.hide();
        }
    }
    Ok(result)
}
#[tauri::command]
async fn open_pin(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    force: bool,
) -> Result<String, String> {
    let path = state
        .data
        .lock()
        .unwrap()
        .config
        .pins
        .iter()
        .find(|p| p.id == id)
        .map(|p| p.path.clone())
        .ok_or("固定项已移除")?;
    open_path(&app, &state, path, force).await
}
#[tauri::command]
async fn activate_window(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<String, String> {
    let result = state.service.request(Request::Activate(id)).await?;
    if state.data.lock().unwrap().config.panel.mode == "popup" {
        state.user_hidden.store(true, Ordering::SeqCst);
        if let Some(w) = app.get_window("main") {
            let _ = w.hide();
        }
    }
    Ok(result)
}
#[tauri::command]
async fn merge_windows(state: State<'_, AppState>) -> Result<String, String> {
    state.service.request(Request::Merge).await
}
#[tauri::command]
fn set_interacting(state: State<AppState>, value: bool) {
    state.interacting.store(value, Ordering::SeqCst);
}
#[tauri::command]
fn hide_panel(app: AppHandle, state: State<AppState>) {
    state.user_hidden.store(true, Ordering::SeqCst);
    if let Some(w) = app.get_window("main") {
        let _ = w.hide();
    }
}
#[tauri::command]
fn export_config(state: State<AppState>, path: String) -> Result<(), String> {
    let c = state.data.lock().unwrap().config.clone();
    std::fs::write(
        path,
        serde_json::to_vec_pretty(&c).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
#[tauri::command]
async fn import_config(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<View, String> {
    let _guard = state.transaction.lock().await;
    let bytes = tauri::async_runtime::spawn_blocking(move || {
        let meta = std::fs::metadata(&path).map_err(|e| e.to_string())?;
        if meta.len() > 2_000_000 {
            return Err("配置文件过大".into());
        }
        std::fs::read(path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;
    let mut next: Config =
        serde_json::from_slice(&bytes).map_err(|e| format!("配置格式错误：{e}"))?;
    next.validate()?;
    // Import retains offline folders; paths are checked when opened, never silently dropped.
    commit(&app, &state, next, true)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            panel::reveal(app, true)
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            get_state,
            list_monitors,
            validate_path,
            save_config,
            undo,
            open_pin,
            activate_window,
            merge_windows,
            set_interacting,
            hide_panel,
            export_config,
            import_config,
            take_intents,
            ui_ready
        ])
        .setup(|app| {
            let dir = config::directory();
            let (mut config, mut warning) = config::load(&dir).map_err(std::io::Error::other)?;
            match app.autolaunch().is_enabled() {
                Ok(enabled) => config.autostart = enabled,
                Err(e) => warning = Some(format!("无法读取开机自启状态：{e}")),
            }
            let service = Service::start(app.handle().clone());
            app.manage(AppState {
                data: Mutex::new(Data {
                    config: config.clone(),
                    revision: 0,
                    undo: vec![],
                    warning,
                }),
                service,
                dir,
                transaction: tokio::sync::Mutex::new(()),
                path_slots: Arc::new(tokio::sync::Semaphore::new(4)),
                collapsed: AtomicBool::new(false),
                interacting: AtomicBool::new(false),
                user_hidden: AtomicBool::new(false),
                rendering: AtomicBool::new(false),
                intents: Mutex::new(vec![]),
                ui_mounts: AtomicUsize::new(0),
            });
            if let Err(e) = bind_hotkeys(app.handle(), &config.hotkeys) {
                app.state::<AppState>().data.lock().unwrap().warning = Some(e);
            }
            let (show_label, settings_label, quit_label, tooltip) = tray_labels(&config.language);
            let show = MenuItem::with_id(app, "show", show_label, true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", settings_label, true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", quit_label, true, None::<&str>)?;
            let _ = TRAY_ITEMS.set(TrayItems {
                show: show.clone(),
                settings: settings.clone(),
                quit: quit.clone(),
            });
            let menu = Menu::with_items(app, &[&show, &settings, &quit])?;
            TrayIconBuilder::with_id("main")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip(tooltip)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => panel::toggle(app),
                    "settings" => {
                        panel::reveal(app, true);
                        queue_intent(app, "settings", String::new());
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        panel::toggle(tray.app_handle());
                    }
                })
                .build(app)?;
            panel::reveal(app.handle(), false);
            panel::watch(app.handle().clone());
            service::availability_scanner(app.handle().clone());
            #[cfg(debug_assertions)]
            lifecycle_smoke(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                window
                    .state::<AppState>()
                    .user_hidden
                    .store(true, Ordering::SeqCst);
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("泊岸启动失败");
}

#[cfg(debug_assertions)]
fn lifecycle_smoke(app: AppHandle) {
    if std::env::var_os("BERTH_LIFECYCLE_TEST").is_none() {
        return;
    }
    std::thread::spawn(move || {
        let result = (|| -> Result<serde_json::Value, String> {
            let state = app.state::<AppState>();
            let started = std::time::Instant::now();
            while state.ui_mounts.load(Ordering::SeqCst) < 1 {
                if started.elapsed() > Duration::from_secs(15) {
                    return Err("Initial UI did not mount".into());
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            let pins = state.data.lock().unwrap().config.pins.clone();
            let initial_ms = started.elapsed().as_millis();
            state.data.lock().unwrap().config.panel.pinned = true;
            std::thread::sleep(Duration::from_secs(3));
            panel::place(&app, false)?;
            std::thread::sleep(Duration::from_secs(18));
            if app.get_webview("main").is_some() {
                return Err("Collapsed UI was not released".into());
            }
            if app.get_window("main").is_none() {
                return Err("Native edge was lost".into());
            }
            let restore = std::time::Instant::now();
            panel::reveal(&app, false);
            queue_intent(&app, "settings", String::new());
            while state.ui_mounts.load(Ordering::SeqCst) < 2
                || !state.interacting.load(Ordering::SeqCst)
            {
                if restore.elapsed() > Duration::from_secs(15) {
                    return Err("Restored UI or pending settings action did not mount".into());
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            let restored_ms = restore.elapsed().as_millis();
            if state.data.lock().unwrap().config.pins != pins {
                return Err("Pinned folders changed during UI lifecycle".into());
            }
            std::thread::sleep(Duration::from_secs(8));
            Ok(
                serde_json::json!({"initialReadyMs":initial_ms,"restoredReadyMs":restored_ms,"uiMounts":state.ui_mounts.load(Ordering::SeqCst),"pinsPreserved":true,"pendingSettingsDelivered":true,"nativeEdgePreserved":true,"collapsedWebviewReleased":true}),
            )
        })();
        let (code, value) = match result {
            Ok(value) => (0, value),
            Err(error) => (1, serde_json::json!({"error":error})),
        };
        let path = app.state::<AppState>().dir.join("lifecycle-result.json");
        let _ = std::fs::write(path, serde_json::to_vec_pretty(&value).unwrap());
        app.exit(code);
    });
}
