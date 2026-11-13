import "./style.css";
import { call, native } from "./api";
import { icon } from "./icons";
import {
  addPin,
  escapeHtml as e,
  folderName,
  matches,
  movePin,
  pathKey,
} from "./model";
import type { Config, Pin, Snapshot, View } from "./types";

const root = document.querySelector<HTMLDivElement>("#app")!;
document.body.classList.toggle("native", native);
root.innerHTML = `<main class="app">${!native ? '<div class="preview-banner">桌面界面预览 · 不执行系统操作</div>' : ""}<header class="top" data-tauri-drag-region><div class="brand-icon">${icon("anchor")}</div><div class="brand" data-tauri-drag-region><strong data-tauri-drag-region>泊岸</strong><small data-tauri-drag-region>BERTH / YOUR FOLDERS</small></div><button class="icon-button" id="keep" aria-label="固定展开" title="固定展开">${icon("pin")}</button><button class="icon-button" id="settings" aria-label="设置" title="设置">${icon("settings")}</button></header><label class="search">${icon("search")}<input id="search" aria-label="搜索文件夹" placeholder="找一个文件夹…" autocomplete="off"><kbd>/</kbd></label><div class="notice" id="warning" role="status"></div><nav class="tabs" aria-label="文件夹列表"><button class="tab active" data-tab="pins">已固定 <span class="count" id="pin-count">0</span></button><button class="tab" data-tab="windows">已打开 <span class="count" id="window-count">0</span></button></nav><section class="list" id="list" aria-label="文件夹"></section><footer class="footer"><button class="add-button" id="add">${icon("plus")} 固定文件夹</button><div class="footnote"><span><i class="tiny-dot"></i> 每个目录，都有停靠的位置</span><button id="undo" class="icon-button" aria-label="撤销" title="撤销 Ctrl+Z" disabled>${icon("undo")}</button></div></footer></main><div class="drop-overlay">松开，让文件夹停靠在这里</div><dialog id="dialog"></dialog><div class="toast" role="status" aria-live="polite"></div>`;
const $ = <T extends HTMLElement = HTMLElement>(selector: string) =>
  document.querySelector<T>(selector)!;
const dialog = $<HTMLDialogElement>("#dialog");
let view: View;
let tab: "pins" | "windows" = "pins";
let query = "";
let windowFilter: string | null = null;
let queue = Promise.resolve();
let toastTimer: ReturnType<typeof setTimeout>;
function toast(message: unknown) {
  $(".toast").textContent = String(
    message instanceof Error ? message.message : message,
  );
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => ($(".toast").textContent = ""), 5000);
}
function accept(next: View) {
  if (!view || next.revision >= view.revision) {
    view = next;
    if (native)
      document.documentElement.classList.toggle("collapsed", !!next.collapsed);
    render();
  }
}
function mutate(fn: (config: Config) => void): Promise<void> {
  const operation = queue.then(async () => {
    const config = structuredClone(view.config);
    fn(config);
    accept(
      await call<View>("save_config", { config, revision: view.revision }),
    );
  });
  queue = operation.catch(async (error) => {
    toast(error);
    accept(await call<View>("get_state"));
  });
  return operation;
}
const counts = (path: string) =>
  view.snapshot.windows.filter((w) => pathKey(w.path) === pathKey(path));
