INSERT INTO mxx_system_menu (parent_id, type, route_name, path, component, perm, status, redirect, sort, icon, create_time, updated_time)
VALUES (0, 'FOLDER', 'statistics', '/statistics', '', '', 1, '', 7, 'lucide:bar-chart-3', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT DO NOTHING;

INSERT INTO mxx_system_menu (parent_id, type, route_name, path, component, perm, status, redirect, sort, icon, create_time, updated_time)
VALUES (
    (SELECT id FROM mxx_system_menu WHERE path='/statistics' AND type='FOLDER'),
    'MENU', 'statistics-index', '/statistics/index', 'statistics/index', 'statistics:view', 1, '', 1,
    'lucide:layout-dashboard', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
)
ON CONFLICT DO NOTHING;

INSERT INTO mxx_system_menu (parent_id, type, route_name, path, component, perm, status, redirect, sort, icon, create_time, updated_time)
VALUES (
    (SELECT id FROM mxx_system_menu WHERE path='/statistics' AND type='FOLDER'),
    'MENU', 'statistics-performance', '/statistics/performance', 'statistics/performance/index', 'statistics:performance:view', 1, '', 2,
    'lucide:target', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
)
ON CONFLICT DO NOTHING;

INSERT INTO mxx_system_menu (parent_id, type, route_name, path, component, perm, status, redirect, sort, icon, create_time, updated_time)
VALUES (
    (SELECT id FROM mxx_system_menu WHERE path='/statistics' AND type='FOLDER'),
    'MENU', 'statistics-customer', '/statistics/customer', 'statistics/customer/index', 'statistics:customer:view', 1, '', 3,
    'lucide:users', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
)
ON CONFLICT DO NOTHING;

INSERT INTO mxx_system_menu (parent_id, type, route_name, path, component, perm, status, redirect, sort, icon, create_time, updated_time)
VALUES (
    (SELECT id FROM mxx_system_menu WHERE path='/statistics' AND type='FOLDER'),
    'MENU', 'statistics-employee', '/statistics/employee', 'statistics/employee/index', 'statistics:employee:view', 1, '', 4,
    'lucide:user', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
)
ON CONFLICT DO NOTHING;

INSERT INTO mxx_system_menu (parent_id, type, route_name, path, component, perm, status, redirect, sort, icon, create_time, updated_time)
VALUES (
    (SELECT id FROM mxx_system_menu WHERE path='/statistics' AND type='FOLDER'),
    'MENU', 'statistics-contract', '/statistics/contract', 'statistics/contract/index', 'statistics:contract:view', 1, '', 5,
    'lucide:file-text', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
)
ON CONFLICT DO NOTHING;

INSERT INTO mxx_system_menu (parent_id, type, route_name, path, component, perm, status, redirect, sort, icon, create_time, updated_time)
VALUES (
    (SELECT id FROM mxx_system_menu WHERE path='/statistics' AND type='FOLDER'),
    'MENU', 'statistics-payment', '/statistics/payment', 'statistics/payment/index', 'statistics:payment:view', 1, '', 6,
    'lucide:wallet', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
)
ON CONFLICT DO NOTHING;

SELECT id, parent_id, type, path, icon FROM mxx_system_menu WHERE path LIKE '/statistics%' AND deleted=0 ORDER BY parent_id, sort;