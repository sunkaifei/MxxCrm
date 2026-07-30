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
      const encoded = encode(body);
      options.headers['Content-Type'] = 'application/msgpack';
      options.headers['Content-Length'] = encoded.length;
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
      req.write(encode(body));
    }
    req.end();
  });
}

// First, get menu ID=263 details
fetchUrl('/api/system/menu/list').then(data => {
  const menus = data.data || [];
  // Find menu 263 and its children
  function findInTree(nodes) {
    for (const n of nodes) {
      if (n.value === 263 || n.id === 263) {
        return n;
      }
      if (n.children && n.children.length > 0) {
        const found = findInTree(n.children);
        if (found) return found;
      }
    }
    return null;
  }
  const menu263 = findInTree(menus);
  if (menu263) {
    console.log('Menu 263:', JSON.stringify(menu263, null, 2));
  } else {
    console.log('Menu 263 not found in tree');
  }

  // Also search the flat list for any menus with parent_id=263
  const flatMenus = menus;
  const children = flatMenus.filter(m => m.parent_id === 263);
  console.log('\nDirect children of 263:', children.length);
  children.forEach(c => console.log('  ', JSON.stringify(c)));

  // Print the full menu 263 details from flat list
  const menu = flatMenus.find(m => m.id === 263);
  if (menu) {
    console.log('\nMenu 263 details:', JSON.stringify(menu, null, 2));
  }
}).catch(e => console.error('Error:', e.message));
