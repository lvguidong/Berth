use serde::{Deserialize, Serialize};
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;
#[cfg(target_os = "macos")]
pub use macos::*;
#[cfg(windows)]
pub use windows::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FolderWindow {
    pub id: String,
    pub path: String,
    pub title: String,
    pub minimized: bool,
}

// Paths are arguments, never shell commands. Explorer and Finder own all file browsing.
pub fn open_folder(path: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        macos::new_window(path)
    }
    #[cfg(windows)]
    let mut cmd = std::process::Command::new("explorer.exe");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        if path.contains('"') || path.contains('\0') {
            return Err("路径包含无效字符".into());
        }
        // Explorer's legacy parser requires the switch outside the quoted path.
        // Command::arg("/n,<path>") quotes the entire switch when a folder contains spaces.
        cmd.raw_arg(format!("/n,\"{path}\""));
    }
    #[cfg(windows)]
    cmd.spawn()
        .map(|_| ())
        .map_err(|e| format!("无法打开文件夹：{e}"))
}
