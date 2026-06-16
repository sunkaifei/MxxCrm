-- ================================================
-- Mxx-CRM 菜单数据初始化脚本
-- ================================================

-- 系统管理子菜单
INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
(67, 'User Management', 'C', 'Admin', '/system/admin', 'system/admin/index', 'system:admin:list', 1, 'User', 0),
(67, 'Role Management', 'C', 'Role', '/system/role', 'system/role/index', 'system:role:list', 2, 'Team', 0),
(67, 'Menu Management', 'C', 'Menu', '/system/menu', 'system/menu/index', 'system:menu:list', 3, 'Menu', 0),
(67, 'Dept Management', 'C', 'Dept', '/system/dept', 'system/dept/index', 'system:dept:list', 4, 'Building', 0),
(67, 'Post Management', 'C', 'Post', '/system/post', 'system/post/index', 'system:post:list', 5, 'UserFilled', 0),
(67, 'System Config', 'C', 'Config', '/system/config', 'system/config/index', 'system:config:list', 6, 'Monitor', 0),
(67, 'Data Dict', 'C', 'Dict', '/system/dict', 'system/dict/index', 'system:dict:list', 7, 'Database', 0),
(67, 'Notice', 'C', 'Notice', '/system/notice', 'system/notice/index', 'system:notice:list', 8, 'Bell', 0),
(67, 'System Log', 'C', 'Log', '/system/log', 'system/log/index', 'system:log:list', 9, 'Files', 0);

-- CRM子菜单
INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
(89, 'Lead Management', 'C', 'Lead', '/crm/lead', 'crm/lead/index', 'crm:lead:list', 1, 'Search', 0),
(89, 'Customer Management', 'C', 'Customer', '/crm/customer', 'crm/customer/index', 'crm:customer:list', 2, 'User', 0),
(89, 'Contact Management', 'C', 'Contact', '/crm/contact', 'crm/contact/index', 'crm:contact:list', 3, 'UserFilled', 0),
(89, 'Opportunity Management', 'C', 'Opportunity', '/crm/opportunity', 'crm/opportunity/index', 'crm:opportunity:list', 4, 'Target', 0),
(89, 'Contract Management', 'C', 'Contract', '/crm/contract', 'crm/contract/index', 'crm:contract:list', 5, 'FileText', 0),
(89, 'Lead Pool', 'C', 'LeadPool', '/crm/leadPool', 'crm/leadPool/index', 'crm:leadPool:list', 6, 'Pool', 0),
(89, 'Followup', 'C', 'Followup', '/crm/followup', 'crm/followup/index', 'crm:followup:list', 7, 'ClockCircle', 0);

-- Sale子菜单
INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
(112, 'Order Management', 'C', 'Order', '/sale/order', 'sale/order/index', 'sale:order:list', 1, 'FileText', 0),
(112, 'Order Item', 'C', 'OrderItem', '/sale/orderItem', 'sale/orderItem/index', 'sale:orderItem:list', 2, 'Table', 0),
(112, 'Payment', 'C', 'Payment', '/sale/payment', 'sale/payment/index', 'sale:payment:list', 3, 'Wallet', 0);

-- Product子菜单
INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
(125, 'Product List', 'C', 'ProductMain', '/product/list', 'product/list/index', 'product:list', 1, 'Grid', 0),
(125, 'Product Category', 'C', 'ProductCategory', '/product/category', 'product/category/index', 'product:category:list', 2, 'FolderOpen', 0),
(125, 'Inventory', 'C', 'Inventory', '/product/inventory', 'product/inventory/index', 'product:inventory:list', 3, 'Box', 0),
(125, 'Warehouse', 'C', 'Warehouse', '/product/warehouse', 'product/warehouse/index', 'product:warehouse:list', 4, 'Home', 0);

