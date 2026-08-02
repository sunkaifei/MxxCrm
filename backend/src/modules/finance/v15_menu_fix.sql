-- v15 菜单修复：工资详情 sort + 团队提成补全 route_name/component/icon
-- 1. 工资详情菜单 sort 调整为 99（排在列表之后，避免菜单排序错乱）
UPDATE mxx_system_menu SET sort = 99 WHERE id = 326 AND deleted = 0;

-- 2. 团队提成菜单补全 route_name、component、icon
UPDATE mxx_system_menu
SET route_name = 'FinanceTeamCommission',
    component = 'finance/team-commission/index',
    icon = 'lucide:users',
    sort = 45
WHERE perm = 'finance:team-commission:list' AND deleted = 0;
