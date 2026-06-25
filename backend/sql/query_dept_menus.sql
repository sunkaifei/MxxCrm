SELECT id, parent_id, name, type, route_name, path, component, perm, sort FROM mxx_system_menu WHERE deleted = 0 AND (path LIKE '%dept%' OR perm LIKE '%dept%') ORDER BY parent_id, sort;
