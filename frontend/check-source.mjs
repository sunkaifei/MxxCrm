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
  console.warn('登录成功:', !!token);

  for (const cid of [1, 2, 3, 7, 8]) {
    const custResp = await fetch(
      `http://127.0.0.1:8080/api/system/customer/info?id=${cid}`,
      {
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    const custBuf = new Uint8Array(await custResp.arrayBuffer());
    const custData = decode(custBuf);
    console.warn(
      `客户 id=${cid} source=${custData?.data?.source} (${typeof custData?.data?.source}) industry=${custData?.data?.industry} level=${custData?.data?.level}`,
    );

    const oppResp = await fetch(
      `http://127.0.0.1:8080/api/system/opportunity/list?listType=customer&page=1&pageSize=10&customerId=${cid}`,
      {
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    const oppBuf = new Uint8Array(await oppResp.arrayBuffer());
    const oppData = decode(oppBuf);
    console.warn(
      `  商机: total=${oppData?.data?.total}, items=${oppData?.data?.items?.length}`,
    );
    if (oppData?.data?.items) {
      for (const item of oppData.data.items) {
        console.warn(
          `    id=${item.id} source=${item.source} (${typeof item.source}) title=${item.title}`,
        );
      }
    }
  }
}
main().catch(console.error);
