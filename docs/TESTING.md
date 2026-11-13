# 验证记录

日期：2026-11-13（Asia/Shanghai）。系统：Windows 11 家庭中文版，10.0.26200 x64；Node 22.21.1，Rust 1.96.0。

## 验证口径

前端单测验证纯业务逻辑；Playwright 验证显式网页预览中的交互，不替代系统集成。Rust 配置/路径测试使用临时目录。`explorer_lifecycle` 在真实 Windows 桌面打开并关闭它自己创建的临时窗口，不触碰原有资源管理器窗口。

## 已完成

| 检查 | 结果 |
| --- | --- |
| TypeScript 类型检查与 Vite 生产构建 | 通过 |
| Vitest | 7 项通过：路径身份、重复固定、分组排序、搜索、HTML 转义 |
| Playwright | 5 条用户流程通过：新增/重命名/刷新持久化/取消/撤销、重复和相对路径、分组搜索折叠、恶意别名/设置、拖动与快捷键撤销 |
| Rust 单元测试 | 13 项通过，另有 1 项默认忽略的真实窗口测试 |
| Rust 原生 Explorer 集成测试 | 单独运行通过：新开、快速重复点击不多开、已有窗口切换、最小化还原、强制新开、关闭状态发现 |
| 原生切换耗时 | 缓存优化后一次实测约 57ms；不是所有机器/网络路径的性能保证 |
| Cargo fmt / Clippy（警告视为错误） | 通过 |
| npm audit | 0 个已知漏洞 |
| Windows NSIS 安装包 | 最终版本成功生成，1,410,624 字节，约 1.35MiB；包含 MIT 许可页（外部 WebView2 Runtime 不计入包体） |
| 发布版启动 | 单次本地测得窗口句柄出现约 708ms；不等同于严格冷缓存可操作时间 |
| 发布版单实例 | 第二次启动正常退出，保留 1 个应用进程和 1 个轻量守护进程 |
| Mac 原生适配 Rust 类型检查 | Apple Silicon 与 Intel 目标均通过独立 `tools/platform-check`；不含 WebView 链接和 Apple SDK 验证 |
| 完整 Mac 应用构建尝试 | 当前 Windows 环境在 Objective-C 编译工具链步骤受阻；没有宣称成功 |
| 界面视觉检查 | 明暗主题预览截图已检查；原生设置界面曾成功显示。用户按 Esc 后停止桌面自动操作 |

## 内存测量

必须计入守护进程、主进程及所有后代 WebView 进程。`tools/soak.ps1` 同时记录工作集和私有内存，支持用 `-RootProcessId` 隔离测试实例。

优化前的展开面板 2 分钟样本：约 410–520MiB 进程树工作集，约 182–229MiB 私有内存；只看主进程会错误地得出“不到 50MiB”。因此不能声称原始实现满足内存目标。

已加入收起/隐藏 5 秒后释放 WebView、仅保留原生边缘窗口的机制；唤起时重新挂载界面，配置与撤销历史保存在 Rust 后台。对应生命周期验证已检查原生边缘保留、界面重新挂载、目录不丢失与延迟设置操作送达。

生命周期测试已执行两轮通过。调试构建收起时样本约 47MiB 工作集、8MiB 私有内存（含测试控制台）；后台并行编译时恢复耗时 8184ms，停止编译后的空闲复测为 1005ms。该数据说明内存与恢复延迟之间的取舍；固定展开场景仍不满足 50MiB，不能把收起数据推广到所有状态。

最终发布构建独立复测：收起后稳定样本约 **38.7MiB 工作集 / 8.7MiB 私有内存**，包含应用和守护进程；WebView 子进程已释放。展开态仍约数百 MiB。本地原始记录位于 `artifacts/final-release-idle.csv`（不纳入源码仓库）。

```powershell
.\tools\lifecycle.ps1
```

此脚本使用单独应用标识与隔离配置；应用自身验证 WebView 释放/重建，不注入鼠标键盘。调试版完成后自动退出。生命周期测试入口在发布构建中不启用。

## 复现命令

```powershell
npm ci
npx playwright install chromium
.\tools\check.ps1 -Native
```

不操作真实窗口时省略 `-Native`。

跨平台适配层类型检查（无需 Apple SDK，但不代表应用可运行）：

```sh
rustup target add aarch64-apple-darwin x86_64-apple-darwin
cargo check --manifest-path tools/platform-check/Cargo.toml --target aarch64-apple-darwin
cargo check --manifest-path tools/platform-check/Cargo.toml --target x86_64-apple-darwin
```

长稳测量：启动发布版并固定 10 个目录，执行下列命令；24 小时报告中排除启动预热，检查私有内存趋势、崩溃和操作响应。

```powershell
.\tools\soak.ps1 -Minutes 1440 -IntervalSeconds 60
# 7 天：-Minutes 10080
```

## 尚未完成的发布验收

- 完整 macOS 两架构编译、Finder 自动化授权/拒绝、Dock/菜单栏/Spaces、最小化还原及全屏。
- Windows 10、新装 Windows 11、真实 UNC/映射盘离线、权限隔离窗口与跨虚拟桌面。
- 真实高 DPI 双显示器拔插（当前逻辑根据 Tauri 工作区计算，不能用代码检查替代硬件验证）。
- 电脑重启后的自启与 10 项数据验收；本次没有自动修改用户开机启动项或重启电脑。
- 24 小时内存增长低于 10MB、7 天无崩溃；短时采样无法替代。
- 数字签名、公证、GitHub 实际构建与发布、新机器安装验证。
- Windows 11 多标签页的准确切换仍为已知限制，详见实现对照。

完整源码、测试和本地安装包可交付，但上述项目不能标记为“全部验收通过”。
