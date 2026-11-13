export interface Pin {
  id: string;
  path: string;
  alias: string;
  color: string;
  groupId: string | null;
  order: number;
}
export interface Group {
  id: string;
  name: string;
  collapsed: boolean;
}
export interface Config {
  version: number;
  language: "zh" | "en";
  panel: {
    mode: "sidebar" | "float" | "popup";
    edge: "left" | "right";
    pinned: boolean;
    width: number;
    height: number;
    x: number | null;
    y: number | null;
    opacity: number;
    alwaysOnTop: boolean;
    hideFullscreen: boolean;
    monitor: string | null;
  };
  autostart: boolean;
  hotkeys: { toggle: string; direct: boolean; capture: string };
  groups: Group[];
  pins: Pin[];
}
export interface FolderWindow {
  id: string;
  path: string;
  title: string;
  minimized: boolean;
}
export interface Snapshot {
  windows: FolderWindow[];
  warning: string | null;
  available: Record<string, boolean>;
}
export interface View {
  config: Config;
  revision: number;
  canUndo: boolean;
  warning: string | null;
  snapshot: Snapshot;
  configPath: string;
  collapsed?: boolean;
}
export const defaultConfig = (): Config => ({
  version: 1,
  language: "zh",
  panel: {
    mode: "sidebar",
    edge: "right",
    pinned: false,
    width: 320,
    height: 680,
    x: null,
    y: null,
    opacity: 1,
    alwaysOnTop: true,
    hideFullscreen: true,
    monitor: null,
  },
  autostart: false,
  hotkeys: { toggle: "Ctrl+Alt+Space", direct: false, capture: "Ctrl+Alt+P" },
  groups: [],
  pins: [],
});
