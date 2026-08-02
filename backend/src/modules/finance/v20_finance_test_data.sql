-- ============================================================
-- 财务模块测试数据补全脚本
-- 重点：工资发放、工资条、提成分配、资金池、费用明细、调薪记录、财务统计
-- 原则：不修改已有的规则、税务、社保等已完善数据，只补全缺失数据
-- 幂等设计：可重复执行
-- ============================================================

-- ============================================================
-- 1. 补全调薪记录（salary_adjustment，原0条）
-- 为各员工创建调薪记录，关联员工ID
-- ============================================================
INSERT INTO mxx_finance_salary_adjustment (id, employee_id, adjustment_date, adjustment_type, old_base_salary, new_base_salary, old_position_allowance, new_position_allowance, old_performance_base, new_performance_base, adjustment_reason, approver_id, approver_name, approve_time, status, create_time)
VALUES
    (200, 3, '2026-01-15', 1, 12000.00, 15000.00, 2000.00, 3000.00, 5000.00, 6000.00, '年度绩效考核优秀，薪资上调', 3, '超级管理员', '2026-01-20', 3, NOW()),
    (201, 5, '2026-01-15', 1, 10000.00, 12000.00, 1500.00, 2000.00, 4000.00, 5000.00, '年度绩效考核良好，薪资上调', 3, '超级管理员', '2026-01-20', 3, NOW()),
    (202, 6, '2026-01-15', 1, 8000.00, 9000.00, 1000.00, 1500.00, 3000.00, 4000.00, '年度绩效考核合格，薪资上调', 3, '超级管理员', '2026-01-20', 3, NOW()),
    (203, 7, '2026-01-15', 1, 5000.00, 6000.00, 800.00, 1000.00, 2000.00, 3000.00, '年度绩效考核合格，薪资上调', 3, '超级管理员', '2026-01-20', 3, NOW()),
    (204, 12, '2026-01-15', 1, 8000.00, 10000.00, 1200.00, 1500.00, 3000.00, 4000.00, '年度绩效考核良好，薪资上调', 3, '超级管理员', '2026-01-20', 3, NOW()),
    (205, 3, '2026-07-01', 2, 15000.00, 16000.00, 3000.00, 3500.00, 6000.00, 7000.00, '年中调薪，职位晋升', 3, '超级管理员', '2026-07-05', 3, NOW()),
    (206, 6, '2026-07-01', 1, 9000.00, 9500.00, 1500.00, 1800.00, 4000.00, 4500.00, '年中调薪，绩效提升', 3, '超级管理员', '2026-07-05', 3, NOW())
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 2. 补全资金池（commission_pool，原0条）
-- 创建3个资金池，关联部门
-- ============================================================
INSERT INTO mxx_finance_commission_pool (id, pool_name, department_id, manager_id, total_amount, used_amount, status, description, create_time, update_time, deleted)
VALUES
    (200, '销售部团建资金池', 111, 5, 50000.00, 12000.00, 1, '销售部门团建活动资金，来源于团队提成分配', NOW(), NOW(), 0),
    (201, '管理层激励资金池', 100, 3, 80000.00, 25000.00, 1, '管理层激励奖金池，来源于管理分润', NOW(), NOW(), 0),
    (202, '总经理团队激励池', 110, 3, 100000.00, 30000.00, 1, '总经理掌控的团队激励资金池', NOW(), NOW(), 0)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 3. 补全资金池流水（commission_pool_log，原0条）
