# Contributing to Berth

请先阅读需求文档、README 和 `docs/IMPLEMENTATION.md`。改动应保持离线、仅限目录、不修改用户文件、准确切换优先的原则。

## 开发与提交

1. 使用 `npm ci` 安装锁定依赖；不要提交 `node_modules`、构建产物、私人配置或日志。
2. 使用 `BERTH_CONFIG_DIR` 创建隔离测试配置；不要使用其他人的真实路径作为测试数据。
3. Rust 负责系统集成、校验与落盘；前端只负责交互。新增系统操作应经后端校验，不能将路径拼接到 shell 命令中。
4. 运行前端测试、端到端测试、Rust 测试、`cargo fmt --check` 和 Clippy。修改系统层时补充对应平台实测。
5. PR 说明具体用户行为变化、测试证据、未验证的平台与风险。不要把 mock/UI 测试写成原生窗口测试，也不要用单进程内存代替整棵进程树。

## 结构

- `src/`：无框架 TypeScript UI；`api.ts` 区分原生调用与显式浏览器预览。
- `src-tauri/src/config.rs`：配置、版本校验、损坏备份和原子保存。
- `src-tauri/src/service.rs`：串行窗口服务、请求截止时间、有界后台路径检查。
- `src-tauri/src/platform/`：Windows COM / macOS Apple Events 适配。
- `src-tauri/src/panel.rs`：边缘显隐、工作区、多显示器和悬浮几何信息。
- `tests/ui/`：Playwright 用户流程回归。
- `.github/workflows/`：跨平台检查与草稿预发布构建。

Keep contributions focused, include meaningful regression coverage and document platform-specific behavior. Contributions are licensed under MIT.
