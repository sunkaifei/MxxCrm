// ============================================================================
// i18n 校验门禁（三道闸）：防止「引用了不存在的语言包键」复发
//   1. zh-CN 与 en-US 键集合对称（按文件逐一比对，缺文件/缺键都算错）
//   2. 源码中 $t('xxx') 字面量引用的键必须在中英文都存在（镜像运行时合并口径）
//   3. 数据库菜单/权限节点 name 中的 i18n 键必须在中英文都存在
//      （连库校验，连接串取 backend/config/config.ini；库不可达时降级为警告）
// 用法：node scripts/check-i18n.mjs [--skip-db]
// 聚合入口：frontend/ 下 pnpm run check（含本脚本，见 check:i18n）
// 说明：输出/注释统一中文；退出码非 0 即存在错误，CI 与本地均应拦截
// ============================================================================
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const LOCALES_DIR = ['src/locales/langs', 'apps/web-antd/src/locales/langs']
  .map((p) => {
    // 兼容从 app 目录或仓库根运行：从 cwd 向上查找
    let dir = process.cwd();
    for (let i = 0; i < 6; i++) {
      const candidate = path.join(dir, p);
      if (fs.existsSync(candidate)) return candidate;
      dir = path.dirname(dir);
    }
    return null;
  })
  .find(Boolean);
if (!LOCALES_DIR) {
  console.error('[i18n] 未找到 src/locales/langs 目录，请在 apps/web-antd 或仓库根下运行');
  process.exit(1);
}
function findUp(rel) {
  let dir = process.cwd();
  for (let i = 0; i < 8; i++) {
    const candidate = path.join(dir, rel);
    if (fs.existsSync(candidate)) return candidate;
    dir = path.dirname(dir);
  }
  return null;
}
const SRC_DIR = path.join(LOCALES_DIR, '../..');
const CONFIG_INI = findUp('backend/config/config.ini');
const SKIP_DB = process.argv.includes('--skip-db');

const LOCALES = ['zh-CN', 'en-US'];
// 与运行时 translateName 一致：目录级 key 允许 .title 兜底
const KEY_RE = /^[a-zA-Z][\w-]*(\.[a-zA-Z][\w-]*)+$/;

let errors = 0;
let warnings = 0;
const err = (msg) => { errors++; console.log(`  ❌ ${msg}`); };
const warn = (msg) => { warnings++; console.log(`  ⚠️  ${msg}`); };

// ---------- 运行时合并口径复刻（locales/index.ts：page-* 深度合并进 page 命名空间） ----------
function deepMerge(target, source) {
  for (const [k, v] of Object.entries(source)) {
    if (v && typeof v === 'object' && !Array.isArray(v) && target[k] && typeof target[k] === 'object' && !Array.isArray(target[k])) {
      deepMerge(target[k], v);
    } else {
      target[k] = v;
    }
  }
  return target;
}

function loadLocale(locale) {
  const dir = path.join(LOCALES_DIR, locale);
  const files = fs.readdirSync(dir).filter((f) => f.endsWith('.json'));
  // 运行时完整消息 = vben 核心语言包（packages/locales/src/langs）打底，应用语言包覆盖合并
  const coreDir = findUp('packages/locales/src/langs');
  const merged = {};
  if (coreDir && fs.existsSync(path.join(coreDir, locale))) {
    for (const f of fs.readdirSync(path.join(coreDir, locale)).filter((x) => x.endsWith('.json'))) {
      merged[f.slice(0, -5)] = JSON.parse(fs.readFileSync(path.join(coreDir, locale, f), 'utf8'));
    }
  }
  for (const f of files) {
    const content = JSON.parse(fs.readFileSync(path.join(dir, f), 'utf8'));
    if (f.startsWith('page-')) {
      merged.page = merged.page || {};
      deepMerge(merged.page, content);
    } else {
      // 与运行时 mergeLocaleMessage 一致：同名文件深度合并覆盖核心包
      merged[f.slice(0, -5)] = deepMerge(merged[f.slice(0, -5)] || {}, content);
    }
  }
  return { files, merged };
}

function walk(obj, segs) {
  let cur = obj;
  for (const s of segs) {
    if (cur && typeof cur === 'object' && s in cur) cur = cur[s];
    else return undefined;
  }
  return cur;
}

// 键是否存在：叶子命中，或目录级键带 .title 兜底（与角色权限树 translateName 同口径）
function keyExists(merged, key) {
  const direct = walk(merged, key.split('.'));
  if (direct !== undefined && (typeof direct !== 'object' || direct === null)) return true;
  const withTitle = walk(merged, [...key.split('.'), 'title']);
  return withTitle !== undefined && (typeof withTitle !== 'object' || withTitle === null);
}

function collectKeys(obj, prefix = '') {
  const keys = [];
  for (const [k, v] of Object.entries(obj || {})) {
    const full = prefix ? `${prefix}.${k}` : k;
    if (v && typeof v === 'object' && !Array.isArray(v)) keys.push(...collectKeys(v, full));
    else keys.push(full);
  }
  return keys;
}

