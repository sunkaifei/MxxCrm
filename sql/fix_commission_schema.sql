-- 修复提成规则表结构
-- 执行: psql -h 115.190.210.106 -p 5432 -U postgres -d mxxcrm_data -f fix_commission_schema.sql

-- 1. 添加 rule_type 字段 (规则类型: 1=个人提成 2=团队分配)
ALTER TABLE mxx_finance_commission_rule
ADD COLUMN IF NOT EXISTS rule_type INT DEFAULT 1;

-- 2. 添加 calc_base_type 字段 (计算基准: 1=个人月累计 2=团队月累计 3=单笔合同 4=单笔回款)
ALTER TABLE mxx_finance_commission_rule
ADD COLUMN IF NOT EXISTS calc_base_type INT DEFAULT 1;

-- 3. 添加 deleted 字段 (软删除: 0=未删除 1=已删除)
ALTER TABLE mxx_finance_commission_rule
ADD COLUMN IF NOT EXISTS deleted INT NOT NULL DEFAULT 0;

-- 4. 创建提成规则成员表 (如果不存在)
CREATE TABLE IF NOT EXISTS mxx_finance_commission_rule_member (
    id BIGSERIAL PRIMARY KEY,
    rule_id BIGINT NOT NULL,
    member_type INT NOT NULL,  -- 成员类型: 1=业务员 2=直属经理 3=部门总监 4=其他
    member_name VARCHAR(100) NOT NULL,
    distribution_type INT NOT NULL DEFAULT 1,  -- 分配类型: 1=固定比例
    fixed_rate NUMERIC(6,4) NOT NULL DEFAULT 0,  -- 固定比例(如0.6000=60%)
    sort INT NOT NULL DEFAULT 0,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_member_rule FOREIGN KEY (rule_id) REFERENCES mxx_finance_commission_rule(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_member_rule_id ON mxx_finance_commission_rule_member (rule_id);

-- 5. 更新现有数据，设置默认值
UPDATE mxx_finance_commission_rule SET rule_type = 1 WHERE rule_type IS NULL;
UPDATE mxx_finance_commission_rule SET calc_base_type = 1 WHERE calc_base_type IS NULL;
UPDATE mxx_finance_commission_rule SET deleted = 0 WHERE deleted IS NULL;

-- 6. 验证修改结果
SELECT '=== 修改后的表结构 ===' as info;
SELECT column_name, data_type, is_nullable, column_default
FROM information_schema.columns
WHERE table_name = 'mxx_finance_commission_rule'
ORDER BY ordinal_position;
