# Changelog

All notable changes to Berth are documented here. Versions follow [Semantic Versioning](https://semver.org/); the project is currently in the 0.x preview series, so minor versions may contain breaking changes.

## [0.1.0] — 2026-11-13

First public preview release. Version `0.1.0` is defined consistently in `package.json`, `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml`, and is published as the Git tag [`v0.1.0`](https://github.com/lvguidong/Berth/releases/tag/v0.1.0).

### Added

- Bilingual interface: English and Simplified Chinese, switchable from **Settings → Language**. The choice is stored in `config.json` (`language: "zh" | "en"`), follows config import/export, and also updates the native tray menu.
- English README (`README.en.md`) covering installation, the release package, memory behavior and limitations.
- macOS arm64 DMG published as a GitHub Release asset, with a `SHA256SUMS` checksum file.
- Folder pinning with drag and drop, aliases, colors, groups, reordering, search and stale-path relocation.
- Explorer/Finder window discovery, switching, duplicate-window merging, session undo, and three panel modes (edge sidebar, floating window, hotkey popup).
- Configurable global shortcuts, tray integration, opt-in launch at login, single instance, atomic config writes and corrupt-config backups.

### Fixed

- **macOS: blank panel after the WebView was released.** The idle-memory optimization closed the WebView after five seconds; WKWebView child views cannot be reliably re-attached, so waking the panel produced an empty 320×680 window. macOS now keeps the WebView alive (`release_ui` is a no-op on macOS); Windows keeps the original recycling behavior.
- macOS DMG packaging: Tauri's `bundle_dmg.sh` fails on some macOS versions while parsing temporary disk device names. The `.app` is still produced; the DMG is generated from a staging folder with `hdiutil create -format UDZO`.

### Notes

- Version 0.1.0 is built and verified on an Apple Silicon Mac. Windows packages are source-buildable but not attached to this release.
- Builds are unsigned and not notarized. macOS requires the Automation (Finder) permission for window reuse.
- The CI packaging workflow now runs on manual dispatch only, so pushing a version tag does not create a competing draft prerelease.

### Tests

- `npm test`: 11 passed (model 7, i18n 4).
- `cargo test`: 13 passed.
- `npm run build` (tsc + Vite) and `cargo check` pass; the release `.app` was launched on macOS and its panel contents verified by screenshot.

[0.1.0]: https://github.com/lvguidong/Berth/releases/tag/v0.1.0
