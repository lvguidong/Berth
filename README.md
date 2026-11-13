# 泊岸 · Berth

每个目录，都有停靠的位置。

泊岸是一个离线运行的轻量桌面文件夹侧栏。点击固定目录，有窗口就切换，没有就打开。使用 Tauri 2、Rust、TypeScript 与系统 WebView，面向 Windows 10/11 和 macOS 12+。

[English](README.en.md) · [需求文档](泊岸（Berth）需求文档.md) · [验证说明](docs/TESTING.md) · [实现与限制](docs/IMPLEMENTATION.md)

<img src="docs/images/preview-light.png" width="280" alt="泊岸亮色界面预览"> <img src="docs/images/preview-dark.png" width="280" alt="泊岸暗色界面预览">

[界面操作演示（WebM）](docs/images/demo.webm) — 图像与视频均为显式标注的网页预览；系统窗口行为由原生集成测试验证。

## 使用

1. Windows：运行安装包，或解压免安装包后运行 `berth.exe`。需要系统 WebView2 Runtime；安装版会在缺失时引导安装。
2. macOS：在对应架构的 DMG 中把 Berth 拖到 Applications。首次切换访达窗口时，在系统的“隐私与安全性 → 自动化”中允许 Berth 控制 Finder。
3. 把文件夹拖入侧栏，或点击“固定文件夹”，输入路径或浏览选择。
4. 点击目录切换；Shift+点击或中键强制新开。圆点表示已打开，数字表示多个窗口。

默认在屏幕右侧，鼠标离开 800ms 后收成细线，靠近即展开。点击顶部图钉可固定展开。托盘图标左键显示/隐藏，右键可打开设置或退出。

侧栏收起或隐藏 5 秒后会释放 WebView，以降低空闲常驻内存；原生边缘和后台配置仍然保留。本机发布版收起状态测得约 38.7MiB（包含所有子进程）。再次展开需重载界面，空闲复测约 1 秒，速度取决于 WebView2/WebKit 与机器负载。固定展开时不会释放界面，仍占用数百 MiB。

当前为 **0.1.0 预览版**。Windows 已在本地编译和验证，macOS 适配与 Intel/Apple Silicon 构建流程已提供，但尚未经 Mac 实机验收，也尚未上传 GitHub Releases。未签名安装包可能显示系统发布者提示。

## 已有功能

- 只固定文件夹：拖入、手动输入、系统选择器、别名、颜色、分组与拖动排序。
- 路径失效提示与重新定位，撤销最近 50 次整理操作（本次运行期间）。
- 窗口列表、已开状态、最小化还原、重复窗口合并（操作前确认）。
- 贴边、悬浮、热键弹出三种形态；显示器选择、工作区约束与悬浮位置保存。
- 搜索、可设置的全局热键、固定当前资源管理器目录、前 9 项直达。
- 托盘、可选开机自启、单实例、有限次数崩溃恢复。
- 原子保存 JSON、损坏配置备份、导入导出、系统明暗主题。

默认快捷键：

| 操作 | 快捷键 |
| --- | --- |
| 呼出 / 隐藏 | Ctrl+Alt+Space |
| 固定当前资源管理器 / Finder 目录 | Ctrl+Alt+P |
| 前 9 个固定项（需在设置中启用） | Ctrl+Alt+1～9 |
| 面板内搜索 | / |
| 撤销整理（输入框外） | Ctrl+Z / Cmd+Z |

快捷键冲突会提示，不会静默保存失败的设置。悬浮窗口可从标题栏拖动、边缘调整大小；右键固定项可编辑、复制路径或取消固定。拖动每行左侧的手柄，可排序或放到分组标题上。

## 数据与隐私

应用不联网、不上传路径、没有账号、遥测或自动更新。所有窗口操作限制在资源管理器或 Finder。只保存配置，不修改固定目录内的文件。构建工具安装依赖和 Windows 安装程序获取 WebView2 属于安装阶段的联网行为。

- Windows：`%APPDATA%\Berth\config.json`
- macOS：`~/Library/Application Support/Berth/config.json`
- 损坏配置会改名保留为 `config.corrupt-<UUID>.json`；不会直接覆盖损坏内容。
- 导入配置会替换现有固定项和设置，包括自启和热键；可以撤销。
- 开机自启默认关闭，由用户在设置中开启。

测试可设置 `BERTH_CONFIG_DIR` 使用独立配置目录，避免覆盖日常配置。状态和撤销历史不写入配置。

## 从源码运行

需要 Node.js 22.12+、npm、Rust 1.90+。Windows 需 Visual Studio C++ Build Tools、Windows SDK 与 WebView2；macOS 需 Xcode Command Line Tools。

```sh
npm ci
npm run desktop
```

仅查看界面：`npm run dev`。网页模式显式显示“桌面界面预览”，不会操作系统窗口，数据仅存于浏览器。

```sh
npm run build
npm test
npx playwright install chromium
npm run test:e2e
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

Windows 原生集成测试会创建临时目录并打开、最小化、还原和关闭**它自己创建的窗口**：

```sh
cargo test --manifest-path src-tauri/Cargo.toml explorer_lifecycle -- --ignored --nocapture --test-threads=1
```

构建 Windows 安装包：`npm run bundle -- --bundles nsis`。产物位于 `src-tauri/target/release/bundle/nsis/`；可执行文件为 `src-tauri/target/release/berth.exe`。macOS 使用 `npm run bundle -- --target aarch64-apple-darwin` 或 `--target x86_64-apple-darwin`。

推送 `v*` 标签时，GitHub Actions 构建 Windows NSIS/免安装 ZIP 以及两个 Mac 架构的 DMG，上传到**草稿预发布**供检查；不会直接公开正式发布。

## 已知边界

- Windows 11 多标签页：能够枚举的路径会列出；如果同一顶层窗口有多个路径，当前版本提示手动选择标签或 Shift+点击新开，不声称已准确切换到该标签。不会自动合并这类窗口。
- 不同虚拟桌面的切换依赖系统前台窗口策略；权限隔离或前台限制可能使置前失败，此时给出提示并闪烁任务栏。
- 网络目录使用有界后台检查与 3 秒超时；系统文件调用本身无法取消。极端网络故障可能占用全部 4 个路径检查槽位，恢复网络后自动恢复。
- Windows 状态轮询每 500ms；路径可用性每 2 秒检查。macOS 的 Apple Events 速度与授权会影响状态延迟；拒绝授权时保留打开目录能力，但不能复用/标记已有窗口，强制新开也无法保证。
- 10MB 安装包、50MB 常驻内存、24 小时/7 天稳定性是验收目标，不能仅凭短时测试声明满足。内存应包含 WebView 子进程，详见验证说明。

第三阶段的工作区、最近目录和终端/VS Code 打开方式尚未纳入本版。项目采用 [MIT License](LICENSE)。
