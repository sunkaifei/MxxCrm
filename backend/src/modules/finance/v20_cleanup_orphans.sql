-- ============================================================
-- 清理历史遗留孤立数据
-- 这些数据引用了已不存在的salary_record（id 1-36）
-- ============================================================

-- 1. 清理孤立的工资项目值（引用已删除的工资记录31-36）
DELETE FROM mxx_finance_salary_item_value
WHERE salary_record_id NOT IN (SELECT id FROM mxx_finance_salary_record);

-- 2. 清理孤立的工资确认记录（引用已删除的工资记录1,12,13）
DELETE FROM mxx_finance_salary_confirm
WHERE salary_record_id NOT IN (SELECT id FROM mxx_finance_salary_record);

-- 3. 清理孤立的工资税务明细（引用已删除的工资记录1-36）
DELETE FROM mxx_finance_salary_tax_detail
WHERE salary_record_id NOT IN (SELECT id FROM mxx_finance_salary_record);

-- 4. 清理孤立的提成分配记录（salary_record_id引用已删除的记录）
DELETE FROM mxx_finance_commission_allocation
WHERE salary_record_id IS NOT NULL AND salary_record_id > 0
  AND salary_record_id NOT IN (SELECT id FROM mxx_finance_salary_record);

-- 5. 验证清理结果
SELECT 'item_value_orphan' as t, count(*) FROM mxx_finance_salary_item_value siv
LEFT JOIN mxx_finance_salary_record sr ON sr.id=siv.salary_record_id WHERE sr.id IS NULL
UNION ALL
SELECT 'confirm_orphan', count(*) FROM mxx_finance_salary_confirm sc
LEFT JOIN mxx_finance_salary_record sr ON sr.id=sc.salary_record_id WHERE sr.id IS NULL
UNION ALL
SELECT 'tax_detail_orphan', count(*) FROM mxx_finance_salary_tax_detail std
LEFT JOIN mxx_finance_salary_record sr ON sr.id=std.salary_record_id WHERE sr.id IS NULL;
