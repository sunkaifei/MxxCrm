-- ================================================
-- Mxx-CRM 菜单数据初始化脚本
-- 重新创建完整的菜单树结构
-- ================================================

-- ================================================
-- 1. 清理现有菜单数据（保留顶级菜单）
-- ================================================

DELETE FROM mxx_system_menu WHERE parent_id != 0;

-- ================================================
-- 2. 获取顶级菜单ID
-- ================================================

-- 创建临时表存储顶级菜单ID
CREATE TEMP TABLE menu_ids AS (
    SELECT id, name FROM mxx_system_menu WHERE parent_id = 0
);

-- ================================================
-- 3. 系统管理菜单（父菜单ID从临时表获取）
-- ================================================

INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
((SELECT id FROM menu_ids WHERE name='系统管理'), '用户管理', 'C', 'Admin', '/system/admin', 'system/admin/index', 'system:admin:list', 1, 'User', 0),
((SELECT id FROM menu_ids WHERE name='系统管理'), '角色管理', 'C', 'Role', '/system/role', 'system/role/index', 'system:role:list', 2, 'Team', 0),
((SELECT id FROM menu_ids WHERE name='系统管理'), '菜单管理', 'C', 'Menu', '/system/menu', 'system/menu/index', 'system:menu:list', 3, 'Menu', 0),
((SELECT id FROM menu_ids WHERE name='系统管理'), '部门管理', 'C', 'Dept', '/system/dept', 'system/dept/index', 'system:dept:list', 4, 'Building', 0),
((SELECT id FROM menu_ids WHERE name='系统管理'), '岗位管理', 'C', 'Post', '/system/post', 'system/post/index', 'system:post:list', 5, 'UserFilled', 0),
((SELECT id FROM menu_ids WHERE name='系统管理'), '系统配置', 'C', 'Config', '/system/config', 'system/config/index', 'system:config:list', 6, 'Monitor', 0),
((SELECT id FROM menu_ids WHERE name='系统管理'), '数据字典', 'C', 'Dict', '/system/dict', 'system/dict/index', 'system:dict:list', 7, 'Database', 0),
((SELECT id FROM menu_ids WHERE name='系统管理'), '通知公告', 'C', 'Notice', '/system/notice', 'system/notice/index', 'system:notice:list', 8, 'Bell', 0),
((SELECT id FROM menu_ids WHERE name='系统管理'), '系统日志', 'C', 'Log', '/system/log', 'system/log/index', 'system:log:list', 9, 'Files', 0);

-- ================================================
-- 4. 用户管理按钮权限
-- ================================================

INSERT INTO mxx_system_menu (parent_id, name, type, perm, sort, status) VALUES
((SELECT id FROM mxx_system_menu WHERE name='用户管理' AND route_name='Admin'), '新增用户', 'F', 'system:admin:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE name='用户管理' AND route_name='Admin'), '编辑用户', 'F', 'system:admin:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE name='用户管理' AND route_name='Admin'), '删除用户', 'F', 'system:admin:delete', 3, 0),
((SELECT id FROM mxx_system_menu WHERE name='用户管理' AND route_name='Admin'), '分配角色', 'F', 'system:admin:assignRole', 4, 0),
((SELECT id FROM mxx_system_menu WHERE name='用户管理' AND route_name='Admin'), '分配部门', 'F', 'system:admin:assignDept', 5, 0);

-- ================================================
-- 5. 角色管理按钮权限
-- ================================================

INSERT INTO mxx_system_menu (parent_id, name, type, perm, sort, status) VALUES
((SELECT id FROM mxx_system_menu WHERE name='角色管理' AND route_name='Role'), '新增角色', 'F', 'system:role:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE name='角色管理' AND route_name='Role'), '编辑角色', 'F', 'system:role:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE name='角色管理' AND route_name='Role'), '删除角色', 'F', 'system:role:delete', 3, 0),
((SELECT id FROM mxx_system_menu WHERE name='角色管理' AND route_name='Role'), '分配菜单', 'F', 'system:role:assignMenu', 4, 0);

-- ================================================
-- 6. 菜单管理按钮权限
-- ================================================

INSERT INTO mxx_system_menu (parent_id, name, type, perm, sort, status) VALUES
((SELECT id FROM mxx_system_menu WHERE name='菜单管理' AND route_name='Menu'), '新增菜单', 'F', 'system:menu:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE name='菜单管理' AND route_name='Menu'), '编辑菜单', 'F', 'system:menu:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE name='菜单管理' AND route_name='Menu'), '删除菜单', 'F', 'system:menu:delete', 3, 0);

-- ================================================
-- 7. CRM客户管理菜单
-- ================================================

INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
((SELECT id FROM menu_ids WHERE name='CRM客户管理'), '线索管理', 'C', 'Lead', '/crm/lead', 'crm/lead/index', 'crm:lead:list', 1, 'Search', 0),
((SELECT id FROM menu_ids WHERE name='CRM客户管理'), '客户管理', 'C', 'Customer', '/crm/customer', 'crm/customer/index', 'crm:customer:list', 2, 'User', 0),
((SELECT id FROM menu_ids WHERE name='CRM客户管理'), '联系人管理', 'C', 'Contact', '/crm/contact', 'crm/contact/index', 'crm:contact:list', 3, 'UserFilled', 0),
((SELECT id FROM menu_ids WHERE name='CRM客户管理'), '商机管理', 'C', 'Opportunity', '/crm/opportunity', 'crm/opportunity/index', 'crm:opportunity:list', 4, 'Target', 0),
((SELECT id FROM menu_ids WHERE name='CRM客户管理'), '合同管理', 'C', 'Contract', '/crm/contract', 'crm/contract/index', 'crm:contract:list', 5, 'FileText', 0),
((SELECT id FROM menu_ids WHERE name='CRM客户管理'), '线索池', 'C', 'LeadPool', '/crm/leadPool', 'crm/leadPool/index', 'crm:leadPool:list', 6, 'Pool', 0),
((SELECT id FROM menu_ids WHERE name='CRM客户管理'), '跟进记录', 'C', 'Followup', '/crm/followup', 'crm/followup/index', 'crm:followup:list', 7, 'ClockCircle', 0);

-- ================================================
-- 8. CRM按钮权限
-- ================================================

INSERT INTO mxx_system_menu (parent_id, name, type, perm, sort, status) VALUES
((SELECT id FROM mxx_system_menu WHERE name='线索管理' AND route_name='Lead'), '新增线索', 'F', 'crm:lead:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE name='线索管理' AND route_name='Lead'), '编辑线索', 'F', 'crm:lead:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE name='线索管理' AND route_name='Lead'), '删除线索', 'F', 'crm:lead:delete', 3, 0),
((SELECT id FROM mxx_system_menu WHERE name='线索管理' AND route_name='Lead'), '转为客户', 'F', 'crm:lead:convert', 4, 0),
((SELECT id FROM mxx_system_menu WHERE name='线索管理' AND route_name='Lead'), '分配线索', 'F', 'crm:lead:assign', 5, 0),
((SELECT id FROM mxx_system_menu WHERE name='客户管理' AND route_name='Customer'), '新增客户', 'F', 'crm:customer:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE name='客户管理' AND route_name='Customer'), '编辑客户', 'F', 'crm:customer:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE name='客户管理' AND route_name='Customer'), '删除客户', 'F', 'crm:customer:delete', 3, 0),
((SELECT id FROM mxx_system_menu WHERE name='客户管理' AND route_name='Customer'), '分配客户', 'F', 'crm:customer:assign', 4, 0),
((SELECT id FROM mxx_system_menu WHERE name='商机管理' AND route_name='Opportunity'), '新增商机', 'F', 'crm:opportunity:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE name='商机管理' AND route_name='Opportunity'), '编辑商机', 'F', 'crm:opportunity:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE name='商机管理' AND route_name='Opportunity'), '删除商机', 'F', 'crm:opportunity:delete', 3, 0),
((SELECT id FROM mxx_system_menu WHERE name='合同管理' AND route_name='Contract'), '新增合同', 'F', 'crm:contract:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE name='合同管理' AND route_name='Contract'), '编辑合同', 'F', 'crm:contract:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE name='合同管理' AND route_name='Contract'), '删除合同', 'F', 'crm:contract:delete', 3, 0);

-- ================================================
-- 9. 销售管理菜单
-- ================================================

INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
((SELECT id FROM menu_ids WHERE name='销售管理'), '订单管理', 'C', 'Order', '/sale/order', 'sale/order/index', 'sale:order:list', 1, 'FileText', 0),
((SELECT id FROM menu_ids WHERE name='销售管理'), '订单明细', 'C', 'OrderItem', '/sale/orderItem', 'sale/orderItem/index', 'sale:orderItem:list', 2, 'Table', 0),
((SELECT id FROM menu_ids WHERE name='销售管理'), '支付记录', 'C', 'Payment', '/sale/payment', 'sale/payment/index', 'sale:payment:list', 3, 'Wallet', 0);

-- ================================================
-- 10. 销售管理按钮权限
-- ================================================

