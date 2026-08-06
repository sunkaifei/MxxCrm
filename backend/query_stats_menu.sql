SELECT id, parent_id, name, type, path, component, perm
FROM mxx_system_menu
WHERE deleted = 0
AND (
  name LIKE '%dashboard%' OR name LIKE '%statistics%' OR name LIKE '%report%'
  OR name LIKE '%analysis%' OR name LIKE '%overview%' OR name LIKE '%performance%'
  OR path LIKE '/dashboard%' OR path LIKE '/statistics%' OR path LIKE '/report%'
  OR path LIKE '/inventory-report%' OR path LIKE '/finance/statistic%'
  OR path LIKE '/salary-analysis%'
)
ORDER BY parent_id, sort, id;
