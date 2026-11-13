/**
 * Lightweight UI localization for Berth.
 *
 * The source of truth is `config.language` (persisted in config.json by the
 * Rust backend). A localStorage copy is used for the very first paint, before
 * `get_state` resolves. Changing the language in Settings saves the config and
 * reloads the WebView, so every label is rendered from scratch.
 */
export type Lang = "zh" | "en";

const STORAGE_KEY = "berth.language";

function initial(): Lang {
  try {
    return localStorage.getItem(STORAGE_KEY) === "en" ? "en" : "zh";
  } catch {
    return "zh";
  }
}

let current: Lang = initial();

export function language(): Lang {
  return current;
}

export function setLanguage(next: string): void {
  current = next === "en" ? "en" : "zh";
  try {
    localStorage.setItem(STORAGE_KEY, current);
  } catch {
    /* storage can be unavailable in restricted environments */
  }
}

/** Pick a string at render time. */
export function t(zh: string, en: string): string {
  return current === "en" ? en : zh;
}

/** Chinese source text -> English UI text. */
const DICT: Record<string, string> = {
  // chrome / header
  "桌面界面预览 · 不执行系统操作": "Desktop preview · no system actions",
  泊岸: "Berth",
  固定展开: "Keep open",
  设置: "Settings",
  搜索文件夹: "Search folders",
  "找一个文件夹…": "Find a folder…",
  文件夹列表: "Folder list",
  文件夹: "Folders",
  已固定: "Pinned",
  已打开: "Open",
  固定文件夹: "Pin folder",
  "每个目录，都有停靠的位置": "Every folder has a place to dock",
  撤销: "Undo",
  "撤销 Ctrl+Z": "Undo Ctrl+Z",
  "松开，让文件夹停靠在这里": "Release to dock the folders here",
  // pin rows / windows
  "路径不可用，点击可重新定位": "Path unavailable, click to relocate",
  "，路径不可用": ", path unavailable",
  " · 路径不可用，点击可重新定位": " · Path unavailable, click to relocate",
  "路径不可用 · 点击重新定位": "Path unavailable · click to relocate",
  查看全部窗口: "Show all windows",
  全部窗口: "All windows",
  合并重复窗口: "Merge duplicate windows",
  "已最小化 · ": "Minimized · ",
  固定此目录: "Pin this folder",
  没有匹配的窗口: "No matching windows",
  此刻很清静: "All quiet for now",
  "此刻，桌面很清静": "All quiet on the desktop",
  打开的文件夹会出现在这里: "Open folders will appear here",
  "随时切换，也可以一键固定": "Switch anytime, or pin with one click",
  把文件夹拖到这里: "Drag folders here",
  为常用目录留一个位置: "Save a spot for your go-to folders",
  "下次，一点就到": "One click next time",
  也可以从下方添加: "Or add one below",
  新建分组: "New group",
  常用目录: "Frequent",
  没有找到这个文件夹: "No folder found",
  "试试目录名、别名或路径中的关键词": "Try a name, alias or path keyword",
  "整理分组": "Edit group",
  关闭: "Close",
  // dialogs
  留一个位置: "Save a spot",
  "固定常用文件夹。文件仍留在原处，泊岸只记住它的位置。":
    "Pin frequently used folders. Files stay where they are; Berth only remembers the location.",
  文件夹路径: "Folder path",
  输入完整路径: "Enter the full path",
  "浏览文件夹…": "Browse folders…",
  "别名 · 可选": "Alias · optional",
  给它一个容易记住的名字: "Give it a memorable name",
  取消: "Cancel",
  "已固定，随时一点就到": "Pinned — one click away",
  整理文件夹: "Edit folder",
  别名: "Alias",
  "文件夹路径 · 可重新定位": "Folder path · can be relocated",
  "浏览…": "Browse…",
  分组: "Group",
  颜色标签: "Color tag",
  上移: "Move up",
  下移: "Move down",
  复制路径: "Copy path",
  取消固定: "Unpin",
  保存: "Save",
  已复制路径: "Path copied",
  "复制失败，请手动选择路径复制": "Copy failed; select and copy the path manually",
  "已取消固定 · 可按 Ctrl+Z 撤销": "Unpinned · Ctrl+Z to undo",
  请输入别名: "Please enter an alias",
  分组名称: "Group name",
  删除分组: "Delete group",
  请输入分组名称: "Please enter a group name",
  让泊岸适合你: "Make Berth yours",
  // settings
  面板: "Panel",
  界面语言: "Language",
  简体中文: "Simplified Chinese",
  显示方式: "Display mode",
  贴边侧栏: "Edge sidebar",
  悬浮小窗: "Floating window",
  热键弹出: "Hotkey popup",
  显示器: "Display",
  跟随当前屏幕: "Follow current display",
  侧栏位置: "Sidebar edge",
  屏幕左侧: "Left side",
  屏幕右侧: "Right side",
  保持置顶: "Always on top",
  全屏时自动隐藏: "Hide in fullscreen",
  面板透明度: "Panel opacity",
  启动与快捷键: "Startup & shortcuts",
  登录电脑时启动: "Launch at login",
  "显示 / 隐藏面板 · 留空关闭": "Show / hide panel · leave empty to disable",
  "固定当前文件夹 · 留空关闭": "Pin current folder · leave empty to disable",
  "前 9 项直达 · Ctrl+Alt+1~9": "First 9 items · Ctrl+Alt+1~9",
  保存设置: "Save settings",
  "你的数据，只留在这里": "Your data stays here",
  导入配置: "Import config",
  导出配置: "Export config",
  "泊岸 0.1.0 · 离线运行，不收集使用数据。":
    "Berth 0.1.0 · Offline, no usage data collected.",
  "关闭面板后，可从托盘或快捷键再次呼出。":
    "After hiding, reopen it from the tray or shortcut.",
  收起到托盘: "Hide to tray",
  "请在桌面应用中导出配置": "Export is only available in the desktop app",
  配置已导出: "Config exported",
  "导入将替换当前固定项和设置，也会应用自启与快捷键。可以通过撤销恢复当前配置。":
    "Import replaces pins and settings, including startup and shortcuts. Undo restores the current config.",
  选择配置并导入: "Choose a config file",
  "请在桌面应用中导入配置": "Import is only available in the desktop app",
  "路径不可用，请重新定位或取消固定。":
    "Path unavailable. Relocate it or unpin the folder.",
  正在打开: "Opening",
  "正在打开，请稍候": "Opening, please wait",
  "同一目录保留最近使用的窗口，关闭其他重复窗口。不会修改文件；含多个标签页的窗口不会自动关闭。":
    "Keep the most recently used window per folder and close duplicates. No files are changed; windows with multiple tabs are never closed automatically.",
  "启动失败，请从托盘退出后重新打开。":
    "Startup failed. Quit from the tray and reopen.",
  // browser preview
  "网页预览：数据仅保存在本浏览器":
    "Browser preview: data is stored in this browser only",
  "请在桌面应用中使用此功能":
    "This feature is only available in the desktop app",
  请输入完整的文件夹路径: "Please enter a full folder path",
  "这是网页预览，请在桌面应用中打开和切换文件夹":
    "This is a browser preview; open and switch folders in the desktop app",
  这个文件夹已经固定了: "This folder is already pinned",
  "网页预览中请手动输入路径":
    "In the browser preview, enter a path manually",
  选择要固定的文件夹: "Choose a folder to pin",
  // backend messages surfaced in toast / warnings
  "配置刚刚发生变化，请重试": "Config changed, please retry",
  没有可撤销的操作: "Nothing to undo",
  固定项已移除: "Pin removed",
  配置文件过大: "Config file is too large",
  "正在检查其他路径，请稍后再试": "Checking other paths, please try again",
  "路径检查超时，网络目录可能离线":
    "Path check timed out; network folders may be offline",
  "路径不可用：文件夹不存在、已离线或无访问权限":
    "Path unavailable: folder missing, offline, or inaccessible",
  "路径包含无效字符": "Path contains invalid characters",
  无法确定用户目录: "Cannot determine the home directory",
  "窗口服务忙，请稍后重试": "Window service is busy, please retry",
  "操作已超时，请重试": "Operation timed out, please retry",
  "窗口操作超时，请检查网络路径或系统权限":
    "Window operation timed out; check network paths or permissions",
  "访达响应超时，请检查自动化权限或网络卷连接":
    "Finder timed out; check Automation permission or network volumes",
  "无法确认已有窗口，未新开窗口：": "Could not confirm existing windows; no new window opened: ",
  面板窗口不存在: "Panel window not found",
  "不支持此配置版本": "Unsupported config version",
  面板设置无效: "Invalid panel settings",
  配置项目数量过多: "Too many config items",
  分组无效或重复: "Invalid or duplicate group",
  "目录必须是绝对路径，且编号不能重复":
    "Folders must use absolute paths with unique IDs",
  配置包含重复目录: "Config contains duplicate folders",
  "别名不能为空或过长": "Alias cannot be empty or too long",
  "窗口已关闭，请重试": "Window closed, please retry",
  "窗口目录已变化，请重试": "Window folder changed, please retry",
  窗口已关闭: "Window closed",
  "包含多个标签页的窗口不会自动关闭":
    "Windows with multiple tabs are not closed automatically",
  "请先切换到一个资源管理器文件夹窗口":
    "Switch to a File Explorer folder window first",
  "该窗口包含多个资源管理器标签页；请在窗口中选择对应标签，或按 Shift 点击新开独立窗口":
    "This window has multiple Explorer tabs; select the tab in the window, or Shift-click to open a separate window",
  "系统限制了窗口置前，已闪烁提示；请点击任务栏中的资源管理器":
    "The system prevented focusing the window; click Explorer in the taskbar",
  // partial phrases, used by tr() to translate composite backend errors
  "无法打开文件夹：": "Cannot open folder: ",
  "呼出快捷键不可用：": "Toggle shortcut unavailable: ",
  "固定当前目录的快捷键不可用：": "Capture shortcut unavailable: ",
  "目录直达快捷键被占用：": "Direct shortcuts unavailable: ",
  "无法更改开机自启：": "Cannot change launch-at-login: ",
  "保存失败，修改未生效：": "Save failed, changes not applied: ",
  "无法读取开机自启状态：": "Cannot read launch-at-login state: ",
  "配置格式错误：": "Invalid config: ",
  "无法读取配置：": "Cannot read config: ",
  "配置无法读取（": "Cannot read config (",
  "），已保留备份：": "), backup kept: ",
  "面板恢复失败：": "Failed to restore panel: ",
  "无法备份损坏配置：": "Cannot back up corrupt config: ",
  "无法读取访达窗口。请在 系统设置 → 隐私与安全性 → 自动化 中允许泊岸控制访达。":
    "Cannot read Finder windows. Allow Berth to control Finder in System Settings → Privacy & Security → Automation.",
  发生权限违例: "permission violation",
  权限不足: "insufficient permission",
  // Backend errors are concatenated without a separator after a full stop.
  ".execution": ". execution",
};

