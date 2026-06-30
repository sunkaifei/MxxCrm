SELECT id, parent_id, name, path, type 
FROM mxx_system_menu 
WHERE (parent_id = 0 AND name LIKE '%财务%') OR path LIKE '/finance%' 
ORDER BY id;