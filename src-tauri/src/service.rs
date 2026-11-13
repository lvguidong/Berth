use crate::{
    paths,
    platform::{self, Explorer, FolderWindow},
};
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    sync::{mpsc, Arc, Mutex},
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Default, Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub windows: Vec<FolderWindow>,
    pub warning: Option<String>,
    pub available: HashMap<String, bool>,
}
pub enum Request {
    Open(String, bool),
    Activate(String),
    Merge,
    Capture,
}
#[derive(Default)]
pub struct OpenTracker {
    pending: HashMap<String, Instant>,
}
impl OpenTracker {
    pub fn open(&mut self, explorer: &Explorer, path: &str, force: bool) -> Result<String, String> {
        let key = paths::key(path);
        if !force {
            let rows = explorer.recent();
            if let Ok(rows) = &rows {
                if let Some(w) = rows.iter().find(|w| paths::key(&w.path) == key) {
                    explorer.activate(&w.id)?;
                    return Ok("activated".into());
                }
            }
            #[cfg(windows)]
            if let Err(e) = rows {
                return Err(format!("无法确认已有窗口，未新开窗口：{e}"));
            }
            if self
                .pending
                .get(&key)
                .is_some_and(|t| t.elapsed() < Duration::from_secs(4))
            {
                return Ok("opening".into());
            }
            // A fresh snapshot before opening catches windows created since the last poll.
            if let Ok(rows) = explorer.list() {
                if let Some(w) = rows.iter().find(|w| paths::key(&w.path) == key) {
                    explorer.activate(&w.id)?;
                    return Ok("activated".into());
                }
            }
        }
        platform::open_folder(path)?;
        self.pending.insert(key, Instant::now());
        Ok("opened".into())
    }
}
type Job = (Request, mpsc::SyncSender<Result<String, String>>, Instant);
#[derive(Clone)]
pub struct Service {
    tx: mpsc::SyncSender<Job>,
    pub snapshot: Arc<Mutex<Snapshot>>,
}
impl Service {
    pub fn start(app: AppHandle) -> Self {
        let (tx, rx) = mpsc::sync_channel::<Job>(16);
        let snapshot = Arc::new(Mutex::new(Snapshot::default()));
        let shared = snapshot.clone();
        std::thread::spawn(move || {
            let explorer = match Explorer::new() {
                Ok(e) => e,
                Err(e) => {
                    shared.lock().unwrap().warning = Some(e);
                    return;
                }
            };
            let mut tracker = OpenTracker::default();
            let mut next_poll = Instant::now();
            loop {
                match rx.recv_timeout(next_poll.saturating_duration_since(Instant::now())) {
                    Ok((request, reply, deadline)) => {
                        if Instant::now() > deadline {
                            let _ = reply.send(Err("操作已超时，请重试".into()));
                            continue;
                        }
                        let result = match request {
                            Request::Activate(id) => {
                                explorer.activate(&id).map(|_| "activated".into())
                            }
                            Request::Capture => explorer.foreground_path(),
                            Request::Merge => (|| {
                                let rows = explorer.list()?;
                                let mut seen = HashSet::new();
                                let mut count = 0;
                                for w in rows {
                                    if !seen.insert(paths::key(&w.path)) {
                                        explorer.close(&w.id)?;
                                        count += 1;
                                    }
                                }
                                Ok(format!("已合并 {count} 个重复窗口"))
                            })(),
                            Request::Open(path, force) => tracker.open(&explorer, &path, force),
                        };
                        let _ = reply.send(result);
                        next_poll = Instant::now();
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
                if Instant::now() >= next_poll {
                    let mut next = shared.lock().unwrap().clone();
                    match explorer.list() {
                        Ok(w) => {
                            next.windows = w;
                            next.warning = None;
                        }
                        Err(e) => {
                            next.windows.clear();
                            next.warning = Some(e);
                        }
                    }
                    tracker.pending.retain(|key, t| {
                        t.elapsed() < Duration::from_secs(4)
                            && !next.windows.iter().any(|w| paths::key(&w.path) == *key)
                    });
                    let mut state = shared.lock().unwrap();
                    // Keep the independent file availability scanner's latest values.
                    next.available = state.available.clone();
                    if *state != next {
                        *state = next.clone();
                        let _ = app.emit("snapshot", &next);
                    }
                    next_poll = Instant::now() + Duration::from_millis(500);
                }
            }
        });
        Self { tx, snapshot }
    }
    pub async fn request(&self, request: Request) -> Result<String, String> {
        let (tx, rx) = mpsc::sync_channel(1);
        self.tx
            .try_send((request, tx, Instant::now() + Duration::from_secs(4)))
            .map_err(|_| "窗口服务忙，请稍后重试".to_string())?;
        tauri::async_runtime::spawn_blocking(move || {
            rx.recv_timeout(Duration::from_secs(4))
                .map_err(|_| "窗口操作超时，请检查网络路径或系统权限".to_string())?
        })
        .await
        .map_err(|e| e.to_string())?
    }
}

pub fn availability_scanner(app: AppHandle) {
    // Fixed number of workers: unreachable network volumes cannot create unbounded threads.
    let (tx, rx) = mpsc::sync_channel::<(String, String)>(1000);
    let rx = Arc::new(Mutex::new(rx));
    let in_flight = Arc::new(Mutex::new(HashSet::<String>::new()));
    for _ in 0..4 {
        let app = app.clone();
        let rx = rx.clone();
        let pending = in_flight.clone();
        std::thread::spawn(move || loop {
            let Ok((id, path)) = rx.lock().unwrap().recv() else {
                break;
            };
            let available = std::path::Path::new(&path).is_dir();
            let state = app.state::<crate::AppState>();
            let mut s = state.service.snapshot.lock().unwrap();
            let changed = s.available.insert(id.clone(), available) != Some(available);
            if changed {
                let _ = app.emit("snapshot", &*s);
            }
            pending.lock().unwrap().remove(&id);
        });
    }
    std::thread::spawn(move || loop {
        let state = app.state::<crate::AppState>();
        let pins = state.data.lock().unwrap().config.pins.clone();
        let ids: HashSet<_> = pins.iter().map(|p| p.id.clone()).collect();
        state
            .service
            .snapshot
            .lock()
            .unwrap()
            .available
            .retain(|id, _| ids.contains(id));
        for p in pins {
            let mut pending = in_flight.lock().unwrap();
            if pending.insert(p.id.clone()) && tx.try_send((p.id.clone(), p.path)).is_err() {
                pending.remove(&p.id);
            }
        }
        std::thread::sleep(Duration::from_secs(2));
    });
}
