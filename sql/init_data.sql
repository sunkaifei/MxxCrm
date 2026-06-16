-- ================================================
-- Mxx-CRM 数据库初始化数据脚本
-- 添加默认角色、部门、岗位、菜单和测试账户
-- ================================================

-- ================================================
-- 1. 默认角色数据
-- ================================================

INSERT INTO mxx_system_role (role_name, role_key, sort, data_scope, status, remark) VALUES
('超级管理员', 'super_admin', 1, 1, 1, '拥有系统全部权限'),
('系统管理员', 'system_admin', 2, 1, 1, '拥有系统管理权限'),
('销售总监', 'sales_director', 3, 1, 1, '拥有销售管理权限'),
('销售经理', 'sales_manager', 4, 2, 1, '拥有客户和销售权限'),
('业务员', 'sales_rep', 5, 3, 1, '拥有客户跟进权限'),
('采购专员', 'purchase_staff', 6, 3, 1, '拥有采购管理权限')
ON CONFLICT DO NOTHING;

-- ================================================
-- 2. 默认部门数据
-- ================================================

INSERT INTO mxx_system_dept (parent_id, ancestors, dept_name, code, sort, leader, status) VALUES
(0, '', '总公司', 'ROOT', 1, '系统管理员', 0),
(1, '1', '销售部', 'SALES', 2, '销售总监', 0),
(1, '1', '采购部', 'PURCHASE', 3, '采购经理', 0),
(1, '1', '财务部', 'FINANCE', 4, '财务经理', 0),
(1, '1', '技术部', 'TECH', 5, '技术总监', 0),
(2, '1,2', '销售一组', 'SALES_01', 6, '销售经理', 0),
(2, '1,2', '销售二组', 'SALES_02', 7, '销售经理', 0)
ON CONFLICT DO NOTHING;

-- ================================================
-- 3. 默认岗位数据
-- ================================================

INSERT INTO mxx_system_post (post_code, post_name, sort, status) VALUES
('SA', '系统管理员', 1, 0),
('SD', '销售总监', 2, 0),
('SM', '销售经理', 3, 0),
('SR', '业务员', 4, 0),
('PD', '采购经理', 5, 0),
('PS', '采购专员', 6, 0),
('FD', '财务经理', 7, 0),
('TD', '技术总监', 8, 0)
ON CONFLICT DO NOTHING;

-- ================================================
-- 4. 默认管理员账户
-- ================================================

INSERT INTO mxx_system_admin (user_name, nick_name, user_type, email, mobile, gender, password, status, sort) VALUES
('admin', '超级管理员', 1, 'admin@mxxcrm.com', '13800138000', 0, '$2b$12$8kVReYyYJFw2NE9Sv5jQwuV0KiTZ9GP4FnsPLoxdKFgEtV2J0Myn6', 0, 1),
('system', '系统管理员', 0, 'system@mxxcrm.com', '13800138001', 0, '$2b$12$8kVReYyYJFw2NE9Sv5jQwuV0KiTZ9GP4FnsPLoxdKFgEtV2J0Myn6', 0, 2),
('sales', '销售总监', 0, 'sales@mxxcrm.com', '13800138002', 0, '$2b$12$8kVReYyYJFw2NE9Sv5jQwuV0KiTZ9GP4FnsPLoxdKFgEtV2J0Myn6', 0, 3),
('manager', '销售经理', 0, 'manager@mxxcrm.com', '13800138003', 0, '$2b$12$8kVReYyYJFw2NE9Sv5jQwuV0KiTZ9GP4FnsPLoxdKFgEtV2J0Myn6', 0, 4),
('rep', '业务员', 0, 'rep@mxxcrm.com', '13800138004', 0, '$2b$12$8kVReYyYJFw2NE9Sv5jQwuV0KiTZ9GP4FnsPLoxdKFgEtV2J0Myn6', 0, 5),
('purchase', '采购专员', 0, 'purchase@mxxcrm.com', '13800138005', 0, '$2b$12$8kVReYyYJFw2NE9Sv5jQwuV0KiTZ9GP4FnsPLoxdKFgEtV2J0Myn6', 0, 6)
ON CONFLICT DO NOTHING;

-- ================================================
-- 5. 用户角色关联
-- ================================================

INSERT INTO mxx_system_admin_role_merge (admin_id, role_id) VALUES
((SELECT id FROM mxx_system_admin WHERE user_name='admin'), (SELECT id FROM mxx_system_role WHERE role_key='super_admin')),
((SELECT id FROM mxx_system_admin WHERE user_name='system'), (SELECT id FROM mxx_system_role WHERE role_key='system_admin')),
((SELECT id FROM mxx_system_admin WHERE user_name='sales'), (SELECT id FROM mxx_system_role WHERE role_key='sales_director')),
((SELECT id FROM mxx_system_admin WHERE user_name='manager'), (SELECT id FROM mxx_system_role WHERE role_key='sales_manager')),
((SELECT id FROM mxx_system_admin WHERE user_name='rep'), (SELECT id FROM mxx_system_role WHERE role_key='sales_rep')),
((SELECT id FROM mxx_system_admin WHERE user_name='purchase'), (SELECT id FROM mxx_system_role WHERE role_key='purchase_staff'))
ON CONFLICT DO NOTHING;