/** Longest phrases first, so a short word never breaks up a longer message. */
const PHRASES: [string, string][] = Object.entries(DICT).sort(
  (a, b) => b[0].length - a[0].length,
);

/** Regex rules for strings that contain dynamic values (names, counts, paths). */
const PATTERNS: [RegExp, string][] = [
  [/^已固定 (\d+) 个文件夹$/, "Pinned $1 folders"],
  [/^已合并 (\d+) 个重复窗口$/, "Merged $1 duplicate windows"],
  [/^项目 (\d+)$/, "Item $1"],
  [/^编辑分组 ([\s\S]*)$/, "Edit group $1"],
  [/^编辑 ([\s\S]*)$/, "Edit $1"],
  [/^拖动 ([\s\S]*)$/, "Drag $1"],
  [/^固定 ([\s\S]*)$/, "Pin $1"],
  [/^已最小化 · /, "Minimized · "],
  [/ · 路径不可用，点击可重新定位$/, " · Path unavailable, click to relocate"],
  [/，路径不可用$/, ", path unavailable"],
];

/**
 * Translate a whole string. `deep` also replaces known phrases inside the
 * string, which is needed for backend errors made of a label plus a detail.
 */
export function tr(text: string, deep = true): string {
  if (current !== "en" || !text) return text;
  const exact = DICT[text];
  if (exact !== undefined) return exact;
  for (const [pattern, replacement] of PATTERNS) {
    if (pattern.test(text)) return text.replace(pattern, replacement);
  }
  if (!deep) return text;
  let out = text;
  for (const [zh, en] of PHRASES) {
    if (out.includes(zh)) out = out.split(zh).join(en);
  }
  return out;
}

const ATTRS = ["placeholder", "title", "aria-label"];

/**
 * Translate a freshly rendered DOM subtree. Text nodes only use exact matches
 * and patterns, so user-provided folder names and paths are left untouched.
 */
export function localize(root: ParentNode = document): void {
  if (current !== "en") return;
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  const texts: Text[] = [];
  while (walker.nextNode()) texts.push(walker.currentNode as Text);
  for (const node of texts) {
    const value = node.nodeValue || "";
    const trimmed = value.trim();
    const exact = DICT[trimmed];
    if (exact !== undefined) {
      node.nodeValue = value.replace(trimmed, exact);
      continue;
    }
    const translated = tr(value, false);
    if (translated !== value) node.nodeValue = translated;
  }
  const elements =
    root instanceof Element
      ? [root, ...Array.from(root.querySelectorAll<Element>("*"))]
      : Array.from(root.querySelectorAll<Element>("*"));
  for (const el of elements) {
    for (const attr of ATTRS) {
      const value = el.getAttribute(attr);
      if (value) {
        const translated = tr(value);
        if (translated !== value) el.setAttribute(attr, translated);
      }
    }
  }
}
