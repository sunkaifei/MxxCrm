-- ============================================================
-- v41: 补齐缺失的按钮权限码
-- 解决：内容模型字段、媒体分类操作"没权限"问题
-- 执行目标：生产数据库 mxxcrm_data
-- ============================================================

-- ============================================================
-- 1. 补齐 content:field 按钮权限（挂在 id=740 内容模型管理下）
-- 后端 content_model_field_admin_controller 使用 content:field:* 权限码
-- 菜单表仅有 content:field:list (id=745)，缺少 add/update/delete/view
-- ============================================================
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status) VALUES
  (786, 740, 'page.website.contentModel.button.fieldAdd', 'BUTTON', 'content:field:add', 6, 1),
  (787, 740, 'page.website.contentModel.button.fieldEdit', 'BUTTON', 'content:field:update', 7, 1),
  (788, 740, 'page.website.contentModel.button.fieldDelete', 'BUTTON', 'content:field:delete', 8, 1),
  (789, 740, 'page.website.contentModel.button.fieldView', 'BUTTON', 'content:field:view', 9, 1)
ON CONFLICT (id) DO UPDATE SET parent_id=EXCLUDED.parent_id, name=EXCLUDED.name, type=EXCLUDED.type, perm=EXCLUDED.perm;

-- ============================================================
-- 2. 补齐 website:media:category 按钮权限（挂在 id=730 媒体库管理下）
-- 后端 website_media_category_admin_controller 使用 website:media:category:* 权限码
-- 菜单表仅有 website:media:category:list (id=735)，缺少 add/update/delete/view
-- ============================================================
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status) VALUES
  (790, 730, 'page.website.media.button.categoryAdd', 'BUTTON', 'website:media:category:add', 6, 1),
  (791, 730, 'page.website.media.button.categoryEdit', 'BUTTON', 'website:media:category:update', 7, 1),
  (792, 730, 'page.website.media.button.categoryDelete', 'BUTTON', 'website:media:category:delete', 8, 1),
  (793, 730, 'page.website.media.button.categoryView', 'BUTTON', 'website:media:category:view', 9, 1)
ON CONFLICT (id) DO UPDATE SET parent_id=EXCLUDED.parent_id, name=EXCLUDED.name, type=EXCLUDED.type, perm=EXCLUDED.perm;

-- ============================================================
-- 3. 将新增菜单权限分配给超级管理员角色（role_id=4）
-- 注意：超级管理员(user_type=1)走 find_all 分支自动获得所有菜单权限，
-- 此处分配仅为非超级管理员角色保持一致性
-- ============================================================
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id, status)
SELECT 4, id, 1 FROM mxx_system_menu m
WHERE m.id IN (786, 787, 788, 789, 790, 791, 792, 793)
AND NOT EXISTS (
  SELECT 1 FROM mxx_system_role_menu_merge rmm
  WHERE rmm.role_id = 4 AND rmm.menu_id = m.id
);