-- ================================================
-- 6. 用户部门关联
-- ================================================

INSERT INTO mxx_system_admin_dept_merge (admin_id, dept_id) VALUES
((SELECT id FROM mxx_system_admin WHERE user_name='admin'), (SELECT id FROM mxx_system_dept WHERE code='ROOT')),
((SELECT id FROM mxx_system_admin WHERE user_name='system'), (SELECT id FROM mxx_system_dept WHERE code='TECH')),
((SELECT id FROM mxx_system_admin WHERE user_name='sales'), (SELECT id FROM mxx_system_dept WHERE code='SALES')),
((SELECT id FROM mxx_system_admin WHERE user_name='manager'), (SELECT id FROM mxx_system_dept WHERE code='SALES_01')),
((SELECT id FROM mxx_system_admin WHERE user_name='rep'), (SELECT id FROM mxx_system_dept WHERE code='SALES_01')),
((SELECT id FROM mxx_system_admin WHERE user_name='purchase'), (SELECT id FROM mxx_system_dept WHERE code='PURCHASE'))
ON CONFLICT DO NOTHING;

-- ================================================
-- 7. 用户岗位关联
-- ================================================

INSERT INTO mxx_system_admin_post_merge (admin_id, post_id) VALUES
((SELECT id FROM mxx_system_admin WHERE user_name='admin'), (SELECT id FROM mxx_system_post WHERE post_code='SA')),
((SELECT id FROM mxx_system_admin WHERE user_name='system'), (SELECT id FROM mxx_system_post WHERE post_code='SA')),
((SELECT id FROM mxx_system_admin WHERE user_name='sales'), (SELECT id FROM mxx_system_post WHERE post_code='SD')),
((SELECT id FROM mxx_system_admin WHERE user_name='manager'), (SELECT id FROM mxx_system_post WHERE post_code='SM')),
((SELECT id FROM mxx_system_admin WHERE user_name='rep'), (SELECT id FROM mxx_system_post WHERE post_code='SR')),
((SELECT id FROM mxx_system_admin WHERE user_name='purchase'), (SELECT id FROM mxx_system_post WHERE post_code='PS'))
ON CONFLICT DO NOTHING;

-- ================================================
-- 8. 默认菜单数据（仪表盘）
-- ================================================

INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
(0, 'page.dashboard.title', 'FOLDER', 'Dashboard', '/dashboard', '', '', 0, 'Dashboard', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Dashboard'), 'page.dashboard.analytics', 'MENU', 'Analytics', '/analytics', 'dashboard/analytics/index', 'dashboard:analytics', 1, 'AreaChart', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Dashboard'), 'page.dashboard.workspace', 'MENU', 'Workspace', '/workspace', 'dashboard/workspace/index', 'dashboard:workspace', 2, 'Workspace', 0)
ON CONFLICT DO NOTHING;

-- ================================================
-- 9. 默认菜单数据（系统管理）
-- ================================================

INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
(0, 'page.system.title', 'FOLDER', 'System', '/system', '', '', 1, 'Setting', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='System'), 'page.system.user.title', 'MENU', 'Admin', '/system/admin', 'system/admin/index', 'system:admin:list', 1, 'User', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='System'), 'page.system.role.title', 'MENU', 'Role', '/system/role', 'system/role/index', 'system:role:list', 2, 'Team', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='System'), 'page.system.menu.title', 'MENU', 'Menu', '/system/menu', 'system/menu/index', 'system:menu:list', 3, 'Menu', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='System'), 'page.system.dept.title', 'MENU', 'Dept', '/system/dept', 'system/dept/index', 'system:dept:list', 4, 'Building', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='System'), 'page.system.post.title', 'MENU', 'Post', '/system/post', 'system/post/index', 'system:post:list', 5, 'UserFilled', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='System'), 'page.system.config.title', 'MENU', 'Config', '/system/config', 'system/config/index', 'system:config:list', 6, 'Monitor', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='System'), 'page.system.dict.title', 'MENU', 'Dict', '/system/dict', 'system/dict/index', 'system:dict:list', 7, 'Database', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='System'), 'page.system.notice.title', 'MENU', 'Notice', '/system/notice', 'system/notice/index', 'system:notice:list', 8, 'Bell', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='System'), 'page.system.log.title', 'MENU', 'Log', '/system/log', 'system/log/index', 'system:log:list', 9, 'Files', 0)
ON CONFLICT DO NOTHING;

