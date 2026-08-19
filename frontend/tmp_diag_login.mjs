import { chromium } from 'playwright';

const BASE = 'http://localhost:5668';
const EDGE = 'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe';
const PROFILE = process.env.TEMP + '\\pw-edge-profile-diag';

async function main() {
  const context = await chromium.launchPersistentContext(PROFILE, {
    executablePath: EDGE,
    headless: true,
    viewport: { width: 1680, height: 950 },
    args: ['--no-first-run', '--no-default-browser-check', '--disable-sync'],
  });
  const page = context.pages()[0] ?? (await context.newPage());
  page.setDefaultTimeout(30000);

  const consoleMsgs = [];
  page.on('console', (m) => consoleMsgs.push(`[${m.type()}] ${m.text().slice(0, 200)}`));
  page.on('requestfailed', (r) => consoleMsgs.push(`[reqfail] ${r.url()} ${r.failure()?.errorText}`));

  await page.goto(BASE + '/auth/login', { waitUntil: 'domcontentloaded' });
  await page.waitForTimeout(3000);

  // 打印登录页表单结构
  const inputs = await page.locator('input').all();
  for (let i = 0; i < inputs.length; i++) {
    const inp = inputs[i];
    const type = await inp.getAttribute('type');
    const ph = await inp.getAttribute('placeholder');
    console.log(`input[${i}] type=${type} placeholder=${ph}`);
  }
  const btns = await page.locator('button').allTextContents();
  console.log('buttons:', JSON.stringify(btns.filter(Boolean)));
  const bodyText = await page.locator('body').innerText();
  console.log('--- body text (first 600) ---');
  console.log(bodyText.slice(0, 600));

  // 尝试登录
  await page.locator('input').first().fill('admin');
  await page.locator('input[type="password"]').first().fill('admin123');
  await page.locator('button[type="submit"]').first().click();
  await page.waitForTimeout(6000);

  console.log('--- after login url:', page.url());
  const bodyText2 = await page.locator('body').innerText();
  console.log('--- body text after login (first 800) ---');
  console.log(bodyText2.slice(0, 800));

  console.log('--- console messages (last 30) ---');
  console.log(consoleMsgs.slice(-30).join('\n'));

  await context.close();
}

main().catch((e) => { console.error('DIAG ERROR:', e.message); process.exit(1); });
