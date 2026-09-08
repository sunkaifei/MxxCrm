#!/usr/bin/env node
/**
 * 批3: 权限码三处一致性体检（G9 验收项 0 的 CI 化）
 *
 * 对比三处权限码集合：
 *   ① 后端路由守卫 require_permission("X")
 *   ② 数据库菜单 mxx_system_menu.perm（授权来源）
 *   ③ 前端 v-access:code / hasAccessByCodes / hasAccessCode("X")
 *
 * 判定：后端码 ⊆ 菜单 perm；前端码 ⊆ 菜单 perm。任一差集非空 exit 1。
 *
 * 用法：node scripts/check-perm-codes.mjs
 * 说明：后端提取剔除注释行与 #[cfg(test)] 测试代码（约定测试 mod 在文件末尾），
 *       避免文档示例（require_permission("perm")）与单测（test:perm）假码误报。
 */
import { execSync } from 'node:child_process';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '../../../..');
const backendSrc = join(root, 'backend', 'src');
const frontendSrc = join(root, 'frontend', 'apps', 'web-antd', 'src');

function walk(dir, exts, out = []) {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) walk(p, exts, out);
    else if (exts.some((e) => name.endsWith(e))) out.push(p);
  }
  return out;
}

// ① 后端守卫码（剔除注释行与测试代码）
const backendCodes = new Set();
for (const f of walk(backendSrc, ['.rs'])) {
  const src = readFileSync(f, 'utf8');
  const cut = src.indexOf('#[cfg(test)]');
  const codeOnly = cut >= 0 ? src.slice(0, cut) : src;
  const noComments = codeOnly
    .split('\n')
    .filter((l) => !l.trim().startsWith('//'))
    .join('\n');
  for (const m of noComments.matchAll(/require_permission\("([^"]+)"\)/g)) backendCodes.add(m[1]);
}

// ③ 前端判定码
const frontendCodes = new Set();
for (const f of walk(frontendSrc, ['.ts', '.vue'])) {
  const src = readFileSync(f, 'utf8');
  for (const m of src.matchAll(/v-access:code="\[([^\]]+)\]"/g)) {
    for (const c of m[1].matchAll(/'([^']+)'/g)) frontendCodes.add(c[1]);
  }
  for (const m of src.matchAll(/hasAccessByCodes\(\[([^\]]+)\]\)/g)) {
    for (const c of m[1].matchAll(/'([^']+)'/g)) frontendCodes.add(c[1]);
  }
  for (const m of src.matchAll(/hasAccessCode\('([^']+)'\)/g)) frontendCodes.add(m[1]);
}

// ② 数据库菜单 perm
const env = { ...process.env, PGPASSWORD: process.env.PGPASSWORD || '123456' };
const sql = "SELECT DISTINCT perm FROM mxx_system_menu WHERE perm IS NOT NULL AND perm <> '' AND deleted = 0";
let dbCodes = new Set();
try {
  const out = execSync(
    `psql -U postgres -h 127.0.0.1 -d mxxcrm_data -t -A -c ${JSON.stringify(sql)}`,
    { encoding: 'utf8', env },
  );
  dbCodes = new Set(out.split(/\r?\n/).map((s) => s.trim()).filter(Boolean));
} catch (e) {
  console.error('❌ 无法读取数据库菜单权限码：', e.message);
  process.exit(2);
}

const backendMissing = [...backendCodes].filter((c) => !dbCodes.has(c)).sort();
const frontendMissing = [...frontendCodes].filter((c) => !dbCodes.has(c)).sort();

console.log(`后端守卫码: ${backendCodes.size} 个 | 前端判定码: ${frontendCodes.size} 个 | 菜单 perm: ${dbCodes.size} 个`);
if (backendMissing.length) {
  console.error('❌ 后端守卫码缺少对应菜单 perm（无法授权）:', JSON.stringify(backendMissing, null, 2));
}
if (frontendMissing.length) {
  console.error('❌ 前端判定码缺少对应菜单 perm（码不存在）:', JSON.stringify(frontendMissing, null, 2));
}
if (!backendMissing.length && !frontendMissing.length) {
  console.log('✅ 权限码三处一致性核验通过');
} else {
  process.exit(1);
}
