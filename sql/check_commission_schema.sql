-- 检查提成规则相关表结构
-- 执行: psql -h 115.190.210.106 -p 5432 -U postgres -d mxxcrm_data -f check_commission_schema.sql

-- 1. 查看提成规则表结构
SELECT '=== mxx_finance_commission_rule 表结构 ===' as info;
SELECT column_name, data_type, is_nullable, column_default
FROM information_schema.columns
WHERE table_name = 'mxx_finance_commission_rule'
ORDER BY ordinal_position;

-- 2. 查看提成阶梯表结构
SELECT '=== mxx_finance_commission_tier 表结构 ===' as info;
SELECT column_name, data_type, is_nullable, column_default
FROM information_schema.columns
WHERE table_name = 'mxx_finance_commission_tier'
ORDER BY ordinal_position;

-- 3. 查看提成规则成员表结构
SELECT '=== mxx_finance_commission_rule_member 表结构 ===' as info;
SELECT column_name, data_type, is_nullable, column_default
FROM information_schema.columns
WHERE table_name = 'mxx_finance_commission_rule_member'
ORDER BY ordinal_position;

-- 4. 查看现有数据
SELECT '=== 现有提成规则数据 ===' as info;
SELECT * FROM mxx_finance_commission_rule WHERE deleted = 0 LIMIT 5;

-- 5. 查看现有阶梯数据
SELECT '=== 现有阶梯数据 ===' as info;
SELECT * FROM mxx_finance_commission_tier LIMIT 5;

-- 6. 查看现有成员数据
SELECT '=== 现有成员数据 ===' as info;
SELECT * FROM mxx_finance_commission_rule_member LIMIT 5;
