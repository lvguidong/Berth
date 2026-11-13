import { describe, it, expect } from "vitest";
import { addPin, movePin, pathKey, matches, escapeHtml } from "./model";
import { defaultConfig } from "./types";
describe("folder identity", () => {
  it("matches drive, UNC, extended paths and trailing separators", () => {
    expect(pathKey("C:/Work/")).toBe(pathKey("c:\\work"));
    expect(pathKey("\\\\?\\UNC\\server\\share\\")).toBe(
      pathKey("\\\\SERVER\\share"),
    );
    expect(pathKey("\\\\?\\C:\\work")).toBe(pathKey("c:\\work"));
  });
  it("preserves Unix case-sensitive volumes and root", () => {
    expect(pathKey("/Volumes/A")).not.toBe(pathKey("/Volumes/a"));
    expect(pathKey("/")).toBe("/");
  });
  it("rejects duplicate folders without discarding an existing alias", () => {
    const c = defaultConfig();
    addPin(c, "C:\\work", "项目");
    expect(() => addPin(c, "c:/WORK/")).toThrow("已经固定");
    expect(c.pins).toHaveLength(1);
    expect(c.pins[0].alias).toBe("项目");
  });
});
describe("organizing", () => {
  it("moves items into and out of groups with stable order", () => {
    const c = defaultConfig();
    const a = addPin(c, "/a"),
      b = addPin(c, "/b"),
      d = addPin(c, "/d");
    b.groupId = "g";
    movePin(c, d.id, b.id);
    expect(c.pins.map((p) => p.id)).toEqual([a.id, d.id, b.id]);
    expect(d.groupId).toBe("g");
    movePin(c, d.id, null, null);
    expect(d.groupId).toBeNull();
    expect(c.pins.map((p) => p.order)).toEqual([0, 1, 2]);
  });
  it("ignores invalid and self drops without losing pins", () => {
    const c = defaultConfig();
    const a = addPin(c, "/a");
    movePin(c, a.id, a.id);
    movePin(c, "unknown", a.id);
    expect(c.pins).toHaveLength(1);
  });
  it("searches aliases, paths, groups and subsequences", () => {
    expect(matches("prja", "Project Alpha")).toBe(true);
    expect(matches("设计 2026", "设计稿", "D:/2026")).toBe(true);
    expect(matches("zzz", "Project Alpha")).toBe(false);
  });
  it("escapes user content in HTML and attributes", () => {
    expect(escapeHtml('<img src=x onerror="alert(1)">')).toBe(
      "&lt;img src=x onerror=&quot;alert(1)&quot;&gt;",
    );
  });
});