-- Purchase子菜单
INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
(139, 'Supplier', 'C', 'Supplier', '/purchase/supplier', 'purchase/supplier/index', 'purchase:supplier:list', 1, 'Shop', 0),
(139, 'Purchase Order', 'C', 'PurchaseOrder', '/purchase/po', 'purchase/po/index', 'purchase:po:list', 2, 'FileText', 0),
(139, 'Purchase Item', 'C', 'PurchaseItem', '/purchase/item', 'purchase/item/index', 'purchase:item:list', 3, 'Table', 0);

-- Attachment子菜单
INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
(151, 'File Management', 'C', 'File', '/attachment/file', 'attachment/file/index', 'attachment:file:list', 1, 'File', 0);

-- Dashboard子菜单
INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
(155, 'Overview', 'C', 'Overview', '/dashboard/overview', 'dashboard/overview/index', 'dashboard:overview', 1, 'BarChart', 0),
(155, 'Sales Stats', 'C', 'SalesStats', '/dashboard/sales', 'dashboard/sales/index', 'dashboard:sales', 2, 'TrendChart', 0),
(155, 'Customer Stats', 'C', 'CustomerStats', '/dashboard/customer', 'dashboard/customer/index', 'dashboard:customer', 3, 'PieChart', 0);

-- Buttons for Admin
INSERT INTO mxx_system_menu (parent_id, name, type, perm, sort, status) VALUES
((SELECT id FROM mxx_system_menu WHERE route_name='Admin'), 'Create', 'F', 'system:admin:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Admin'), 'Edit', 'F', 'system:admin:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Admin'), 'Delete', 'F', 'system:admin:delete', 3, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Admin'), 'Assign Role', 'F', 'system:admin:assignRole', 4, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Admin'), 'Assign Dept', 'F', 'system:admin:assignDept', 5, 0);

-- Buttons for Role
INSERT INTO mxx_system_menu (parent_id, name, type, perm, sort, status) VALUES
((SELECT id FROM mxx_system_menu WHERE route_name='Role'), 'Create', 'F', 'system:role:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Role'), 'Edit', 'F', 'system:role:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Role'), 'Delete', 'F', 'system:role:delete', 3, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Role'), 'Assign Menu', 'F', 'system:role:assignMenu', 4, 0);

-- Buttons for CRM
INSERT INTO mxx_system_menu (parent_id, name, type, perm, sort, status) VALUES
((SELECT id FROM mxx_system_menu WHERE route_name='Lead'), 'Create', 'F', 'crm:lead:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Lead'), 'Edit', 'F', 'crm:lead:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Lead'), 'Delete', 'F', 'crm:lead:delete', 3, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Lead'), 'Convert', 'F', 'crm:lead:convert', 4, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Lead'), 'Assign', 'F', 'crm:lead:assign', 5, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Customer'), 'Create', 'F', 'crm:customer:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Customer'), 'Edit', 'F', 'crm:customer:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Customer'), 'Delete', 'F', 'crm:customer:delete', 3, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Customer'), 'Assign', 'F', 'crm:customer:assign', 4, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Opportunity'), 'Create', 'F', 'crm:opportunity:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Opportunity'), 'Edit', 'F', 'crm:opportunity:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Opportunity'), 'Delete', 'F', 'crm:opportunity:delete', 3, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Contract'), 'Create', 'F', 'crm:contract:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Contract'), 'Edit', 'F', 'crm:contract:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Contract'), 'Delete', 'F', 'crm:contract:delete', 3, 0);

-- Buttons for Sale
INSERT INTO mxx_system_menu (parent_id, name, type, perm, sort, status) VALUES
((SELECT id FROM mxx_system_menu WHERE route_name='Order'), 'Create', 'F', 'sale:order:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Order'), 'Edit', 'F', 'sale:order:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Order'), 'Delete', 'F', 'sale:order:delete', 3, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Order'), 'Confirm', 'F', 'sale:order:confirm', 4, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Order'), 'Ship', 'F', 'sale:order:ship', 5, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Order'), 'Complete', 'F', 'sale:order:complete', 6, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Payment'), 'Create', 'F', 'sale:payment:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Payment'), 'Edit', 'F', 'sale:payment:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Payment'), 'Delete', 'F', 'sale:payment:delete', 3, 0);