INSERT INTO mxx_system_menu (parent_id, name, type, perm, sort, status) VALUES
((SELECT id FROM mxx_system_menu WHERE name='订单管理' AND route_name='Order'), '新增订单', 'F', 'sale:order:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE name='订单管理' AND route_name='Order'), '编辑订单', 'F', 'sale:order:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE name='订单管理' AND route_name='Order'), '删除订单', 'F', 'sale:order:delete', 3, 0),
((SELECT id FROM mxx_system_menu WHERE name='订单管理' AND route_name='Order'), '确认订单', 'F', 'sale:order:confirm', 4, 0),
((SELECT id FROM mxx_system_menu WHERE name='订单管理' AND route_name='Order'), '发货', 'F', 'sale:order:ship', 5, 0),
((SELECT id FROM mxx_system_menu WHERE name='订单管理' AND route_name='Order'), '完成订单', 'F', 'sale:order:complete', 6, 0),
((SELECT id FROM mxx_system_menu WHERE name='支付记录' AND route_name='Payment'), '新增支付', 'F', 'sale:payment:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE name='支付记录' AND route_name='Payment'), '编辑支付', 'F', 'sale:payment:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE name='支付记录' AND route_name='Payment'), '删除支付', 'F', 'sale:payment:delete', 3, 0);

-- ================================================
-- 11. 产品管理菜单
-- ================================================

INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
((SELECT id FROM menu_ids WHERE name='产品管理'), '产品列表', 'C', 'ProductMain', '/product/list', 'product/list/index', 'product:list', 1, 'Grid', 0),
((SELECT id FROM menu_ids WHERE name='产品管理'), '产品分类', 'C', 'ProductCategory', '/product/category', 'product/category/index', 'product:category:list', 2, 'FolderOpen', 0),
((SELECT id FROM menu_ids WHERE name='产品管理'), '库存管理', 'C', 'Inventory', '/product/inventory', 'product/inventory/index', 'product:inventory:list', 3, 'Box', 0),
((SELECT id FROM menu_ids WHERE name='产品管理'), '仓库管理', 'C', 'Warehouse', '/product/warehouse', 'product/warehouse/index', 'product:warehouse:list', 4, 'Home', 0);

-- ================================================
-- 12. 产品管理按钮权限
-- ================================================

INSERT INTO mxx_system_menu (parent_id, name, type, perm, sort, status) VALUES
((SELECT id FROM mxx_system_menu WHERE name='产品列表' AND route_name='ProductMain'), '新增产品', 'F', 'product:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE name='产品列表' AND route_name='ProductMain'), '编辑产品', 'F', 'product:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE name='产品列表' AND route_name='ProductMain'), '删除产品', 'F', 'product:delete', 3, 0),
((SELECT id FROM mxx_system_menu WHERE name='产品分类' AND route_name='ProductCategory'), '新增分类', 'F', 'product:category:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE name='产品分类' AND route_name='ProductCategory'), '编辑分类', 'F', 'product:category:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE name='产品分类' AND route_name='ProductCategory'), '删除分类', 'F', 'product:category:delete', 3, 0),
((SELECT id FROM mxx_system_menu WHERE name='仓库管理' AND route_name='Warehouse'), '新增仓库', 'F', 'product:warehouse:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE name='仓库管理' AND route_name='Warehouse'), '编辑仓库', 'F', 'product:warehouse:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE name='仓库管理' AND route_name='Warehouse'), '删除仓库', 'F', 'product:warehouse:delete', 3, 0);

-- ================================================
-- 13. 采购管理菜单
-- ================================================

INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
((SELECT id FROM menu_ids WHERE name='采购管理'), '供应商管理', 'C', 'Supplier', '/purchase/supplier', 'purchase/supplier/index', 'purchase:supplier:list', 1, 'Shop', 0),
((SELECT id FROM menu_ids WHERE name='采购管理'), '采购单管理', 'C', 'PurchaseOrder', '/purchase/po', 'purchase/po/index', 'purchase:po:list', 2, 'FileText', 0),
((SELECT id FROM menu_ids WHERE name='采购管理'), '采购明细', 'C', 'PurchaseItem', '/purchase/item', 'purchase/item/index', 'purchase:item:list', 3, 'Table', 0);

-- ================================================
-- 14. 采购管理按钮权限
-- ================================================

