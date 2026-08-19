import fs from 'node:fs';
import path from 'node:path';
import { chromium } from 'playwright';

const BASE = 'http://localhost:5668';
const EDGE = 'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe';
const PROFILE = path.join(process.env.TEMP, 'pw-edge-profile');
const SHOTS = path.join(process.cwd(), 'tmp_shots');
fs.mkdirSync(SHOTS, { recursive: true });

const results = [];
function record(name, ok, detail = '') {
  results.push({ name, ok });
  console.log(`${ok ? 'PASS' : 'FAIL'} | ${name}${detail ? ' | ' + detail : ''}`);
}

async function shot(page, name) {
  await page.screenshot({ path: path.join(SHOTS, name + '.png'), fullPage: false });
}

async function getMainHeaders(page) {
  return page.locator('.vxe-table--main-wrapper .vxe-table--header th .vxe-cell').allTextContents();
}

async function main() {
  const context = await chromium.launchPersistentContext(PROFILE, {
    executablePath: EDGE,
    headless: true,
    viewport: { width: 1680, height: 950 },
    args: ['--no-first-run', '--no-default-browser-check', '--disable-sync'],
  });
  const page = context.pages()[0] ?? (await context.newPage());
  page.setDefaultTimeout(30000);

  try {
    // ---------- 1. 登录 ----------
    await page.goto(BASE + '/auth/login', { waitUntil: 'domcontentloaded' });
    await page.waitForTimeout(3000);
    await shot(page, '01-login-page');

    const userInput = page.locator('input').first();
    const pwdInput = page.locator('input[type="password"]').first();
    await userInput.fill('admin');
    await pwdInput.fill('admin123');
    await page.locator('button[type="submit"], button:has-text("登录")').first().click();
    await page.waitForTimeout(6000);
    await shot(page, '02-after-login');

    const urlNow = page.url();
    record('AC-LOGIN 管理员登录', !urlNow.includes('/auth/login'), urlNow);

    // ---------- 2. 员工管理页：20列 ----------
    await page.goto(BASE + '/system/admin', { waitUntil: 'domcontentloaded' });
    await page.waitForSelector('.vxe-table--render-wrapper', { timeout: 30000 });
    await page.waitForTimeout(4000);
    await shot(page, '03-user-list');

    const headers = (await getMainHeaders(page)).map((s) => s.trim()).filter(Boolean);
    record('AC-3 管理员默认20列', headers.length === 20, `实际 ${headers.length} 列: ${headers.join(',')}`);

    const hasSensitive = ['手机号', '邮箱', '最后登录IP'].every((t) => headers.includes(t));
    record('AC-3 敏感列(手机号/邮箱/IP)可见', hasSensitive);

    // ---------- 3. 工具栏按钮 ----------
    const btn = page.locator('button', { hasText: '列显示设置' });
    const btnCount = await btn.count();
    record('AC-5 工具栏"列显示设置"按钮', btnCount > 0, `count=${btnCount}`);

    // ---------- 4. 打开抽屉验证结构 ----------
    await btn.first().click();
    await page.waitForSelector('.ant-drawer-content', { timeout: 15000 });
    await page.waitForTimeout(1500);
    await shot(page, '04-drawer-open');

    const drawer = page.locator('.ant-drawer-content');
    const levelLabels = await drawer.locator('.level-label').allTextContents();
    const levelsOk = ['管理员', '人事', '部门负责人', '普通员工'].every((l) =>
      levelLabels.some((t) => t.trim() === l),
    );
    record('AC-6 抽屉四角色分组', levelsOk, levelLabels.join(','));

    const cbCount = await drawer.locator('.column-checkbox').count();
    record('AC-6 每组20列checkbox', cbCount === 80, `count=${cbCount}`);

    // AC-11 操作列禁用：四组中的"操作"checkbox 均应 disabled
    const actionCbs = drawer.locator('.column-checkbox', { hasText: '操作' });
    const actionCount = await actionCbs.count();
    let disabledCount = 0;
    for (let i = 0; i < actionCount; i++) {
      const cls = await actionCbs.nth(i).getAttribute('class');
      if (cls && cls.includes('disabled')) disabledCount++;
    }
    record('AC-11 操作列开关禁用(4组)', actionCount === 4 && disabledCount === 4, `action=${actionCount}, disabled=${disabledCount}`);

    // ---------- 5. AC-7 修改配置：admin 组取消"邮箱" ----------
    const adminBlock = drawer.locator('.level-block').filter({
      has: page.locator('.level-label', { hasText: '管理员' }),
    });
    const emailCb = adminBlock.locator('.column-checkbox', { hasText: '邮箱' }).first();
    const checkedBefore = await emailCb.locator('input').isChecked();
    record('AC-7 前置:邮箱初始勾选', checkedBefore === true, `checked=${checkedBefore}`);
    await emailCb.click();
    await page.waitForTimeout(500);
    await shot(page, '05-email-unchecked');

    await drawer.locator('.footer-bar button', { hasText: '保存' }).first().click();
    await page.waitForTimeout(3000);
    await shot(page, '06-after-save');

    const headers2 = (await getMainHeaders(page)).map((s) => s.trim()).filter(Boolean);
    record('AC-7 保存后列即时生效', headers2.length === 19 && !headers2.includes('邮箱'), `实际 ${headers2.length} 列`);

    // ---------- 6. AC-8 刷新页面持久化 ----------
    await page.reload({ waitUntil: 'domcontentloaded' });
    await page.waitForSelector('.vxe-table--render-wrapper', { timeout: 30000 });
    await page.waitForTimeout(4000);
    await shot(page, '07-after-reload');

    const headers3 = (await getMainHeaders(page)).map((s) => s.trim()).filter(Boolean);
    record('AC-8 刷新后配置持久化', headers3.length === 19 && !headers3.includes('邮箱'), `实际 ${headers3.length} 列`);

    // ---------- 7. 恢复默认 ----------
    const btn2 = page.locator('button', { hasText: '列显示设置' });
    await btn2.first().click();
    await page.waitForSelector('.ant-drawer-content', { timeout: 15000 });
    await page.waitForTimeout(1500);
    const drawer2 = page.locator('.ant-drawer-content');
    const adminBlock2 = drawer2.locator('.level-block').filter({
      has: page.locator('.level-label', { hasText: '管理员' }),
    });
    await adminBlock2.locator('.level-header button', { hasText: '恢复默认' }).first().click();
    await page.waitForTimeout(500);
    await drawer2.locator('.footer-bar button', { hasText: '保存' }).first().click();
    await page.waitForTimeout(3000);
    await shot(page, '08-restored');

    const headers4 = (await getMainHeaders(page)).map((s) => s.trim()).filter(Boolean);
    record('恢复默认后回到20列', headers4.length === 20, `实际 ${headers4.length} 列`);
  } catch (e) {
    record('EXCEPTION 异常中断', false, e.message);
    try { await shot(page, '99-exception'); } catch {}
  } finally {
    await context.close();
  }

  const failed = results.filter((r) => !r.ok);
  console.log(`\n===== E2E 结果: ${results.length - failed.length}/${results.length} PASS =====`);
  if (failed.length) {
    console.log('FAILED:', failed.map((f) => f.name).join(' | '));
  }
  process.exit(failed.length ? 1 : 0);
}

main();
