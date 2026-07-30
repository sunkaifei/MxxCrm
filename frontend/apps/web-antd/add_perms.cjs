const http = require('http');
const { decode, encode } = require('@msgpack/msgpack');
const fs = require('fs');

const token = fs.readFileSync('C:/Users/Administrator/AppData/Local/Temp/mxx_token.txt', 'utf-8').trim();

function fetchUrl(path, method='GET', body=null) {
  return new Promise((resolve, reject) => {
    const options = {
      hostname: 'localhost',
      port: 8080,
      path: path,
      method: method,
      headers: { 'Authorization': `Bearer ${token}` }
    };
    if (body) {
      const jsonStr = JSON.stringify(body);
      options.headers['Content-Type'] = 'application/json';
      options.headers['Content-Length'] = Buffer.byteLength(jsonStr);
    }
    const req = http.request(options, res => {
      const chunks = [];
      res.on('data', c => chunks.push(c));
      res.on('end', () => {
        const buf = Buffer.concat(chunks);
        try {
          const decoded = decode(new Uint8Array(buf));
          resolve(decoded);
        } catch(e) {
          reject(e);
        }
      });
    });
    req.on('error', reject);
    if (body) {
      req.write(JSON.stringify(body));
    }
    req.end();
  });
}

async function addPermission(perm, name, sort) {
  const body = {
    parentId: 263,
    type: 'BUTTON',
    perm: perm,
    status: 1,
    path: '',
    component: '',
    routeName: '',
    redirect: '',
    meta: {
      name: name,
      sort: sort,
      icon: '',
      hideInMenu: 1,
    },
  };
  const res = await fetchUrl('/api/system/menu/add', 'POST', body);
  console.log(`Add ${perm}:`, JSON.stringify(res));
  return res;
}

async function main() {
  // Add the three performance-plan permission codes
  await addPermission('statistics:performance-plan:view', '销售计划查看', 1);
  await addPermission('statistics:performance-plan:manage', '销售计划管理', 2);
  await addPermission('statistics:performance-plan:approve', '销售计划审批', 3);

  // Now clear the permission cache for admin user (id=3) so the new permissions take effect
  // We can do this by calling the auth/codes endpoint which should reload permissions
  console.log('\nDone! New permissions added.');
}

main().catch(e => console.error('Error:', e.message));
