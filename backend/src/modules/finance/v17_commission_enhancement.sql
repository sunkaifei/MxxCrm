-- ============================================================
-- v17_commission_enhancement.sql
-- 提成系统增强：6 种提成模式 + 团建资金池 + 提成分配
-- 幂等迁移，可重复执行
-- ============================================================

-- ============================================================
-- 1. commission_rule 表扩展（10 个新字段）
-- ============================================================
ALTER TABLE mxx_finance_commission_rule ADD COLUMN IF NOT EXISTS
  commission_category SMALLINT NOT NULL DEFAULT 1;
-- 提成性质: 1=个人提成 2=管理分润 3=团队激励奖金 4=团建资金池 5=总提成再分配 6=利润提成

ALTER TABLE mxx_finance_commission_rule ADD COLUMN IF NOT EXISTS
  beneficiary_role SMALLINT NOT NULL DEFAULT 1;
-- 受益岗位: 1=销售本人 2=直属主管 3=部门经理 4=总监 5=总经理 6=自定义岗位

ALTER TABLE mxx_finance_commission_rule ADD COLUMN IF NOT EXISTS
  calc_method SMALLINT NOT NULL DEFAULT 1;
-- 计算方式: 1=按比例 2=固定金额(达标后) 3=阶梯累进 4=超额递增

ALTER TABLE mxx_finance_commission_rule ADD COLUMN IF NOT EXISTS
  bonus_target DECIMAL(12,2) DEFAULT 0;
-- 达标门槛(calc_method=2时使用)

ALTER TABLE mxx_finance_commission_rule ADD COLUMN IF NOT EXISTS
  bonus_fixed_amount DECIMAL(12,2) DEFAULT 0;
-- 固定奖金金额(calc_method=2时使用)

ALTER TABLE mxx_finance_commission_rule ADD COLUMN IF NOT EXISTS
  commission_cap DECIMAL(12,2);
-- 单笔提成封顶(NULL=不封顶)

ALTER TABLE mxx_finance_commission_rule ADD COLUMN IF NOT EXISTS
  commission_floor DECIMAL(12,2);
-- 月度提成保底(NULL=不保底)

ALTER TABLE mxx_finance_commission_rule ADD COLUMN IF NOT EXISTS
  customer_category VARCHAR(20);
-- 客户分类筛选: new=仅新客户 old=仅老客户 NULL=全部

ALTER TABLE mxx_finance_commission_rule ADD COLUMN IF NOT EXISTS
  defer_months INT NOT NULL DEFAULT 0;
-- 递延发放月数: 0=随当月发 N=分N个月递延

ALTER TABLE mxx_finance_commission_rule ADD COLUMN IF NOT EXISTS
  pool_id BIGINT;
-- 关联资金池ID(category=4时使用)

COMMENT ON COLUMN mxx_finance_commission_rule.commission_category IS '提成性质: 1=个人提成 2=管理分润 3=团队激励奖金 4=团建资金池 5=总提成再分配 6=利润提成';
COMMENT ON COLUMN mxx_finance_commission_rule.beneficiary_role IS '受益岗位: 1=销售本人 2=直属主管 3=部门经理 4=总监 5=总经理 6=自定义岗位';
COMMENT ON COLUMN mxx_finance_commission_rule.calc_method IS '计算方式: 1=按比例 2=固定金额(达标后) 3=阶梯累进 4=超额递增';
COMMENT ON COLUMN mxx_finance_commission_rule.bonus_target IS '达标门槛(calc_method=2时使用)';
COMMENT ON COLUMN mxx_finance_commission_rule.bonus_fixed_amount IS '固定奖金金额(calc_method=2时使用)';
COMMENT ON COLUMN mxx_finance_commission_rule.commission_cap IS '单笔提成封顶(NULL=不封顶)';
COMMENT ON COLUMN mxx_finance_commission_rule.commission_floor IS '月度提成保底(NULL=不保底)';
COMMENT ON COLUMN mxx_finance_commission_rule.customer_category IS '客户分类筛选: new=仅新客户 old=仅老客户 NULL=全部';
COMMENT ON COLUMN mxx_finance_commission_rule.defer_months IS '递延发放月数: 0=随当月发 N=分N个月递延';
COMMENT ON COLUMN mxx_finance_commission_rule.pool_id IS '关联资金池ID(category=4时使用)';