-- 用户管理按钮权限
INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
((SELECT id FROM mxx_system_menu WHERE route_name='Admin'), 'button.system.admin.create', 'BUTTON', '', '', '', 'system:admin:create', 1, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Admin'), 'button.system.admin.edit', 'BUTTON', '', '', '', 'system:admin:edit', 2, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Admin'), 'button.system.admin.delete', 'BUTTON', '', '', '', 'system:admin:delete', 3, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Admin'), 'button.system.admin.assignRole', 'BUTTON', '', '', '', 'system:admin:assignRole', 4, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Admin'), 'button.system.admin.assignDept', 'BUTTON', '', '', '', 'system:admin:assignDept', 5, '', 0),
-- 角色管理按钮权限
((SELECT id FROM mxx_system_menu WHERE route_name='Role'), 'button.system.role.create', 'BUTTON', '', '', '', 'system:role:create', 1, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Role'), 'button.system.role.edit', 'BUTTON', '', '', '', 'system:role:edit', 2, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Role'), 'button.system.role.delete', 'BUTTON', '', '', '', 'system:role:delete', 3, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Role'), 'button.system.role.assignMenu', 'BUTTON', '', '', '', 'system:role:assignMenu', 4, '', 0),
-- 菜单管理按钮权限
((SELECT id FROM mxx_system_menu WHERE route_name='Menu'), 'button.system.menu.create', 'BUTTON', '', '', '', 'system:menu:create', 1, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Menu'), 'button.system.menu.edit', 'BUTTON', '', '', '', 'system:menu:edit', 2, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Menu'), 'button.system.menu.delete', 'BUTTON', '', '', '', 'system:menu:delete', 3, '', 0)
ON CONFLICT DO NOTHING;

-- ================================================
-- 10. 默认菜单数据（CRM客户管理）
-- ================================================

INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
(0, 'page.crm.title', 'FOLDER', 'Crm', '/crm', '', '', 2, 'Users', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Crm'), 'page.crm.lead.title', 'MENU', 'Lead', '/crm/lead', 'crm/lead/index', 'crm:lead:list', 1, 'Search', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Crm'), 'page.crm.customer.title', 'MENU', 'Customer', '/crm/customer', 'crm/customer/index', 'crm:customer:list', 2, 'User', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Crm'), 'page.crm.contact.title', 'MENU', 'Contact', '/crm/contact', 'crm/contact/index', 'crm:contact:list', 3, 'UserFilled', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Crm'), 'page.crm.opportunity.title', 'MENU', 'Opportunity', '/crm/opportunity', 'crm/opportunity/index', 'crm:opportunity:list', 4, 'Target', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Crm'), 'page.crm.contract.title', 'MENU', 'Contract', '/crm/contract', 'crm/contract/index', 'crm:contract:list', 5, 'FileText', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Crm'), 'page.crm.leadPool.title', 'MENU', 'LeadPool', '/crm/leadPool', 'crm/leadPool/index', 'crm:leadPool:list', 6, 'Pool', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Crm'), 'page.crm.followup.title', 'MENU', 'Followup', '/crm/followup', 'crm/followup/index', 'crm:followup:list', 7, 'ClockCircle', 0)
ON CONFLICT DO NOTHING;

-- 线索管理按钮权限
INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
((SELECT id FROM mxx_system_menu WHERE route_name='Lead'), 'button.crm.lead.create', 'BUTTON', '', '', '', 'crm:lead:create', 1, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Lead'), 'button.crm.lead.edit', 'BUTTON', '', '', '', 'crm:lead:edit', 2, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Lead'), 'button.crm.lead.delete', 'BUTTON', '', '', '', 'crm:lead:delete', 3, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Lead'), 'button.crm.lead.convert', 'BUTTON', '', '', '', 'crm:lead:convert', 4, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Lead'), 'button.crm.lead.assign', 'BUTTON', '', '', '', 'crm:lead:assign', 5, '', 0),
-- 客户管理按钮权限
((SELECT id FROM mxx_system_menu WHERE route_name='Customer'), 'button.crm.customer.create', 'BUTTON', '', '', '', 'crm:customer:create', 1, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Customer'), 'button.crm.customer.edit', 'BUTTON', '', '', '', 'crm:customer:edit', 2, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Customer'), 'button.crm.customer.delete', 'BUTTON', '', '', '', 'crm:customer:delete', 3, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Customer'), 'button.crm.customer.assign', 'BUTTON', '', '', '', 'crm:customer:assign', 4, '', 0),
-- 商机管理按钮权限
((SELECT id FROM mxx_system_menu WHERE route_name='Opportunity'), 'button.crm.opportunity.create', 'BUTTON', '', '', '', 'crm:opportunity:create', 1, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Opportunity'), 'button.crm.opportunity.edit', 'BUTTON', '', '', '', 'crm:opportunity:edit', 2, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Opportunity'), 'button.crm.opportunity.delete', 'BUTTON', '', '', '', 'crm:opportunity:delete', 3, '', 0),
-- 合同管理按钮权限
((SELECT id FROM mxx_system_menu WHERE route_name='Contract'), 'button.crm.contract.create', 'BUTTON', '', '', '', 'crm:contract:create', 1, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Contract'), 'button.crm.contract.edit', 'BUTTON', '', '', '', 'crm:contract:edit', 2, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Contract'), 'button.crm.contract.delete', 'BUTTON', '', '', '', 'crm:contract:delete', 3, '', 0)
ON CONFLICT DO NOTHING;

