const http = require('http');
const { decode } = require('@msgpack/msgpack');

const token = require('fs').readFileSync('C:/Users/Administrator/AppData/Local/Temp/mxx_token.txt', 'utf-8').trim();

function fetchMenuList() {
  return new Promise((resolve, reject) => {
    const options = {
      hostname: 'localhost',
      port: 8080,
      path: '/api/system/menu/list',
      method: 'GET',
      headers: { 'Authorization': `Bearer ${token}` }
    };
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
    req.end();
  });
}

fetchMenuList().then(data => {
  const menus = data.data || [];
  // Find statistics/performance related menus
  const perfMenus = menus.filter(m => 
    (m.path && (m.path.includes('statistics') || m.path.includes('performance'))) ||
    (m.perm && m.perm.includes('performance')) ||
    (m.name && (m.name.includes('业绩') || m.name.includes('统计')))
  );
  console.log('Found', perfMenus.length, 'performance/statistics menus:');
  perfMenus.forEach(m => {
    console.log(`  ID=${m.id}, parent=${m.parent_id}, type=${m.menu_type}, name=${m.name}, path=${m.path}, perm=${m.perm}, route_name=${m.route_name}`);
  });
}).catch(e => console.error('Error:', e.message));
