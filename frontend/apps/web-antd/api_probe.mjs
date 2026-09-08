import { createHmac } from 'node:crypto';
import { decode } from '@msgpack/msgpack';

const secret = '7425d38dd1d0a754a3607c9d007e03907c2ffe9da7b5c56354083f047c6931e8';
const b64 = (obj) => Buffer.from(JSON.stringify(obj)).toString('base64url');
const now = Math.floor(Date.now() / 1000);
const payload = {
  id: 3,
  username: 'admin',
  aud: 'mxx_B2B',
  exp: now + 3600,
  iat: now,
  iss: 'mxx_B2B_admin',
  nbf: now,
  sub: 'mxx_B2B_token',
  jti: 'ignore',
};
const data = b64({ typ: 'JWT', alg: 'HS256' }) + '.' + b64(payload);
const sig = createHmac('sha256', secret).update(data).digest('base64url');
const token = data + '.' + sig;

async function probe(url) {
  const resp = await fetch(url, {
    headers: { Authorization: `Bearer ${token}` },
  });
  const buf = await resp.arrayBuffer();
  const obj = decode(new Uint8Array(buf));
  return obj;
}

for (const scope of ['all', 'mine', 'recycle']) {
  const url = `http://127.0.0.1:8088/api/system/alert/list?page=1&pageSize=20&scope=${scope}`;
  try {
    const obj = await probe(url);
    const d = obj?.data || {};
    console.log(`\n===== scope=${scope} code=${obj?.code} msg=${obj?.msg} total=${d?.total} crossViewEnabled=${d?.crossViewEnabled} isManagement=${d?.isManagement}`);
    for (const it of d?.items || []) {
      console.log(
        `  id=${it.id} pid=${it.productId} wh=${it.warehouseId} whName=${it.warehouseName} type=${it.alertType} qty=${it.quantity} avail=${it.availableQuantity} min=${it.alertMinQuantity} max=${it.alertMaxQuantity} stale=${it.obsoleteDays}`,
      );
    }
  } catch (e) {
    console.log(`\n===== scope=${scope} ERROR: ${e.message}`);
  }
}