-- ================================================
-- 11. 默认菜单数据（销售管理）
-- ================================================

INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
(0, 'page.sale.title', 'FOLDER', 'Sale', '/sale', '', '', 3, 'ShoppingCart', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Sale'), 'page.sale.order.title', 'MENU', 'Order', '/sale/order', 'sale/order/index', 'sale:order:list', 1, 'FileText', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Sale'), 'page.sale.orderItem.title', 'MENU', 'OrderItem', '/sale/orderItem', 'sale/orderItem/index', 'sale:orderItem:list', 2, 'Table', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Sale'), 'page.sale.payment.title', 'MENU', 'Payment', '/sale/payment', 'sale/payment/index', 'sale:payment:list', 3, 'Wallet', 0)
ON CONFLICT DO NOTHING;

-- 订单管理按钮权限
INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
((SELECT id FROM mxx_system_menu WHERE route_name='Order'), 'button.sale.order.create', 'BUTTON', '', '', '', 'sale:order:create', 1, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Order'), 'button.sale.order.edit', 'BUTTON', '', '', '', 'sale:order:edit', 2, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Order'), 'button.sale.order.delete', 'BUTTON', '', '', '', 'sale:order:delete', 3, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Order'), 'button.sale.order.confirm', 'BUTTON', '', '', '', 'sale:order:confirm', 4, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Order'), 'button.sale.order.ship', 'BUTTON', '', '', '', 'sale:order:ship', 5, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Order'), 'button.sale.order.complete', 'BUTTON', '', '', '', 'sale:order:complete', 6, '', 0),
-- 支付记录按钮权限
((SELECT id FROM mxx_system_menu WHERE route_name='Payment'), 'button.sale.payment.create', 'BUTTON', '', '', '', 'sale:payment:create', 1, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Payment'), 'button.sale.payment.edit', 'BUTTON', '', '', '', 'sale:payment:edit', 2, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Payment'), 'button.sale.payment.delete', 'BUTTON', '', '', '', 'sale:payment:delete', 3, '', 0)
ON CONFLICT DO NOTHING;

-- ================================================
-- 12. 默认菜单数据（产品管理）
-- ================================================

INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
(0, 'page.product.title', 'FOLDER', 'Product', '/product', '', '', 4, 'Package', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Product'), 'page.product.list.title', 'MENU', 'ProductMain', '/product/list', 'product/list/index', 'product:list', 1, 'Grid', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Product'), 'page.product.category.title', 'MENU', 'ProductCategory', '/product/category', 'product/category/index', 'product:category:list', 2, 'FolderOpen', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Product'), 'page.product.inventory.title', 'MENU', 'Inventory', '/product/inventory', 'product/inventory/index', 'product:inventory:list', 3, 'Box', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Product'), 'page.product.warehouse.title', 'MENU', 'Warehouse', '/product/warehouse', 'product/warehouse/index', 'product:warehouse:list', 4, 'Home', 0)
ON CONFLICT DO NOTHING;

-- 产品列表按钮权限
INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
((SELECT id FROM mxx_system_menu WHERE route_name='ProductMain'), 'button.product.create', 'BUTTON', '', '', '', 'product:create', 1, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='ProductMain'), 'button.product.edit', 'BUTTON', '', '', '', 'product:edit', 2, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='ProductMain'), 'button.product.delete', 'BUTTON', '', '', '', 'product:delete', 3, '', 0),
-- 产品分类按钮权限
((SELECT id FROM mxx_system_menu WHERE route_name='ProductCategory'), 'button.product.category.create', 'BUTTON', '', '', '', 'product:category:create', 1, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='ProductCategory'), 'button.product.category.edit', 'BUTTON', '', '', '', 'product:category:edit', 2, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='ProductCategory'), 'button.product.category.delete', 'BUTTON', '', '', '', 'product:category:delete', 3, '', 0),
-- 仓库管理按钮权限
((SELECT id FROM mxx_system_menu WHERE route_name='Warehouse'), 'button.product.warehouse.create', 'BUTTON', '', '', '', 'product:warehouse:create', 1, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Warehouse'), 'button.product.warehouse.edit', 'BUTTON', '', '', '', 'product:warehouse:edit', 2, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Warehouse'), 'button.product.warehouse.delete', 'BUTTON', '', '', '', 'product:warehouse:delete', 3, '', 0)
ON CONFLICT DO NOTHING;

-- ================================================
-- 13. 默认菜单数据（采购管理）
-- ================================================

INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
(0, 'page.purchase.title', 'FOLDER', 'Purchase', '/purchase', '', '', 5, 'Shopping', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Purchase'), 'page.purchase.supplier.title', 'MENU', 'Supplier', '/purchase/supplier', 'purchase/supplier/index', 'purchase:supplier:list', 1, 'Shop', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Purchase'), 'page.purchase.po.title', 'MENU', 'PurchaseOrder', '/purchase/po', 'purchase/po/index', 'purchase:po:list', 2, 'FileText', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Purchase'), 'page.purchase.item.title', 'MENU', 'PurchaseItem', '/purchase/item', 'purchase/item/index', 'purchase:item:list', 3, 'Table', 0)
ON CONFLICT DO NOTHING;

-- 供应商管理按钮权限
INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
((SELECT id FROM mxx_system_menu WHERE route_name='Supplier'), 'button.purchase.supplier.create', 'BUTTON', '', '', '', 'purchase:supplier:create', 1, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Supplier'), 'button.purchase.supplier.edit', 'BUTTON', '', '', '', 'purchase:supplier:edit', 2, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Supplier'), 'button.purchase.supplier.delete', 'BUTTON', '', '', '', 'purchase:supplier:delete', 3, '', 0),
-- 采购单管理按钮权限
((SELECT id FROM mxx_system_menu WHERE route_name='PurchaseOrder'), 'button.purchase.po.create', 'BUTTON', '', '', '', 'purchase:po:create', 1, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='PurchaseOrder'), 'button.purchase.po.edit', 'BUTTON', '', '', '', 'purchase:po:edit', 2, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='PurchaseOrder'), 'button.purchase.po.delete', 'BUTTON', '', '', '', 'purchase:po:delete', 3, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='PurchaseOrder'), 'button.purchase.po.confirm', 'BUTTON', '', '', '', 'purchase:po:confirm', 4, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='PurchaseOrder'), 'button.purchase.po.receive', 'BUTTON', '', '', '', 'purchase:po:receive', 5, '', 0)
ON CONFLICT DO NOTHING;

-- ================================================
-- 14. 默认菜单数据（附件管理）
-- ================================================

INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
(0, 'page.attachment.title', 'FOLDER', 'Attachment', '/attachment', '', '', 6, 'Paperclip', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Attachment'), 'page.attachment.file.title', 'MENU', 'File', '/attachment/file', 'attachment/file/index', 'attachment:file:list', 1, 'File', 0)
ON CONFLICT DO NOTHING;

-- 文件管理按钮权限
INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
((SELECT id FROM mxx_system_menu WHERE route_name='File'), 'button.attachment.file.upload', 'BUTTON', '', '', '', 'attachment:file:upload', 1, '', 0),
((SELECT id FROM mxx_system_menu WHERE route_name='File'), 'button.attachment.file.delete', 'BUTTON', '', '', '', 'attachment:file:delete', 2, '', 0)
ON CONFLICT DO NOTHING;

-- ================================================
-- 15. 角色菜单关联（超级管理员拥有所有权限）
-- ================================================

INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='super_admin'), id 
FROM mxx_system_menu WHERE deleted=0
ON CONFLICT DO NOTHING;

-- ================================================
-- 16. 角色菜单关联（系统管理员）
-- ================================================

INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='system_admin'), id 
FROM mxx_system_menu WHERE perm LIKE 'system:%' AND deleted=0
ON CONFLICT DO NOTHING;

-- ================================================
-- 17. 角色菜单关联（销售相关角色）
-- ================================================

INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='sales_director'), id 
FROM mxx_system_menu WHERE perm LIKE 'crm:%' OR perm LIKE 'sale:%' OR perm LIKE 'dashboard:%' AND deleted=0
ON CONFLICT DO NOTHING;

INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='sales_manager'), id 
FROM mxx_system_menu WHERE perm LIKE 'crm:%' OR perm LIKE 'sale:%' AND deleted=0
ON CONFLICT DO NOTHING;

INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='sales_rep'), id 
FROM mxx_system_menu WHERE perm LIKE 'crm:lead:%' OR perm LIKE 'crm:customer:%' OR perm LIKE 'crm:followup:%' AND deleted=0
ON CONFLICT DO NOTHING;

-- ================================================
-- 18. 角色菜单关联（采购专员）
-- ================================================

INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='purchase_staff'), id 
FROM mxx_system_menu WHERE perm LIKE 'purchase:%' OR perm LIKE 'product:%' AND deleted=0
ON CONFLICT DO NOTHING;

-- ================================================
-- 19. 更新菜单树路径
-- ================================================

UPDATE mxx_system_menu SET tree_path = CASE 
    WHEN parent_id = 0 THEN id::TEXT
    ELSE (SELECT tree_path FROM mxx_system_menu WHERE id = parent_id) || ',' || id::TEXT
END WHERE parent_id IS NOT NULL;

-- ================================================
-- 20. 插入示例数据
-- ================================================

INSERT INTO mxx_product_category (parent_id, name, sort_order) VALUES
(0, '电子产品', 1),
(0, '服装鞋帽', 2),
(0, '家居用品', 3),
(1, '手机数码', 1),
(1, '电脑配件', 2),
(2, '男装', 1),
(2, '女装', 2),
(3, '厨房用品', 1),
(3, '卧室用品', 2)
ON CONFLICT DO NOTHING;