console.log('▶ 闸门1/3：中英文键集合对称性');
const locales = {};
for (const locale of LOCALES) {
  if (!fs.existsSync(path.join(LOCALES_DIR, locale))) {
    console.error(`  ❌ 缺少语言目录 ${locale}`);
    process.exit(1);
  }
  locales[locale] = loadLocale(locale);
}
const [zh, en] = [locales['zh-CN'], locales['en-US']];
const allFiles = [...new Set([...zh.files, ...en.files])];
for (const f of allFiles) {
  if (!zh.files.includes(f)) { err(`${f} 仅存在于 en-US，zh-CN 缺文件`); continue; }
  if (!en.files.includes(f)) { err(`${f} 仅存在于 zh-CN，en-US 缺文件`); continue; }
  const zhKeys = new Set(collectKeys(JSON.parse(fs.readFileSync(path.join(LOCALES_DIR, 'zh-CN', f), 'utf8'))));
  const enKeys = new Set(collectKeys(JSON.parse(fs.readFileSync(path.join(LOCALES_DIR, 'en-US', f), 'utf8'))));
  for (const k of zhKeys) if (!enKeys.has(k)) err(`${f} en-US 缺键：${k}`);
  for (const k of enKeys) if (!zhKeys.has(k)) err(`${f} zh-CN 缺键：${k}`);
}
if (errors === 0) console.log('  ✅ 对称');

console.log('▶ 闸门2/3：源码 $t() 引用检查');
let refCount = 0;
function scanDir(dir) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, entry.name);
    if (entry.isDirectory()) { scanDir(p); continue; }
    if (!/\.(vue|ts|tsx)$/.test(entry.name)) continue;
    const text = fs.readFileSync(p, 'utf8');
    for (const m of text.matchAll(/\$t\(\s*['"]([a-zA-Z][\w.-]+)['"]/g)) {
      refCount++;
      const key = m[1];
      for (const locale of LOCALES) {
        if (!keyExists(locales[locale].merged, key)) {
          err(`[闸门2] ${path.relative(SRC_DIR, p)} 引用的键在 ${locale} 不存在：${key}`);
        }
      }
    }
  }
}
scanDir(SRC_DIR);
if (errors === 0) console.log(`  ✅ 已扫描 ${refCount} 处引用，均存在`);

console.log(`▶ 闸门3/3：数据库菜单/权限节点 name 检查${SKIP_DB ? '（--skip-db 跳过）' : ''}`);
if (SKIP_DB) {
  warn('已跳过数据库校验，菜单 name 是否有翻译未验证');
} else {
  let dbUrl = process.env.I18N_DB_URL || '';
  if (!dbUrl && fs.existsSync(CONFIG_INI)) {
    const ini = fs.readFileSync(CONFIG_INI, 'utf8');
    const m = ini.match(/^url=(postgres:\/\/[^\s#]+)$/m);
    if (m) [dbUrl] = [m[1]];
  }
  if (!dbUrl) {
    warn('未找到数据库连接串（backend/config/config.ini 或环境变量 I18N_DB_URL），跳过菜单校验');
  } else {
    try {
      const u = new URL(dbUrl);
      const out = execFileSync(
        'psql',
        ['-U', u.username, '-h', u.hostname, '-p', u.port || '5432', '-d', u.pathname.slice(1), '-t', '-A', '-c',
          "SELECT DISTINCT name FROM mxx_system_menu WHERE deleted = 0 AND name ~ '^[a-zA-Z][a-zA-Z0-9_-]*(\\.[a-zA-Z][a-zA-Z0-9_-]*)+$'"],
        { env: { ...process.env, PGPASSWORD: decodeURIComponent(u.password || '') }, encoding: 'utf8' },
      );
      const names = out.split('\n').map((s) => s.trim()).filter(Boolean);
      let menuErrs = 0;
      for (const name of names) {
        for (const locale of LOCALES) {
          if (!keyExists(locales[locale].merged, name)) {
            menuErrs++;
            err(`[闸门3] 菜单 name 在 ${locale} 无翻译：${name}`);
          }
        }
      }
      if (menuErrs === 0) console.log(`  ✅ 已校验 ${names.length} 个菜单键名，均有中英文翻译`);
    } catch (e) {
      warn(`数据库不可达或 psql 缺失，跳过菜单校验：${String(e.message).split('\n')[0]}`);
    }
  }
}

// ---------- 结构约定提示（仅警告，不拦截） ----------
console.log('▶ 结构约定提示');
for (const locale of LOCALES) {
  for (const f of locales[locale].files.filter((x) => x.startsWith('page-'))) {
    const tops = Object.keys(JSON.parse(fs.readFileSync(path.join(LOCALES_DIR, locale, f), 'utf8')));
    const expected = f.slice(5, -5);
    if (!(tops.length === 1 && tops[0] === expected)) {
      warn(`${locale}/${f} 顶层键为 ${tops.join(',')}（与文件名不一致，历史结构不迁移；新增键请放到 page-{模块}.json 且顶层键与文件名一致）`);
    }
  }
}

console.log('──────────────────────────────');
if (errors > 0) {
  console.log(`❌ i18n 校验失败：${errors} 个错误${warnings ? `，${warnings} 个警告` : ''}`);
  process.exit(1);
}
console.log(`✅ i18n 校验通过${warnings ? `（${warnings} 个警告）` : ''}`);
