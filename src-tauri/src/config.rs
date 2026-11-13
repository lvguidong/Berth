use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Panel {
    pub mode: String,
    pub edge: String,
    pub pinned: bool,
    pub width: f64,
    pub height: f64,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub opacity: f64,
    pub always_on_top: bool,
    pub hide_fullscreen: bool,
    pub monitor: Option<String>,
}
impl Default for Panel {
    fn default() -> Self {
        Self {
            mode: "sidebar".into(),
            edge: "right".into(),
            pinned: false,
            width: 320.,
            height: 680.,
            x: None,
            y: None,
            opacity: 1.,
            always_on_top: true,
            hide_fullscreen: true,
            monitor: None,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Pin {
    pub id: String,
    pub path: String,
    pub alias: String,
    pub color: String,
    pub group_id: Option<String>,
    pub order: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub collapsed: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Hotkeys {
    pub toggle: String,
    pub direct: bool,
    pub capture: String,
}
impl Default for Hotkeys {
    fn default() -> Self {
        Self {
            toggle: "Ctrl+Alt+Space".into(),
            direct: false,
            capture: "Ctrl+Alt+P".into(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub version: u32,
    /// UI language: "zh" (Simplified Chinese) or "en" (English).
    pub language: String,
    pub panel: Panel,
    pub autostart: bool,
    pub hotkeys: Hotkeys,
    pub groups: Vec<Group>,
    pub pins: Vec<Pin>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            language: "zh".into(),
            panel: Panel::default(),
            autostart: false,
            hotkeys: Hotkeys::default(),
            groups: vec![],
            pins: vec![],
        }
    }
}

impl Config {
    pub fn validate(&mut self) -> Result<(), String> {
        if self.version != 1 {
            return Err("不支持此配置版本".into());
        }
        if self.language != "en" {
            self.language = "zh".into();
        }
        if !["sidebar", "float", "popup"].contains(&self.panel.mode.as_str())
            || !["left", "right"].contains(&self.panel.edge.as_str())
        {
            return Err("面板设置无效".into());
        }
        self.panel.width = self.panel.width.clamp(280., 800.);
        self.panel.height = self.panel.height.clamp(400., 1200.);
        self.panel.opacity = self.panel.opacity.clamp(0.5, 1.);
        if self.pins.len() > 1000 || self.groups.len() > 100 {
            return Err("配置项目数量过多".into());
        }
        let mut ids = std::collections::HashSet::new();
        for g in &self.groups {
            if g.id.is_empty() || !ids.insert(&g.id) || g.name.trim().is_empty() {
                return Err("分组无效或重复".into());
            }
        }
        let mut pin_ids = std::collections::HashSet::new();
        let mut paths = std::collections::HashSet::new();
        self.pins.sort_by_key(|p| p.order);
        for (i, p) in self.pins.iter_mut().enumerate() {
            if p.id.is_empty()
                || !pin_ids.insert(p.id.clone())
                || p.path.contains('\0')
                || !Path::new(&p.path).is_absolute()
            {
                return Err("目录必须是绝对路径，且编号不能重复".into());
            }
            if !paths.insert(crate::paths::key(&p.path)) {
                return Err("配置包含重复目录".into());
            }
            if p.alias.trim().is_empty() || p.alias.len() > 512 {
                return Err("别名不能为空或过长".into());
            }
            if p.group_id.as_ref().is_some_and(|id| !ids.contains(id)) {
                p.group_id = None;
            }
            if p.color.len() != 7
                || !p.color.starts_with('#')
                || !p.color[1..].chars().all(|c| c.is_ascii_hexdigit())
            {
                p.color = "#75a99c".into();
            }
            p.order = i;
        }
        Ok(())
    }
}

pub fn directory() -> PathBuf {
    // The override is reserved for isolated development/integration test profiles.
    std::env::var_os("BERTH_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("Berth")
        })
}
pub fn load(dir: &Path) -> Result<(Config, Option<String>), String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let path = dir.join("config.json");
    if !path.exists() {
        return Ok((Config::default(), None));
    }
    let data = fs::read(&path).map_err(|e| format!("无法读取配置：{e}"))?;
    let parsed = serde_json::from_slice::<Config>(&data)
        .map_err(|e| e.to_string())
        .and_then(|mut c| {
            c.validate()?;
            Ok(c)
        });
    match parsed {
        Ok(c) => Ok((c, None)),
        Err(e) => {
            let backup = dir.join(format!("config.corrupt-{}.json", uuid::Uuid::new_v4()));
            fs::rename(&path, &backup).map_err(|err| format!("无法备份损坏配置：{err}"))?;
            Ok((
                Config::default(),
                Some(format!(
                    "配置无法读取（{e}），已保留备份：{}",
                    backup.display()
                )),
            ))
        }
    }
}
pub fn save(dir: &Path, config: &Config) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let tmp = dir.join("config.json.tmp");
    let bytes = serde_json::to_vec_pretty(config).map_err(|e| e.to_string())?;
    let mut file = fs::File::create(&tmp).map_err(|e| e.to_string())?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())?;
    drop(file);
    fs::rename(tmp, dir.join("config.json")).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip_and_replace() {
        let dir = tempfile::tempdir().unwrap();
        let mut c = Config::default();
        save(dir.path(), &c).unwrap();
        c.panel.edge = "left".into();
        save(dir.path(), &c).unwrap();
        assert_eq!(load(dir.path()).unwrap().0, c);
    }
    #[test]
    fn corruption_is_preserved() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("config.json"), b"{ broken").unwrap();
        let (c, warning) = load(dir.path()).unwrap();
        assert!(warning.is_some());
        assert!(c.pins.is_empty());
        assert_eq!(
            fs::read(
                fs::read_dir(dir.path())
                    .unwrap()
                    .next()
                    .unwrap()
                    .unwrap()
                    .path()
            )
            .unwrap(),
            b"{ broken"
        );
    }
    #[test]
    fn future_version_is_not_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("config.json"), br#"{"version":99}"#).unwrap();
        assert!(load(dir.path()).unwrap().1.is_some());
    }
    #[test]
    fn rejects_invalid_modes() {
        let mut c = Config::default();
        c.panel.mode = "bad".into();
        assert!(c.validate().is_err());
    }

    fn pin(path: &Path, id: &str) -> Pin {
        Pin {
            id: id.into(),
            path: path.display().to_string(),
            alias: id.into(),
            color: "#75a99c".into(),
            group_id: None,
            order: 0,
        }
    }
    #[test]
    fn ten_folders_survive_a_restart() {
        let root = tempfile::tempdir().unwrap();
        let data = tempfile::tempdir().unwrap();
        let mut c = Config::default();
        for i in 0..10 {
            let folder = root.path().join(format!("项目 {i}"));
            fs::create_dir(&folder).unwrap();
            let mut p = pin(&folder, &format!("p{i}"));
            p.order = i;
            c.pins.push(p);
        }
        c.validate().unwrap();
        save(data.path(), &c).unwrap();
        assert_eq!(load(data.path()).unwrap().0, c);
    }
    #[test]
    fn rejects_duplicate_paths_and_ids() {
        let root = tempfile::tempdir().unwrap();
        let mut c = Config::default();
        c.pins = vec![pin(root.path(), "a"), pin(root.path(), "b")];
        assert!(c.validate().is_err());
        c.pins[1].path = root.path().join("different").display().to_string();
        c.pins[1].id = "a".into();
        assert!(c.validate().is_err());
    }
    #[test]
    fn repairs_orphan_groups_and_color_without_removing_offline_pins() {
        let root = tempfile::tempdir().unwrap();
        let mut c = Config::default();
        let mut p = pin(&root.path().join("offline"), "a");
        p.color = "url(evil)".into();
        p.group_id = Some("missing".into());
        c.pins.push(p);
        c.validate().unwrap();
        assert_eq!(c.pins[0].group_id, None);
        assert_eq!(c.pins[0].color, "#75a99c");
        assert_eq!(c.pins.len(), 1);
    }
    #[test]
    fn failed_write_preserves_last_valid_config() {
        let dir = tempfile::tempdir().unwrap();
        let c = Config::default();
        save(dir.path(), &c).unwrap();
        fs::create_dir(dir.path().join("config.json.tmp")).unwrap();
        let mut next = c.clone();
        next.panel.edge = "left".into();
        assert!(save(dir.path(), &next).is_err());
        assert_eq!(load(dir.path()).unwrap().0, c);
    }
}
