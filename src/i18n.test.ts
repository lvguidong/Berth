import { describe, expect, it } from "vitest";
import { language, setLanguage, t, tr } from "./i18n";

describe("UI language", () => {
  it("starts in Chinese", () => {
    expect(language()).toBe("zh");
    expect(t("设置", "Settings")).toBe("设置");
    expect(tr("设置")).toBe("设置");
  });

  it("translates labels and dynamic messages in English", () => {
    setLanguage("en");
    expect(language()).toBe("en");
    expect(t("设置", "Settings")).toBe("Settings");
    expect(tr("已固定 3 个文件夹")).toBe("Pinned 3 folders");
    expect(tr("已合并 2 个重复窗口")).toBe("Merged 2 duplicate windows");
    expect(tr("拖动 我的项目")).toBe("Drag 我的项目");
    expect(tr("无法读取配置：disk error")).toBe("Cannot read config: disk error");
    // Longer phrases must win over short words such as 设置 / 泊岸.
    expect(
      tr(
        "无法读取访达窗口。请在 系统设置 → 隐私与安全性 → 自动化 中允许泊岸控制访达。execution error",
      ),
    ).toBe(
      "Cannot read Finder windows. Allow Berth to control Finder in System Settings → Privacy & Security → Automation. execution error",
    );
  });

  it("does not rewrite unknown user data", () => {
    expect(tr("/Users/me/设计文档")).toBe("/Users/me/设计文档");
  });

  it("switches back to Chinese", () => {
    setLanguage("zh");
    expect(tr("已固定 3 个文件夹")).toBe("已固定 3 个文件夹");
  });
});
