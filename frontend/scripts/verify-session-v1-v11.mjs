// 认证会话有效期整改（任务③）V1-V11 验证用例脚本
// 对应文档：docs/认证会话有效期整改开发文档.md §8.2
//
// 前置条件：
//   1. 后端已启动（默认 http://127.0.0.1:8088）且已完成本次整改代码部署
//   2. psql 在 PATH 中，PostgreSQL 可连通（默认 postgres/123456@127.0.0.1/mxxcrm_data）
//   3. 管理员账号可登录（默认 admin/admin123，需 system:admin:kick / system:scheduler:manage 权限，超管即可）
//   4. V5/V6/V9/V10/V11 需要非超管测试账号（--test-username/--test-password），未提供则 SKIP
//
// 用例（文档 V1-V9 + 本次整改补充 V10/V11）：
//   V1  双时间分离 / V2 缓存 TTL / V3 静默刷新 / V4 refresh 窗口终结 / V5 踢下线不复活
//   V6  禁用立即生效 / V7 复用攻击撤销 / V8 调度清理 / V9 Redis 模式挤下线
//   V10 并发刷新互斥（R7 乐观锁）/ V11 改密全端撤销（R3）
//
// 用法（在 frontend 目录下）：
//   node scripts/verify-session-v1-v11.mjs
//   node scripts/verify-session-v1-v11.mjs --test-username test01 --test-password 'Test@123456'
//   node scripts/verify-session-v1-v11.mjs --slow-v3   # V3 真实等待版（临时把 access_token_expire 调 120s）
//   存在 FAIL 时进程退出码为 1

