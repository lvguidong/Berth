import { invoke, isTauri } from "@tauri-apps/api/core";
import { defaultConfig, type Config, type View } from "./types";
export const native = isTauri();
let history: Config[] = [];
let preview: View = {
  config: defaultConfig(),
  revision: 0,
  canUndo: false,
  warning: null,
  snapshot: { windows: [], warning: null, available: {} },
  configPath: "网页预览：数据仅保存在本浏览器",
};
if (!native) {
  try {
    preview.config =
      JSON.parse(localStorage.getItem("berth-preview") || "null") ||
      defaultConfig();
  } catch {
    /* start empty */
  }
}
export async function call<T>(
  command: string,
  args: Record<string, unknown> = {},
): Promise<T> {
  if (native) return invoke<T>(command, args);
  switch (command) {
    case "get_state":
      return structuredClone(preview) as T;
    case "save_config":
      history.push(structuredClone(preview.config));
      preview.config = structuredClone(args.config as Config);
      preview.revision++;
      break;
    case "undo":
      if (history.length) preview.config = history.pop()!;
      preview.revision++;
      break;
    case "validate_path": {
      const p = String(args.path).trim();
      if (!/^(?:[a-z]:[\\/]|\\\\|\/)/i.test(p))
        throw new Error("请输入完整的文件夹路径");
      return p as T;
    }
    case "open_pin":
    case "activate_window":
      throw new Error("这是网页预览，请在桌面应用中打开和切换文件夹");
    case "set_interacting":
    case "hide_panel":
      return undefined as T;
    default:
      throw new Error("请在桌面应用中使用此功能");
  }
  preview.canUndo = history.length > 0;
  localStorage.setItem("berth-preview", JSON.stringify(preview.config));
  return structuredClone(preview) as T;
}