-- 为每个资金池创建入账和支出流水
-- ============================================================
INSERT INTO mxx_finance_commission_pool_log (id, pool_id, log_type, amount, source_rule_id, source_employee_id, source_year, source_month, usage_description, usage_date, operator_id, create_time, deleted)
VALUES
    -- 销售部团建资金池流水
    (200, 200, 1, 50000.00, 11, 5, 2026, 6, '6月团队提成分配入账', NULL, 3, NOW(), 0),
    (201, 200, 2, 8000.00, NULL, NULL, 2026, 7, '7月销售部季度团建活动支出', '2026-07-15', 5, NOW(), 0),
    (202, 200, 2, 4000.00, NULL, NULL, 2026, 7, '7月销售部生日会支出', '2026-07-20', 5, NOW(), 0),
    -- 管理层激励资金池流水
    (203, 201, 1, 80000.00, 3, 3, 2026, 6, '6月管理分润入账', NULL, 3, NOW(), 0),
    (204, 201, 2, 15000.00, NULL, NULL, 2026, 7, '7月管理层激励奖金发放', '2026-07-10', 3, NOW(), 0),
    (205, 201, 2, 10000.00, NULL, NULL, 2026, 7, '7月优秀员工奖励', '2026-07-25', 3, NOW(), 0),
    -- 总经理团队激励池流水
    (206, 202, 1, 100000.00, 11, 3, 2026, 6, '6月团队激励奖金入账', NULL, 3, NOW(), 0),
    (207, 202, 2, 20000.00, NULL, NULL, 2026, 7, '7月跨部门协作奖励', '2026-07-12', 3, NOW(), 0),
    (208, 202, 2, 10000.00, NULL, NULL, 2026, 7, '7月特别贡献奖', '2026-07-28', 3, NOW(), 0)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 4. 补全提成分配记录（commission_allocation，原0条）
-- 关联提成结果和工资记录
-- 注：分配总额必须等于提成结果金额，避免数据不一致
-- ============================================================
INSERT INTO mxx_finance_commission_allocation (id, commission_result_id, allocator_id, employee_id, employee_name, amount, allocate_method, employee_payment, team_total_payment, salary_record_id, year, month, remark, create_time, deleted)
VALUES
    -- 基于现有commission_result(id=1, amount=3390)进行分配，合计3390
    (200, 1, 5, 7, '业务员', 2034.00, 1, 2034.00, 3390.00, 40, 2026, 7, '合同10提成分配，业务员获得60%', NOW(), 0),
    (201, 1, 5, 6, '销售经理', 847.50, 1, 847.50, 3390.00, NULL, 2026, 7, '合同10提成分配，销售经理获得25%', NOW(), 0),
    (202, 1, 5, 3, '销售总监', 508.50, 1, 508.50, 3390.00, NULL, 2026, 7, '合同10提成分配，销售总监获得15%', NOW(), 0)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 5. 补全费用明细项（expense_item，原0条）
-- 为现有费用申请(expense id 100-106)创建明细项
-- 实际字段：id,expense_id,item_date,item_amount,item_category,item_description,item_attachment,create_time
-- ============================================================
INSERT INTO mxx_finance_expense_item (id, expense_id, item_date, item_amount, item_category, item_description, item_attachment, create_time)
SELECT
    200 + ROW_NUMBER() OVER (ORDER BY e.id) as id,
    e.id as expense_id,
    e.apply_date as item_date,
    e.amount as item_amount,
    COALESCE(et.type_code, 'OTHER') as item_category,
    COALESCE(et.type_name, '其他费用') || '明细' as item_description,
    NULL as item_attachment,
    NOW() as create_time
