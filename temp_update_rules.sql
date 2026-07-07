-- 兼容升级：给旧规则添加 rule_type 和 calc_base_type
UPDATE mxx_finance_commission_rule 
SET rule_type = 1, calc_base_type = 1 
WHERE rule_type IS NULL AND deleted = 0;

-- 为旧规则（ID 1-3）添加默认的成员记录（100%给业务员）
-- 先检查是否已有成员记录
INSERT INTO mxx_finance_commission_rule_member (rule_id, member_type, member_name, distribution_type, fixed_rate, sort)
SELECT 
    r.id, 
    1, 
    '业务员', 
    1, 
    1.0000, 
    1
FROM mxx_finance_commission_rule r
LEFT JOIN mxx_finance_commission_rule_member m ON r.id = m.rule_id
WHERE r.id IN (1, 2, 3) 
AND r.deleted = 0
AND m.id IS NULL;

-- 删除测试用的 ID 10-11（可选，如果想保留演示数据可以注释掉）
-- DELETE FROM mxx_finance_commission_rule_member WHERE rule_id IN (10, 11);
-- DELETE FROM mxx_finance_commission_tier WHERE rule_id IN (10, 11);
-- DELETE FROM mxx_finance_commission_rule WHERE id IN (10, 11);

-- 查看最终规则列表
SELECT 
    id, 
    rule_name, 
    department_id, 
    post_id, 
    rule_type,
    calc_base_type,
    trigger_condition, 
    enabled
FROM mxx_finance_commission_rule 
WHERE deleted = 0 
ORDER BY id;
