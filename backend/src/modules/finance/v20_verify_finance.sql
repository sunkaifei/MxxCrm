-- ============================================================
-- 财务模块数据完整性验证脚本
-- 验证所有外键关联无断点，重点检查工资、提成、资金池、费用等
-- ============================================================

\echo '========== 1. 各表数据统计（补全后） =========='
SELECT 'salary_record' as tbl, count(*) as cnt FROM mxx_finance_salary_record
UNION ALL SELECT 'salary_item', count(*) FROM mxx_finance_salary_item
UNION ALL SELECT 'salary_item_value', count(*) FROM mxx_finance_salary_item_value
UNION ALL SELECT 'salary_config', count(*) FROM mxx_finance_salary_config
UNION ALL SELECT 'salary_confirm', count(*) FROM mxx_finance_salary_confirm
UNION ALL SELECT 'salary_adjustment', count(*) FROM mxx_finance_salary_adjustment
UNION ALL SELECT 'salary_tax_detail', count(*) FROM mxx_finance_salary_tax_detail
UNION ALL SELECT 'salary_calc_log', count(*) FROM mxx_finance_salary_calc_log
UNION ALL SELECT 'payslip', count(*) FROM mxx_finance_payslip
UNION ALL SELECT 'commission_result', count(*) FROM mxx_finance_commission_result
UNION ALL SELECT 'commission_detail', count(*) FROM mxx_finance_commission_detail
UNION ALL SELECT 'commission_allocation', count(*) FROM mxx_finance_commission_allocation
UNION ALL SELECT 'commission_pool', count(*) FROM mxx_finance_commission_pool
UNION ALL SELECT 'commission_pool_log', count(*) FROM mxx_finance_commission_pool_log
UNION ALL SELECT 'attendance_record', count(*) FROM mxx_finance_attendance_record
UNION ALL SELECT 'bank_payment_file', count(*) FROM mxx_finance_bank_payment_file
UNION ALL SELECT 'expense', count(*) FROM mxx_finance_expense
UNION ALL SELECT 'expense_item', count(*) FROM mxx_finance_expense_item
UNION ALL SELECT 'finance_statistics', count(*) FROM mxx_finance_statistics
ORDER BY 1;

\echo '========== 2. 工资记录关联验证 =========='
-- 2.1 salary_record → employee (employee_id 必填)
SELECT 'salary_record_employee_invalid' as broken, count(*) as cnt
FROM mxx_finance_salary_record sr
LEFT JOIN mxx_system_admin a ON a.id = sr.employee_id AND a.deleted=0
WHERE sr.deleted=0 AND sr.employee_id > 0 AND a.id IS NULL;

-- 2.2 salary_item_value → salary_record (必填)
SELECT 'item_value_no_record' as broken, count(*) as cnt
FROM mxx_finance_salary_item_value siv
LEFT JOIN mxx_finance_salary_record sr ON sr.id = siv.salary_record_id AND sr.deleted=0
WHERE sr.id IS NULL;

-- 2.3 salary_item_value → salary_item (必填)
SELECT 'item_value_no_item' as broken, count(*) as cnt
FROM mxx_finance_salary_item_value siv
LEFT JOIN mxx_finance_salary_item si ON si.id = siv.item_id
WHERE si.id IS NULL;

-- 2.4 salary_confirm → salary_record (必填)
SELECT 'confirm_no_record' as broken, count(*) as cnt
FROM mxx_finance_salary_confirm sc
LEFT JOIN mxx_finance_salary_record sr ON sr.id = sc.salary_record_id AND sr.deleted=0
WHERE sr.id IS NULL;

-- 2.5 salary_tax_detail → salary_record (必填)
SELECT 'tax_detail_no_record' as broken, count(*) as cnt
FROM mxx_finance_salary_tax_detail std
LEFT JOIN mxx_finance_salary_record sr ON sr.id = std.salary_record_id AND sr.deleted=0
WHERE sr.id IS NULL;

-- 2.6 payslip → salary_record (必填)
SELECT 'payslip_no_record' as broken, count(*) as cnt
FROM mxx_finance_payslip p
LEFT JOIN mxx_finance_salary_record sr ON sr.id = p.salary_record_id AND sr.deleted=0
WHERE sr.id IS NULL;

\echo '========== 3. 提成关联验证 =========='
-- 3.1 commission_result → rule (rule_id 必填)
SELECT 'result_no_rule' as broken, count(*) as cnt
FROM mxx_finance_commission_result cr
LEFT JOIN mxx_finance_commission_rule r ON r.id = cr.rule_id AND r.deleted=0
WHERE r.id IS NULL;

-- 3.2 commission_result → salary_record (可选，但若有值必须有效)
SELECT 'result_invalid_salary' as broken, count(*) as cnt
FROM mxx_finance_commission_result cr
LEFT JOIN mxx_finance_salary_record sr ON sr.id = cr.salary_record_id AND sr.deleted=0
WHERE cr.salary_record_id IS NOT NULL AND cr.salary_record_id > 0 AND sr.id IS NULL;