FROM mxx_finance_expense e
LEFT JOIN mxx_finance_expense_type et ON et.id = e.expense_type
WHERE e.deleted = 0
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 6. 补全7月工资记录（salary_record，现有只有员工7的7月工资）
-- 为员工3/5/6/12创建7月工资记录
-- ============================================================
INSERT INTO mxx_finance_salary_record (
    id, employee_id, employee_name, department_name, year, month,
    base_salary, commission_amount, performance_bonus, deduction_amount, total_salary,
    social_insurance_personal, housing_fund_personal, social_insurance_company, housing_fund_company,
    tax_amount, net_salary, team_commission_amount, bonus_amount, allocated_commission, deferred_commission,
    status, employee_confirmed, confirmed_time, remark, created_by, create_time, updated_by, update_time, deleted
)
VALUES
    (200, 3, '超级管理员', '总公司', 2026, 7, 16000.00, 0.00, 7000.00, 0.00, 23000.00, 3200.00, 1920.00, 4000.00, 2400.00, 2100.00, 15780.00, 1500.00, 2000.00, 1500.00, 500.00, 2, 1, NOW(), '7月工资，含管理分润', 3, NOW(), 3, NOW(), 0),
    (201, 5, '销售总监', '销售部', 2026, 7, 12000.00, 3390.00, 5000.00, 0.00, 20390.00, 2400.00, 1440.00, 3000.00, 1800.00, 1750.00, 13800.00, 2000.00, 1000.00, 2000.00, 0.00, 2, 1, NOW(), '7月工资，含合同10提成', 3, NOW(), 3, NOW(), 0),
    (202, 6, '销售经理', '销售部', 2026, 7, 9500.00, 847.50, 4500.00, 100.00, 14747.50, 1900.00, 1140.00, 2375.00, 1425.00, 950.00, 10757.50, 1200.00, 500.00, 1200.00, 0.00, 2, 1, NOW(), '7月工资，含团队提成分配', 3, NOW(), 3, NOW(), 0),
    (203, 12, '财务专员', '财务部', 2026, 7, 10000.00, 0.00, 4000.00, 0.00, 14000.00, 2000.00, 1200.00, 2500.00, 1500.00, 1020.00, 9780.00, 0.00, 1000.00, 0.00, 0.00, 2, 1, NOW(), '7月工资，财务部门', 3, NOW(), 3, NOW(), 0)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 7. 补全7月工资条（payslip，现有只有员工7的7月工资条）