-- 数据迁移：把现有 rule_type 映射到 commission_category + beneficiary_role
-- rule_type=1(个人提成) → category=1, role=1
-- rule_type=2(团队分成) → category=2, role=2
-- rule_type=3(部门经理) → category=2, role=3
-- rule_type=4(总监)     → category=2, role=4
-- rule_type=5(团队长)   → category=2, role=2
UPDATE mxx_finance_commission_rule SET commission_category = 1, beneficiary_role = 1 WHERE rule_type = 1 AND commission_category = 1;
UPDATE mxx_finance_commission_rule SET commission_category = 2, beneficiary_role = 2 WHERE rule_type = 2 AND commission_category = 1;
UPDATE mxx_finance_commission_rule SET commission_category = 2, beneficiary_role = 3 WHERE rule_type = 3 AND commission_category = 1;
UPDATE mxx_finance_commission_rule SET commission_category = 2, beneficiary_role = 4 WHERE rule_type = 4 AND commission_category = 1;
UPDATE mxx_finance_commission_rule SET commission_category = 2, beneficiary_role = 2 WHERE rule_type = 5 AND commission_category = 1;

-- ============================================================
-- 2. commission_result 表扩展（6 个新字段）
-- ============================================================
ALTER TABLE mxx_finance_commission_result ADD COLUMN IF NOT EXISTS
  commission_category SMALLINT DEFAULT 1;
ALTER TABLE mxx_finance_commission_result ADD COLUMN IF NOT EXISTS
  beneficiary_role SMALLINT DEFAULT 1;
ALTER TABLE mxx_finance_commission_result ADD COLUMN IF NOT EXISTS
  manager_level INT;
ALTER TABLE mxx_finance_commission_result ADD COLUMN IF NOT EXISTS
  allocate_status SMALLINT DEFAULT 0;
ALTER TABLE mxx_finance_commission_result ADD COLUMN IF NOT EXISTS
  allocated_amount DECIMAL(12,2) DEFAULT 0;
ALTER TABLE mxx_finance_commission_result ADD COLUMN IF NOT EXISTS
  pool_id BIGINT;
ALTER TABLE mxx_finance_commission_result ADD COLUMN IF NOT EXISTS
  cost_amount DECIMAL(12,2);

COMMENT ON COLUMN mxx_finance_commission_result.commission_category IS '提成性质(冗余自规则表)';
COMMENT ON COLUMN mxx_finance_commission_result.beneficiary_role IS '受益岗位';
COMMENT ON COLUMN mxx_finance_commission_result.manager_level IS '管理者层级(1=主管 2=经理 3=总监)';
COMMENT ON COLUMN mxx_finance_commission_result.allocate_status IS '分配状态: 0=无需分配 1=待分配 2=已分配';
COMMENT ON COLUMN mxx_finance_commission_result.allocated_amount IS '已分配金额';
COMMENT ON COLUMN mxx_finance_commission_result.pool_id IS '关联资金池ID(category=4)';
COMMENT ON COLUMN mxx_finance_commission_result.cost_amount IS '成本金额(category=6利润提成)';

-- ============================================================
-- 3. salary_record 表扩展（3 个新字段）
-- ============================================================
ALTER TABLE mxx_finance_salary_record ADD COLUMN IF NOT EXISTS
  bonus_amount DECIMAL(12,2) NOT NULL DEFAULT 0;
ALTER TABLE mxx_finance_salary_record ADD COLUMN IF NOT EXISTS
  allocated_commission DECIMAL(12,2) NOT NULL DEFAULT 0;
ALTER TABLE mxx_finance_salary_record ADD COLUMN IF NOT EXISTS
  deferred_commission DECIMAL(12,2) NOT NULL DEFAULT 0;

COMMENT ON COLUMN mxx_finance_salary_record.bonus_amount IS '团队激励奖金(category=3)';
COMMENT ON COLUMN mxx_finance_salary_record.allocated_commission IS '手动分配提成(category=5)';
COMMENT ON COLUMN mxx_finance_salary_record.deferred_commission IS '递延提成金额';

-- ============================================================
-- 4. 新增团建资金池表
-- ============================================================
CREATE TABLE IF NOT EXISTS mxx_finance_commission_pool (
  id BIGSERIAL PRIMARY KEY,
  pool_name VARCHAR(100) NOT NULL,
  department_id BIGINT,
  manager_id BIGINT,
  total_amount DECIMAL(12,2) NOT NULL DEFAULT 0,
  used_amount DECIMAL(12,2) NOT NULL DEFAULT 0,
  balance DECIMAL(12,2) GENERATED ALWAYS AS (total_amount - used_amount) STORED,
  status SMALLINT NOT NULL DEFAULT 1,
  description TEXT,
  create_time TIMESTAMP NOT NULL DEFAULT NOW(),
  update_time TIMESTAMP NOT NULL DEFAULT NOW(),
  deleted SMALLINT NOT NULL DEFAULT 0
);

COMMENT ON TABLE mxx_finance_commission_pool IS '团建资金池';
COMMENT ON COLUMN mxx_finance_commission_pool.pool_name IS '资金池名称';
COMMENT ON COLUMN mxx_finance_commission_pool.department_id IS '归属部门';
COMMENT ON COLUMN mxx_finance_commission_pool.manager_id IS '管理人';
COMMENT ON COLUMN mxx_finance_commission_pool.total_amount IS '累计存入金额';
COMMENT ON COLUMN mxx_finance_commission_pool.used_amount IS '已使用金额';
COMMENT ON COLUMN mxx_finance_commission_pool.balance IS '余额(自动计算)';
COMMENT ON COLUMN mxx_finance_commission_pool.status IS '状态: 1=活跃 2=冻结 3=已关闭';

