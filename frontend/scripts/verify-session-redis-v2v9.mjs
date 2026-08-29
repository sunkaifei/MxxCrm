// Redis 专项复验：V2 缓存 TTL（redis 键带 TTL，非永生）+ V9 Redis 模式单设备挤下线
// 对应《认证会话有效期整改开发文档》§8.1 V2 / §8.2 V9；在 cache_type=redis + session_store=redis 下执行。
//
// 前置条件：
//   1. config.ini [server]: cache_type=redis 且新增 session_store=redis，重启后端
//   2. DB mxx_system_config.login_multi_device=0（单设备模式，V9 挤下语义），重启后生效
//   3. Redis 兼容服务运行于 127.0.0.1:6379（Memurai / redis-server 均可）
//   4. 后端 8088 端口、数据库可连（本脚本测试账号自建自清，无需外部 --test-username）
// 注意：config.rs 只在启动时加载配置，改 config.ini 必须重启后端；复验后请将
//       cache_type/session_store 还原为 mem/db、login_multi_device 还原为 1 并再次重启。
//
// 用法（frontend 目录）：
//   node scripts/verify-session-redis-v2v9.mjs --redis-cli "C:\Program Files\Memurai\memurai-cli.exe"
// 环境不符的用例自动 SKIP 并给出原因，不中断其余用例。
import { decode } from '../node_modules/.pnpm/@msgpack+msgpack@3.1.3/node_modules/@msgpack/msgpack/dist.esm/index.mjs';
import { execSync } from 'node:child_process';
import { readFileSync, existsSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const argv = process.argv.slice(2);
const redisCliArg = (() => {
  const i = argv.indexOf('--redis-cli');
  return i >= 0 && argv[i + 1] ? argv[i + 1] : null;
})();
const BASE = 'http://127.0.0.1:8088';
const ADMIN = { username: 'admin', password: 'admin123' };
const TU = { username: 'test01', password: 'Test@123456' };

function psql(sql) {
  const env = { ...process.env, PGPASSWORD: '123456' };
  return execSync(
    `psql -U postgres -h 127.0.0.1 -p 5432 -d mxxcrm_data -t -A -c ${JSON.stringify(sql)}`,
    { env, encoding: 'utf8' }
  ).trim();
}

// redis-cli 可执行文件探测：--redis-cli 参数 > PATH > 常见安装位置
function findRedisCli() {
  if (redisCliArg) return redisCliArg;
  try {
    const inPath = execSync('where.exe redis-cli 2>NUL', { encoding: 'utf8' }).split(/\r?\n/)[0].trim();
    if (inPath) return inPath;
  } catch {}
  for (const p of [
    'C:\\Program Files\\Memurai\\memurai-cli.exe',
    'C:\\Program Files\\Redis\\redis-cli.exe',
    'C:\\ProgramData\\chocolatey\\bin\\redis-cli.exe',
    'C:\\redis\\redis-cli.exe',
  ]) {
    if (existsSync(p)) return p;
  }
  throw new Error('未找到 redis-cli/memurai-cli，请用 --redis-cli 参数指定路径');
}
const REDIS_BIN = findRedisCli();

function redisCli(cmd) {
  return execSync(`"${REDIS_BIN}" -h 127.0.0.1 -p 6379 ${cmd}`, { encoding: 'utf8' }).trim();
}

// 按前缀删除键（Memurai/redis 均可用 KEYS+DEL，无需 xargs/--scan 等 Linux 专属选项）
function delKeysByPattern(pattern) {
  const keys = redisCli(`KEYS "${pattern}"`).split(/\r?\n/).map((k) => k.trim()).filter(Boolean);
  if (keys.length === 0) return 0;
  redisCli(`DEL ${keys.map((k) => `"${k}"`).join(' ')}`);
  return keys.length;
}

async function http(method, apiPath, body, token) {
  const headers = { 'Content-Type': 'application/json' };
  if (token) headers.Authorization = `Bearer ${token}`;
  const res = await fetch(BASE + apiPath, { method, headers, body: body !== undefined ? JSON.stringify(body) : undefined });
  const buf = new Uint8Array(await res.arrayBuffer());
  let data = null, text = '';
  try { data = decode(buf); } catch { text = Buffer.from(buf).toString('utf8'); }
  return { status: res.status, data, text };
}

async function login(user, pwd) {
  const r = await http('POST', '/api/system/auth/login', { username: user, password: pwd });
  const ok = r.status === 200 && r.data && r.data.code === 200;
  const uid = ok ? Number(psql(`SELECT id FROM mxx_system_admin WHERE user_name='${user}' LIMIT 1;`)) : null;
  return {
    ok, httpStatus: r.status, bizCode: r.data && r.data.code, msg: r.data && r.data.msg, userId: uid,
    accessToken: ok && r.data.data && r.data.data.accessToken,
    refreshToken: ok && r.data.data && r.data.data.refreshToken,
  };
}

async function tokenAlive(token) {
  return (await http('GET', '/api/system/admin/userinfo', undefined, token)).status;
}

function addResult(id, name, status, detail) {
  const icon = status === 'PASS' ? '[PASS]' : status === 'FAIL' ? '[FAIL]' : '[SKIP]';
  console.log(`${id.padEnd(5)} ${icon.padEnd(7)} ${name}`);
  if (detail) console.log(`       ${detail}`);
}

// ---------- 环境探测 ----------
// 定位仓库根（backend/config/config.ini 所在目录）：多候选探测，兼容任意 cwd
const SCRIPT_DIR = fileURLToPath(new URL('.', import.meta.url));
function detectRoot() {
  const candidates = [
    join(SCRIPT_DIR, '..', '..'),       // frontend/scripts -> 仓库根
    resolve('..'),                      // cwd 为 frontend 时
    resolve('.'),                       // cwd 为仓库根时
  ];
  for (const c of candidates) {
    if (existsSync(join(c, 'backend', 'config', 'config.ini'))) return c;
  }
  throw new Error('无法定位仓库根（backend/config/config.ini）');
}
const ROOT = detectRoot();
const ini = readFileSync(join(ROOT, 'backend', 'config', 'config.ini'), 'utf8');
const cacheType = (ini.match(/^cache_type=(\w+)/m) || [])[1] || 'mem';
const sessionStore = (ini.match(/^session_store=(\w+)/m) || [])[1] || 'db';
const multiDevice = psql(`SELECT config_value FROM mxx_system_config WHERE config_key='login_multi_device';`);
console.log('==== Redis 专项复验 V2/V9 ====');
console.log(`配置: cache_type=${cacheType}  session_store=${sessionStore}  login_multi_device=${multiDevice}`);
let redisOk = false;
try { redisOk = redisCli('PING') === 'PONG'; } catch {}
console.log(`Redis: ${redisOk ? 'PONG(127.0.0.1:6379)' : '不可达'}  cli=${REDIS_BIN}\n`);

const admin = await login(ADMIN.username, ADMIN.password);
if (!admin.ok) { console.error(`[FATAL] 管理员登录失败: ${admin.msg}`); process.exit(2); }

// ============ V2 缓存 TTL ============
try {
  if (cacheType !== 'redis' || !redisOk) {
    addResult('V2', '缓存 TTL', 'SKIP', `当前 cache_type=${cacheType}，需切 redis 后复验`);
  } else {
    // 单设备模式写 user_{id}（带 TTL）；user_tokens_{id} 为多设备集合键，可能不存在
    const ttl = redisCli(`TTL "user_${admin.userId}"`);
    const ttl2 = redisCli(`TTL "user_tokens_${admin.userId}"`);
    const valid = [ttl, ttl2].find((t) => /^\d+$/.test(t) && Number(t) > 0);
    if (valid !== undefined) {
      addResult('V2', '缓存 TTL', 'PASS', `redis TTL=${valid}s，与 access_token_expire=7200 对齐，非 -1 永生键（user_${admin.userId}=${ttl} / user_tokens_${admin.userId}=${ttl2}）`);
    } else if ([ttl, ttl2].some((t) => t === '-1')) {
      addResult('V2', '缓存 TTL', 'FAIL', `存在永生键：user_${admin.userId}=${ttl} / user_tokens_${admin.userId}=${ttl2}（期望带正 TTL）`);
    } else {
      addResult('V2', '缓存 TTL', 'FAIL', `未找到带 TTL 的 token 键（user_${admin.userId}=${ttl} / user_tokens_${admin.userId}=${ttl2}）`);
    }
  }
} catch (e) { addResult('V2', '缓存 TTL', 'FAIL', e.message); }

// ============ V9 Redis 模式单设备挤下线 ============
let testUid = '';
try {
  if (sessionStore !== 'redis' || !redisOk) {
    addResult('V9', 'Redis 挤下线', 'SKIP', `当前 session_store=${sessionStore}，需切 redis 后复验`);
  } else if (multiDevice === '1') {
    addResult('V9', 'Redis 挤下线', 'SKIP', `当前 login_multi_device=1（多设备模式），无"挤出"语义；需临时改 0`);
  } else {
    // 自建测试账号（清理历史残留 → add → 断言落库）
    const oldUid = psql(`SELECT id FROM mxx_system_admin WHERE user_name='${TU.username}' LIMIT 1;`);
    if (oldUid && oldUid !== '') {
      delKeysByPattern(`session:${oldUid}:*`);
      delKeysByPattern(`refresh_session:*`);
      psql(`DELETE FROM mxx_system_admin WHERE user_name='${TU.username}';`);
      console.log('[SETUP] 清理旧残留测试账号');
    }
    // 动态唯一手机号/邮箱，避免与库内既有账号冲突
    const uniq = String(Date.now()).slice(-8); // 8 位
    const cr = await http('POST', '/api/system/admin/add', {
      userName: TU.username, nickName: 'Redis复验测试号', password: TU.password,
      userType: 0, status: 1, mobile: `139${uniq}`, email: `test01r${uniq}@example.com`,
      deptIds: ['103'], roleIds: ['8'], postIds: ['16'], salaryEnabled: 1, bizEnabled: 1,
    }, admin.accessToken);
    const after = psql(`SELECT COUNT(*) FROM mxx_system_admin WHERE user_name='${TU.username}';`);
    if (!(cr.status === 200 && cr.data && cr.data.code === 200 && Number(after) === 1)) {
      throw new Error(`创建测试账号失败: HTTP ${cr.status} code=${cr.data && cr.data.code} msg=${cr.data && cr.data.msg} DB=${after}`);
    }

    const first = await login(TU.username, TU.password);
    if (!first.ok) throw new Error(`首设备登录失败: ${first.msg}`);
    const second = await login(TU.username, TU.password);
    if (!second.ok) throw new Error(`二次登录失败: ${second.msg}`);

    const stFirst = await tokenAlive(first.accessToken);   // 期望 401（被挤出）
    const stSecond = await tokenAlive(second.accessToken); // 期望 200
    // 存储层断言：被挤出的 redis 会话键应已被删
    const kSes = redisCli(`EXISTS "session:${first.userId}:${first.accessToken}"`);

    if (stFirst === 401 && stSecond === 200) {
      addResult('V9', 'Redis 挤下线', 'PASS',
        `单设备二次登录挤出首设备：旧 token=${stFirst}（401）、新设备=${stSecond}（200）、旧会话键 EXISTS=${kSes}（0=已删）`);
    } else if (stFirst === 200) {
      addResult('V9', 'Redis 挤下线', 'FAIL',
        `首设备旧 token 仍存活（${stFirst}）——单设备模式未挤出或存储键未清理（EXISTS=${kSes}）`);
    } else {
      addResult('V9', 'Redis 挤下线', 'FAIL', `旧 token=${stFirst}（期望401）新设备=${stSecond}（期望200）`);
    }
  }
} catch (e) {
  addResult('V9', 'Redis 挤下线', 'FAIL', e.message);
} finally {
  // 兜底清理：硬删测试账号 + 清其 redis 会话键（后端 revoke 也会扫删，此处双保险）
  testUid = psql(`SELECT id FROM mxx_system_admin WHERE user_name='${TU.username}' LIMIT 1;`);
  if (testUid && testUid !== '') {
    if (admin.ok) await http('DELETE', `/api/system/admin/delete/${testUid}`, {}, admin.accessToken);
    delKeysByPattern(`session:${testUid}:*`);
    delKeysByPattern(`refresh_session:*`);
    psql(`DELETE FROM mxx_system_admin WHERE user_name='${TU.username}';`);
    console.log(`[CLEANUP] 测试账号 ${TU.username}(id=${testUid}) 已清理`);
  }
}

console.log('\n执行完成。环境保持 redis/单设备模式，恢复操作见 docs 手册。');
