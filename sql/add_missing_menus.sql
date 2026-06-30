-- =============================================================
-- Add missing menus to mxx_system_menu
-- Attachment: 分类管理（父级 Attachment id=151）
-- =============================================================
INSERT INTO public.mxx_system_menu (id, parent_id, name, type, route_name, path, component, icon, perm, status) VALUES
(337, 151, 'page.attachment.category.title', 'MENU', 'AttachmentCategory', '/attachment/category', 'attachment/category/index', 'lucide:folder-tree', 'attachment:category:list', 1);

-- 分类管理按钮权限
INSERT INTO public.mxx_system_menu (id, parent_id, name, type, route_name, path, component, icon, perm, status) VALUES
(338, 337, 'page.attachment.category.button.create', 'BUTTON', 'CategoryCreate', '', 'attachment/category/index', '', 'attachment:category:create', 1),
(339, 337, 'page.attachment.category.button.edit', 'BUTTON', 'CategoryEdit', '', 'attachment/category/index', '', 'attachment:category:update', 1),
(340, 337, 'page.attachment.category.button.delete', 'BUTTON', 'CategoryDelete', '', 'attachment/category/index', '', 'attachment:category:delete', 1);

-- =============================================================
-- System: 标签管理（父级 System id=67）
-- =============================================================
INSERT INTO public.mxx_system_menu (id, parent_id, name, type, route_name, path, component, icon, perm, status) VALUES
(341, 67, 'page.system.tag.title', 'MENU', 'SystemTag', '/system/tag', 'system/tag/index', 'lucide:tag', 'system:tag:list', 1);

INSERT INTO public.mxx_system_menu (id, parent_id, name, type, route_name, path, component, icon, perm, status) VALUES
(342, 341, 'page.system.tag.button.create', 'BUTTON', 'TagCreate', '', 'system/tag/index', '', 'system:tag:create', 1),
(343, 341, 'page.system.tag.button.edit', 'BUTTON', 'TagEdit', '', 'system/tag/index', '', 'system:tag:update', 1),
(344, 341, 'page.system.tag.button.delete', 'BUTTON', 'TagDelete', '', 'system/tag/index', '', 'system:tag:delete', 1);

-- =============================================================
-- Shop: 商城模块（全新父级）
-- =============================================================
INSERT INTO public.mxx_system_menu (id, parent_id, name, type, route_name, path, component, icon, perm, status) VALUES
(345, 0, 'page.shop.title', 'FOLDER', 'Shop', '/shop', '', 'lucide:shopping-cart', 'shop:index', 1);

-- 商城子菜单
INSERT INTO public.mxx_system_menu (id, parent_id, name, type, route_name, path, component, icon, perm, status) VALUES
(346, 345, 'page.shop.category', 'MENU', 'ShopCategory', '/shop/category', 'shop/category/index', 'lucide:folder-tree', 'shop:category:list', 1),
(347, 345, 'page.shop.supplierAudit', 'MENU', 'ShopSupplierAudit', '/shop/supplier-audit', 'shop/supplier-audit/index', 'lucide:user-check', 'shop:supplier-audit:list', 1),
(348, 345, 'page.shop.goodsAudit', 'MENU', 'ShopGoodsAudit', '/shop/goods-audit', 'shop/goods-audit/index', 'lucide:file-search', 'shop:goods-audit:list', 1),
(349, 345, 'page.shop.goods', 'MENU', 'ShopGoods', '/shop/goods', 'shop/goods/index', 'lucide:package', 'shop:goods:list', 1),
(350, 345, 'page.shop.shopList', 'MENU', 'ShopList', '/shop/shop-list', 'shop/shop-list/index', 'lucide:store', 'shop:shop:list', 1),
(351, 345, 'page.shop.orderList', 'MENU', 'ShopOrderList', '/shop/order-list', 'shop/order-list/index', 'lucide:shopping-bag', 'shop:order:list', 1),
(352, 345, 'page.shop.settlement', 'MENU', 'ShopSettlement', '/shop/settlement', 'shop/settlement/index', 'lucide:wallet', 'shop:settlement:list', 1),
(353, 345, 'page.shop.promotion', 'MENU', 'ShopPromotion', '/shop/promotion', 'shop/promotion/index', 'lucide:megaphone', 'shop:promotion:list', 1),
(354, 345, 'page.shop.commissionConfig', 'MENU', 'ShopCommissionConfig', '/shop/commission-config', 'shop/commission-config/index', 'lucide:percent', 'shop:commission:list', 1);

-- 商城按钮权限（分类）
INSERT INTO public.mxx_system_menu (id, parent_id, name, type, route_name, path, component, icon, perm, status) VALUES
(355, 346, 'page.shop.category.button.create', 'BUTTON', 'ShopCategoryCreate', '', 'shop/category/index', '', 'shop:category:create', 1),
(356, 346, 'page.shop.category.button.edit', 'BUTTON', 'ShopCategoryEdit', '', 'shop/category/index', '', 'shop:category:edit', 1),
(357, 346, 'page.shop.category.button.delete', 'BUTTON', 'ShopCategoryDelete', '', 'shop/category/index', '', 'shop:category:delete', 1);

-- 商城按钮权限（商品）
INSERT INTO public.mxx_system_menu (id, parent_id, name, type, route_name, path, component, icon, perm, status) VALUES
(358, 349, 'page.shop.goods.button.create', 'BUTTON', 'ShopGoodsCreate', '', 'shop/goods/index', '', 'shop:goods:create', 1),
(359, 349, 'page.shop.goods.button.edit', 'BUTTON', 'ShopGoodsEdit', '', 'shop/goods/index', '', 'shop:goods:edit', 1),
(360, 349, 'page.shop.goods.button.delete', 'BUTTON', 'ShopGoodsDelete', '', 'shop/goods/index', '', 'shop:goods:delete', 1);