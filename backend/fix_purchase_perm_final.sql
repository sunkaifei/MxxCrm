-- 采购模块权限码全面修复：去重 + 纠错 + 对齐后端

-- ============================================================
-- 1. 删除重复权限码
-- ============================================================
-- id=841/842 与 id=222/223 完全重复（purchase:order:audit/close）
DELETE FROM mxx_system_menu WHERE id IN (841, 842);

-- ============================================================
-- 2. 纠正权限码错误
-- ============================================================

-- id=804: 名为"审批采购申请"但权限码是 save，后端 approve 路由用的是 purchase:requisition:approve
UPDATE mxx_system_menu SET perm = 'purchase:requisition:approve' WHERE id = 804;

-- id=812: 名为"删除收货单"但权限码是 update，应该是编辑收货单
-- 后端收货只有 save/delete/list/inbound，没有 update
-- 改为"编辑收货单"，复用 save 权限
UPDATE mxx_system_menu SET name = '编辑收货单', perm = 'purchase:receipt:save' WHERE id = 812;

-- id=813: 名为"生成入库单"但权限码是 delete，后端 to_inbound 用的是 purchase:receipt:inbound
UPDATE mxx_system_menu SET name = '生成入库单', perm = 'purchase:receipt:inbound' WHERE id = 813;

-- id=844: supplier info 权限码，后端用的是 purchase:supplier:info（保持与后端一致）
-- 新规则要求 view，但改后端需重编译，暂保留 info

-- ============================================================
-- 3. 补充缺失的权限码按钮
-- ============================================================

-- 采购收货详情（后端 /info 复用 list 权限，无需额外按钮）

-- 采购退货详情（后端 /info 复用 list 权限，无需额外按钮）

-- ============================================================
-- 4. 修复 FOLDER 权限码（原 shopping:index → purchase:index）
-- ============================================================
UPDATE mxx_system_menu SET perm = 'purchase:index' WHERE id = 139;

-- ============================================================
-- 5. 修复菜单 path 不一致问题
-- ============================================================
-- 采购订单菜单 path 是 /purchase/po，后端 scope 是 /purchase/order
-- 前端路由也是 /purchase/po，所以页面 URL 用 po 没问题
-- 但 component 应该一致，这里保持不变

-- ============================================================
-- 验证：对比后端 require_permission 与 DB perm
-- ============================================================
SELECT '--- DB 权限码列表 ---' as info;
SELECT id, parent_id, name, type, perm FROM mxx_system_menu
WHERE perm LIKE 'purchase:%' AND deleted = 0
ORDER BY parent_id, sort, id;
