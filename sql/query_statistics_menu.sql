SELECT id, parent_id, name, path, component, type 
FROM mxx_system_menu 
WHERE path LIKE '%statistics%' OR component LIKE '%statistics%' 
ORDER BY parent_id, id;