INSERT INTO mxx_product_main (product_no, name, category_id, unit, cost_price, sale_price, is_active) VALUES
('PRD000001', '智能手机 X1', 4, '台', 800.00, 1299.00, true),
('PRD000002', '笔记本电脑 Pro', 5, '台', 3500.00, 4999.00, true),
('PRD000003', '运动T恤', 6, '件', 50.00, 99.00, true),
('PRD000004', '休闲运动鞋', 6, '双', 80.00, 169.00, true),
('PRD000005', '不粘锅套装', 8, '套', 150.00, 299.00, true),
('PRD000006', '床上四件套', 9, '套', 180.00, 359.00, true)
ON CONFLICT DO NOTHING;

INSERT INTO mxx_inventory_warehouse (name, code, address) VALUES
('主仓库', 'WH001', '深圳市南山区科技园'),
('备用仓库', 'WH002', '深圳市宝安区工业园'),
('广州仓库', 'WH003', '广州市天河区')
ON CONFLICT DO NOTHING;

INSERT INTO mxx_inventory_stock (product_id, warehouse_id, quantity, reserved_quantity) VALUES
((SELECT id FROM mxx_product_main WHERE product_no='PRD000001'), 1, 100, 10),
((SELECT id FROM mxx_product_main WHERE product_no='PRD000001'), 2, 50, 5),
((SELECT id FROM mxx_product_main WHERE product_no='PRD000002'), 1, 30, 2),
((SELECT id FROM mxx_product_main WHERE product_no='PRD000003'), 1, 500, 20),
((SELECT id FROM mxx_product_main WHERE product_no='PRD000005'), 3, 200, 15)
ON CONFLICT DO NOTHING;

INSERT INTO mxx_purchase_supplier (supplier_no, name, contact_person, phone, email) VALUES
('SUP000001', '深圳电子科技有限公司', '张经理', '0755-12345678', 'zhang@sztech.com'),
('SUP000002', '广州服装贸易公司', '李总', '020-87654321', 'li@gztrade.com'),
('SUP000003', '佛山家居用品厂', '王厂长', '0757-11223344', 'wang@fshome.com')
ON CONFLICT DO NOTHING;

INSERT INTO mxx_crm_customer (customer_no, company_name, short_name, country, region, industry, level, assigned_to) VALUES
('CUST20240100001', '阿里巴巴集团', '阿里巴巴', '中国', '浙江省杭州市', 'ecommerce', 'svip', (SELECT id FROM mxx_system_admin WHERE user_name='manager')),
('CUST20240100002', '腾讯科技', '腾讯', '中国', '广东省深圳市', 'ecommerce', 'svip', (SELECT id FROM mxx_system_admin WHERE user_name='manager')),
('CUST20240100003', '华为技术有限公司', '华为', '中国', '广东省深圳市', 'manufacturer', 'vip', (SELECT id FROM mxx_system_admin WHERE user_name='rep')),
('CUST20240100004', '字节跳动', '字节', '中国', '北京市', 'ecommerce', 'vip', (SELECT id FROM mxx_system_admin WHERE user_name='rep')),
('CUST20240100005', '小米科技', '小米', '中国', '北京市', 'manufacturer', 'normal', (SELECT id FROM mxx_system_admin WHERE user_name='rep'))
ON CONFLICT DO NOTHING;

INSERT INTO mxx_crm_contact (customer_id, name, title, email, mobile, is_primary) VALUES
((SELECT id FROM mxx_crm_customer WHERE customer_no='CUST20240100001'), '马云', 'CEO', 'jack@alibaba.com', '13800138001', true),
((SELECT id FROM mxx_crm_customer WHERE customer_no='CUST20240100002'), '马化腾', 'CEO', 'pony@tencent.com', '13800138002', true),
((SELECT id FROM mxx_crm_customer WHERE customer_no='CUST20240100003'), '任正非', 'CEO', 'ren@huawei.com', '13800138003', true),
((SELECT id FROM mxx_crm_customer WHERE customer_no='CUST20240100004'), '张一鸣', 'CEO', 'zhang@bytedance.com', '13800138004', true),
((SELECT id FROM mxx_crm_customer WHERE customer_no='CUST20240100005'), '雷军', 'CEO', 'lei@xiaomi.com', '13800138005', true)
ON CONFLICT DO NOTHING;

