const { Client } = require('pg');
const client = new Client({
  connectionString: 'postgres://postgres:jyXyh2618BjSkf@115.190.210.106:5432/mxxcrm_data'
});

client.connect()
  .then(() => client.query("SELECT id, name, perm, route_name FROM mxx_system_menu WHERE perm LIKE '%performance-plan%' AND deleted=0"))
  .then(r => {
    console.log('performance-plan menu rows:', r.rows.length);
    r.rows.forEach(row => console.log(JSON.stringify(row)));
    return client.query("SELECT id, name, perm FROM mxx_system_menu WHERE perm LIKE '%performance%' AND deleted=0 ORDER BY id");
  })
  .then(r => {
    console.log('\nAll performance menu rows:', r.rows.length);
    r.rows.forEach(row => console.log(JSON.stringify(row)));
    client.end();
  })
  .catch(e => {
    console.error('Error:', e.message);
    client.end();
  });
