-- 更新采购子菜单图标
UPDATE mxx_system_menu SET icon = 'lucide:building-2' WHERE id = 174;
UPDATE mxx_system_menu SET icon = 'lucide:file-text' WHERE id = 175;
UPDATE mxx_system_menu SET icon = 'lucide:clipboard-list' WHERE id = 800;
UPDATE mxx_system_menu SET icon = 'lucide:package-check' WHERE id = 810;
UPDATE mxx_system_menu SET icon = 'lucide:undo-2' WHERE id = 820;
UPDATE mxx_system_menu SET icon = 'lucide:warehouse' WHERE id = 830;
UPDATE mxx_system_menu SET icon = 'lucide:bar-chart-3' WHERE id = 840;

SELECT id, name, path, icon FROM mxx_system_menu WHERE path LIKE '/purchase%' AND deleted = 0 ORDER BY id;
