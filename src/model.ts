import type { Pin, Config } from "./types";
export function pathKey(path: string): string {
  if (/^(?:[a-z]:[\\/]|\\\\)/i.test(path))
    return path
      .replaceAll("/", "\\")
      .replace(/^\\\\\?\\UNC\\/i, "\\\\")
      .replace(/^\\\\\?\\/, "")
      .replace(/\\+$/, "")
      .toLowerCase();
  return path === "/" ? "/" : path.replace(/\/+$/, "");
}
export function folderName(path: string): string {
  return (
    path
      .replace(/[\\/]+$/, "")
      .split(/[\\/]/)
      .at(-1) || path
  );
}
export function matches(query: string, ...values: string[]): boolean {
  const haystack = values.join(" ").toLocaleLowerCase();
  return query
    .trim()
    .toLocaleLowerCase()
    .split(/\s+/)
    .every((word) => {
      if (haystack.includes(word)) return true;
      let i = 0;
      for (const c of haystack) if (c === word[i]) i++;
      return i === word.length;
    });
}
export function addPin(config: Config, path: string, alias = ""): Pin {
  if (config.pins.some((p) => pathKey(p.path) === pathKey(path)))
    throw new Error("这个文件夹已经固定了");
  const pin: Pin = {
    id: crypto.randomUUID(),
    path,
    alias: alias.trim() || folderName(path),
    color: "#75a99c",
    groupId: null,
    order: config.pins.length,
  };
  config.pins.push(pin);
  return pin;
}
export function movePin(
  config: Config,
  id: string,
  targetId: string | null,
  groupId?: string | null,
): void {
  const index = config.pins.findIndex((p) => p.id === id);
  if (index < 0 || targetId === id) return;
  const [pin] = config.pins.splice(index, 1);
  if (groupId !== undefined) pin.groupId = groupId;
  const target = config.pins.findIndex((p) => p.id === targetId);
  if (target >= 0) {
    if (groupId === undefined) pin.groupId = config.pins[target].groupId;
    config.pins.splice(target, 0, pin);
  } else config.pins.push(pin);
  config.pins.forEach((p, i) => (p.order = i));
}
export const escapeHtml = (s: unknown): string =>
  String(s ?? "").replace(
    /[&<>"']/g,
    (c) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[
        c
      ]!,
  );