INSERT INTO mxx_system_menu (parent_id, name, type, perm, sort, status) VALUES
((SELECT id FROM mxx_system_menu WHERE name='供应商管理' AND route_name='Supplier'), '新增供应商', 'F', 'purchase:supplier:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE name='供应商管理' AND route_name='Supplier'), '编辑供应商', 'F', 'purchase:supplier:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE name='供应商管理' AND route_name='Supplier'), '删除供应商', 'F', 'purchase:supplier:delete', 3, 0),
((SELECT id FROM mxx_system_menu WHERE name='采购单管理' AND route_name='PurchaseOrder'), '新增采购单', 'F', 'purchase:po:create', 1, 0),
((SELECT id FROM mxx_system_menu WHERE name='采购单管理' AND route_name='PurchaseOrder'), '编辑采购单', 'F', 'purchase:po:edit', 2, 0),
((SELECT id FROM mxx_system_menu WHERE name='采购单管理' AND route_name='PurchaseOrder'), '删除采购单', 'F', 'purchase:po:delete', 3, 0),
((SELECT id FROM mxx_system_menu WHERE name='采购单管理' AND route_name='PurchaseOrder'), '确认采购', 'F', 'purchase:po:confirm', 4, 0),
((SELECT id FROM mxx_system_menu WHERE name='采购单管理' AND route_name='PurchaseOrder'), '入库', 'F', 'purchase:po:receive', 5, 0);

-- ================================================
-- 15. 附件管理菜单
-- ================================================

INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
((SELECT id FROM menu_ids WHERE name='附件管理'), '文件管理', 'C', 'File', '/attachment/file', 'attachment/file/index', 'attachment:file:list', 1, 'File', 0);

-- ================================================
-- 16. 附件管理按钮权限
-- ================================================

INSERT INTO mxx_system_menu (parent_id, name, type, perm, sort, status) VALUES
((SELECT id FROM mxx_system_menu WHERE name='文件管理' AND route_name='File'), '上传文件', 'F', 'attachment:file:upload', 1, 0),
((SELECT id FROM mxx_system_menu WHERE name='文件管理' AND route_name='File'), '删除文件', 'F', 'attachment:file:delete', 2, 0);

-- ================================================
-- 17. 仪表盘菜单
-- ================================================

INSERT INTO mxx_system_menu (parent_id, name, type, route_name, path, component, perm, sort, icon, status) VALUES
((SELECT id FROM menu_ids WHERE name='仪表盘'), '数据概览', 'C', 'Overview', '/dashboard/overview', 'dashboard/overview/index', 'dashboard:overview', 1, 'BarChart', 0),
((SELECT id FROM menu_ids WHERE name='仪表盘'), '销售统计', 'C', 'SalesStats', '/dashboard/sales', 'dashboard/sales/index', 'dashboard:sales', 2, 'TrendChart', 0),
((SELECT id FROM menu_ids WHERE name='仪表盘'), '客户分析', 'C', 'CustomerStats', '/dashboard/customer', 'dashboard/customer/index', 'dashboard:customer', 3, 'PieChart', 0);

-- ================================================
-- 18. 重新建立角色菜单关联
-- ================================================

TRUNCATE TABLE mxx_system_role_menu_merge RESTART IDENTITY;

-- 超级管理员拥有所有权限
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='super_admin'), id 
FROM mxx_system_menu WHERE deleted=0;

-- 系统管理员拥有系统管理权限
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='system_admin'), id 
FROM mxx_system_menu WHERE perm LIKE 'system:%' AND deleted=0;

-- 销售总监拥有CRM和销售权限
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='sales_director'), id 
FROM mxx_system_menu WHERE (perm LIKE 'crm:%' OR perm LIKE 'sale:%' OR perm LIKE 'dashboard:%') AND deleted=0;

-- 销售经理拥有CRM和销售权限
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='sales_manager'), id 
FROM mxx_system_menu WHERE (perm LIKE 'crm:%' OR perm LIKE 'sale:%') AND deleted=0;

-- 业务员拥有线索、客户和跟进权限
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='sales_rep'), id 
FROM mxx_system_menu WHERE (perm LIKE 'crm:lead:%' OR perm LIKE 'crm:customer:%' OR perm LIKE 'crm:followup:%') AND deleted=0;

-- 采购专员拥有采购和产品权限
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT (SELECT id FROM mxx_system_role WHERE role_key='purchase_staff'), id 
FROM mxx_system_menu WHERE (perm LIKE 'purchase:%' OR perm LIKE 'product:%') AND deleted=0;

-- ================================================
-- 19. 更新菜单树路径
-- ================================================

UPDATE mxx_system_menu SET tree_path = CASE 
    WHEN parent_id = 0 THEN id::TEXT
    ELSE (SELECT COALESCE(tree_path, '') FROM mxx_system_menu WHERE id = parent_id) || ',' || id::TEXT
END WHERE parent_id IS NOT NULL;

-- ================================================
-- 20. 验证结果
-- ================================================

SELECT '菜单初始化完成' AS result;