-- ============================================================
-- v40: 网站管理后台菜单与权限补齐修复脚本
-- 解决：前端页面提示"接口不存在"或"没权限"的问题
-- 修复：补齐缺失的菜单注册 + 修复错误的 component 路径 + 统一权限码
-- 执行目标：生产数据库 mxxcrm_data
-- ============================================================

-- ============================================================
-- 1. 修复已有菜单的错误配置
-- ============================================================

-- 1.1 修复 id=350 站点设置：component 指向错误文件 + 权限码不匹配
-- 前端实际文件：views/website/site/settings.vue
-- 后端权限码：system:site:view / system:site:update
UPDATE mxx_system_menu SET
  component = 'views/website/site/settings.vue',
  perm = 'system:site:view',
  name = 'page.website.list',
  path = '/website/list'
WHERE id = 350;

-- 1.2 修复 id=652 通知配置：component 指向不存在的文件
-- 通知配置已合并到站点设置页，此菜单应隐藏（不删除，保留权限码）
UPDATE mxx_system_menu SET
  hide_in_menu = 1,
  component = NULL
WHERE id = 652;

-- ============================================================
-- 2. 补齐缺失的 website 模块菜单（7 个页面）
-- 父菜单 id=345（网站管理）
-- ============================================================

-- 2.1 Banner 管理（id=700-705）
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, path, component, icon, sort, status) VALUES
  (700, 345, 'page.website.bannerTitle', 'MENU', 'website:banner:list', '/website/banner', 'views/website/banner/index.vue', 'lucide-image', 20, 1)
ON CONFLICT (id) DO UPDATE SET parent_id=EXCLUDED.parent_id, name=EXCLUDED.name, type=EXCLUDED.type, perm=EXCLUDED.perm, path=EXCLUDED.path, component=EXCLUDED.component;

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status) VALUES
  (701, 700, 'page.website.banner.button.view', 'BUTTON', 'website:banner:view', 1, 1),
  (702, 700, 'page.website.banner.button.add', 'BUTTON', 'website:banner:add', 2, 1),
  (703, 700, 'page.website.banner.button.edit', 'BUTTON', 'website:banner:update', 3, 1),
  (704, 700, 'page.website.banner.button.delete', 'BUTTON', 'website:banner:delete', 4, 1)
ON CONFLICT (id) DO UPDATE SET parent_id=EXCLUDED.parent_id, name=EXCLUDED.name, type=EXCLUDED.type, perm=EXCLUDED.perm;

-- 2.2 区块管理（id=710-714）
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, path, component, icon, sort, status) VALUES
  (710, 345, 'page.website.blockTitle', 'MENU', 'website:block:list', '/website/block', 'views/website/block/index.vue', 'lucide-layout-dashboard', 21, 1)
ON CONFLICT (id) DO UPDATE SET parent_id=EXCLUDED.parent_id, name=EXCLUDED.name, type=EXCLUDED.type, perm=EXCLUDED.perm, path=EXCLUDED.path, component=EXCLUDED.component;

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status) VALUES
  (711, 710, 'page.website.block.button.view', 'BUTTON', 'website:block:view', 1, 1),
  (712, 710, 'page.website.block.button.add', 'BUTTON', 'website:block:add', 2, 1),
  (713, 710, 'page.website.block.button.edit', 'BUTTON', 'website:block:update', 3, 1),
  (714, 710, 'page.website.block.button.delete', 'BUTTON', 'website:block:delete', 4, 1)
ON CONFLICT (id) DO UPDATE SET parent_id=EXCLUDED.parent_id, name=EXCLUDED.name, type=EXCLUDED.type, perm=EXCLUDED.perm;

-- 2.3 自定义页面管理（id=720-724）
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, path, component, icon, sort, status) VALUES
  (720, 345, 'page.website.pageTitle', 'MENU', 'website:page:list', '/website/page', 'views/website/page/index.vue', 'lucide-file-text', 22, 1)
ON CONFLICT (id) DO UPDATE SET parent_id=EXCLUDED.parent_id, name=EXCLUDED.name, type=EXCLUDED.type, perm=EXCLUDED.perm, path=EXCLUDED.path, component=EXCLUDED.component;

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status) VALUES
  (721, 720, 'page.website.page.button.view', 'BUTTON', 'website:page:view', 1, 1),
  (722, 720, 'page.website.page.button.add', 'BUTTON', 'website:page:add', 2, 1),
  (723, 720, 'page.website.page.button.edit', 'BUTTON', 'website:page:update', 3, 1),
  (724, 720, 'page.website.page.button.delete', 'BUTTON', 'website:page:delete', 4, 1)
