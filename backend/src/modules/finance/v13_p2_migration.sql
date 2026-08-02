-- ============================================================
-- v13 P2 待优化项迁移 SQL（2026-07-31）
-- 包含 P2-2 工资趋势分析菜单 + P2-3 提成规则维度扩展 + P2-6 调度器重试
-- 数据库：PostgreSQL
-- ============================================================

-- ============ P2-6: 定时任务重试机制字段 ============
-- 为 mxx_system_scheduler_job 新增 3 个字段：max_retries、retry_interval_base、last_retry_count
ALTER TABLE mxx_system_scheduler_job ADD COLUMN IF NOT EXISTS max_retries INTEGER DEFAULT 3;
ALTER TABLE mxx_system_scheduler_job ADD COLUMN IF NOT EXISTS retry_interval_base INTEGER DEFAULT 60;
ALTER TABLE mxx_system_scheduler_job ADD COLUMN IF NOT EXISTS last_retry_count INTEGER DEFAULT 0;

-- 为现有任务设置默认重试次数（3 次，间隔基数 60 秒，最大延迟 60*2^2=240 秒）
UPDATE mxx_system_scheduler_job
SET max_retries = 3,
    retry_interval_base = 60,
    last_retry_count = 0
WHERE max_retries IS NULL;


-- ============ P2-3: 提成规则扩展产品线/区域/客户类型维度 ============
-- 为 mxx_finance_commission_rule 新增 3 个可选维度字段
ALTER TABLE mxx_finance_commission_rule ADD COLUMN IF NOT EXISTS product_line VARCHAR(100);
ALTER TABLE mxx_finance_commission_rule ADD COLUMN IF NOT EXISTS region_code VARCHAR(50);
ALTER TABLE mxx_finance_commission_rule ADD COLUMN IF NOT EXISTS customer_type VARCHAR(50);

-- 注释（PostgreSQL 语法）
COMMENT ON COLUMN mxx_finance_commission_rule.product_line IS 'P2-3: 产品线维度（可选，用于按产品线差异化提成）';
COMMENT ON COLUMN mxx_finance_commission_rule.region_code IS 'P2-3: 区域编码维度（可选，用于按区域差异化提成）';
COMMENT ON COLUMN mxx_finance_commission_rule.customer_type IS 'P2-3: 客户类型维度（可选，如 VIP/普通/战略等）';


-- ============ P2-2: 工资历史趋势分析页菜单 ============
-- 新增菜单入口（parent_id=500 为财务父菜单）
-- 列名对齐 v6_migration.sql 中的实际表结构
INSERT INTO mxx_system_menu (id, parent_id, menu_name, route_name, route_path, component, menu_type, perms, icon, sort, is_show, created_at)
SELECT 610, 500, '工资趋势分析', 'FinanceSalaryAnalysis', 'salary-analysis', 'finance/salary-analysis/index', 2, 'finance:salary:list', 'mdi:chart-line', 74, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 610);

-- 查看权限按钮（工资分析只需 list 权限，不引入新的 manage 权限）
INSERT INTO mxx_system_menu (id, parent_id, menu_name, menu_type, perms, sort, is_show, created_at)
SELECT 611, 610, '查看', 3, 'finance:salary:list', 1, 1, NOW()
WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE id = 611);

-- ============ 验证 SQL ============
-- 查询确认所有迁移项已落地
SELECT
    (SELECT COUNT(*) FROM information_schema.columns
     WHERE table_name = 'mxx_system_scheduler_job'
       AND column_name IN ('max_retries', 'retry_interval_base', 'last_retry_count')) AS scheduler_p2_6_columns,
    (SELECT COUNT(*) FROM information_schema.columns
     WHERE table_name = 'mxx_finance_commission_rule'
       AND column_name IN ('product_line', 'region_code', 'customer_type')) AS commission_p2_3_columns,
    (SELECT COUNT(*) FROM mxx_system_menu WHERE id IN (610, 611)) AS p2_2_menu_count;