-- ============================================================
-- 5. 新增资金池流水表
-- ============================================================
CREATE TABLE IF NOT EXISTS mxx_finance_commission_pool_log (
  id BIGSERIAL PRIMARY KEY,
  pool_id BIGINT NOT NULL,
  log_type SMALLINT NOT NULL,
  amount DECIMAL(12,2) NOT NULL,
  source_rule_id BIGINT,
  source_employee_id BIGINT,
  source_year INT,
  source_month INT,
  usage_description TEXT,
  usage_date DATE,
  operator_id BIGINT,
  create_time TIMESTAMP NOT NULL DEFAULT NOW(),
  deleted SMALLINT NOT NULL DEFAULT 0
);

COMMENT ON TABLE mxx_finance_commission_pool_log IS '资金池流水';
COMMENT ON COLUMN mxx_finance_commission_pool_log.log_type IS '类型: 1=存入(提成归集) 2=支出(团建活动)';
COMMENT ON COLUMN mxx_finance_commission_pool_log.usage_description IS '支出事由';

-- ============================================================
-- 6. 新增提成分配记录表
-- ============================================================
CREATE TABLE IF NOT EXISTS mxx_finance_commission_allocation (
  id BIGSERIAL PRIMARY KEY,
  commission_result_id BIGINT NOT NULL,
  allocator_id BIGINT NOT NULL,
  employee_id BIGINT NOT NULL,
  employee_name VARCHAR(100),
  amount DECIMAL(12,2) NOT NULL,
  allocate_method SMALLINT NOT NULL DEFAULT 3,
  employee_payment DECIMAL(12,2),
  team_total_payment DECIMAL(12,2),
  salary_record_id BIGINT,
  year INT NOT NULL,
  month INT NOT NULL,
  remark TEXT,
  create_time TIMESTAMP NOT NULL DEFAULT NOW(),
  deleted SMALLINT NOT NULL DEFAULT 0
);

COMMENT ON TABLE mxx_finance_commission_allocation IS '提成分配记录';
COMMENT ON COLUMN mxx_finance_commission_allocation.allocate_method IS '分配方式: 1=平均 2=按业绩比例 3=手动';

-- ============================================================
-- 7. 菜单数据（团建资金池菜单）
-- ============================================================
INSERT INTO mxx_system_menu (parent_id, name, type, path, perm, status, sort, icon, create_time, update_time, deleted)
SELECT 315, 'page.finance.commissionPool.title', 'menu', 'commission-pool', 'finance:commission-pool:list', 1, 55, 'lucide:wallet', NOW(), NOW(), 0
WHERE NOT EXISTS (
  SELECT 1 FROM mxx_system_menu WHERE perm = 'finance:commission-pool:list' AND deleted = 0
);

-- 关联到财务角色(id=4)和超级管理员(id=10)
INSERT INTO mxx_system_role_menu_merge (role_id, menu_id)
SELECT r.role_id, m.id
FROM mxx_system_menu m
CROSS JOIN (SELECT 4 AS role_id UNION SELECT 10 AS role_id) r
WHERE m.perm = 'finance:commission-pool:list' AND m.deleted = 0
  AND NOT EXISTS (
    SELECT 1 FROM mxx_system_role_menu_merge rm
    WHERE rm.role_id = r.role_id AND rm.menu_id = m.id
  );

-- ============================================================
-- 8. 验证
-- ============================================================
SELECT 'commission_rule columns' AS check_item,
  count(*) AS added
FROM information_schema.columns
WHERE table_name = 'mxx_finance_commission_rule'
  AND column_name IN ('commission_category','beneficiary_role','calc_method','bonus_target','bonus_fixed_amount','commission_cap','commission_floor','customer_category','defer_months','pool_id');

SELECT 'commission_result columns' AS check_item,
  count(*) AS added
FROM information_schema.columns
WHERE table_name = 'mxx_finance_commission_result'
  AND column_name IN ('commission_category','beneficiary_role','manager_level','allocate_status','allocated_amount','pool_id','cost_amount');

SELECT 'salary_record columns' AS check_item,
  count(*) AS added
FROM information_schema.columns
WHERE table_name = 'mxx_finance_salary_record'
  AND column_name IN ('bonus_amount','allocated_commission','deferred_commission');

SELECT 'new tables' AS check_item,
  count(*) AS created
FROM information_schema.tables
WHERE table_name IN ('mxx_finance_commission_pool','mxx_finance_commission_pool_log','mxx_finance_commission_allocation');
