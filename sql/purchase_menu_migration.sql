-- 采购模块菜单数据迁移
-- 2026-08-05

-- 采购申请 (requisition) - MENU
INSERT INTO mxx_system_menu (id, parent_id, name, type, route_name, path, component, perm, status, sort) VALUES
(800, 139, 'page.purchase.requisition.title', 'MENU', 'PurchaseRequisition', '/purchase/requisition', 'purchase/requisition/index', 'purchase:requisition:list', 1, 3);

-- 采购申请按钮权限
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, component, status, sort) VALUES
(801, 800, '新增采购申请', 'BUTTON', 'purchase:requisition:save', 'purchase/requisition/index', 1, 1),
(802, 800, '编辑采购申请', 'BUTTON', 'purchase:requisition:update', 'purchase/requisition/index', 1, 2),
(803, 800, '删除采购申请', 'BUTTON', 'purchase:requisition:delete', 'purchase/requisition/index', 1, 3),
(804, 800, '审批采购申请', 'BUTTON', 'purchase:requisition:approve', 'purchase/requisition/index', 1, 4),
(805, 800, '转采购订单', 'BUTTON', 'purchase:requisition:convert', 'purchase/requisition/index', 1, 5);

-- 采购收货 (receipt) - MENU
INSERT INTO mxx_system_menu (id, parent_id, name, type, route_name, path, component, perm, status, sort) VALUES
(810, 139, 'page.purchase.receipt.title', 'MENU', 'PurchaseReceipt', '/purchase/receipt', 'purchase/receipt/index', 'purchase:receipt:list', 1, 4);

-- 采购收货按钮权限
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, component, status, sort) VALUES
(811, 810, '新增收货单', 'BUTTON', 'purchase:receipt:save', 'purchase/receipt/index', 1, 1),
(812, 810, '删除收货单', 'BUTTON', 'purchase:receipt:delete', 'purchase/receipt/index', 1, 2),
(813, 810, '生成入库单', 'BUTTON', 'purchase:receipt:inbound', 'purchase/receipt/index', 1, 3);

-- 采购退货 (return) - MENU
INSERT INTO mxx_system_menu (id, parent_id, name, type, route_name, path, component, perm, status, sort) VALUES
(820, 139, 'page.purchase.return.title', 'MENU', 'PurchaseReturn', '/purchase/return', 'purchase/return/index', 'purchase:return:list', 1, 5);

-- 采购退货按钮权限
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, component, status, sort) VALUES
(821, 820, '新增退货单', 'BUTTON', 'purchase:return:save', 'purchase/return/index', 1, 1),
(822, 820, '编辑退货单', 'BUTTON', 'purchase:return:update', 'purchase/return/index', 1, 2),
(823, 820, '删除退货单', 'BUTTON', 'purchase:return:delete', 'purchase/return/index', 1, 3);

-- 备货计划 (stock-plan) - MENU
INSERT INTO mxx_system_menu (id, parent_id, name, type, route_name, path, component, perm, status, sort) VALUES
(830, 139, 'page.purchase.stockPlan.title', 'MENU', 'PurchaseStockPlan', '/purchase/stock-plan', 'purchase/stock-plan/index', 'purchase:stock_plan:list', 1, 6);

-- 备货计划按钮权限
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, component, status, sort) VALUES
(831, 830, '新增备货计划', 'BUTTON', 'purchase:stock_plan:save', 'purchase/stock-plan/index', 1, 1),
(832, 830, '编辑备货计划', 'BUTTON', 'purchase:stock_plan:update', 'purchase/stock-plan/index', 1, 2),
(833, 830, '删除备货计划', 'BUTTON', 'purchase:stock_plan:delete', 'purchase/stock-plan/index', 1, 3);

-- 采购报表 (report) - MENU
INSERT INTO mxx_system_menu (id, parent_id, name, type, route_name, path, component, perm, status, sort) VALUES
(840, 139, 'page.purchase.report.title', 'MENU', 'PurchaseReport', '/purchase/report', 'purchase/report/index', 'purchase:report:list', 1, 7);

-- PO 审核/关闭按钮权限（已有菜单 id=175）
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, component, status, sort) VALUES
(841, 175, '审核采购订单', 'BUTTON', 'purchase:order:audit', 'purchase/po/index', 1, 10),
(842, 175, '关闭采购订单', 'BUTTON', 'purchase:order:close', 'purchase/po/index', 1, 11);