-- 为新增的7月工资记录创建工资条
-- ============================================================
INSERT INTO mxx_finance_payslip (id, salary_record_id, employee_id, year, month, total_salary, social_insurance_personal, tax_amount, net_salary, detail_json, send_status, send_channels, send_time, read_time, confirm_time, password_protected, password_hash, withdraw_time, withdraw_reason, withdrawn_by, create_time)
VALUES
    (200, 200, 3, 2026, 7, 23000.00, 5120.00, 2100.00, 15780.00, '{"items":[{"name":"基本工资","amount":16000},{"name":"绩效奖金","amount":7000},{"name":"管理分润","amount":1500},{"name":"奖金","amount":2000},{"name":"社保个人","amount":-5120},{"name":"个人所得税","amount":-2100}]}'::json, 1, 'sms,email', NOW(), NULL, NOW(), 1, NULL, NULL, NULL, NULL, NOW()),
    (201, 201, 5, 2026, 7, 20390.00, 3840.00, 1750.00, 13800.00, '{"items":[{"name":"基本工资","amount":12000},{"name":"提成","amount":3390},{"name":"绩效奖金","amount":5000},{"name":"团队提成","amount":2000},{"name":"奖金","amount":1000},{"name":"社保个人","amount":-3840},{"name":"个人所得税","amount":-1750}]}'::json, 1, 'sms,email', NOW(), NULL, NOW(), 1, NULL, NULL, NULL, NULL, NOW()),
    (202, 202, 6, 2026, 7, 14747.50, 3040.00, 950.00, 10757.50, '{"items":[{"name":"基本工资","amount":9500},{"name":"提成","amount":847.5},{"name":"绩效奖金","amount":4500},{"name":"团队提成","amount":1200},{"name":"奖金","amount":500},{"name":"考勤扣款","amount":-100},{"name":"社保个人","amount":-3040},{"name":"个人所得税","amount":-950}]}'::json, 1, 'sms', NOW(), NULL, NOW(), 1, NULL, NULL, NULL, NULL, NOW()),
    (203, 203, 12, 2026, 7, 14000.00, 3200.00, 1020.00, 9780.00, '{"items":[{"name":"基本工资","amount":10000},{"name":"绩效奖金","amount":4000},{"name":"奖金","amount":1000},{"name":"社保个人","amount":-3200},{"name":"个人所得税","amount":-1020}]}'::json, 0, NULL, NULL, NULL, NULL, 0, NULL, NULL, NULL, NULL, NOW())
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 8. 补全7月工资确认记录（salary_confirm）
-- 为新增的7月工资记录创建确认记录
-- ============================================================
INSERT INTO mxx_finance_salary_confirm (id, salary_record_id, employee_id, employee_name, year, month, action, reason, status, handler_id, handler_name, handle_time, handle_remark, create_time)
VALUES
    (200, 200, 3, '超级管理员', 2026, 7, 1, '工资核对无误', 2, 3, '超级管理员', NOW(), '已确认', NOW()),
    (201, 201, 5, '销售总监', 2026, 7, 1, '工资核对无误', 2, 5, '销售总监', NOW(), '已确认', NOW()),
    (202, 202, 6, '销售经理', 2026, 7, 1, '工资核对无误', 2, 6, '销售经理', NOW(), '已确认', NOW()),
    (203, 203, 12, '财务专员', 2026, 7, 2, '社保扣款金额有疑问，申请重新核算', 1, NULL, NULL, NULL, NULL, NOW())
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 9. 补全7月工资税务明细（salary_tax_detail）
-- 为新增的7月工资记录创建税务明细
-- ============================================================
INSERT INTO mxx_finance_salary_tax_detail (id, salary_record_id, employee_id, year, month, monthly_income, monthly_threshold, monthly_special_deduction, monthly_other_deduction, cumulative_income, cumulative_taxable, applicable_rate, quick_deduction, cumulative_tax_should, cumulative_tax_paid, monthly_tax, create_time)
VALUES
    (200, 200, 3, 2026, 7, 23000.00, 5000.00, 3000.00, 0.00, 161000.00, 124000.00, 0.10, 2520.00, 9880.00, 7780.00, 2100.00, NOW()),
    (201, 201, 5, 2026, 7, 20390.00, 5000.00, 2000.00, 0.00, 142730.00, 111730.00, 0.10, 2520.00, 8653.00, 6903.00, 1750.00, NOW()),
    (202, 202, 6, 2026, 7, 14747.50, 5000.00, 1500.00, 0.00, 103222.50, 81722.50, 0.10, 2520.00, 5652.25, 4702.25, 950.00, NOW()),
    (203, 203, 12, 2026, 7, 14000.00, 5000.00, 1000.00, 0.00, 98000.00, 79000.00, 0.10, 2520.00, 5380.00, 4360.00, 1020.00, NOW())
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 10. 补全7月考勤记录（attendance_record，现有只有6月）
-- 为员工3/5/6/7/12创建7月考勤记录
-- ============================================================
INSERT INTO mxx_finance_attendance_record (id, employee_id, year, month, work_days, actual_work_days, late_count, early_leave_count, absent_count, personal_leave_days, sick_leave_days, annual_leave_days, overtime_hours_weekday, overtime_hours_weekend, overtime_hours_holiday, data_source, create_time)
VALUES
    (200, 3, 2026, 7, 23.0, 23.0, 0, 0, 0, 0.0, 0.0, 0.0, 10.0, 8.0, 0.0, 1, NOW()),
    (201, 5, 2026, 7, 23.0, 22.0, 1, 0, 0, 0.0, 1.0, 0.0, 8.0, 6.0, 0.0, 1, NOW()),
    (202, 6, 2026, 7, 23.0, 21.0, 2, 1, 0, 1.0, 0.0, 0.0, 12.0, 4.0, 0.0, 1, NOW()),
    (203, 7, 2026, 7, 23.0, 20.0, 3, 2, 0, 2.0, 1.0, 0.0, 6.0, 8.0, 0.0, 1, NOW()),
    (204, 12, 2026, 7, 23.0, 23.0, 0, 0, 0, 0.0, 0.0, 0.0, 4.0, 0.0, 0.0, 1, NOW())
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 11. 补全7月银行代发文件（bank_payment_file）
-- ============================================================
INSERT INTO mxx_finance_bank_payment_file (id, year, month, bank_type, file_name, file_path, file_format, total_count, total_amount, status, creator_id, creator_name, create_time)
VALUES
    (200, 2026, 7, 'icbc', '2026年7月工资代发文件_icbc.txt', '/upload/bank/202607_icbc.txt', 'txt', 4, 50017.50, 2, 3, '超级管理员', NOW()),
    (201, 2026, 7, 'cmb', '2026年7月工资代发文件_cmb.txt', '/upload/bank/202607_cmb.txt', 'txt', 4, 50017.50, 1, 3, '超级管理员', NOW())
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 12. 补全财务统计（finance_statistics，原0条）
-- 实际字段：id,stat_date,stat_type,total_income,success_amount,refund_amount,member_fee_amount,order_count,success_count,refund_count,create_time,update_time
-- ============================================================
INSERT INTO mxx_finance_statistics (id, stat_date, stat_type, total_income, success_amount, refund_amount, member_fee_amount, order_count, success_count, refund_count, create_time, update_time)
VALUES
    (200, '2026-06-01', 1, 2500000.00, 800000.00, 0.00, 50000.00, 8, 6, 0, NOW(), NOW()),
    (201, '2026-06-01', 2, 1800000.00, 800000.00, 0.00, 50000.00, 6, 4, 0, NOW(), NOW()),
    (202, '2026-07-01', 1, 1200000.00, 720000.00, 49000.00, 30000.00, 5, 4, 1, NOW(), NOW()),
    (203, '2026-07-01', 2, 1100000.00, 720000.00, 49000.00, 30000.00, 4, 3, 1, NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 13. 补全工资计算日志（salary_calc_log，补充7月）
-- ============================================================
INSERT INTO mxx_finance_salary_calc_log (id, year, month, trigger_type, result, generated_count, error_message, elapsed_ms, operator_id, operator_name, create_time)
VALUES
    (200, 2026, 7, 1, 1, 5, NULL, 1520, 3, '超级管理员', NOW()),
    (201, 2026, 7, 2, 1, 5, NULL, 980, 3, '超级管理员', NOW())
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 14. 更新序列（确保自增ID不冲突）
-- ============================================================
SELECT setval('mxx_finance_salary_adjustment_id_seq', GREATEST((SELECT MAX(id) FROM mxx_finance_salary_adjustment), 206));
SELECT setval('mxx_finance_commission_pool_id_seq', GREATEST((SELECT MAX(id) FROM mxx_finance_commission_pool), 202));
SELECT setval('mxx_finance_commission_pool_log_id_seq', GREATEST((SELECT MAX(id) FROM mxx_finance_commission_pool_log), 208));
SELECT setval('mxx_finance_commission_allocation_id_seq', GREATEST((SELECT MAX(id) FROM mxx_finance_commission_allocation), 202));
SELECT setval('mxx_finance_expense_item_id_seq', GREATEST((SELECT MAX(id) FROM mxx_finance_expense_item), 210));
SELECT setval('mxx_finance_salary_record_id_seq', GREATEST((SELECT MAX(id) FROM mxx_finance_salary_record), 203));
SELECT setval('mxx_finance_payslip_id_seq', GREATEST((SELECT MAX(id) FROM mxx_finance_payslip), 203));
SELECT setval('mxx_finance_salary_confirm_id_seq', GREATEST((SELECT MAX(id) FROM mxx_finance_salary_confirm), 203));
SELECT setval('mxx_finance_salary_tax_detail_id_seq', GREATEST((SELECT MAX(id) FROM mxx_finance_salary_tax_detail), 203));
SELECT setval('mxx_finance_attendance_record_id_seq', GREATEST((SELECT MAX(id) FROM mxx_finance_attendance_record), 204));
SELECT setval('mxx_finance_bank_payment_file_id_seq', GREATEST((SELECT MAX(id) FROM mxx_finance_bank_payment_file), 201));
SELECT setval('mxx_finance_statistics_id_seq', GREATEST((SELECT MAX(id) FROM mxx_finance_statistics), 201));
SELECT setval('mxx_finance_salary_calc_log_id_seq', GREATEST((SELECT MAX(id) FROM mxx_finance_salary_calc_log), 201));
