-- 查看所有提成规则
SELECT 
    id, 
    rule_name, 
    department_id, 
    post_id, 
    rule_type,
    calc_base_type,
    trigger_condition, 
    enabled,
    description
FROM mxx_finance_commission_rule 
WHERE deleted = 0 
ORDER BY id;

-- 查看规则成员分配
SELECT 
    m.id,
    m.rule_id,
    r.rule_name,
    m.member_type,
    m.member_name,
    m.fixed_rate,
    m.sort
FROM mxx_finance_commission_rule_member m
JOIN mxx_finance_commission_rule r ON m.rule_id = r.id
ORDER BY m.rule_id, m.sort;

-- 查看阶梯配置
SELECT 
    t.id,
    t.rule_id,
    r.rule_name,
    t.min_amount,
    t.max_amount,
    t.commission_rate,
    t.sort
FROM mxx_finance_commission_tier t
JOIN mxx_finance_commission_rule r ON t.rule_id = r.id
ORDER BY t.rule_id, t.sort;
