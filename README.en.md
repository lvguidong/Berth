# Berth

A quiet harbor for your folders. Pin folders to an edge panel; click to focus an existing Explorer/Finder window or open a new one. Built with Tauri 2, Rust, TypeScript and the system WebView.

[中文](README.md) · [Testing](docs/TESTING.md) · [Implementation status](docs/IMPLEMENTATION.md)

<img src="docs/images/preview-light.png" width="280" alt="Light theme preview"> <img src="docs/images/preview-dark.png" width="280" alt="Dark theme preview">

[UI walkthrough (WebM)](docs/images/demo.webm). These are labeled browser previews; OS window behavior is covered separately by native integration tests.

## Getting started

On Windows, run the NSIS installer or unzip the portable build and run `berth.exe`. WebView2 Runtime is required. On macOS, use the DMG matching your architecture and move Berth into Applications. Grant Finder Automation permission when prompted to enable window discovery and switching.

Drag a folder into the panel or use the add button. Click to switch/open; Shift+click or middle-click to force a new window. A dot means the folder is open. Hover shows the full path. Use the top pin button to keep the sidebar expanded. The tray icon toggles the panel; its menu opens settings or quits.

After five seconds collapsed or hidden, Berth releases its WebView and keeps only the native edge and Rust backend. Waking the UI reloads the WebView (about one second in an idle local test, longer under load). Pinned-open panels keep the WebView; their memory use is higher.

Version 0.1.0 is a preview. Windows is locally built and tested. macOS adapters and CI targets for Apple Silicon and Intel are included but have not been tested on a Mac. No public GitHub release has been published. Builds are unsigned.

## Features

Folder aliases, colors, groups, reorder handles, search, window list, duplicate-window merging with confirmation, session undo, sidebar/floating/popup modes, monitor selection, fullscreen hiding, configurable global shortcuts, tray integration, opt-in login startup, single instance and bounded crash recovery.

Default shortcuts: `Ctrl+Alt+Space` toggles Berth, `Ctrl+Alt+P` pins the foreground folder. Optional `Ctrl+Alt+1–9` opens the first nine pins. `/` focuses search; `Ctrl/Cmd+Z` undoes an organization change outside input fields.

## Development

Node.js 22.12+, Rust 1.90+ and platform prerequisites (Windows C++ Build Tools/SDK/WebView2, or macOS Xcode Command Line Tools):

```sh
npm ci
npm run desktop
```

`npm run dev` is an explicitly labeled browser preview and never operates OS windows.

```sh
npm run build
npm test
npx playwright install chromium
npm run test:e2e
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
npm run bundle -- --bundles nsis
```

The ignored `explorer_lifecycle` integration test requires an interactive Windows session and only closes temporary windows it created. Tag builds generate a draft prerelease with Windows NSIS/ZIP and macOS ARM64/x64 DMG packages.

## Privacy and persistence

No network access, accounts, telemetry or automatic update client. The application never changes the contents of pinned folders. Configuration is stored atomically as local JSON:

- Windows: `%APPDATA%\Berth\config.json`
- macOS: `~/Library/Application Support/Berth/config.json`

Corrupted configuration is preserved under a unique backup name. Import replaces configuration, including startup/shortcut settings, and can be undone. Use `BERTH_CONFIG_DIR` for isolated test profiles. Build dependencies and the install-time WebView2 download require network access; the app itself does not.

## Limitations

Windows 11 tabs are not yet reliably selected: ambiguous multi-tab windows produce guidance instead of claiming success and are never auto-merged. Cross-virtual-desktop activation depends on system policy. Network-path checks time out with bounded workers. Without macOS Automation permission, opening folders remains available but discovery/reuse and guaranteed force-new are unavailable.

The 10MB installer, 50MB resident memory, sub-300ms switching and 24-hour/7-day stability requirements are targets, not blanket claims. Memory must include WebView processes. Workspaces, recent folders and terminal/VS Code actions are not part of this release.

Licensed under the [MIT License](LICENSE).
