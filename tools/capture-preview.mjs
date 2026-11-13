import { chromium } from '@playwright/test';
import { mkdir } from 'node:fs/promises';
await mkdir('docs/images',{recursive:true});
const browser=await chromium.launch();
const context=await browser.newContext({viewport:{width:340,height:760},colorScheme:'light',recordVideo:{dir:'artifacts/videos',size:{width:340,height:760}}});
const page=await context.newPage();
await page.goto('http://127.0.0.1:5173');
for(const [path,alias] of [['C:\\Projects\\Berth','泊岸 · 源码'],['C:\\Design\\Brand','品牌与设计'],['C:\\Documents\\Notes','灵感笔记']]){
  await page.getByRole('button',{name:'固定文件夹',exact:true}).click();
  await page.getByRole('textbox',{name:'文件夹路径',exact:true}).fill(path);
  await page.getByRole('textbox',{name:'别名 · 可选'}).fill(alias);
  await page.getByRole('dialog').getByRole('button',{name:'固定文件夹',exact:true}).click();
  await page.getByRole('dialog').waitFor({state:'hidden'});
}
await page.locator('.toast').waitFor({state:'hidden'});
await page.screenshot({path:'docs/images/preview-light.png'});
await page.emulateMedia({colorScheme:'dark'});
await page.screenshot({path:'docs/images/preview-dark.png'});
await page.getByRole('textbox',{name:'搜索文件夹'}).fill('设计');
await page.getByRole('button',{name:'品牌与设计',exact:true}).waitFor();
await page.getByRole('textbox',{name:'搜索文件夹'}).fill('');
const video=page.video();await context.close();await video.saveAs('docs/images/demo.webm');await browser.close();
