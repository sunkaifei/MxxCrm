DELETE FROM mxx_system_menu WHERE parent_id = (SELECT id FROM mxx_system_menu WHERE route_name = 'Dashboard');

INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
((SELECT id FROM mxx_system_menu WHERE route_name = 'Dashboard'), 'Analytics', 'C', 'Analytics', '/analytics', 'dashboard/analytics/index', 'dashboard:analytics', 1, 'AreaChart', 0),
((SELECT id FROM mxx_system_menu WHERE route_name = 'Dashboard'), 'Workspace', 'C', 'Workspace', '/workspace', 'dashboard/workspace/index', 'dashboard:workspace', 2, 'Workspace', 0);

INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='super_admin'), id 
FROM mxx_system_menu WHERE perm LIKE 'dashboard:%' AND deleted=0
ON CONFLICT DO NOTHING;

INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='system_admin'), id 
FROM mxx_system_menu WHERE perm LIKE 'dashboard:%' AND deleted=0
ON CONFLICT DO NOTHING;

INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='sales_director'), id 
FROM mxx_system_menu WHERE perm LIKE 'dashboard:%' AND deleted=0
ON CONFLICT DO NOTHING;

SELECT 'Dashboard routes fixed' AS result;