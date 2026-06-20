-- ================================================
-- 销售管理菜单更新：新增报价单、发票、销售业绩
-- 运行前请确认 Sale 父菜单的 ID
-- ================================================

-- 步骤1：更新现有菜单排序，为报价单腾出 sort=1 的位置
UPDATE mxx_system_menu SET sort = 2 WHERE route_name = 'Order' AND parent_id = (SELECT id FROM mxx_system_menu WHERE route_name = 'Sale');
UPDATE mxx_system_menu SET sort = 3 WHERE route_name = 'OrderItem' AND parent_id = (SELECT id FROM mxx_system_menu WHERE route_name = 'Sale');
UPDATE mxx_system_menu SET sort = 4 WHERE route_name = 'Payment' AND parent_id = (SELECT id FROM mxx_system_menu WHERE route_name = 'Sale');

-- 步骤2：新增报价单管理菜单
INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
((SELECT id FROM mxx_system_menu WHERE route_name='Sale'), 'page.sale.quotation.title', 'MENU', 'Quotation', '/sale/quotation', 'sale/quotation/index', 'sale:quotation:list', 1, 'FileText', 0)
ON CONFLICT DO NOTHING;

-- 步骤3：新增发票管理菜单
INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
((SELECT id FROM mxx_system_menu WHERE route_name='Sale'), 'page.sale.invoice.title', 'MENU', 'Invoice', '/sale/invoice', 'sale/invoice/index', 'sale:invoice:list', 5, 'Receipt', 0)
ON CONFLICT DO NOTHING;

-- 步骤4：新增销售业绩菜单
INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
((SELECT id FROM mxx_system_menu WHERE route_name='Sale'), 'page.sale.performance.title', 'MENU', 'Performance', '/sale/performance', 'sale/performance/index', 'sale:performance:view', 6, 'BarChart', 0)
ON CONFLICT DO NOTHING;

-- 步骤5：报价单按钮权限
INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
((SELECT id FROM mxx_system_menu WHERE route_name='Quotation'), 'button.sale.quotation.create', 'BUTTON', '', '', '', 'sale:quotation:create', 1, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Quotation'), 'button.sale.quotation.edit', 'BUTTON', '', '', '', 'sale:quotation:edit', 2, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Quotation'), 'button.sale.quotation.delete', 'BUTTON', '', '', '', 'sale:quotation:delete', 3, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Quotation'), 'button.sale.quotation.send', 'BUTTON', '', '', '', 'sale:quotation:send', 4, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Quotation'), 'button.sale.quotation.accept', 'BUTTON', '', '', '', 'sale:quotation:accept', 5, '', 0)
ON CONFLICT DO NOTHING;

-- 步骤6：发票按钮权限
INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
((SELECT id FROM mxx_system_menu WHERE route_name='Invoice'), 'button.sale.invoice.create', 'BUTTON', '', '', '', 'sale:invoice:create', 1, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Invoice'), 'button.sale.invoice.edit', 'BUTTON', '', '', '', 'sale:invoice:edit', 2, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Invoice'), 'button.sale.invoice.delete', 'BUTTON', '', '', '', 'sale:invoice:delete', 3, '', 0)
ON CONFLICT DO NOTHING;

-- 步骤7：更新 tree_path（如果需要）
-- UPDATE mxx_system_menu SET tree_path = ... WHERE parent_id = (SELECT id FROM mxx_system_menu WHERE route_name='Sale');