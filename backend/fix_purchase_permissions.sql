-- 1. 补全采购子菜单的按钮权限（之前缺失的菜单ID: 800,810,820,830,840）

-- 采购申请按钮
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, component, status, sort, create_time, deleted)
VALUES
  (801, 800, 'page.purchase.requisition.button.create', 'BUTTON', 'purchase:requisition:create', 'purchase/requisition/index', 1, 1, NOW(), 0),
  (802, 800, 'page.purchase.requisition.button.edit', 'BUTTON', 'purchase:requisition:edit', 'purchase/requisition/index', 1, 2, NOW(), 0),
  (803, 800, 'page.purchase.requisition.button.delete', 'BUTTON', 'purchase:requisition:delete', 'purchase/requisition/index', 1, 3, NOW(), 0),
  (804, 800, 'page.purchase.requisition.button.submit', 'BUTTON', 'purchase:requisition:submit', 'purchase/requisition/index', 1, 4, NOW(), 0),
  (805, 800, 'page.purchase.requisition.button.approve', 'BUTTON', 'purchase:requisition:approve', 'purchase/requisition/index', 1, 5, NOW(), 0),
ON CONFLICT (id) DO NOTHING;

-- 采购收货按钮
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, component, status, sort, create_time, deleted)
VALUES
  (811, 810, 'page.purchase.receipt.button.create', 'BUTTON', 'purchase:receipt:create', 'purchase/receipt/index', 1, 1, NOW(), 0),
  (812, 810, 'page.purchase.receipt.button.edit', 'BUTTON', 'purchase:receipt:edit', 'purchase/receipt/index', 1, 2, NOW(), 0),
  (813, 810, 'page.purchase.receipt.button.delete', 'BUTTON', 'purchase:receipt:delete', 'purchase/receipt/index', 1, 3, NOW(), 0),
  (814, 810, 'page.purchase.receipt.button.confirm', 'BUTTON', 'purchase:receipt:confirm', 'purchase/receipt/index', 1, 4, NOW(), 0),
ON CONFLICT (id) DO NOTHING;

-- 采购退货按钮
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, component, status, sort, create_time, deleted)
VALUES
  (821, 820, 'page.purchase.return.button.create', 'BUTTON', 'purchase:return:create', 'purchase/return/index', 1, 1, NOW(), 0),
  (822, 820, 'page.purchase.return.button.edit', 'BUTTON', 'purchase:return:edit', 'purchase/return/index', 1, 2, NOW(), 0),
  (823, 820, 'page.purchase.return.button.delete', 'BUTTON', 'purchase:return:delete', 'purchase/return/index', 1, 3, NOW(), 0),
  (824, 820, 'page.purchase.return.button.confirm', 'BUTTON', 'purchase:return:confirm', 'purchase/return/index', 1, 4, NOW(), 0),
ON CONFLICT (id) DO NOTHING;

-- 备货计划按钮
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, component, status, sort, create_time, deleted)
VALUES
  (831, 830, 'page.purchase.stockPlan.button.create', 'BUTTON', 'purchase:stock_plan:create', 'purchase/stock-plan/index', 1, 1, NOW(), 0),
  (832, 830, 'page.purchase.stockPlan.button.edit', 'BUTTON', 'purchase:stock_plan:edit', 'purchase/stock-plan/index', 1, 2, NOW(), 0),
  (833, 830, 'page.purchase.stockPlan.button.delete', 'BUTTON', 'purchase:stock_plan:delete', 'purchase/stock-plan/index', 1, 3, NOW(), 0),
ON CONFLICT (id) DO NOTHING;

-- 采购报表按钮（报表一般只有查看权限，不需要额外按钮）
-- 报表菜单840本身已有 purchase:report:list 权限

-- 2. 给超级管理员(4)和系统管理员(5)分配所有缺失的采购菜单权限
-- 所有采购相关的菜单ID: 139,174-175,216-223,800-805,810-814,820-824,830-833,840

-- 超级管理员(role_id=4)
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT 4, m.id FROM mxx_system_menu m
WHERE (m.id IN (139,800,801,802,803,804,805,810,811,812,813,814,820,821,822,823,824,830,831,832,833,840))
  AND NOT EXISTS (SELECT 1 FROM mxx_system_role_menu_merge rm WHERE rm.role_id = 4 AND rm.menu_id = m.id)
ON CONFLICT DO NOTHING;

-- 系统管理员(role_id=5)
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT 5, m.id FROM mxx_system_menu m
WHERE (m.id IN (139,800,801,802,803,804,805,810,811,812,813,814,820,821,822,823,824,830,831,832,833,840))
  AND NOT EXISTS (SELECT 1 FROM mxx_system_role_menu_merge rm WHERE rm.role_id = 5 AND rm.menu_id = m.id)
ON CONFLICT DO NOTHING;

-- 采购专员(role_id=9)
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT 9, m.id FROM mxx_system_menu m
WHERE (m.id IN (139,800,801,802,803,804,805,810,811,812,813,814,820,821,822,823,824,830,831,832,833,840))
  AND NOT EXISTS (SELECT 1 FROM mxx_system_role_menu_merge rm WHERE rm.role_id = 9 AND rm.menu_id = m.id)
ON CONFLICT DO NOTHING;

-- 业务员(role_id=8) - 只加采购订单列表和申请
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT 8, m.id FROM mxx_system_menu m
WHERE (m.id IN (139,800,801,802,803,804,805,830))
  AND NOT EXISTS (SELECT 1 FROM mxx_system_role_menu_merge rm WHERE rm.role_id = 8 AND rm.menu_id = m.id)
ON CONFLICT DO NOTHING;

-- 验证结果
SELECT rm.role_id, count(*) as total_perm FROM mxx_system_role_menu_merge rm
JOIN mxx_system_menu m ON rm.menu_id = m.id
WHERE m.id IN (139,174,175,216,217,218,219,220,221,222,223,
               800,801,802,803,804,805,810,811,812,813,814,
               820,821,822,823,824,830,831,832,833,840)
GROUP BY rm.role_id ORDER BY rm.role_id;
