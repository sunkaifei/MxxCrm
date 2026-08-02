-- v31: 交易型后台管理菜单注册（阶段8）
-- 包含：订单管理、退款管理、前台用户管理 菜单 + 按钮权限
-- 父级菜单：网站管理(id=345)
-- 幂等设计：所有 INSERT 使用 WHERE NOT EXISTS

-- =====================================================
-- 订单管理菜单 + 按钮权限
-- =====================================================
INSERT INTO mxx_system_menu (id, parent_id, name, path, type, route_name, component, perm, icon, sort, status, create_time)
SELECT 630, 345, 'page.website.orderTitle', '/website/order', 'MENU', 'WebsiteOrder', 'views/website/order/index.vue', 'website:order:list', 'lucide-shopping-cart', 10, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 630);

-- 订单按钮权限
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 631, 630, 'page.website.order.button.view', 'BUTTON', 'website:order:view', 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 631);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 632, 630, 'page.website.order.button.edit', 'BUTTON', 'website:order:update', 2, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 632);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 633, 630, 'page.website.order.button.ship', 'BUTTON', 'website:order:ship', 3, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 633);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 634, 630, 'page.website.order.button.delete', 'BUTTON', 'website:order:delete', 4, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 634);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 635, 630, 'page.website.order.button.deliveryList', 'BUTTON', 'website:delivery:list', 5, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 635);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 636, 630, 'page.website.order.button.deliveryView', 'BUTTON', 'website:delivery:view', 6, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 636);

-- =====================================================
-- 退款管理菜单 + 按钮权限
-- =====================================================
INSERT INTO mxx_system_menu (id, parent_id, name, path, type, route_name, component, perm, icon, sort, status, create_time)
SELECT 637, 345, 'page.website.refundTitle', '/website/refund', 'MENU', 'WebsiteRefund', 'views/website/refund/index.vue', 'website:refund:list', 'lucide-undo-2', 11, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 637);

-- 退款按钮权限
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 638, 637, 'page.website.refund.button.view', 'BUTTON', 'website:refund:view', 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 638);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 639, 637, 'page.website.refund.button.handle', 'BUTTON', 'website:refund:handle', 2, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 639);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 640, 637, 'page.website.refund.button.markRefunded', 'BUTTON', 'website:refund:refund', 3, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 640);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 641, 637, 'page.website.refund.button.delete', 'BUTTON', 'website:refund:delete', 4, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 641);

-- =====================================================
-- 前台用户管理菜单 + 按钮权限
-- =====================================================
INSERT INTO mxx_system_menu (id, parent_id, name, path, type, route_name, component, perm, icon, sort, status, create_time)
SELECT 642, 345, 'page.website.userTitle', '/website/user', 'MENU', 'WebsiteUser', 'views/website/user/index.vue', 'website:user:list', 'lucide-users', 12, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 642);

-- 前台用户按钮权限
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 643, 642, 'page.website.user.button.view', 'BUTTON', 'website:user:view', 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 643);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 644, 642, 'page.website.user.button.create', 'BUTTON', 'website:user:create', 2, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 644);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 645, 642, 'page.website.user.button.edit', 'BUTTON', 'website:user:update', 3, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 645);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 646, 642, 'page.website.user.button.resetPassword', 'BUTTON', 'website:user:reset', 4, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 646);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 647, 642, 'page.website.user.button.status', 'BUTTON', 'website:user:status', 5, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 647);

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status, create_time)
SELECT 648, 642, 'page.website.user.button.delete', 'BUTTON', 'website:user:delete', 6, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 648);

-- =====================================================
-- 授权：将交易型菜单授予 super_admin(1) 角色
-- =====================================================
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT 1, m.id FROM mxx_system_menu m
WHERE m.id BETWEEN 630 AND 648
AND NOT EXISTS (SELECT 1 FROM mxx_system_role_menu_merge rm WHERE rm.role_id = 1 AND rm.menu_id = m.id);

-- =====================================================
-- 验证
-- =====================================================
SELECT id, parent_id, name, path, type, perm, component, sort
FROM mxx_system_menu
WHERE id BETWEEN 630 AND 648
ORDER BY id;
