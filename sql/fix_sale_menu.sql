-- 销售菜单修复脚本 (v2 修正字段名)
-- 正确字段：id, parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted
-- 日期：2026-06-28

BEGIN;

-- 1. 更新商机菜单(163)：从CRM菜单(89)移到销售管理(112)下，路径改为/sale/opportunity，排序0
UPDATE mxx_system_menu SET parent_id = 112, path = '/sale/opportunity', sort = 0, icon = 'lucide:target' WHERE id = 163;

-- 2. 更新合同菜单(164)：路径改为/sale/contract，排序2
UPDATE mxx_system_menu SET path = '/sale/contract', sort = 2, icon = 'lucide:file-text' WHERE id = 164;

-- 3. 更新订单菜单(167)排序为3，图标改为lucide格式
UPDATE mxx_system_menu SET sort = 3, icon = 'lucide:shopping-cart' WHERE id = 167;

-- 4. 更新回款菜单(169)排序为4
UPDATE mxx_system_menu SET sort = 4, icon = 'lucide:wallet' WHERE id = 169;

-- 5. 隐藏订单明细菜单(168)（不需要作为独立菜单，明细在订单详情页查看）
UPDATE mxx_system_menu SET hide_in_menu = 1, sort = 99 WHERE id = 168;

-- 6. 添加报价单菜单（ID从301开始，当前最大ID=300）
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
VALUES (301, 112, '', 'page.sale.quotation.title', 'MENU', 'Quotation', '/sale/quotation', 'sale/quotation/index', 'sale:quotation:list', 1, 0, 0, 0, 0, 0, 0, 1, 'lucide:file-text', '', NULL, NOW(), NOW(), 0);

-- 7. 添加发票菜单
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
VALUES (302, 112, '', 'page.sale.invoice.title', 'MENU', 'SaleInvoice', '/sale/invoice', 'sale/invoice/index', 'sale:invoice:list', 1, 0, 0, 0, 0, 0, 0, 5, 'lucide:receipt', '', NULL, NOW(), NOW(), 0);

-- 8. 添加业绩菜单
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
VALUES (303, 112, '', 'page.sale.performance.title', 'MENU', 'SalePerformance', '/sale/performance', 'sale/performance/index', 'sale:performance:view', 1, 0, 0, 0, 0, 0, 0, 6, 'lucide:bar-chart-3', '', NULL, NOW(), NOW(), 0);

-- 9. 添加报价单按钮权限
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted) VALUES
(304, 301, '', 'page.sale.quotation.button.create', 'BUTTON', 'QuotationCreate', '', 'sale/quotation/index', 'sale:quotation:create', 1, 0, 0, 0, 0, 0, 0, 1, '', '', NULL, NOW(), NOW(), 0),
(305, 301, '', 'page.sale.quotation.button.edit', 'BUTTON', 'QuotationEdit', '', 'sale/quotation/index', 'sale:quotation:edit', 1, 0, 0, 0, 0, 0, 0, 2, '', '', NULL, NOW(), NOW(), 0),
(306, 301, '', 'page.sale.quotation.button.delete', 'BUTTON', 'QuotationDelete', '', 'sale/quotation/index', 'sale:quotation:delete', 1, 0, 0, 0, 0, 0, 0, 3, '', '', NULL, NOW(), NOW(), 0),
(307, 301, '', 'page.sale.quotation.button.send', 'BUTTON', 'QuotationSend', '', 'sale/quotation/index', 'sale:quotation:send', 1, 0, 0, 0, 0, 0, 0, 4, '', '', NULL, NOW(), NOW(), 0);

-- 10. 添加发票按钮权限
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted) VALUES
(308, 302, '', 'page.sale.invoice.button.create', 'BUTTON', 'InvoiceCreate', '', 'sale/invoice/index', 'sale:invoice:create', 1, 0, 0, 0, 0, 0, 0, 1, '', '', NULL, NOW(), NOW(), 0),
(309, 302, '', 'page.sale.invoice.button.edit', 'BUTTON', 'InvoiceEdit', '', 'sale/invoice/index', 'sale:invoice:edit', 1, 0, 0, 0, 0, 0, 0, 2, '', '', NULL, NOW(), NOW(), 0),
(310, 302, '', 'page.sale.invoice.button.delete', 'BUTTON', 'InvoiceDelete', '', 'sale/invoice/index', 'sale:invoice:delete', 1, 0, 0, 0, 0, 0, 0, 3, '', '', NULL, NOW(), NOW(), 0);

-- 11. 添加合同按钮权限（如果不存在）
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
SELECT 311, 164, '', 'page.crm.contract.button.create', 'BUTTON', 'ContractCreate', '', 'crm/contract/index', 'crm:contract:create', 1, 0, 0, 0, 0, 0, 0, 1, '', '', NULL, NOW(), NOW(), 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'crm:contract:create');

INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
SELECT 312, 164, '', 'page.crm.contract.button.edit', 'BUTTON', 'ContractEdit', '', 'crm/contract/index', 'crm:contract:edit', 1, 0, 0, 0, 0, 0, 0, 2, '', '', NULL, NOW(), NOW(), 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'crm:contract:edit');

INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted)
SELECT 313, 164, '', 'page.crm.contract.button.delete', 'BUTTON', 'ContractDelete', '', 'crm/contract/index', 'crm:contract:delete', 1, 0, 0, 0, 0, 0, 0, 3, '', '', NULL, NOW(), NOW(), 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'crm:contract:delete');

-- 12. 添加业绩按钮权限（查看）
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive, sort, icon, redirect, params, create_time, update_time, deleted) VALUES
(314, 303, '', 'page.sale.performance.button.export', 'BUTTON', 'PerformanceExport', '', 'sale/performance/index', 'sale:performance:export', 1, 0, 0, 0, 0, 0, 0, 1, '', '', NULL, NOW(), NOW(), 0);

-- 13. 给超级管理员角色(role_id=1)分配新菜单权限
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id) VALUES
(1, 301), (1, 302), (1, 303),
(1, 304), (1, 305), (1, 306), (1, 307),
(1, 308), (1, 309), (1, 310),
(1, 314);

COMMIT;

-- 最终菜单顺序（父ID=112 销售管理，hide_in_menu=0的）：
-- sort=0: 商机 (163) - Target/lucide:target
-- sort=1: 报价单 (301) - lucide:file-text
-- sort=2: 合同 (164) - lucide:file-text
-- sort=3: 订单 (167) - lucide:shopping-cart
-- sort=4: 回款 (169) - lucide:wallet
-- sort=5: 发票 (302) - lucide:receipt
-- sort=6: 业绩 (303) - lucide:bar-chart-3
