import { decode } from './node_modules/.pnpm/@msgpack+msgpack@3.1.3/node_modules/@msgpack/msgpack/dist.esm/index.mjs';

async function main() {
  const loginResp = await fetch('http://127.0.0.1:8080/api/system/auth/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username: 'admin', password: 'admin123' }),
  });
  const loginBuf = new Uint8Array(await loginResp.arrayBuffer());
  const loginData = decode(loginBuf);
  const token = loginData?.data?.accessToken;

  // 客户 id=1 详情 - 仔细查看所有字段类型
  const custResp = await fetch('http://127.0.0.1:8080/api/system/customer/info?id=1', {
    headers: { Authorization: `Bearer ${token}` },
  });
  const custBuf = new Uint8Array(await custResp.arrayBuffer());
  const custData = decode(custBuf);
  const d = custData?.data || {};
  console.log('=== 客户 id=1 所有枚举/数字字段 ===');
  console.log('customerType:', d.customerType, typeof d.customerType);
  console.log('source:', d.source, typeof d.source);
  console.log('industry:', d.industry, typeof d.industry);
  console.log('level:', d.level, typeof d.level);
  console.log('currency:', d.currency, typeof d.currency);
  console.log('gender:', d.gender, typeof d.gender);

  // 商机列表 - customerId=1
  const oppResp = await fetch('http://127.0.0.1:8080/api/system/opportunity/list?listType=customer&page=1&pageSize=10&customerId=1', {
    headers: { Authorization: `Bearer ${token}` },
  });
  const oppBuf = new Uint8Array(await oppResp.arrayBuffer());
  const oppData = decode(oppBuf);
  console.log('\n=== 商机列表 ===');
  console.log('total:', oppData?.data?.total, typeof oppData?.data?.total);
  if (oppData?.data?.items) {
    for (const item of oppData.data.items) {
      console.log(`id=${item.id}, source=${item.source} (${typeof item.source}), stage=${item.stage} (${typeof item.stage})`);
    }
  }
}
main().catch(console.error);