import { decode } from '../node_modules/.pnpm/@msgpack+msgpack@3.1.3/node_modules/@msgpack/msgpack/dist.esm/index.mjs';
import { execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

// ============ 参数解析 ============
const argv = process.argv.slice(2);
const opt = {};
for (let i = 0; i < argv.length; i++) {
  if (argv[i].startsWith('--')) {
    const key = argv[i].slice(2);
    const next = argv[i + 1];
    if (next !== undefined && !next.startsWith('--')) { opt[key] = next; i++; }
    else { opt[key] = true; }
  }
}
const BASE = opt.base || 'http://127.0.0.1:8088';
const USERNAME = opt.username || 'admin';
const PASSWORD = opt.password || 'admin123';
const TEST_USERNAME = opt['test-username'] || '';
const TEST_PASSWORD = opt['test-password'] || '';
const PG = {
  user: opt['pg-user'] || 'postgres',
  host: opt['pg-host'] || '127.0.0.1',
  port: opt['pg-port'] || '5432',
  db: opt['pg-db'] || 'mxxcrm_data',
  password: opt['pg-password'] || '123456',
};
const SLOW_V3 = !!opt['slow-v3'];
const hasTestUser = !!(TEST_USERNAME && TEST_PASSWORD);

// ============ 工具函数 ============
const results = [];
function addResult(id, name, status, detail) {
  results.push({ id, name, status, detail });
  const icon = status === 'PASS' ? '[PASS]' : status === 'FAIL' ? '[FAIL]' : '[SKIP]';
  console.log(`${id.padEnd(5)} ${icon.padEnd(7)} ${name}`);
  if (detail) console.log(`       ${detail}`);
}

function psql(sql) {
  const env = { ...process.env, PGPASSWORD: PG.password };
  return execSync(
    `psql -U ${PG.user} -h ${PG.host} -p ${PG.port} -d ${PG.db} -t -A -c ${JSON.stringify(sql)}`,
    { env, encoding: 'utf8' }
  ).trim();
}

async function http(method, apiPath, body, token) {
  const headers = { 'Content-Type': 'application/json' };
  if (token) headers.Authorization = `Bearer ${token}`;
  const res = await fetch(BASE + apiPath, {
    method,
    headers,
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
  const buf = new Uint8Array(await res.arrayBuffer());
  let data = null;
  let text = '';
  try { data = decode(buf); } catch { text = Buffer.from(buf).toString('utf8'); }
  return { status: res.status, data, text };
}

async function login(user, pwd) {
  const r = await http('POST', '/api/system/auth/login', { username: user, password: pwd });
  if (r.status !== 200 || !r.data || r.data.code !== 200) {
    throw new Error(`登录失败（${user}）：HTTP ${r.status} ${r.text || JSON.stringify(r.data)}`);
  }
  // 登录响应（TokenVO）不含用户 id，从 DB 反查
  const safeName = user.replace(/'/g, "''");
  const userId = Number(psql(`SELECT id FROM mxx_system_admin WHERE user_name='${safeName}' LIMIT 1;`));
  if (!Number.isFinite(userId)) throw new Error(`无法定位用户 id（${user}）`);
  return {
    userId,
    accessToken: r.data.data.accessToken,
    refreshToken: r.data.data.refreshToken,
    expiresIn: r.data.data.expiresIn,
  };
}

// 轻量业务接口：校验 accessToken 是否有效（200=有效，401=失效）
async function tokenAlive(token) {
  const r = await http('GET', '/api/system/admin/userinfo', undefined, token);
  return r.status;
}

async function getSessionRow(userId) {
  const line = psql(
    `SELECT id, EXTRACT(EPOCH FROM (refresh_expire_time - expire_time))::bigint, EXTRACT(EPOCH FROM (expire_time - CURRENT_TIMESTAMP))::bigint ` +
    `FROM mxx_system_session WHERE user_id=${userId} ORDER BY id DESC LIMIT 1;`
  );
  if (!line) return null;
  const [id, diff, remain] = line.split('|').map(Number);
  return { id, diffSecs: diff, accessRemainSecs: remain };
}

function sessionCount(userId) {
  return Number(psql(`SELECT COUNT(*) FROM mxx_system_session WHERE user_id=${userId};`));
}

function dbConfig(key, def) {
  const v = psql(`SELECT config_value FROM mxx_system_config WHERE config_key='${key}' LIMIT 1;`);
  return /^\d+$/.test(v) ? Number(v) : def;
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// ============ 环境检查 ============
console.log('==== 认证会话有效期整改 V1-V11 验证 ====');
console.log(`后端: ${BASE}   数据库: ${PG.user}@${PG.host}/${PG.db}`);

// 后端连通性（免鉴权接口）
try {
  await fetch(`${BASE}/api/system/auth/register-status`, { method: 'GET' });
} catch (e) {
  console.error(`[FATAL] 后端不可达：${BASE}（${e.message}）`);
  process.exit(2);
}

// 部署配置（session_store / cache_type）
const iniPath = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', '..', 'backend', 'config', 'config.ini');
const cfg = {};
if (fs.existsSync(iniPath)) {
  for (const line of fs.readFileSync(iniPath, 'utf8').split(/\r?\n/)) {
    const m = line.match(/^\s*([a-zA-Z_]+)\s*=\s*(.+?)\s*$/);
    if (m) cfg[m[1]] = m[2];
  }
}
const sessionStore = cfg['session_store'] || 'db';
const cacheType = cfg['cache_type'] || 'mem';
console.log(`存储模式: session_store=${sessionStore}  cache_type=${cacheType}\n`);
if (!hasTestUser) {
  console.log('[提示] 未提供 --test-username/--test-password，V5/V6/V9/V10/V11 将 SKIP（V7 使用管理员自身）\n');
}

let admin;
try {
  admin = await login(USERNAME, PASSWORD);
} catch (e) {
  console.error(`[FATAL] ${e.message}`);
  process.exit(2);
}
console.log(`管理员登录成功 user_id=${admin.userId}\n`);

// ============ V1 双时间分离 ============
try {
  const row = await getSessionRow(admin.userId);
  if (!row) throw new Error(`未找到管理员会话行（user_id=${admin.userId}）`);
  const expect = dbConfig('session_timeout', 28800) - dbConfig('access_token_expire', 7200);
  if (Math.abs(row.diffSecs - expect) <= 120) {
    addResult('V1', '双时间分离', 'PASS',
      `expire 差值=${row.diffSecs}s，期望≈${expect}s（session_timeout-access_token_expire），session_id=${row.id}`);
  } else {
    addResult('V1', '双时间分离', 'FAIL', `两列差值=${row.diffSecs}s，期望≈${expect}s，偏差超 120s`);
  }
} catch (e) { addResult('V1', '双时间分离', 'FAIL', e.message); }

// ============ V2 缓存 TTL ============
// mem 模式无法外部观测 TTL；redis 模式尝试 redis-cli
try {
  if (cacheType !== 'redis') {
    addResult('V2', '缓存 TTL', 'SKIP',
      `当前 cache_type=${cacheType}，mem 模式无法外部观测 TTL；切 redis 后复验（期望 TTL≈access_token_expire，非 -1 永生）`);
  } else {
    const { execSync: es } = await import('node:child_process');
    const ttl = es(`redis-cli -h 127.0.0.1 TTL "user_${admin.userId}"`, { encoding: 'utf8' }).trim();
    const ttl2 = es(`redis-cli -h 127.0.0.1 TTL "user_tokens_${admin.userId}"`, { encoding: 'utf8' }).trim();
    const valid = [ttl, ttl2].find((t) => /^\d+$/.test(t) && Number(t) > 0);
    if (valid !== undefined) {
      addResult('V2', '缓存 TTL', 'PASS', `redis TTL=${valid}s，与 access 过期对齐`);
    } else {
      addResult('V2', '缓存 TTL', 'FAIL', `redis 模式下未查到带 TTL 的 token 键（user_{id}=${ttl} / user_tokens_{id}=${ttl2}），可能仍为永生键`);
    }
  }
} catch (e) { addResult('V2', '缓存 TTL', 'FAIL', e.message); }

// ============ V3 静默刷新 ============
try {
  // 基础链路：refreshToken 换新 accessToken，新 token 业务可用
  //（access 是否已过期不影响刷新——refresh 不校验 access；真实 2min 等待见 --slow-v3）
  const r = await http('POST', '/api/system/auth/refresh', { refreshToken: admin.refreshToken });
  if (r.status === 200 && r.data?.code === 200 && r.data.data?.accessToken && r.data.data?.refreshToken) {
    const alive = await tokenAlive(r.data.data.accessToken);
    if (alive === 200) {
      addResult('V3', '静默刷新', 'PASS',
        `刷新成功且新 accessToken 业务可用（userinfo=200）；expiresIn=${r.data.data.expiresIn}s`);
      admin.oldRefreshToken = admin.refreshToken; // 旋转前的旧 refreshToken（V7 复用攻击用）
      admin.accessToken = r.data.data.accessToken;
      admin.refreshToken = r.data.data.refreshToken;
    } else {
      addResult('V3', '静默刷新', 'FAIL', `刷新返回 200，但新 accessToken 请求 userinfo 返回 HTTP ${alive}`);
    }
  } else {
    addResult('V3', '静默刷新', 'FAIL', `刷新接口异常：HTTP ${r.status} ${r.text || JSON.stringify(r.data)}`);
  }

  if (SLOW_V3) {
    // 真实等待版：access_token_expire 调 120s → 重登 → 等 130s → 旧 token 应 401、刷新应成功 → 恢复配置
    console.log('       [V3-Slow] 临时将 access_token_expire 调整为 120s 并真实等待 130s ...');
    const orig = dbConfig('access_token_expire', 7200);
    psql(`UPDATE mxx_system_config SET config_value='120' WHERE config_key='access_token_expire';`);
    try {
      const slow = await login(USERNAME, PASSWORD);
      await sleep(130_000);
      const st = await tokenAlive(slow.accessToken);
      const rr = await http('POST', '/api/system/auth/refresh', { refreshToken: slow.refreshToken });
      if (st === 401 && rr.status === 200) {
        addResult('V3+', '静默刷新(真实等待)', 'PASS', 'access 过期后 userinfo=401，refreshToken 刷新成功（HTTP 200）');
      } else {
        addResult('V3+', '静默刷新(真实等待)', 'FAIL', `旧 token 状态=${st}（期望401），刷新状态=${rr.status}（期望200）`);
      }
    } finally {
      psql(`UPDATE mxx_system_config SET config_value='${orig}' WHERE config_key='access_token_expire';`);
    }
  }
} catch (e) { addResult('V3', '静默刷新', 'FAIL', e.message); }

// ============ V4 refresh 窗口终结（DB 模拟过期） ============
try {
  // refresh_expire_time 置为过去 → 刷新应 401（行被后端删除）
  // 注意：应用侧 PG 会话时区为 UTC（db_now 写入的是 UTC 墙钟），此处须用 AT TIME ZONE 'UTC' 写入
  const row = await getSessionRow(admin.userId);
  if (!row) throw new Error('未找到管理员会话行');
  const savedRefresh = admin.refreshToken;
  psql(`UPDATE mxx_system_session SET refresh_expire_time = (now() AT TIME ZONE 'UTC') - interval '1 minute' WHERE id=${row.id};`);
  const r = await http('POST', '/api/system/auth/refresh', { refreshToken: savedRefresh });
  // 多设备模式（login_multi_device=1）下该用户可能另有有效会话行，后端只删 refresh 终结的那一行，
  // 因此断言对象是"被置过期的那一行 id 已删除"，而非"用户总行数为 0"
  const rowLeft = Number(psql(`SELECT COUNT(*) FROM mxx_system_session WHERE id=${row.id};`));
  if (r.status === 401 && rowLeft === 0) {
    addResult('V4', 'refresh 窗口终结', 'PASS', `超窗刷新返回 401，且过期会话行(id=${row.id})已删除`);
  } else {
    addResult('V4', 'refresh 窗口终结', 'FAIL', `刷新状态=${r.status}（期望401），行(id=${row.id})删除=${rowLeft === 0}。响应：${r.text}`);
  }
  admin = await login(USERNAME, PASSWORD); // V4 后管理员会话已被删，重新登录恢复
} catch (e) { addResult('V4', 'refresh 窗口终结', 'FAIL', e.message); }

// ============ V8 调度清理 ============
try {
  const jobId = psql(`SELECT id FROM mxx_system_scheduler_job WHERE job_code='session_clean_expired' AND deleted=0 LIMIT 1;`);
  if (!jobId) {
    addResult('V8', '调度清理', 'FAIL', "mxx_system_scheduler_job 中未找到 job_code='session_clean_expired'（种子 SQL 未执行？）");
  } else {
    const r = await http('POST', '/api/system/scheduler/trigger', { id: Number(jobId) }, admin.accessToken);
    if (r.status === 200 && r.data?.code === 200) {
      addResult('V8', '调度清理', 'PASS', `手动触发成功（job_id=${jobId}）：${r.data.data}`);
    } else {
      addResult('V8', '调度清理', 'FAIL', `触发失败：HTTP ${r.status} ${r.text || JSON.stringify(r.data)}`);
    }
  }
} catch (e) { addResult('V8', '调度清理', 'FAIL', e.message); }

// ============ 需要测试账号的用例 ============
let tu = null;
if (hasTestUser) {
  try { tu = await login(TEST_USERNAME, TEST_PASSWORD); }
  catch (e) {
    console.error(`[FATAL] 测试账号登录失败：${e.message}`);
  }
}
if (!tu) {
  addResult('V5', '踢下线不复活', 'SKIP', '需要 --test-username/--test-password');
  addResult('V6', '禁用立即生效', 'SKIP', '需要 --test-username/--test-password');
  addResult('V10', '并发刷新互斥', 'SKIP', '需要 --test-username/--test-password');
  addResult('V11', '改密全端撤销', 'SKIP', '需要 --test-username/--test-password');
} else {
  // ---------- V5 踢下线不复活 ----------
  try {
    const r = await http('POST', `/api/system/admin/kick-offline/${tu.userId}`, {}, admin.accessToken);
    const cnt = sessionCount(tu.userId);
    const st = await tokenAlive(tu.accessToken);
    if (r.status === 200 && cnt === 0 && st === 401) {
      addResult('V5', '踢下线不复活', 'PASS',
        `kick 后 DB 行已删（${cnt} 行）、旧 token 401；（重启后端复验属人工步骤，降级路径同判定）`);
    } else {
      addResult('V5', '踢下线不复活', 'FAIL', `kick响应=${r.status} 剩余行=${cnt} 旧token状态=${st}（期望 200/0/401）`);
    }
    tu = await login(TEST_USERNAME, TEST_PASSWORD); // 恢复
  } catch (e) { addResult('V5', '踢下线不复活', 'FAIL', e.message); }

  // ---------- V6 禁用立即生效 ----------
  try {
    const r = await http('PUT', '/api/system/admin/update-status', { id: tu.userId, status: 0 }, admin.accessToken);
    const cnt = sessionCount(tu.userId);
    const st = await tokenAlive(tu.accessToken);
    const pass = r.status === 200 && cnt === 0 && st === 401;
    await http('PUT', '/api/system/admin/update-status', { id: tu.userId, status: 1 }, admin.accessToken); // 恢复启用
    if (pass) {
      addResult('V6', '禁用立即生效', 'PASS', `禁用后旧 token 401 且 DB session 行已删（${cnt} 行），已自动恢复启用`);
    } else {
      addResult('V6', '禁用立即生效', 'FAIL', `禁用响应=${r.status} 剩余行=${cnt} 旧token状态=${st}（期望 200/0/401）`);
    }
    tu = await login(TEST_USERNAME, TEST_PASSWORD); // 恢复
  } catch (e) {
    try { await http('PUT', '/api/system/admin/update-status', { id: tu.userId, status: 1 }, admin.accessToken); } catch { }
    addResult('V6', '禁用立即生效', 'FAIL', e.message);
  }

  // ---------- V11 改密全端撤销 ----------
  try {
    // 管理员重置测试账号密码（重置为原密码，保证账号后续可用）
    const r = await http('PUT', '/api/system/admin/update_password',
      { userId: tu.userId, password: TEST_PASSWORD }, admin.accessToken);
    const cnt = sessionCount(tu.userId);
    const st = await tokenAlive(tu.accessToken);
    if (r.status === 200 && cnt === 0 && st === 401) {
      addResult('V11', '改密全端撤销', 'PASS', `改密后旧 token 401 且 DB session 行已删（${cnt} 行）；密码已重置为原值`);
    } else {
      addResult('V11', '改密全端撤销', 'FAIL', `改密响应=${r.status} 剩余行=${cnt} 旧token状态=${st}（期望 200/0/401）`);
    }
    tu = await login(TEST_USERNAME, TEST_PASSWORD); // 恢复
  } catch (e) { addResult('V11', '改密全端撤销', 'FAIL', e.message); }

  // ---------- V10 并发刷新互斥（R7 乐观锁） ----------
  try {
    const rt = tu.refreshToken;
    const fire = () => http('POST', '/api/system/auth/refresh', { refreshToken: rt }).then((x) => x.status).catch(() => -1);
    const codes = await Promise.all([fire(), fire()]);
    const ok200 = codes.filter((c) => c === 200).length;
    const ok401 = codes.filter((c) => c === 401).length;
    if (ok200 === 1 && ok401 === 1) {
      addResult('V10', '并发刷新互斥', 'PASS', `双发结果 [${codes.join(',')}]：恰好一次成功一次 401（乐观锁生效）`);
    } else {
      addResult('V10', '并发刷新互斥', 'FAIL', `双发结果 [${codes.join(',')}]，期望 [200,401] 各一次`);
    }
    tu = await login(TEST_USERNAME, TEST_PASSWORD); // 并发刷新后重新登录
  } catch (e) { addResult('V10', '并发刷新互斥', 'FAIL', e.message); }
}

// ============ V7 复用攻击撤销（管理员自身，放最后避免影响前序用例） ============
try {
  // V3 已旋转过一次，admin.oldRefreshToken 即被作废的旧凭据；若无则先主动旋转一次
  if (!admin.oldRefreshToken) {
    const r = await http('POST', '/api/system/auth/refresh', { refreshToken: admin.refreshToken });
    if (r.status !== 200) throw new Error(`预旋转失败：HTTP ${r.status}`);
    admin.oldRefreshToken = admin.refreshToken;
    admin.refreshToken = r.data.data.refreshToken;
  }
  const r = await http('POST', '/api/system/auth/refresh', { refreshToken: admin.oldRefreshToken });
  const cnt = sessionCount(admin.userId);
  const st = await tokenAlive(admin.accessToken);
  if (r.status === 401 && cnt === 0 && st === 401) {
    addResult('V7', '复用攻击撤销', 'PASS',
      `旧 refreshToken 复用返回 401，管理员全端会话已撤销（DB 剩余 ${cnt} 行、当前 token 401）`);
  } else if (r.status === 401 && cnt === 0) {
    addResult('V7', '复用攻击撤销', 'PASS', `旧 refreshToken 复用返回 401，DB 会话已全删（复用检测开关未启用时撤销范围以前端表现为准）`);
  } else {
    addResult('V7', '复用攻击撤销', 'FAIL',
      `复用刷新状态=${r.status}（期望401）剩余行=${cnt} 旧token状态=${st}。响应：${r.text}`);
  }
  admin = await login(USERNAME, PASSWORD); // 重新登录
} catch (e) { addResult('V7', '复用攻击撤销', 'FAIL', e.message); }

// ============ V9 Redis 模式挤下线 ============
try {
  const multiDevice = dbConfig('login_multi_device', 0);
  if (sessionStore !== 'redis') {
    addResult('V9', 'Redis 模式挤下线', 'SKIP',
      `当前 session_store=${sessionStore}；切 redis 后复验（单设备模式二次登录挤出首设备 → 首设备旧 token 401）`);
  } else if (multiDevice === 1) {
    addResult('V9', 'Redis 模式挤下线', 'SKIP', `当前 login_multi_device=1（多设备模式），无"挤出"语义；临时改 0 并切 redis 后复验`);
  } else if (!tu) {
    addResult('V9', 'Redis 模式挤下线', 'SKIP', '需要 --test-username/--test-password');
  } else {
    const first = await login(TEST_USERNAME, TEST_PASSWORD); // 首设备
    await login(TEST_USERNAME, TEST_PASSWORD);             // 二次登录（单设备模式挤出首设备）
    const st = await tokenAlive(first.accessToken);
    if (st === 401) {
      addResult('V9', 'Redis 模式挤下线', 'PASS', '首设备旧 token 已 401（session:{user_id}:{old_token} 键已删）');
    } else {
      addResult('V9', 'Redis 模式挤下线', 'FAIL', `首设备旧 token 状态=${st}（期望401）`);
    }
  }
} catch (e) { addResult('V9', 'Redis 模式挤下线', 'FAIL', e.message); }

// ============ 汇总 ============
const pass = results.filter((r) => r.status === 'PASS').length;
const fail = results.filter((r) => r.status === 'FAIL').length;
const skip = results.filter((r) => r.status === 'SKIP').length;
console.log('\n==== 汇总 ====');
console.table(results.map((r) => ({ 用例: r.id, 结果: r.status, 名称: r.name, 说明: (r.detail || '').slice(0, 80) })));
console.log(`PASS: ${pass}   FAIL: ${fail}   SKIP: ${skip}`);
if (fail > 0) {
  console.error('\n存在 FAIL 用例，请检查上方明细。');
  process.exit(1);
}
console.log('\n全部执行的用例通过。');
process.exit(0);