function row(pin: Pin) {
  const windows = counts(pin.path);
  const available = view.snapshot.available[pin.id] !== false;
  return `<article class="pin-row ${available ? "" : "unavailable"}" data-pin="${e(pin.id)}" title="${e(pin.path)}${available ? "" : " · 路径不可用，点击可重新定位"}"><button class="icon-button grip" aria-label="拖动 ${e(pin.alias)}" data-drag="${e(pin.id)}">${icon("grip")}</button><button class="open-pin" data-open="${e(pin.id)}" aria-label="${e(pin.alias)}${available ? "" : "，路径不可用"}"><span class="folder-icon" style="color:${/^#[0-9a-f]{6}$/i.test(pin.color) ? pin.color : "#75a99c"}">${icon("folder")}</span><span class="pin-info"><span class="pin-name">${e(pin.alias)}</span><span class="pin-path">${available ? e(pin.path) : "路径不可用 · 点击重新定位"}</span></span></button>${windows.length > 1 ? `<button class="windows-count" data-all="${e(pin.id)}" title="查看全部窗口">${windows.length}</button>` : windows.length ? '<span class="status" title="已打开" aria-label="已打开"></span>' : ""}<button class="icon-button row-menu" data-edit="${e(pin.id)}" aria-label="编辑 ${e(pin.alias)}">${icon("more")}</button></article>`;
}
function render() {
  if (!view) return;
  $("#pin-count").textContent = String(view.config.pins.length);
  $("#window-count").textContent = String(view.snapshot.windows.length);
  $("#keep").classList.toggle("active", view.config.panel.pinned);
  $("#keep").setAttribute("aria-pressed", String(view.config.panel.pinned));
  $<HTMLButtonElement>("#undo").disabled = !view.canUndo;
  $("#warning").textContent = [view.warning, view.snapshot.warning]
    .filter(Boolean)
    .join("\n");
  $(".app").style.opacity = String(view.config.panel.opacity);
  document
    .querySelectorAll<HTMLElement>("[data-tab]")
    .forEach((el) => el.classList.toggle("active", el.dataset.tab === tab));
  const list = $("#list");
  if (tab === "windows") {
    const windows = view.snapshot.windows.filter(
      (w) =>
        (!windowFilter || pathKey(w.path) === windowFilter) &&
        matches(query, w.title, w.path),
    );
    list.innerHTML = `<div class="toolbar">${windowFilter ? '<button class="button" data-clear-filter>全部窗口</button>' : "<span></span>"}<button class="button" data-merge ${windows.length < 2 ? "disabled" : ""}>合并重复窗口</button></div>${windows.map((w) => `<article class="pin-row" title="${e(w.path)}"><button class="open-pin" data-window="${e(w.id)}"><span class="folder-icon">${icon("window")}</span><span class="pin-info"><span class="pin-name">${e(w.title || folderName(w.path))}</span><span class="pin-path">${w.minimized ? "已最小化 · " : ""}${e(w.path)}</span></span></button><button class="icon-button" data-pin-window="${e(w.id)}" title="固定此目录" aria-label="固定 ${e(w.title)}">${icon(view.config.pins.some((p) => pathKey(p.path) === pathKey(w.path)) ? "check" : "plus")}</button></article>`).join("")}${!windows.length ? `<div class="empty"><div class="empty-art">${icon("window")}</div><h2>${query ? "没有匹配的窗口" : "此刻，桌面很清静"}</h2><p>打开的文件夹会出现在这里<br>随时切换，也可以一键固定</p></div>` : ""}`;
    return;
  }
  const pins = view.config.pins.filter((p) =>
    matches(
      query,
      p.alias,
      p.path,
      view.config.groups.find((g) => g.id === p.groupId)?.name || "",
    ),
  );
  if (!view.config.pins.length && !view.config.groups.length) {
    list.innerHTML = `<div class="empty"><div class="empty-art">${icon("folder")}</div><h2>把文件夹拖到这里</h2><p>为常用目录留一个位置<br>下次，一点就到</p></div><div class="section-head"><span style="flex:1">也可以从下方添加</span><button class="icon-button" data-add-group title="新建分组" aria-label="新建分组">${icon("plus")}</button></div>`;
    return;
  }
  const section = (id: string | null, name: string, collapsed = false) => {
    const items = pins.filter((p) => p.groupId === id);
    if (query && !items.length) return "";
    return `<section data-group="${e(id || "")}"><div class="section-head"><button class="group-toggle" ${id ? `data-collapse="${e(id)}"` : ""}>${id ? icon("arrow") : ""}${e(name)}<span class="count">${items.length}</span></button>${id ? `<button class="icon-button" data-edit-group="${e(id)}" aria-label="编辑分组 ${e(name)}">${icon("more")}</button>` : `<button class="icon-button" data-add-group title="新建分组" aria-label="新建分组">${icon("plus")}</button>`}</div>${!collapsed || query ? items.map(row).join("") : ""}</section>`;
  };
  list.innerHTML =
    section(null, "常用目录") +
    view.config.groups.map((g) => section(g.id, g.name, g.collapsed)).join("") +
    (!pins.length && query
      ? '<div class="empty"><h2>没有找到这个文件夹</h2><p>试试目录名、别名或路径中的关键词</p></div>'
      : "");
}
function modal(title: string, body: string) {
  if (dialog.open) dialog.close();
  dialog.innerHTML = `<div class="dialog-head"><h2>${e(title)}</h2><button class="icon-button" data-close aria-label="关闭">${icon("close")}</button></div><div class="dialog-body">${body}<div class="error" role="alert"></div></div>`;
  dialog.showModal();
  void call("set_interacting", { value: true });
}
dialog.addEventListener(
  "close",
  () => void call("set_interacting", { value: false }),
);
dialog.addEventListener("click", (event) => {
  if ((event.target as HTMLElement).closest("[data-close]")) dialog.close();
});
async function dialogTask(task: () => Promise<void>, close = true) {
  const button = dialog.querySelector<HTMLButtonElement>("button[type=submit]");
  if (button) button.disabled = true;
  try {
    await task();
    if (close) dialog.close();
  } catch (error) {
    dialog.querySelector(".error")!.textContent = String(
      error instanceof Error ? error.message : error,
    );
  } finally {
    if (button) button.disabled = false;
  }
}
async function browse(): Promise<string | null> {
  if (!native) {
    toast("网页预览中请手动输入路径");
    return null;
  }
  const { open } = await import("@tauri-apps/plugin-dialog");
  return (await open({
    directory: true,
    multiple: false,
    title: "选择要固定的文件夹",
  })) as string | null;
}
async function pinPaths(paths: string[]) {
  const failures: string[] = [];
  for (const path of paths) {
    try {
      const resolved = await call<string>("validate_path", { path });
      await mutate((c) => {
        addPin(c, resolved);
      });
    } catch (error) {
      failures.push(`${folderName(path)}：${String(error)}`);
    }
  }
  toast(
    failures.length ? failures.join("\n") : `已固定 ${paths.length} 个文件夹`,
  );
}
function addDialog() {
  modal(
    "留一个位置",
    `<p>固定常用文件夹。文件仍留在原处，泊岸只记住它的位置。</p><form id="add-form"><label class="field"><span>文件夹路径</span><input name="path" required placeholder="输入完整路径" autofocus></label><button type="button" class="button" id="browse">浏览文件夹…</button><label class="field"><span>别名 · 可选</span><input name="alias" maxlength="120" placeholder="给它一个容易记住的名字"></label><div class="actions"><button type="button" class="button" data-close>取消</button><button type="submit" class="button primary">固定文件夹</button></div></form>`,
  );
  $("#browse").onclick = () =>
    void dialogTask(async () => {
      const path = await browse();
      if (path) $<HTMLInputElement>("[name=path]").value = path;
    }, false);
  $<HTMLFormElement>("#add-form").onsubmit = (event) => {
    event.preventDefault();
    void dialogTask(async () => {
      const data = new FormData(event.currentTarget as HTMLFormElement);
      const path = await call<string>("validate_path", {
        path: String(data.get("path")),
      });
      await mutate((c) => {
        addPin(c, path, String(data.get("alias")));
      });
      toast("已固定，随时一点就到");
    });
  };
}
function editDialog(id: string) {
  const pin = view.config.pins.find((p) => p.id === id);
  if (!pin) return;
  modal(
    "整理文件夹",
    `<form id="edit-form"><label class="field"><span>别名</span><input name="alias" required maxlength="120" value="${e(pin.alias)}"></label><label class="field"><span>文件夹路径 · 可重新定位</span><input name="path" required value="${e(pin.path)}"></label><button type="button" class="button" id="relocate">浏览…</button><div class="field-row"><label class="field"><span>分组</span><select name="group" aria-label="分组"><option value="">常用目录</option>${view.config.groups.map((g) => `<option value="${e(g.id)}" ${g.id === pin.groupId ? "selected" : ""}>${e(g.name)}</option>`).join("")}</select></label><label class="field"><span>颜色标签</span><input name="color" type="color" value="${e(pin.color)}"></label></div><div class="actions"><button type="button" class="button" id="up">上移</button><button type="button" class="button" id="down">下移</button><button type="button" class="button" id="copy">复制路径</button></div><div class="actions"><button type="button" class="button danger" id="remove">取消固定</button><button class="button primary" type="submit">保存</button></div></form>`,
  );
  $("#relocate").onclick = () =>
    void dialogTask(async () => {
      const path = await browse();
      if (path) $<HTMLInputElement>("[name=path]").value = path;
    }, false);
  $("#copy").onclick = () =>
    void navigator.clipboard
      .writeText(pin.path)
      .then(() => toast("已复制路径"))
      .catch(() => toast("复制失败，请手动选择路径复制"));
  $("#remove").onclick = () =>
    void dialogTask(async () => {
      await mutate((c) => {
        c.pins = c.pins.filter((p) => p.id !== id);
        c.pins.forEach((p, i) => (p.order = i));
      });
      toast("已取消固定 · 可按 Ctrl+Z 撤销");
    });
  const shift = async (offset: number) => {
    await mutate((c) => {
      const i = c.pins.findIndex((p) => p.id === id);
      const target = i + offset;
      if (target < 0 || target >= c.pins.length) return;
      [c.pins[i], c.pins[target]] = [c.pins[target], c.pins[i]];
      c.pins.forEach((p, i) => (p.order = i));
    });
  };
  $("#up").onclick = () => void dialogTask(() => shift(-1), false);
  $("#down").onclick = () => void dialogTask(() => shift(1), false);
  $<HTMLFormElement>("#edit-form").onsubmit = (event) => {
    event.preventDefault();
    const data = new FormData(event.currentTarget as HTMLFormElement);
    void dialogTask(() =>
      mutate((c) => {
        const p = c.pins.find((p) => p.id === id)!;
        p.alias = String(data.get("alias")).trim();
        if (!p.alias) throw new Error("请输入别名");
        p.path = String(data.get("path")).trim();
        p.color = String(data.get("color"));
        p.groupId = String(data.get("group")) || null;
      }),
    );
  };
}
function groupDialog(id?: string) {
  const group = view.config.groups.find((g) => g.id === id);
  modal(
    group ? "整理分组" : "新建分组",
    `<form id="group-form"><label class="field"><span>分组名称</span><input name="name" required maxlength="80" value="${e(group?.name || "")}" autofocus></label><div class="actions">${group ? '<button class="button danger" type="button" id="remove-group">删除分组</button>' : ""}<button type="submit" class="button primary">保存</button></div></form>`,
  );
  if (group)
    $("#remove-group").onclick = () =>
      void dialogTask(() =>
        mutate((c) => {
          c.groups = c.groups.filter((g) => g.id !== id);
          c.pins.forEach((p) => {
            if (p.groupId === id) p.groupId = null;
          });
        }),
      );
  $<HTMLFormElement>("#group-form").onsubmit = (event) => {
    event.preventDefault();
    const name = String(
      new FormData(event.currentTarget as HTMLFormElement).get("name"),
    ).trim();
    void dialogTask(() =>
      mutate((c) => {
        if (!name) throw new Error("请输入分组名称");
        const g = c.groups.find((g) => g.id === id);
        if (g) g.name = name;
        else c.groups.push({ id: crypto.randomUUID(), name, collapsed: false });
      }),
    );
  };
}
function settingsDialog() {
  const c = view.config;
  modal(
    "让泊岸适合你",
    `<form id="settings-form"><div class="section-title">面板</div><label class="field"><span>显示方式</span><select name="mode"><option value="sidebar" ${c.panel.mode === "sidebar" ? "selected" : ""}>贴边侧栏</option><option value="float" ${c.panel.mode === "float" ? "selected" : ""}>悬浮小窗</option><option value="popup" ${c.panel.mode === "popup" ? "selected" : ""}>热键弹出</option></select></label><label class="field"><span>显示器</span><select name="monitor"><option value="">跟随当前屏幕</option></select></label><label class="field"><span>侧栏位置</span><select name="edge"><option value="left" ${c.panel.edge === "left" ? "selected" : ""}>屏幕左侧</option><option value="right" ${c.panel.edge === "right" ? "selected" : ""}>屏幕右侧</option></select></label><label class="check-field">保持置顶<input type="checkbox" name="top" ${c.panel.alwaysOnTop ? "checked" : ""}></label><label class="check-field">全屏时自动隐藏<input type="checkbox" name="fullscreen" ${c.panel.hideFullscreen ? "checked" : ""}></label><label class="field"><span>面板透明度</span><input name="opacity" type="range" min="0.5" max="1" step="0.05" value="${c.panel.opacity}"></label><div class="section-title">启动与快捷键</div><label class="check-field">登录电脑时启动<input type="checkbox" name="autostart" ${c.autostart ? "checked" : ""}></label><label class="field"><span>显示 / 隐藏面板 · 留空关闭</span><input name="toggle" value="${e(c.hotkeys.toggle)}" placeholder="Ctrl+Alt+Space"></label><label class="field"><span>固定当前文件夹 · 留空关闭</span><input name="capture" value="${e(c.hotkeys.capture)}" placeholder="Ctrl+Alt+P"></label><label class="check-field">前 9 项直达 · Ctrl+Alt+1~9<input type="checkbox" name="direct" ${c.hotkeys.direct ? "checked" : ""}></label><div class="actions"><button type="submit" class="button primary">保存设置</button></div><div class="section-title">你的数据，只留在这里</div><p class="settings-note">${e(view.configPath)}</p><div class="actions"><button type="button" class="button" id="import">导入配置</button><button type="button" class="button" id="export">导出配置</button></div><p>泊岸 0.1.0 · 离线运行，不收集使用数据。<br>关闭面板后，可从托盘或快捷键再次呼出。</p><button type="button" class="button" id="hide">收起到托盘</button></form>`,
  );
  $<HTMLFormElement>("#settings-form").onsubmit = (event) => {
    event.preventDefault();
    const data = new FormData(event.currentTarget as HTMLFormElement);
    void dialogTask(() =>
      mutate((c) => {
        c.panel.mode = String(data.get("mode")) as Config["panel"]["mode"];
        c.panel.edge = String(data.get("edge")) as "left" | "right";
        c.panel.monitor = String(data.get("monitor")) || null;
        c.panel.alwaysOnTop = data.has("top");
        c.panel.hideFullscreen = data.has("fullscreen");
        c.panel.opacity = Number(data.get("opacity"));
        c.autostart = data.has("autostart");
        c.hotkeys.toggle = String(data.get("toggle")).trim();
        c.hotkeys.capture = String(data.get("capture")).trim();
        c.hotkeys.direct = data.has("direct");
      }),
    );
  };
  if (native)
    void call<string[]>("list_monitors")
      .then((names) => {
        const select =
          dialog.querySelector<HTMLSelectElement>("[name=monitor]");
        if (select) {
          select.innerHTML =
            '<option value="">跟随当前屏幕</option>' +
            names
              .map((name) => `<option value="${e(name)}">${e(name)}</option>`)
              .join("");
          select.value = c.panel.monitor || "";
        }
      })
      .catch(toast);
  $("#hide").onclick = () => {
    dialog.close();
    void call("hide_panel");
  };
  $("#export").onclick = () =>
    void dialogTask(async () => {
      if (!native) throw new Error("请在桌面应用中导出配置");
      const { save } = await import("@tauri-apps/plugin-dialog");
      const path = await save({
        defaultPath: "berth-config.json",
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (path) {
        await call("export_config", { path });
        toast("配置已导出");
      }
    }, false);
  $("#import").onclick = () => {
    modal(
      "导入配置",
      `<p>导入将替换当前固定项和设置，也会应用自启与快捷键。可以通过撤销恢复当前配置。</p><div class="actions"><button class="button" data-close>取消</button><button class="button primary" id="confirm-import">选择配置并导入</button></div>`,
    );
    $("#confirm-import").onclick = () =>
      void dialogTask(async () => {
        if (!native) throw new Error("请在桌面应用中导入配置");
        const { open } = await import("@tauri-apps/plugin-dialog");
        const path = await open({
          multiple: false,
          filters: [{ name: "JSON", extensions: ["json"] }],
        });
        if (path) accept(await call<View>("import_config", { path }));
      });
  };
}
async function openPin(id: string, force = false) {
  const pin = view.config.pins.find((p) => p.id === id);
  if (!pin) return;
  if (view.snapshot.available[id] === false) {
    editDialog(id);
    dialog.querySelector(".error")!.textContent =
      "路径不可用，请重新定位或取消固定。";
    return;
  }
  const row = document.querySelector<HTMLElement>(
    `[data-pin="${CSS.escape(id)}"]`,
  );
  row?.classList.add("busy");
  try {
    const result = await call<string>("open_pin", { id, force });
    if (result === "opening") toast("正在打开，请稍候");
  } catch (error) {
    toast(error);
  } finally {
    row?.classList.remove("busy");
  }
}
$("#add").onclick = addDialog;
$("#settings").onclick = settingsDialog;
$("#keep").onclick = () =>
  void mutate((c) => {
    c.panel.pinned = !c.panel.pinned;
  }).catch(() => {});
$("#undo").onclick = () => {
  queue = queue.then(async () => accept(await call<View>("undo"))).catch(toast);
};
$<HTMLInputElement>("#search").oninput = (event) => {
  query = (event.target as HTMLInputElement).value;
  render();
};
document.querySelectorAll<HTMLElement>("[data-tab]").forEach(
  (el) =>
    (el.onclick = () => {
      tab = el.dataset.tab as typeof tab;
      windowFilter = null;
      render();
    }),
);
$("#list").addEventListener("click", (event) => {
  const target = (event.target as HTMLElement).closest<HTMLElement>("button");
  if (!target) return;
  const d = target.dataset;
  if (d.open) void openPin(d.open, event.shiftKey);
  if (d.edit) editDialog(d.edit);
  if (d.all) {
    windowFilter = pathKey(view.config.pins.find((p) => p.id === d.all)!.path);
    tab = "windows";
    render();
  }
  if (d.window) void call("activate_window", { id: d.window }).catch(toast);
  if (d.pinWindow) {
    const w = view.snapshot.windows.find((w) => w.id === d.pinWindow);
    if (w) void pinPaths([w.path]);
  }
  if (d.collapse)
    void mutate((c) => {
      const g = c.groups.find((g) => g.id === d.collapse)!;
      g.collapsed = !g.collapsed;
    }).catch(() => {});
  if ("addGroup" in d) groupDialog();
  if (d.editGroup) groupDialog(d.editGroup);
  if ("clearFilter" in d) {
    windowFilter = null;
    render();
  }
  if ("merge" in d) {
    modal(
      "合并重复窗口",
      `<p>同一目录保留最近使用的窗口，关闭其他重复窗口。不会修改文件；含多个标签页的窗口不会自动关闭。</p><div class="actions"><button class="button" data-close>取消</button><button class="button primary" id="confirm-merge">合并重复窗口</button></div>`,
    );
    $("#confirm-merge").onclick = () =>
      void dialogTask(async () => {
        toast(await call<string>("merge_windows"));
      });
  }
});
$("#list").addEventListener("auxclick", (event) => {
  if (event.button === 1) {
    event.preventDefault();
    const button = (event.target as HTMLElement).closest<HTMLElement>(
      "[data-open]",
    );
    if (button) void openPin(button.dataset.open!, true);
  }
});
$("#list").addEventListener("contextmenu", (event) => {
  const row = (event.target as HTMLElement).closest<HTMLElement>("[data-pin]");
  if (row) {
    event.preventDefault();
    editDialog(row.dataset.pin!);
  }
});
let dragId: string | null = null;
$("#list").addEventListener("pointerdown", (event) => {
  const handle = (event.target as HTMLElement).closest<HTMLElement>(
    "[data-drag]",
  );
  if (handle) {
    event.preventDefault();
    dragId = handle.dataset.drag!;
    handle.closest(".pin-row")?.classList.add("dragging");
    void call("set_interacting", { value: true });
  }
});
document.addEventListener("pointermove", (event) => {
  if (!dragId) return;
  document
    .querySelectorAll(".drop-target")
    .forEach((e) => e.classList.remove("drop-target"));
  document
    .elementFromPoint(event.clientX, event.clientY)
    ?.closest("[data-pin],[data-group]")
    ?.classList.add("drop-target");
});
document.addEventListener("pointerup", (event) => {
  if (!dragId) return;
  const id = dragId;
  dragId = null;
  const target = document
    .elementFromPoint(event.clientX, event.clientY)
    ?.closest<HTMLElement>("[data-pin],[data-group]");
  if (target)
    void mutate((c) =>
      movePin(
        c,
        id,
        target.dataset.pin || null,
        target.dataset.group === undefined
          ? undefined
          : target.dataset.group || null,
      ),
    ).catch(() => {});
  document
    .querySelectorAll(".drop-target,.dragging")
    .forEach((e) => e.classList.remove("drop-target", "dragging"));
  void call("set_interacting", { value: false });
});
document.addEventListener("pointercancel", () => {
  dragId = null;
  render();
  void call("set_interacting", { value: false });
});
document.addEventListener("keydown", (event) => {
  const editing = (event.target as HTMLElement).matches(
    "input,textarea,select",
  );
  if (event.key === "/" && !editing && !dialog.open) {
    event.preventDefault();
    $("#search").focus();
  }
  if (
    (event.ctrlKey || event.metaKey) &&
    event.key.toLowerCase() === "z" &&
    !editing &&
    !dialog.open
  ) {
    event.preventDefault();
    $("#undo").click();
  }
  if (event.key === "Escape" && !dialog.open) {
    if (query) {
      query = "";
      $<HTMLInputElement>("#search").value = "";
      render();
    } else void call("hide_panel");
  }
});
async function start() {
  if (native) {
    const { listen } = await import("@tauri-apps/api/event");
    await listen<View>("config-changed", (event) => accept(event.payload));
    await listen<Snapshot>("snapshot", (event) => {
      if (view) {
        view.snapshot = event.payload;
        if (!dragId) render();
      }
    });
    await listen<boolean>("collapsed", (event) =>
      document.documentElement.classList.toggle("collapsed", event.payload),
    );
    await listen("settings", settingsDialog);
    await listen<string>("notice", (event) => toast(event.payload));
    await listen<string>("capture", (event) => void pinPaths([event.payload]));
    await listen("intent", () => {
      if (view) void drainIntents();
    });
    const { getCurrentWebviewWindow } =
      await import("@tauri-apps/api/webviewWindow");
    await getCurrentWebviewWindow().onDragDropEvent((event) => {
      const payload = event.payload;
      document.body.classList.toggle(
        "dropping",
        payload.type === "enter" || payload.type === "over",
      );
      if (payload.type === "enter")
        void call("set_interacting", { value: true });
      if (payload.type === "leave")
        void call("set_interacting", { value: false });
      if (payload.type === "drop")
        void pinPaths(payload.paths).finally(() =>
          call("set_interacting", { value: false }),
        );
    });
  }
  accept(await call<View>("get_state"));
  if (native) {
    await call("ui_ready");
    await drainIntents();
  }
}
async function drainIntents() {
  const intents = await call<{ kind: string; value: string }[]>("take_intents");
  for (const intent of intents) {
    if (intent.kind === "settings") settingsDialog();
    if (intent.kind === "capture") await pinPaths([intent.value]);
    if (intent.kind === "notice") toast(intent.value);
  }
}
void start().catch((error) => {
  toast(error);
  $("#warning").textContent = "启动失败，请从托盘退出后重新打开。";
});
