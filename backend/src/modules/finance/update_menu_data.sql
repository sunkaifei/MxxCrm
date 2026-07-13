-- ============================================================
-- 更新菜单数据 - 修复路由路径和组件配置
-- ============================================================

-- 更新仪表盘菜单（添加组件和重定向）
UPDATE mxx_system_menu SET 
    component = 'BasicLayout', 
    redirect = '/analytics' 
WHERE id = 1;

-- 更新分析页菜单（路径改为相对路径）
UPDATE mxx_system_menu SET 
    path = 'analytics' 
WHERE id = 2;

-- 更新工作台菜单（路径改为相对路径）
UPDATE mxx_system_menu SET 
    path = 'workspace' 
WHERE id = 3;

-- 更新财务管理菜单（添加组件和重定向）
UPDATE mxx_system_menu SET 
    component = 'BasicLayout', 
    redirect = '/finance/commission-rule' 
WHERE id = 315;

-- 更新提成规则菜单（路径改为相对路径）
UPDATE mxx_system_menu SET 
    path = 'commission-rule' 
WHERE id = 316;

-- 更新工资核算菜单（路径改为相对路径）
UPDATE mxx_system_menu SET 
    path = 'salary' 
WHERE id = 321;

-- 更新工资详情菜单（路径改为相对路径）
UPDATE mxx_system_menu SET 
    path = 'salary/detail/:id' 
WHERE id = 326;

-- 更新采购付款菜单（路径改为相对路径）
UPDATE mxx_system_menu SET 
    path = 'payment' 
WHERE id = 332;

-- 如果以上ID不存在，执行插入
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, sort, icon, redirect, params, create_time, update_time, deleted)
SELECT 1, 0, '1', 'page.dashboard.title', 'FOLDER', 'Dashboard', '/dashboard', 'BasicLayout', 'dashboard:index', 1, -1, 'lucide:layout-dashboard', '/analytics', NULL, NOW(), NOW(), 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 1);

INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, sort, icon, redirect, params, create_time, update_time, deleted)
SELECT 2, 1, '', 'page.dashboard.analytics', 'MENU', 'Analytics', 'analytics', 'dashboard/analytics/index', 'dashboard:analytics', 1, 1, 'lucide:area-chart', NULL, NULL, NOW(), NOW(), 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 2);

INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, route_name, path, component, perm, status, sort, icon, redirect, params, create_time, update_time, deleted)
SELECT 3, 1, '', 'page.dashboard.workspace', 'MENU', 'Workspace', 'workspace', 'dashboard/workspace/index', 'dashboard:workspace', 1, 2, 'carbon:workspace', NULL, NULL, NOW(), NOW(), 0
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 3);

-- 角色授权
INSERT INTO mxx_system_role_menu_merge (menu_id, role_id, status, create_time, update_time)
SELECT 1, 5, 1, NOW(), NOW() WHERE NOT EXISTS (SELECT 1 FROM mxx_system_role_menu_merge WHERE menu_id = 1 AND role_id = 5);

INSERT INTO mxx_system_role_menu_merge (menu_id, role_id, status, create_time, update_time)
SELECT 2, 5, 1, NOW(), NOW() WHERE NOT EXISTS (SELECT 1 FROM mxx_system_role_menu_merge WHERE menu_id = 2 AND role_id = 5);

INSERT INTO mxx_system_role_menu_merge (menu_id, role_id, status, create_time, update_time)
SELECT 3, 5, 1, NOW(), NOW() WHERE NOT EXISTS (SELECT 1 FROM mxx_system_role_menu_merge WHERE menu_id = 3 AND role_id = 5);

SELECT 'Menu data updated successfully' AS result;
