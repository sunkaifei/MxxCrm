-- 业绩目标计划菜单
INSERT INTO mxx_system_menu (parent_id, type, route_name, path, component, perm, status, redirect, sort, icon, create_time, updated_time)
VALUES (
    (SELECT id FROM mxx_system_menu WHERE path='/statistics' AND type='FOLDER'),
    'MENU', 'statistics-performance-plan', '/statistics/performance-plan', 'statistics/performance-plan/index', 'statistics:performance-plan:view', 1, '', 3,
    'lucide:notebook-pen', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
)
ON CONFLICT DO NOTHING;

SELECT id, parent_id, type, path, icon FROM mxx_system_menu WHERE path LIKE '/statistics%' AND deleted=0 ORDER BY parent_id, sort;