-- Buttons for Product
INSERT INTO mxx_system_menu (parent_id, name, type, perm, sort, status) VALUES
((SELECT id FROM mxx_system_menu WHERE route_name='ProductMain'), 'Create', 'F', 'product:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='ProductMain'), 'Edit', 'F', 'product:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='ProductMain'), 'Delete', 'F', 'product:delete', 3, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='ProductCategory'), 'Create', 'F', 'product:category:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='ProductCategory'), 'Edit', 'F', 'product:category:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='ProductCategory'), 'Delete', 'F', 'product:category:delete', 3, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Warehouse'), 'Create', 'F', 'product:warehouse:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Warehouse'), 'Edit', 'F', 'product:warehouse:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Warehouse'), 'Delete', 'F', 'product:warehouse:delete', 3, 0);

-- Buttons for Purchase
INSERT INTO mxx_system_menu (parent_id, name, type, perm, sort, status) VALUES
((SELECT id FROM mxx_system_menu WHERE route_name='Supplier'), 'Create', 'F', 'purchase:supplier:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Supplier'), 'Edit', 'F', 'purchase:supplier:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='Supplier'), 'Delete', 'F', 'purchase:supplier:delete', 3, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='PurchaseOrder'), 'Create', 'F', 'purchase:po:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='PurchaseOrder'), 'Edit', 'F', 'purchase:po:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='PurchaseOrder'), 'Delete', 'F', 'purchase:po:delete', 3, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='PurchaseOrder'), 'Confirm', 'F', 'purchase:po:confirm', 4, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='PurchaseOrder'), 'Receive', 'F', 'purchase:po:receive', 5, 0);

-- Buttons for Attachment
INSERT INTO mxx_system_menu (parent_id, name, type, perm, sort, status) VALUES
((SELECT id FROM mxx_system_menu WHERE route_name='File'), 'Upload', 'F', 'attachment:file:upload', 1, 0),
((SELECT id FROM mxx_system_menu WHERE route_name='File'), 'Delete', 'F', 'attachment:file:delete', 2, 0);

-- Rebuild role-menu relations
TRUNCATE TABLE mxx_system_role_menu_merge RESTART IDENTITY;

-- Super admin
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id) SELECT (SELECT id FROM mxx_system_role WHERE role_key='super_admin'), id FROM mxx_system_menu WHERE deleted=0;

-- System admin
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id) SELECT (SELECT id FROM mxx_system_role WHERE role_key='system_admin'), id FROM mxx_system_menu WHERE perm LIKE 'system:%' AND deleted=0;

-- Sales director
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id) SELECT (SELECT id FROM mxx_system_role WHERE role_key='sales_director'), id FROM mxx_system_menu WHERE (perm LIKE 'crm:%' OR perm LIKE 'sale:%' OR perm LIKE 'dashboard:%') AND deleted=0;

-- Sales manager
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id) SELECT (SELECT id FROM mxx_system_role WHERE role_key='sales_manager'), id FROM mxx_system_menu WHERE (perm LIKE 'crm:%' OR perm LIKE 'sale:%') AND deleted=0;

-- Sales rep
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id) SELECT (SELECT id FROM mxx_system_role WHERE role_key='sales_rep'), id FROM mxx_system_menu WHERE (perm LIKE 'crm:lead:%' OR perm LIKE 'crm:customer:%' OR perm LIKE 'crm:followup:%') AND deleted=0;

-- Purchase staff
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id) SELECT (SELECT id FROM mxx_system_role WHERE role_key='purchase_staff'), id FROM mxx_system_menu WHERE (perm LIKE 'purchase:%' OR perm LIKE 'product:%') AND deleted=0;

-- Update tree path
UPDATE mxx_system_menu SET tree_path = CASE WHEN parent_id = 0 THEN id::TEXT ELSE (SELECT COALESCE(tree_path, '') FROM mxx_system_menu WHERE id = parent_id) || ',' || id::TEXT END WHERE parent_id IS NOT NULL;

SELECT 'Menu init completed' AS result;