// 二分定位：刷新接口到底校验哪一列
import { decode } from '../node_modules/.pnpm/@msgpack+msgpack@3.1.3/node_modules/@msgpack/msgpack/dist.esm/index.mjs';
import { execSync } from 'node:child_process';

const BASE = 'http://127.0.0.1:8088';
const env = { ...process.env, PGPASSWORD: '123456' };
function psql(sql) {
  return execSync(`psql -U postgres -h 127.0.0.1 -d mxxcrm_data -t -A -c ${JSON.stringify(sql)}`, { env, encoding: 'utf8' }).trim();
}
async function post(path, body) {
  const res = await fetch(BASE + path, {
    method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body),
  });
  const buf = new Uint8Array(await res.arrayBuffer());
  let data = null; try { data = decode(buf); } catch { }
  return { status: res.status, data };
}

const login = await post('/api/system/auth/login', { username: 'admin', password: 'admin123' });
let rt = login.data.data.refreshToken;

// 测试 A：只把 refresh_expire_time 置过去
psql(`UPDATE mxx_system_session SET refresh_expire_time = CURRENT_TIMESTAMP - interval '1 minute' WHERE user_id=3;`);
let r = await post('/api/system/auth/refresh', { refreshToken: rt });
console.log('A 仅 refresh 过期 → refresh 状态:', r.status);
if (r.status === 200) rt = r.data.data.refreshToken; // 旋转后更新凭据

// 测试 B：refresh_expire_time 和 expire_time 都置过去
psql(`UPDATE mxx_system_session SET refresh_expire_time = CURRENT_TIMESTAMP - interval '1 minute', expire_time = CURRENT_TIMESTAMP - interval '1 minute' WHERE user_id=3;`);
r = await post('/api/system/auth/refresh', { refreshToken: rt });
console.log('B 双列均过期 → refresh 状态:', r.status);
if (r.status === 200) rt = r.data.data.refreshToken;

// 测试 C：只把 expire_time 置过去
psql(`UPDATE mxx_system_session SET expire_time = CURRENT_TIMESTAMP - interval '1 minute' WHERE user_id=3;`);
r = await post('/api/system/auth/refresh', { refreshToken: rt });
console.log('C 仅 access 过期 → refresh 状态:', r.status, '（预期 200：refresh 不校验 access 窗口）');