INSERT INTO mxx_crm_lead (company_name, contact_name, title, email, phone, mobile, country, region, industry, source, status, assigned_to) VALUES
('美团', '王兴', 'CEO', 'wang@meituan.com', '010-12345678', '13800138006', '中国', '北京市', 'ecommerce', 'website', 'new', (SELECT id FROM mxx_system_admin WHERE user_name='rep')),
('京东集团', '刘强东', 'CEO', 'liu@jd.com', '010-87654321', '13800138007', '中国', '北京市', 'ecommerce', 'website', 'new', (SELECT id FROM mxx_system_admin WHERE user_name='rep')),
('拼多多', '黄峥', 'CEO', 'huang@pinduoduo.com', '021-11223344', '13800138008', '中国', '上海市', 'ecommerce', 'social', 'following', (SELECT id FROM mxx_system_admin WHERE user_name='manager')),
('网易', '丁磊', 'CEO', 'ding@netease.com', '0571-88888888', '13800138009', '中国', '浙江省杭州市', 'ecommerce', 'referral', 'new', (SELECT id FROM mxx_system_admin WHERE user_name='rep')),
('小红书', '毛文超', 'CEO', 'mao@xiaohongshu.com', '021-99999999', '13800138010', '中国', '上海市', 'social', 'social', 'following', (SELECT id FROM mxx_system_admin WHERE user_name='manager'))
ON CONFLICT DO NOTHING;

INSERT INTO mxx_crm_opportunity (customer_id, name, amount, stage, probability, assigned_to) VALUES
((SELECT id FROM mxx_crm_customer WHERE customer_no='CUST20240100001'), '年度采购项目', 5000000.00, 'negotiation', 80, (SELECT id FROM mxx_system_admin WHERE user_name='manager')),
((SELECT id FROM mxx_crm_customer WHERE customer_no='CUST20240100002'), '云服务采购', 3000000.00, 'proposal', 60, (SELECT id FROM mxx_system_admin WHERE user_name='manager')),
((SELECT id FROM mxx_crm_customer WHERE customer_no='CUST20240100003'), '企业定制项目', 2000000.00, 'needs_analysis', 40, (SELECT id FROM mxx_system_admin WHERE user_name='rep')),
((SELECT id FROM mxx_crm_customer WHERE customer_no='CUST20240100004'), '广告合作项目', 1500000.00, 'qualification', 20, (SELECT id FROM mxx_system_admin WHERE user_name='rep'))
ON CONFLICT DO NOTHING;

INSERT INTO mxx_crm_contract (contract_no, customer_id, opportunity_id, name, amount, status, sign_date) VALUES
('CON20240100001', (SELECT id FROM mxx_crm_customer WHERE customer_no='CUST20240100001'), 1, '2024年度采购合同', 5000000.00, 'signed', '2024-01-15'),
('CON20240100002', (SELECT id FROM mxx_crm_customer WHERE customer_no='CUST20240100002'), 2, '云服务合同', 3000000.00, 'executing', '2024-02-20')
ON CONFLICT DO NOTHING;

INSERT INTO mxx_sale_order (order_no, customer_id, contract_id, order_date, amount, status, payment_status, assigned_to) VALUES
('ORD20240300001', (SELECT id FROM mxx_crm_customer WHERE customer_no='CUST20240100001'), 1, '2024-03-01', 1000000.00, 'completed', 'paid', (SELECT id FROM mxx_system_admin WHERE user_name='manager')),
('ORD20240300002', (SELECT id FROM mxx_crm_customer WHERE customer_no='CUST20240100001'), 1, '2024-03-15', 1500000.00, 'delivered', 'paid', (SELECT id FROM mxx_system_admin WHERE user_name='manager')),
('ORD20240400001', (SELECT id FROM mxx_crm_customer WHERE customer_no='CUST20240100002'), 2, '2024-04-01', 800000.00, 'shipping', 'partial', (SELECT id FROM mxx_system_admin WHERE user_name='rep')),
('ORD20240400002', (SELECT id FROM mxx_crm_customer WHERE customer_no='CUST20240100003'), NULL, '2024-04-10', 500000.00, 'pending', 'unpaid', (SELECT id FROM mxx_system_admin WHERE user_name='rep'))
ON CONFLICT DO NOTHING;

INSERT INTO mxx_sale_order_item (order_id, product_id, product_name, quantity, unit_price, total_amount) VALUES
(1, (SELECT id FROM mxx_product_main WHERE product_no='PRD000001'), '智能手机 X1', 500, 1299.00, 649500.00),
(1, (SELECT id FROM mxx_product_main WHERE product_no='PRD000002'), '笔记本电脑 Pro', 50, 4999.00, 249950.00),
(2, (SELECT id FROM mxx_product_main WHERE product_no='PRD000001'), '智能手机 X1', 800, 1299.00, 1039200.00),
(2, (SELECT id FROM mxx_product_main WHERE product_no='PRD000003'), '运动T恤', 2000, 99.00, 198000.00),
(3, (SELECT id FROM mxx_product_main WHERE product_no='PRD000002'), '笔记本电脑 Pro', 100, 4999.00, 499900.00),
(3, (SELECT id FROM mxx_product_main WHERE product_no='PRD000004'), '休闲运动鞋', 3000, 169.00, 507000.00)
ON CONFLICT DO NOTHING;

