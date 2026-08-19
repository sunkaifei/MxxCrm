// 检查各状态发票的审批详情接口表现（临时）
import { decode } from '../node_modules/.pnpm/@msgpack+msgpack@3.1.3/node_modules/@msgpack/msgpack/dist.esm/index.mjs';

const BASE = 'http://127.0.0.1:8080';
async function post(path, body, token) {
  const headers = { 'Content-Type': 'application/json' };
  if (token) headers.Authorization = `Bearer ${token}`;
  const res = await fetch(BASE + path, { method: 'POST', headers, body: JSON.stringify(body ?? {}) });
  return decode(new Uint8Array(await res.arrayBuffer()));
}
async function get(path, token) {
  const res = await fetch(BASE + path, { headers: { Authorization: `Bearer ${token}` } });
  return decode(new Uint8Array(await res.arrayBuffer()));
}
const login = await post('/api/system/auth/login', { username: 'admin', password: 'admin123' });
const token = login.data.accessToken;

// 113: approval_status=3 无实例（老数据）；114: =2 无实例；106: 草稿+残留撤回实例
for (const id of [106, 113, 114]) {
  const d = await get(`/api/system/sale/invoice/${id}/approval-detail`, token);
  const data = d.data ?? {};
  console.log(`--- 发票 ${id} (approvalStatus=${data.approvalStatus}) ---`);
  const inst = data.instance;
  if (!inst) {
    console.log('  instance: NULL（前端审批页无数据可用）');
  } else {
    console.log(`  instance: id=${inst.id} status=${inst.status} currentNode=${inst.currentNodeKey} submitter=${inst.submitterName}`);
    console.log(`  flowNodes: ${(inst.flowNodes || []).map((n) => n.nodeName).join(' → ')}`);
    console.log(`  logs: ${(inst.logs || []).length} 条`);
  }
}

// 列表 VO 是否带 approvalStatus
const list = await get('/api/system/sale/invoice/list?page=1&pageSize=20&listType=all', token);
const rows = list.data?.list ?? [];
console.log('\n--- 列表 VO ---');
for (const r of rows.slice(0, 15)) {
  console.log(`id=${r.id} title=${(r.title || '').slice(0, 10)} approvalStatus=${r.approvalStatus ?? 'undefined'}`);
}
