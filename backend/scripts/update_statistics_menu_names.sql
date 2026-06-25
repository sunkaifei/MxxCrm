UPDATE mxx_system_menu SET name = 'page.statistics.title' WHERE path='/statistics' AND type='FOLDER';
UPDATE mxx_system_menu SET name = 'page.statistics.index' WHERE path='/statistics/index';
UPDATE mxx_system_menu SET name = 'page.statistics.performance' WHERE path='/statistics/performance';
UPDATE mxx_system_menu SET name = 'page.statistics.customer' WHERE path='/statistics/customer';
UPDATE mxx_system_menu SET name = 'page.statistics.employee' WHERE path='/statistics/employee';
UPDATE mxx_system_menu SET name = 'page.statistics.contract' WHERE path='/statistics/contract';
UPDATE mxx_system_menu SET name = 'page.statistics.payment' WHERE path='/statistics/payment';

SELECT id, parent_id, type, path, name, icon FROM mxx_system_menu WHERE path LIKE '/statistics%' AND deleted=0 ORDER BY parent_id, sort;