/**
 * 生成 lucide 图标子集
 *
 * 背景
 * ----
 * 原实现 `import('@iconify/json/json/lucide.json')` 会把完整 lucide 图标集
 * （0.53 MB JSON / 打包后约 545 KB 独立 chunk）拉进首屏关键路径，且 bootstrap.ts
 * 中同步 await 它，位于 createApp(App) 之前，形成串行瀑布：
 *
 *   index → bootstrap → lucide(545KB) → 登录页渲染
 *
 * 本脚本扫描源码中**实际引用**的 lucide 图标名，从 @iconify/json 抽取子集输出到
 * `src/icons/lucide-subset.json`，通常只有几十 KB。既消除了首屏阻塞体量，
 * 又保留了「本地预加载、不依赖远程 CDN」的原始意图。
 *
 * 用法
 * ----
 *   pnpm --filter @vben/web-antd run gen:icons
 *   （build 前会通过 prebuild 钩子自动执行）
 */
import { createRequire } from 'node:module';
import { readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { dirname, extname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const APP_ROOT = resolve(__dirname, '..');

/** 不管是否扫描到都必须保留的图标：动态表名 / 运行时拼接场景的兜底 */
const ALWAYS_INCLUDE = [
  'user',
  'lock',
  'eye',
  'eye-off',
  'loader-circle',
  'circle-check-big',
  'circle-alert',
  'triangle-alert',
  'x',
  'search',
  'menu',
  'log-out',
  'chevron-down',
  'chevron-right',
  'chevron-left',
  'house',
  'settings',
  'bell',
  'sun',
  'moon',
  'languages',
  'expand',
  'shrink',
  'refresh-cw',
  'arrow-left',
  'arrow-right',
  'plus',
  'trash-2',
  'pencil',
  'save',
  'download',
  'upload',
  'file-text',
  'folder',
  'calendar',
  'clock',
  'layout-grid',
  'list',
  'panel-left',
  'check',
  'circle-x',
  'info',
  'lock-keyhole',
  'database',
  'server',
  'shield',
  'key',
  'mail',
  'phone',
  'building',
  'users',
  'wallet',
  'chart-line',
  'chart-pie',
  'shopping-cart',
  'package',
  'truck',
  'credit-card',
  'receipt',
  'braces',
  'code',
  'terminal',
  'bug',
  'wrench',
];

const SCAN_EXT = new Set(['.ts', '.tsx', '.js', '.jsx', '.vue', '.json']);
const SKIP_DIR = new Set([
  'node_modules',
  'dist',
  '.git',
  '.turbo',
  'coverage',
  'build',
  '.vite',
]);

/** 匹配 'lucide:xxx' / "lucide:xxx" / `lucide:xxx` */
const RE_LUCIDE = /['"`]lucide:([a-z0-9-]+)['"`]/g;

function walk(dir, out = []) {
  let entries;
  try {
    entries = readdirSync(dir, { withFileTypes: true });
  } catch {
    return out;
  }
  for (const entry of entries) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) {
      if (SKIP_DIR.has(entry.name)) continue;
      walk(full, out);
    } else if (entry.isFile() && SCAN_EXT.has(extname(entry.name))) {
      out.push(full);
    }
  }
  return out;
}

function collectUsedNames() {
  const roots = [
    join(APP_ROOT, 'src'),
    resolve(APP_ROOT, '../../packages'),
  ];
  const names = new Set(ALWAYS_INCLUDE);

  for (const root of roots) {
    for (const file of walk(root)) {
      let text;
      try {
        text = readFileSync(file, 'utf8');
      } catch {
        continue;
      }
      let match;
      RE_LUCIDE.lastIndex = 0;
      while ((match = RE_LUCIDE.exec(text)) !== null) {
        names.add(match[1]);
      }
    }
  }
  return names;
}

function buildSubset(lucide, names) {
  const icons = {};
  const aliases = {};
  const missing = [];

  for (const name of names) {
    if (lucide.icons?.[name]) {
      icons[name] = lucide.icons[name];
      continue;
    }
    if (lucide.aliases?.[name]) {
      aliases[name] = lucide.aliases[name];
      const parent = lucide.aliases[name].parent;
      if (lucide.icons?.[parent]) icons[parent] = lucide.icons[parent];
      continue;
    }
    missing.push(name);
  }

  const subset = { prefix: lucide.prefix, icons };
  if (lucide.width) subset.width = lucide.width;
  if (lucide.height) subset.height = lucide.height;
  if (Object.keys(aliases).length > 0) subset.aliases = aliases;

  return { subset, missing };
}

function main() {
  const require = createRequire(import.meta.url);
  const lucide = require('@iconify/json/json/lucide.json');

  const names = collectUsedNames();
  const { subset, missing } = buildSubset(lucide, names);

  const outFile = join(APP_ROOT, 'src', 'icons', 'lucide-subset.json');
  writeFileSync(outFile, `${JSON.stringify(subset)}\n`, 'utf8');

  const full = JSON.stringify(lucide).length;
  const part = JSON.stringify(subset).length;
  console.log('[gen:lucide-subset]');
  console.log(`  引用的图标      : ${names.size}`);
  console.log(`  写入 icons      : ${Object.keys(subset.icons).length}`);
  console.log(`  aliases         : ${Object.keys(subset.aliases || {}).length}`);
  console.log(`  全量体积        : ${(full / 1024).toFixed(1)} KB`);
  console.log(`  子集体积        : ${(part / 1024).toFixed(1)} KB`);
  console.log(
    `  压缩比          : ${((1 - part / full) * 100).toFixed(1)}% ↓`,
  );
  if (missing.length) {
    console.log(`  ⚠ 未命中 ${missing.length} 个: ${missing.join(', ')}`);
  }
}

main();