ON CONFLICT (id) DO UPDATE SET parent_id=EXCLUDED.parent_id, name=EXCLUDED.name, type=EXCLUDED.type, perm=EXCLUDED.perm;

-- 2.4 媒体库管理（id=730-735）
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, path, component, icon, sort, status) VALUES
  (730, 345, 'page.website.mediaTitle', 'MENU', 'website:media:list', '/website/media', 'views/website/media/index.vue', 'lucide-folder-open', 23, 1)
ON CONFLICT (id) DO UPDATE SET parent_id=EXCLUDED.parent_id, name=EXCLUDED.name, type=EXCLUDED.type, perm=EXCLUDED.perm, path=EXCLUDED.path, component=EXCLUDED.component;

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status) VALUES
  (731, 730, 'page.website.media.button.view', 'BUTTON', 'website:media:view', 1, 1),
  (732, 730, 'page.website.media.button.add', 'BUTTON', 'website:media:add', 2, 1),
  (733, 730, 'page.website.media.button.edit', 'BUTTON', 'website:media:update', 3, 1),
  (734, 730, 'page.website.media.button.delete', 'BUTTON', 'website:media:delete', 4, 1),
  (735, 730, 'page.website.media.button.category', 'BUTTON', 'website:media:category:list', 5, 1)
ON CONFLICT (id) DO UPDATE SET parent_id=EXCLUDED.parent_id, name=EXCLUDED.name, type=EXCLUDED.type, perm=EXCLUDED.perm;

-- 2.5 内容模型管理（id=740-745）
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, path, component, icon, sort, status) VALUES
  (740, 345, 'page.website.contentModelTitle', 'MENU', 'content:model:list', '/website/content-model', 'views/website/content-model/index.vue', 'lucide-boxes', 24, 1)
ON CONFLICT (id) DO UPDATE SET parent_id=EXCLUDED.parent_id, name=EXCLUDED.name, type=EXCLUDED.type, perm=EXCLUDED.perm, path=EXCLUDED.path, component=EXCLUDED.component;

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status) VALUES
  (741, 740, 'page.website.contentModel.button.view', 'BUTTON', 'content:model:view', 1, 1),
  (742, 740, 'page.website.contentModel.button.add', 'BUTTON', 'content:model:add', 2, 1),
  (743, 740, 'page.website.contentModel.button.edit', 'BUTTON', 'content:model:update', 3, 1),
  (744, 740, 'page.website.contentModel.button.delete', 'BUTTON', 'content:model:delete', 4, 1),
  (745, 740, 'page.website.contentModel.button.field', 'BUTTON', 'content:field:list', 5, 1)
ON CONFLICT (id) DO UPDATE SET parent_id=EXCLUDED.parent_id, name=EXCLUDED.name, type=EXCLUDED.type, perm=EXCLUDED.perm;

-- 2.6 模板变量管理（id=750-755）
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, path, component, icon, sort, status) VALUES
  (750, 345, 'page.website.templateVarTitle', 'MENU', 'template:var:list', '/website/template-var', 'views/website/template-var/index.vue', 'lucide-variable', 25, 1)
ON CONFLICT (id) DO UPDATE SET parent_id=EXCLUDED.parent_id, name=EXCLUDED.name, type=EXCLUDED.type, perm=EXCLUDED.perm, path=EXCLUDED.path, component=EXCLUDED.component;

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status) VALUES
  (751, 750, 'page.website.templateVar.button.view', 'BUTTON', 'template:var:view', 1, 1),
  (752, 750, 'page.website.templateVar.button.add', 'BUTTON', 'template:var:add', 2, 1),
  (753, 750, 'page.website.templateVar.button.edit', 'BUTTON', 'template:var:update', 3, 1),
  (754, 750, 'page.website.templateVar.button.delete', 'BUTTON', 'template:var:delete', 4, 1)
ON CONFLICT (id) DO UPDATE SET parent_id=EXCLUDED.parent_id, name=EXCLUDED.name, type=EXCLUDED.type, perm=EXCLUDED.perm;

-- 2.7 文章自定义字段管理（id=760-764，挂在文章管理 id=349 下）
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, path, component, icon, sort, status) VALUES
  (760, 349, 'page.website.articleFieldTitle', 'MENU', 'website:article:field', '/website/article-field', 'views/website/article-field/index.vue', 'mdi:form-textbox', 30, 1)
ON CONFLICT (id) DO UPDATE SET parent_id=EXCLUDED.parent_id, name=EXCLUDED.name, type=EXCLUDED.type, perm=EXCLUDED.perm, path=EXCLUDED.path, component=EXCLUDED.component;

INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status) VALUES
  (761, 760, 'page.website.articleField.button.view', 'BUTTON', 'website:article:field:view', 1, 1),
  (762, 760, 'page.website.articleField.button.add', 'BUTTON', 'website:article:field:add', 2, 1),
  (763, 760, 'page.website.articleField.button.edit', 'BUTTON', 'website:article:field:update', 3, 1),
  (764, 760, 'page.website.articleField.button.delete', 'BUTTON', 'website:article:field:delete', 4, 1)
ON CONFLICT (id) DO UPDATE SET parent_id=EXCLUDED.parent_id, name=EXCLUDED.name, type=EXCLUDED.type, perm=EXCLUDED.perm;

-- ============================================================
-- 3. 补齐模板管理和模板数据的按钮权限
-- ============================================================

-- 3.1 模板管理按钮权限（id=770-774，挂在 id=413 模板市场下）
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status) VALUES
  (771, 413, 'page.website.template.button.view', 'BUTTON', 'template:view', 1, 1),
  (772, 413, 'page.website.template.button.add', 'BUTTON', 'template:add', 2, 1),
  (773, 413, 'page.website.template.button.edit', 'BUTTON', 'template:update', 3, 1),
  (774, 413, 'page.website.template.button.delete', 'BUTTON', 'template:delete', 4, 1)
ON CONFLICT (id) DO UPDATE SET parent_id=EXCLUDED.parent_id, name=EXCLUDED.name, type=EXCLUDED.type, perm=EXCLUDED.perm;

-- 3.2 模板数据按钮权限（id=780-784，挂在 id=400 模板数据下）
INSERT INTO mxx_system_menu (id, parent_id, name, type, perm, sort, status) VALUES
  (781, 400, 'page.website.templateData.button.view', 'BUTTON', 'template:data:view', 1, 1),
  (782, 400, 'page.website.templateData.button.add', 'BUTTON', 'template:data:add', 2, 1),
  (783, 400, 'page.website.templateData.button.edit', 'BUTTON', 'template:data:update', 3, 1),
  (784, 400, 'page.website.templateData.button.delete', 'BUTTON', 'template:data:delete', 4, 1),
  (785, 400, 'page.website.templateData.button.revision', 'BUTTON', 'template:revision:list', 5, 1)
ON CONFLICT (id) DO UPDATE SET parent_id=EXCLUDED.parent_id, name=EXCLUDED.name, type=EXCLUDED.type, perm=EXCLUDED.perm;

-- ============================================================
-- 4. 补齐缺失的表字段
-- ============================================================

-- 4.1 G-2.4: mxx_website 添加 mobile_template_id 字段（v37 未执行）
ALTER TABLE mxx_website ADD COLUMN IF NOT EXISTS mobile_template_id BIGINT;
COMMENT ON COLUMN mxx_website.mobile_template_id IS 'G-2.4: 移动端模板ID（NULL/0 时与 template_id 相同）';

-- 4.2 G-1.16: mxx_website_links 添加 link_category 字段（v39 未执行）
ALTER TABLE mxx_website_links ADD COLUMN IF NOT EXISTS link_category VARCHAR(64);
COMMENT ON COLUMN mxx_website_links.link_category IS 'G-1.16: 链接分类（如：partner/friend/media）';
CREATE INDEX IF NOT EXISTS idx_links_category ON mxx_website_links(link_category);

-- ============================================================
-- 5. 将所有新菜单权限分配给超级管理员角色（role_id=4）
-- 确保管理员能立即看到菜单和调用接口
-- 表名：mxx_system_role_menu_merge，主键：(id, role_id, menu_id)
-- 注意：主键含自增 id，无法用 ON CONFLICT (role_id, menu_id)
-- 改用 NOT EXISTS 避免重复插入
-- ============================================================

INSERT INTO mxx_system_role_menu_merge (role_id, menu_id, status)
SELECT 4, id, 1 FROM mxx_system_menu m
WHERE m.id IN (
  700, 701, 702, 703, 704,
  710, 711, 712, 713, 714,
  720, 721, 722, 723, 724,
  730, 731, 732, 733, 734, 735,
  740, 741, 742, 743, 744, 745,
  750, 751, 752, 753, 754,
  760, 761, 762, 763, 764,
  771, 772, 773, 774,
  781, 782, 783, 784, 785
)
AND NOT EXISTS (
  SELECT 1 FROM mxx_system_role_menu_merge rmm
  WHERE rmm.role_id = 4 AND rmm.menu_id = m.id
);