INSERT INTO mxx_sale_payment (payment_no, order_id, customer_id, amount, payment_method, payment_date) VALUES
('PAY20240300001', 1, (SELECT id FROM mxx_crm_customer WHERE customer_no='CUST20240100001'), 1000000.00, 'bank_transfer', '2024-03-02'),
('PAY20240300002', 2, (SELECT id FROM mxx_crm_customer WHERE customer_no='CUST20240100001'), 1500000.00, 'bank_transfer', '2024-03-16'),
('PAY20240400001', 3, (SELECT id FROM mxx_crm_customer WHERE customer_no='CUST20240100002'), 500000.00, 'bank_transfer', '2024-04-02')
ON CONFLICT DO NOTHING;

INSERT INTO mxx_purchase_po (purchase_no, supplier_id, purchase_date, expected_date, amount, status) VALUES
('PO20240300001', 1, '2024-03-01', '2024-03-15', 500000.00, 'completed'),
('PO20240300002', 2, '2024-03-10', '2024-03-25', 300000.00, 'completed'),
('PO20240400001', 1, '2024-04-01', '2024-04-15', 400000.00, 'in_transit'),
('PO20240400002', 3, '2024-04-10', '2024-04-25', 250000.00, 'ordered')
ON CONFLICT DO NOTHING;

INSERT INTO mxx_purchase_item (purchase_id, product_id, product_name, quantity, unit_price, total_amount) VALUES
(1, (SELECT id FROM mxx_product_main WHERE product_no='PRD000001'), '智能手机 X1', 500, 800.00, 400000.00),
(1, (SELECT id FROM mxx_product_main WHERE product_no='PRD000002'), '笔记本电脑 Pro', 20, 3500.00, 70000.00),
(2, (SELECT id FROM mxx_product_main WHERE product_no='PRD000003'), '运动T恤', 3000, 50.00, 150000.00),
(2, (SELECT id FROM mxx_product_main WHERE product_no='PRD000004'), '休闲运动鞋', 1000, 80.00, 80000.00),
(3, (SELECT id FROM mxx_product_main WHERE product_no='PRD000001'), '智能手机 X1', 400, 800.00, 320000.00),
(4, (SELECT id FROM mxx_product_main WHERE product_no='PRD000005'), '不粘锅套装', 1000, 150.00, 150000.00)
ON CONFLICT DO NOTHING;

INSERT INTO mxx_inventory_transaction (product_id, warehouse_id, operation_type, quantity, reference_type, reference_id) VALUES
((SELECT id FROM mxx_product_main WHERE product_no='PRD000001'), 1, 'in', 500, 'purchase', 1),
((SELECT id FROM mxx_product_main WHERE product_no='PRD000002'), 1, 'in', 20, 'purchase', 1),
((SELECT id FROM mxx_product_main WHERE product_no='PRD000003'), 1, 'in', 3000, 'purchase', 2),
((SELECT id FROM mxx_product_main WHERE product_no='PRD000001'), 1, 'out', 500, 'sale', 1),
((SELECT id FROM mxx_product_main WHERE product_no='PRD000002'), 1, 'out', 50, 'sale', 1),
((SELECT id FROM mxx_product_main WHERE product_no='PRD000001'), 1, 'out', 800, 'sale', 2)
ON CONFLICT DO NOTHING;

INSERT INTO mxx_crm_followup (lead_id, customer_id, activity_type, subject, content, next_follow_date) VALUES
(1, NULL, 'call', '初次联系', '电话联系客户，介绍公司产品和服务，客户表示有兴趣', '2024-04-20'),
(2, NULL, 'email', '发送产品资料', '发送产品详细资料和报价单', '2024-04-25'),
(3, NULL, 'meeting', '商务会谈', '与客户进行线下会谈，深入了解需求', '2024-04-30'),
(NULL, 1, 'wechat', '日常跟进', '微信跟进项目进度，客户反馈良好', '2024-04-22'),
(NULL, 2, 'email', '合同跟进', '跟进合同签署情况', '2024-04-28')
ON CONFLICT DO NOTHING;

INSERT INTO mxx_crm_lead_pool (name, description, recycle_days, member_ids) VALUES
('公海池', '所有未分配的线索', 30, (SELECT ARRAY(SELECT id FROM mxx_system_admin WHERE user_name IN ('sales', 'manager', 'rep')))),
('VIP池', '高价值线索专用池', 15, (SELECT ARRAY(SELECT id FROM mxx_system_admin WHERE user_name IN ('sales', 'manager'))))
ON CONFLICT DO NOTHING;

INSERT INTO mxx_attachment_file (file_name, file_path, file_size, file_type, related_type, related_id, uploaded_by) VALUES
('合同文件.pdf', '/storage/upload/contract/202403/contract_001.pdf', 2048000, 'pdf', 'contract', 1, 1),
('产品图片.jpg', '/storage/upload/product/202403/product_001.jpg', 512000, 'image', 'product', 1, 1)
ON CONFLICT DO NOTHING;

-- ================================================
-- 完成
-- ================================================

SELECT '初始化数据完成' AS result;