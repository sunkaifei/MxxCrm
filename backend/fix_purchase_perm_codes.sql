-- 修复采购模块权限码：使数据库 mxx_system_menu 的 perm 与后端 require_permission 完全一致

-- ============================================================
-- 1. 采购订单（原 purchase:po:* → purchase:order:*）
-- ============================================================
UPDATE mxx_system_menu SET perm = 'purchase:order:list'   WHERE id = 175;
UPDATE mxx_system_menu SET perm = 'purchase:order:save'  WHERE id = 219;
UPDATE mxx_system_menu SET perm = 'purchase:order:update' WHERE id = 220;
UPDATE mxx_system_menu SET perm = 'purchase:order:delete' WHERE id = 221;
UPDATE mxx_system_menu SET perm = 'purchase:order:audit'  WHERE id = 222;
UPDATE mxx_system_menu SET perm = 'purchase:order:close'  WHERE id = 223;

-- 新增采购订单缺失的权限码
INSERT INTO mxx_system_menu (parent_id, name, type, perm, component, status, sort, create_time, deleted)
SELECT 175, 'page.purchase.po.button.view', 'BUTTON', 'purchase:order:view', 'purchase/po/index', 1, 7, NOW(), 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'purchase:order:view');

-- ============================================================
-- 2. 供应商（原 purchase:supplier:create → save, 新增 info/view）
-- ============================================================
UPDATE mxx_system_menu SET perm = 'purchase:supplier:save'   WHERE id = 216;
UPDATE mxx_system_menu SET perm = 'purchase:supplier:update'  WHERE id = 217;
UPDATE mxx_system_menu SET perm = 'purchase:supplier:delete'  WHERE id = 218;

-- 新增供应商缺失的权限码
INSERT INTO mxx_system_menu (parent_id, name, type, perm, component, status, sort, create_time, deleted)
SELECT 174, 'page.purchase.supplier.button.info', 'BUTTON', 'purchase:supplier:info', 'purchase/supplier/index', 1, 6, NOW(), 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'purchase:supplier:info');

-- ============================================================
-- 3. 采购申请（对齐权限码）
-- ============================================================
UPDATE mxx_system_menu SET perm = 'purchase:requisition:save'    WHERE id = 801;
UPDATE mxx_system_menu SET perm = 'purchase:requisition:update'   WHERE id = 802;
UPDATE mxx_system_menu SET perm = 'purchase:requisition:delete'   WHERE id = 803;
UPDATE mxx_system_menu SET perm = 'purchase:requisition:save'     WHERE id = 804;
UPDATE mxx_system_menu SET perm = 'purchase:requisition:approve'  WHERE id = 805;

-- 新增缺失
INSERT INTO mxx_system_menu (parent_id, name, type, perm, component, status, sort, create_time, deleted)
SELECT 800, 'page.purchase.requisition.button.view', 'BUTTON', 'purchase:requisition:view', 'purchase/requisition/index', 1, 7, NOW(), 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'purchase:requisition:view');

INSERT INTO mxx_system_menu (parent_id, name, type, perm, component, status, sort, create_time, deleted)
SELECT 800, 'page.purchase.requisition.button.convert', 'BUTTON', 'purchase:requisition:convert', 'purchase/requisition/index', 1, 8, NOW(), 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'purchase:requisition:convert');

-- ============================================================
-- 4. 采购收货（对齐权限码）
-- ============================================================
UPDATE mxx_system_menu SET perm = 'purchase:receipt:save'   WHERE id = 811;
UPDATE mxx_system_menu SET perm = 'purchase:receipt:update'  WHERE id = 812;
UPDATE mxx_system_menu SET perm = 'purchase:receipt:delete'  WHERE id = 813;
UPDATE mxx_system_menu SET perm = 'purchase:receipt:list'    WHERE id = 814;

INSERT INTO mxx_system_menu (parent_id, name, type, perm, component, status, sort, create_time, deleted)
SELECT 810, 'page.purchase.receipt.button.inbound', 'BUTTON', 'purchase:receipt:inbound', 'purchase/receipt/index', 1, 5, NOW(), 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'purchase:receipt:inbound');

-- ============================================================
-- 5. 采购退货（对齐权限码）
-- ============================================================
UPDATE mxx_system_menu SET perm = 'purchase:return:save'   WHERE id = 821;
UPDATE mxx_system_menu SET perm = 'purchase:return:update'  WHERE id = 822;
UPDATE mxx_system_menu SET perm = 'purchase:return:delete'  WHERE id = 823;
UPDATE mxx_system_menu SET perm = 'purchase:return:list'    WHERE id = 824;

-- ============================================================
-- 6. 备货计划（对齐权限码）
-- ============================================================
UPDATE mxx_system_menu SET perm = 'purchase:stock_plan:save'   WHERE id = 831;
UPDATE mxx_system_menu SET perm = 'purchase:stock_plan:update'  WHERE id = 832;
UPDATE mxx_system_menu SET perm = 'purchase:stock_plan:delete'  WHERE id = 833;

INSERT INTO mxx_system_menu (parent_id, name, type, perm, component, status, sort, create_time, deleted)
SELECT 830, 'page.purchase.stockPlan.button.view', 'BUTTON', 'purchase:stock_plan:view', 'purchase/stock-plan/index', 1, 4, NOW(), 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'purchase:stock_plan:view');

INSERT INTO mxx_system_menu (parent_id, name, type, perm, component, status, sort, create_time, deleted)
SELECT 830, 'page.purchase.stockPlan.button.convert', 'BUTTON', 'purchase:stock_plan:convert', 'purchase/stock-plan/index', 1, 5, NOW(), 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'purchase:stock_plan:convert');

-- ============================================================
-- 验证
-- ============================================================
SELECT id, name, perm FROM mxx_system_menu
WHERE (perm LIKE 'purchase:%') AND deleted = 0
ORDER BY parent_id, id;
