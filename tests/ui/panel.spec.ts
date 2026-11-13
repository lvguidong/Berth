import { test, expect, Page } from "@playwright/test";
async function add(page: Page, path: string, alias: string) {
  await page.getByRole("button", { name: "固定文件夹", exact: true }).click();
  await page
    .getByRole("textbox", { name: "文件夹路径", exact: true })
    .fill(path);
  await page.getByRole("textbox", { name: "别名 · 可选" }).fill(alias);
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "固定文件夹", exact: true })
    .click();
  await expect(page.getByRole("dialog")).not.toBeVisible();
}
test.beforeEach(async ({ page }) => {
  await page.goto("/");
});
test("empty state → pin → rename → persistence → unpin → undo", async ({
  page,
}) => {
  await expect(
    page.getByRole("heading", { name: "把文件夹拖到这里" }),
  ).toBeVisible();
  await add(page, "C:\\Projects\\Berth", "泊岸源码");
  await page
    .getByRole("button", { name: "编辑 泊岸源码", exact: true })
    .click();
  await page
    .getByRole("textbox", { name: "别名", exact: true })
    .fill("开发目录");
  await page.getByRole("button", { name: "保存", exact: true }).click();
  await page.reload();
  await expect(
    page.getByRole("button", { name: "开发目录", exact: true }),
  ).toBeVisible();
  await page
    .getByRole("button", { name: "编辑 开发目录", exact: true })
    .click();
  await page.getByRole("button", { name: "取消固定", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "把文件夹拖到这里" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "撤销", exact: true }).click();
  await expect(
    page.getByRole("button", { name: "开发目录", exact: true }),
  ).toBeVisible();
});
test("duplicates and relative paths do not add extra pins", async ({
  page,
}) => {
  await add(page, "C:\\Projects", "项目");
  await page.getByRole("button", { name: "固定文件夹", exact: true }).click();
  await page
    .getByRole("textbox", { name: "文件夹路径", exact: true })
    .fill("c:/PROJECTS/");
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "固定文件夹", exact: true })
    .click();
  await expect(page.getByRole("alert")).toContainText("已经固定");
  await page
    .getByRole("textbox", { name: "文件夹路径", exact: true })
    .fill("relative");
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "固定文件夹", exact: true })
    .click();
  await expect(page.getByRole("alert")).toContainText("完整");
});
test("group, search, collapse and remove group preserve folders", async ({
  page,
}) => {
  await add(page, "C:\\Design", "设计资源");
  await page.getByRole("button", { name: "新建分组", exact: true }).click();
  await page.getByRole("textbox", { name: "分组名称" }).fill("工作");
  await page.getByRole("button", { name: "保存", exact: true }).click();
  await page
    .getByRole("button", { name: "编辑 设计资源", exact: true })
    .click();
  await page
    .getByLabel("分组", { exact: true })
    .selectOption({ label: "工作" });
  await page.getByRole("button", { name: "保存", exact: true }).click();
  await page.getByRole("button", { name: /工作 1/ }).click();
  await expect(
    page.getByRole("button", { name: "设计资源", exact: true }),
  ).not.toBeVisible();
  await page.getByRole("textbox", { name: "搜索文件夹" }).fill("design");
  await expect(
    page.getByRole("button", { name: "设计资源", exact: true }),
  ).toBeVisible();
  await page.getByRole("textbox", { name: "搜索文件夹" }).fill("no-match");
  await expect(
    page.getByRole("heading", { name: "没有找到这个文件夹" }),
  ).toBeVisible();
  await page.getByRole("textbox", { name: "搜索文件夹" }).fill("");
  await page
    .getByRole("button", { name: "编辑分组 工作", exact: true })
    .click();
  await page.getByRole("button", { name: "删除分组" }).click();
  await expect(
    page.getByRole("button", { name: "设计资源", exact: true }),
  ).toBeVisible();
});
test("user aliases are rendered as text, and settings persist", async ({
  page,
}) => {
  await add(page, "C:\\Safe", "<img src=x onerror=alert(1)>");
  await expect(page.locator(".pin-name")).toHaveText(
    "<img src=x onerror=alert(1)>",
  );
  await expect(page.locator(".pin-name img")).toHaveCount(0);
  await page.getByRole("button", { name: "设置", exact: true }).click();
  await page.getByLabel("显示方式").selectOption("float");
  await page.getByLabel("登录电脑时启动").check();
  await page.getByRole("button", { name: "保存设置" }).click();
  await page.reload();
  await page.getByRole("button", { name: "设置", exact: true }).click();
  await expect(page.getByLabel("显示方式")).toHaveValue("float");
  await expect(page.getByLabel("登录电脑时启动")).toBeChecked();
});
test("keyboard undo and pointer reorder work", async ({ page }) => {
  await add(page, "C:\\A", "Alpha");
  await add(page, "C:\\B", "Beta");
  await page.locator("[data-drag]").nth(1).hover();
  const from = await page.locator("[data-drag]").nth(1).boundingBox();
  const to = await page.locator("[data-pin]").nth(0).boundingBox();
  await page.mouse.move(from!.x + 5, from!.y + 5);
  await page.mouse.down();
  await page.mouse.move(to!.x + 40, to!.y + 20, { steps: 8 });
  await page.mouse.up();
  await expect(page.locator(".pin-name").first()).toHaveText("Beta");
  await page.keyboard.press("Control+z");
  await expect(page.locator(".pin-name").first()).toHaveText("Alpha");
});