-- 3.3 commission_detail → salary_record (必填)
SELECT 'detail_no_record' as broken, count(*) as cnt
FROM mxx_finance_commission_detail cd
LEFT JOIN mxx_finance_salary_record sr ON sr.id = cd.salary_record_id AND sr.deleted=0
WHERE sr.id IS NULL;

-- 3.4 commission_allocation → commission_result (必填)
SELECT 'alloc_no_result' as broken, count(*) as cnt
FROM mxx_finance_commission_allocation ca
LEFT JOIN mxx_finance_commission_result cr ON cr.id = ca.commission_result_id
WHERE cr.id IS NULL;

-- 3.5 commission_allocation → salary_record (可选)
SELECT 'alloc_invalid_salary' as broken, count(*) as cnt
FROM mxx_finance_commission_allocation ca
LEFT JOIN mxx_finance_salary_record sr ON sr.id = ca.salary_record_id AND sr.deleted=0
WHERE ca.salary_record_id IS NOT NULL AND ca.salary_record_id > 0 AND sr.id IS NULL;

\echo '========== 4. 资金池关联验证 =========='
-- 4.1 commission_pool_log → commission_pool (必填)
SELECT 'pool_log_no_pool' as broken, count(*) as cnt
FROM mxx_finance_commission_pool_log pl
LEFT JOIN mxx_finance_commission_pool p ON p.id = pl.pool_id AND p.deleted=0
WHERE p.id IS NULL;

-- 4.2 资金池余额一致性验证
SELECT 'pool_balance_inconsistent' as broken, p.id, p.pool_name, p.total_amount, p.used_amount,
       (SELECT COALESCE(SUM(CASE WHEN log_type=1 THEN amount ELSE -amount END), 0)
        FROM mxx_finance_commission_pool_log pl WHERE pl.pool_id=p.id AND pl.deleted=0) as calc_balance
FROM mxx_finance_commission_pool p
WHERE p.deleted=0
  AND p.total_amount - p.used_amount <>
      (SELECT COALESCE(SUM(CASE WHEN log_type=1 THEN amount ELSE -amount END), 0)
       FROM mxx_finance_commission_pool_log pl WHERE pl.pool_id=p.id AND pl.deleted=0);

\echo '========== 5. 费用关联验证 =========='
-- 5.1 expense_item → expense (必填)
SELECT 'expense_item_no_expense' as broken, count(*) as cnt
FROM mxx_finance_expense_item ei
LEFT JOIN mxx_finance_expense e ON e.id = ei.expense_id AND e.deleted=0
WHERE e.id IS NULL;

\echo '========== 6. 7月工资链路完整性验证 =========='
-- 检查7月每个有工资记录的员工是否都有完整的链路
SELECT
    sr.employee_id, sr.employee_name, sr.year, sr.month, sr.net_salary,
    CASE WHEN sc.id IS NOT NULL THEN 'Y' ELSE 'N' END as has_confirm,
    CASE WHEN std.id IS NOT NULL THEN 'Y' ELSE 'N' END as has_tax_detail,
    CASE WHEN p.id IS NOT NULL THEN 'Y' ELSE 'N' END as has_payslip,
    CASE WHEN ar.id IS NOT NULL THEN 'Y' ELSE 'N' END as has_attendance
FROM mxx_finance_salary_record sr
LEFT JOIN mxx_finance_salary_confirm sc ON sc.salary_record_id = sr.id
LEFT JOIN mxx_finance_salary_tax_detail std ON std.salary_record_id = sr.id
LEFT JOIN mxx_finance_payslip p ON p.salary_record_id = sr.id
LEFT JOIN mxx_finance_attendance_record ar ON ar.employee_id = sr.employee_id AND ar.year = sr.year AND ar.month = sr.month
WHERE sr.deleted=0 AND sr.year=2026 AND sr.month=7
ORDER BY sr.employee_id;

\echo '========== 7. 提成分配完整性验证 =========='
-- 检查提成分配金额是否与提成结果一致
SELECT
    cr.id as result_id, cr.commission_amount,
    (SELECT COALESCE(SUM(amount), 0) FROM mxx_finance_commission_allocation ca WHERE ca.commission_result_id=cr.id AND ca.deleted=0) as allocated_total,
    cr.commission_amount - (SELECT COALESCE(SUM(amount), 0) FROM mxx_finance_commission_allocation ca WHERE ca.commission_result_id=cr.id AND ca.deleted=0) as diff
FROM mxx_finance_commission_result cr
WHERE cr.settled = 0
ORDER BY cr.id;

\echo '========== 8. 工资条发送状态统计 =========='
SELECT
    CASE
        WHEN send_status=0 THEN '未发送'
        WHEN send_status=1 THEN '已发送'
        WHEN send_status=2 THEN '已查看'
        WHEN send_status=3 THEN '已确认'
        ELSE '其他'
    END as status,
    count(*) as cnt,
    SUM(net_salary) as total_amount
FROM mxx_finance_payslip
GROUP BY send_status
ORDER BY send_status;
