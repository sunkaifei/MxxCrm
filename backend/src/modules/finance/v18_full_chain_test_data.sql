-- ============================================================
-- 全链路测试数据：打通 Lead→Customer→Contact→Opportunity→Quotation→Order→Contract→Payment→Invoice→Shipment→Refund→Expense
-- 幂等设计：可重复执行，已存在数据自动跳过
-- ============================================================

-- ============================================================
-- 0. 创建费用相关表（如不存在）
-- ============================================================
CREATE TABLE IF NOT EXISTS mxx_finance_expense_type (
    id BIGSERIAL PRIMARY KEY,
    type_name VARCHAR(100),
    type_code VARCHAR(50),
    parent_id BIGINT,
    sort INTEGER DEFAULT 0,
    status SMALLINT DEFAULT 1,
    is_system SMALLINT DEFAULT 0,
    create_time TIMESTAMP DEFAULT NOW(),
    deleted SMALLINT DEFAULT 0
);

CREATE TABLE IF NOT EXISTS mxx_finance_expense (
    id BIGSERIAL PRIMARY KEY,
    expense_no VARCHAR(50),
    title VARCHAR(200),
    expense_type BIGINT,
    applicant_id BIGINT,
    dept_id BIGINT,
    customer_id BIGINT,
    opportunity_id BIGINT,
    order_id BIGINT,
    amount DECIMAL(12,2),
    currency VARCHAR(10) DEFAULT 'CNY',
    apply_date DATE,
    status SMALLINT DEFAULT 1,
    approval_status SMALLINT DEFAULT 0,
    instance_id BIGINT,
    remark TEXT,
    attachment JSONB,
    create_by BIGINT,
    create_time TIMESTAMP DEFAULT NOW(),
    update_time TIMESTAMP DEFAULT NOW(),
    deleted SMALLINT DEFAULT 0
);

-- 费用类型基础数据
INSERT INTO mxx_finance_expense_type (id, type_name, type_code, sort, status, is_system, create_time, deleted)
VALUES
    (1, '差旅费', 'TRAVEL', 1, 1, 1, NOW(), 0),
    (2, '招待费', 'ENTERTAIN', 2, 1, 1, NOW(), 0),
    (3, '交通费', 'TRANSPORT', 3, 1, 1, NOW(), 0),
    (4, '办公费', 'OFFICE', 4, 1, 1, NOW(), 0),
    (5, '市场推广费', 'MARKETING', 5, 1, 1, NOW(), 0)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 1. 修复现有断链数据
-- ============================================================

-- 1.1 修复线索→客户关联（Lead 2,3,4,7 缺少 converted_to_customer_id）
UPDATE mxx_crm_lead SET converted_to_customer_id = 1, converted_at = NOW() WHERE id = 2 AND converted_to_customer_id IS NULL AND deleted = 0;
UPDATE mxx_crm_lead SET converted_to_customer_id = 2, converted_at = NOW() WHERE id = 3 AND converted_to_customer_id IS NULL AND deleted = 0;
UPDATE mxx_crm_lead SET converted_to_customer_id = 3, converted_at = NOW() WHERE id = 4 AND converted_to_customer_id IS NULL AND deleted = 0;
UPDATE mxx_crm_lead SET converted_to_customer_id = 4, converted_at = NOW() WHERE id = 7 AND converted_to_customer_id IS NULL AND deleted = 0;

-- 1.2 修复商机→线索关联（所有商机 lead_id 为 NULL）
UPDATE mxx_crm_opportunity SET lead_id = 2 WHERE id = 1 AND lead_id IS NULL AND deleted = 0;
UPDATE mxx_crm_opportunity SET lead_id = 3 WHERE id = 2 AND lead_id IS NULL AND deleted = 0;
UPDATE mxx_crm_opportunity SET lead_id = 4 WHERE id = 3 AND lead_id IS NULL AND deleted = 0;
UPDATE mxx_crm_opportunity SET lead_id = 7 WHERE id = 4 AND lead_id IS NULL AND deleted = 0;
UPDATE mxx_crm_opportunity SET lead_id = 3 WHERE id = 6 AND lead_id IS NULL AND deleted = 0;
UPDATE mxx_crm_opportunity SET lead_id = 4 WHERE id = 7 AND lead_id IS NULL AND deleted = 0;

-- 1.3 修复合同2的客户ID（原为7/美团，但订单3关联到此合同且客户为2/腾讯，统一为2）
UPDATE mxx_crm_contract SET customer_id = 2 WHERE id = 2 AND deleted = 0;

-- 1.4 修复合同→商机关联（合同5的opportunity_id=100需在商机100创建后更新，移至第4步之后）
UPDATE mxx_crm_contract SET opportunity_id = 2 WHERE id = 2 AND opportunity_id IS NULL AND deleted = 0;
UPDATE mxx_crm_contract SET opportunity_id = 3 WHERE id = 3 AND opportunity_id IS NULL AND deleted = 0;
UPDATE mxx_crm_contract SET opportunity_id = 4 WHERE id = 4 AND opportunity_id IS NULL AND deleted = 0;
UPDATE mxx_crm_contract SET opportunity_id = 1 WHERE id = 6 AND opportunity_id IS NULL AND deleted = 0;

-- 1.5 修复合同→订单关联
UPDATE mxx_crm_contract SET order_id = 1 WHERE id = 1 AND order_id IS NULL AND deleted = 0;
UPDATE mxx_crm_contract SET order_id = 3 WHERE id = 2 AND order_id IS NULL AND deleted = 0;

-- 1.6 修复订单→报价单、商机关联
UPDATE mxx_sale_order SET opportunity_id = 1 WHERE id IN (1, 2) AND opportunity_id IS NULL AND deleted = 0;
UPDATE mxx_sale_order SET opportunity_id = 2 WHERE id = 3 AND opportunity_id IS NULL AND deleted = 0;
UPDATE mxx_sale_order SET opportunity_id = 3 WHERE id = 4 AND opportunity_id IS NULL AND deleted = 0;

-- 1.7 修复回款→合同关联
UPDATE mxx_sale_payment SET contract_id = 1 WHERE id IN (1, 2) AND contract_id IS NULL AND deleted = 0;
UPDATE mxx_sale_payment SET contract_id = 2 WHERE id = 3 AND contract_id IS NULL AND deleted = 0;

-- ============================================================
-- 2. 补全缺失的联系人（客户6,7,8,9 无联系人）
-- ============================================================
INSERT INTO mxx_crm_contact (id, customer_id, name, title, phone, mobile, gender, is_primary, is_billing, is_shipping, create_time, update_time, deleted)
VALUES
    (100, 6, '王经理', '采购经理', '13800001006', '13800001006', 1, true, true, false, NOW(), NOW(), 0),
    (101, 7, '李总', '运营总监', '13800001007', '13800001007', 1, true, true, true, NOW(), NOW(), 0),
    (102, 8, '赵总监', '市场总监', '13800001008', '13800001008', 1, true, true, false, NOW(), NOW(), 0),
    (103, 9, '孙总', '技术总监', '13800001009', '13800001009', 1, true, true, true, NOW(), NOW(), 0)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 3. 补全缺失的线索（客户5,9 无来源线索）
-- ============================================================
INSERT INTO mxx_crm_lead (id, company_name, contact_name, title, phone, mobile, source, status, level, assigned_to, converted_to_customer_id, converted_at, created_by, create_time, update_time, deleted)
VALUES
    (100, '小米科技', '雷军', '采购咨询', '13800001005', '13800001005', 'website'::mxx_lead_source, 3, 2, 5, 5, NOW(), 3, NOW(), NOW(), 0),
    (101, '百度', '孙总', '技术合作', '13800001009', '13800001009', 'exhibition'::mxx_lead_source, 3, 1, 7, 9, NOW(), 3, NOW(), NOW(), 0)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 4. 补全缺失的商机（客户5,7,8,9 无商机）
-- ============================================================
INSERT INTO mxx_crm_opportunity (id, customer_id, contact_id, lead_id, name, description, stage, probability, amount, currency, expected_close_date, assigned_to, source, quote_status, order_status, contract_status, shipment_status, payment_status, invoice_status, created_by, create_time, update_time, deleted)
VALUES
    (100, 5, 5, 100, '小米智能硬件采购项目', '小米科技智能硬件年度采购商机', 4, 80, 339000.00, 1, '2026-09-30', 5, 'website'::mxx_lead_source, 2, 2, 2, 1, 2, 1, 3, NOW(), NOW(), 0),
    (101, 7, 101, 1, '美团年度服务合同', '美团年度IT服务采购商机', 3, 60, 800000.00, 1, '2026-10-15', 6, 'website'::mxx_lead_source, 2, 2, 2, 2, 2, 2, 3, NOW(), NOW(), 0),
    (102, 8, 102, 5, '小红书广告投放合作', '小红书平台广告投放商机', 2, 40, 500000.00, 1, '2026-11-30', 7, 'exhibition'::mxx_lead_source, 1, 0, 0, 0, 0, 0, 3, NOW(), NOW(), 0),
    (103, 9, 103, 101, '百度云计算服务合同', '百度云计算服务年度合同商机', 4, 85, 1200000.00, 1, '2026-09-15', 5, 'exhibition'::mxx_lead_source, 2, 2, 2, 2, 2, 2, 3, NOW(), NOW(), 0)
ON CONFLICT (id) DO NOTHING;

-- 修复报价单2的商机关联
UPDATE mxx_sale_quotation SET opportunity_id = 100 WHERE id = 2 AND opportunity_id IS NULL AND deleted = 0;

-- 修复合同5的商机关联（商机100已创建，现在可以安全更新）
UPDATE mxx_crm_contract SET opportunity_id = 100 WHERE id = 5 AND (opportunity_id IS NULL OR opportunity_id = 0) AND deleted = 0;

-- ============================================================
-- 5. 补全报价单（为各商机创建报价单）
-- ============================================================
INSERT INTO mxx_sale_quotation (id, quotation_no, customer_id, customer_name, contact_id, contact_name, opportunity_id, opportunity_title, title, total_amount, currency, tax_amount, discount_amount, grand_total, valid_until, quotation_date, status, approval_status, payment_terms, delivery_terms, delivery_date, owner_user_id, dept_id, create_by, create_time, update_by, update_time, deleted)
VALUES
    (100, 'QUO-20260701-0100', 1, '阿里巴巴集团', 1, '马云', 1, '年度采购项目', '阿里巴巴年度采购报价单', 904000.00, 1, 104000.00, 0.00, 904000.00, '2026-08-31', '2026-07-01', 2, 3, '合同签订后30天内付款', '签订后60天内交付', '2026-09-30', 5, 111, '3', NOW(), '3', NOW(), 0),
    (101, 'QUO-20260702-0101', 2, '腾讯科技', 2, '马化腾', 2, '云服务采购', '腾讯云服务采购报价单', 565000.00, 1, 65000.00, 0.00, 565000.00, '2026-08-31', '2026-07-02', 2, 3, '分期付款40%/30%/30%', '分批交付', '2026-10-10', 6, 111, '3', NOW(), '3', NOW(), 0),
    (102, 'QUO-20260703-0102', 3, '华为技术有限公司', 3, '任正非', 3, '企业定制项目', '华为企业定制设备报价单', 1356000.00, 1, 156000.00, 0.00, 1356000.00, '2026-08-31', '2026-07-03', 2, 3, '合同签订后支付40%，验收支付60%', '签订后90天内交付', '2026-10-15', 5, 111, '3', NOW(), '3', NOW(), 0),
    (103, 'QUO-20260704-0103', 4, '字节跳动', 4, '张一鸣', 4, '广告合作项目', '字节跳动年度广告服务报价单', 565000.00, 1, 65000.00, 0.00, 565000.00, '2026-08-31', '2026-07-04', 2, 3, '按月结算', '持续服务', '2026-12-31', 6, 111, '3', NOW(), '3', NOW(), 0),
    (104, 'QUO-20260707-0104', 7, '美团', 101, '李总', 101, '美团年度服务合同', '美团年度IT服务报价单', 800000.00, 1, 92000.00, 0.00, 800000.00, '2026-09-30', '2026-07-07', 2, 3, '合同签订后30天内付款', '持续服务', '2027-07-07', 6, 111, '3', NOW(), '3', NOW(), 0),
    (105, 'QUO-20260708-0105', 8, '小红书', 102, '赵总监', 102, '小红书广告投放合作', '小红书广告投放报价单', 500000.00, 1, 57500.00, 0.00, 500000.00, '2026-10-31', '2026-07-08', 1, 0, '按月结算', '持续服务', '2026-12-31', 7, 111, '3', NOW(), '3', NOW(), 0),
    (106, 'QUO-20260709-0106', 9, '百度', 103, '孙总', 103, '百度云计算服务合同', '百度云计算服务报价单', 1200000.00, 1, 138000.00, 0.00, 1200000.00, '2026-08-31', '2026-07-09', 2, 3, '合同签订后40%，验收60%', '分批交付', '2026-10-15', 5, 111, '3', NOW(), '3', NOW(), 0)
ON CONFLICT (id) DO NOTHING;

-- 修复现有订单的报价单关联
UPDATE mxx_sale_order SET quotation_id = 100 WHERE id IN (1, 2) AND quotation_id IS NULL AND deleted = 0;
UPDATE mxx_sale_order SET quotation_id = 101 WHERE id = 3 AND quotation_id IS NULL AND deleted = 0;
UPDATE mxx_sale_order SET quotation_id = 102, contract_id = 3 WHERE id = 4 AND quotation_id IS NULL AND deleted = 0;

-- ============================================================
-- 6. 补全合同（为客户7美团、客户9百度创建合同）—— 必须先于订单，订单外键引用合同
-- ============================================================
INSERT INTO mxx_crm_contract (id, contract_no, customer_id, opportunity_id, order_id, name, contract_type, amount, currency, tax_amount, total_amount, status, start_date, end_date, sign_date, payment_method_type, assigned_to, approval_status, remark, created_by, create_time, updated_by, update_time, deleted)
VALUES
    (107, 'CON-20260707-0107', 7, 101, 108, '美团年度IT服务合同', 3, 708000.00, 1, 92000.00, 800000.00, 2, '2026-07-07', '2027-07-07', '2026-07-07', 1, 6, 3, '美团年度IT服务合同，已审批', 3, NOW(), 3, NOW(), 0),
    (108, 'CON-20260709-0108', 9, 103, 109, '百度云计算服务合同', 1, 1062000.00, 1, 138000.00, 1200000.00, 2, '2026-07-09', '2027-07-09', '2026-07-09', 2, 5, 3, '百度云计算服务年度合同，已审批', 3, NOW(), 3, NOW(), 0)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 7. 补全订单（为合同4,5,6及新客户7,9创建订单）
-- ============================================================
INSERT INTO mxx_sale_order (id, order_no, title, order_type, status, customer_id, customer_name, contact_id, contact_name, opportunity_id, quotation_id, contract_id, order_date, delivery_date, currency, product_amount, discount_amount, shipping_fee, tax_amount, other_fee, total_amount, paid_amount, unpaid_amount, payment_status, payment_method, payment_due_date, shipping_method, owner_user_id, dept_id, approval_status, create_by, create_time, update_by, update_time, deleted)
VALUES
    (105, 'ORD-20260701-0105', '阿里巴巴战略合作订单', 1, 2, 1, '阿里巴巴集团', 1, '马云', 1, 100, 6, '2026-07-01', '2026-12-31', 1, 2212389.38, 0.00, 0.00, 287610.62, 0.00, 2500000.00, 0.00, 2500000.00, 0, 2, '2026-08-31', 1, 5, 111, 3, 3, NOW(), 3, NOW(), 0),
    (106, 'ORD-20260704-0106', '字节跳动年度广告服务订单', 1, 2, 4, '字节跳动', 4, '张一鸣', 4, 103, 4, '2026-07-04', '2026-12-31', 1, 500000.00, 0.00, 0.00, 65000.00, 0.00, 565000.00, 0.00, 565000.00, 0, 2, '2026-08-31', 1, 6, 111, 3, 3, NOW(), 3, NOW(), 0),
    (107, 'ORD-20260705-0107', '小米智能硬件采购订单', 1, 2, 5, '小米科技', 5, '雷军', 100, 2, 5, '2026-07-05', '2026-11-01', 1, 300000.00, 0.00, 0.00, 39000.00, 0.00, 339000.00, 0.00, 339000.00, 0, 2, '2026-08-31', 1, 5, 111, 3, 3, NOW(), 3, NOW(), 0),
    (108, 'ORD-20260707-0108', '美团年度IT服务订单', 1, 2, 7, '美团', 101, '李总', 101, 104, 107, '2026-07-07', '2027-07-07', 1, 708000.00, 0.00, 0.00, 92000.00, 0.00, 800000.00, 800000.00, 0.00, 2, 1, '2026-08-31', 1, 6, 111, 3, 3, NOW(), 3, NOW(), 0),
    (109, 'ORD-20260709-0109', '百度云计算服务订单', 1, 2, 9, '百度', 103, '孙总', 103, 106, 108, '2026-07-09', '2026-10-15', 1, 1062000.00, 0.00, 0.00, 138000.00, 0.00, 1200000.00, 0.00, 1200000.00, 0, 2, '2026-08-31', 1, 5, 111, 3, 3, NOW(), 3, NOW(), 0)
ON CONFLICT (id) DO NOTHING;

-- 修复合同→订单关联（补全）
UPDATE mxx_crm_contract SET order_id = 4 WHERE id = 3 AND order_id IS NULL AND deleted = 0;
UPDATE mxx_crm_contract SET order_id = 106 WHERE id = 4 AND order_id IS NULL AND deleted = 0;
UPDATE mxx_crm_contract SET order_id = 107 WHERE id = 5 AND order_id IS NULL AND deleted = 0;
UPDATE mxx_crm_contract SET order_id = 105 WHERE id = 6 AND order_id IS NULL AND deleted = 0;

-- 修复订单108和109的合同关联
UPDATE mxx_sale_order SET contract_id = 107 WHERE id = 108 AND contract_id IS NULL AND deleted = 0;
UPDATE mxx_sale_order SET contract_id = 108 WHERE id = 109 AND contract_id IS NULL AND deleted = 0;

-- ============================================================
-- 8. 补全回款计划（为新合同创建回款计划）
-- ============================================================
INSERT INTO mxx_crm_contract_payment_plan (contract_id, stage_name, payment_type, plan_amount, plan_date, status, sort, remark, create_time, update_time, deleted)
VALUES
    (107, '首付款', 1, 240000.00, '2026-07-15', 1, 1, '合同签订后支付30%', NOW(), NOW(), 0),
    (107, '进度款', 2, 320000.00, '2027-01-07', 0, 2, '服务满6个月支付40%', NOW(), NOW(), 0),
    (107, '尾款', 4, 240000.00, '2027-07-07', 0, 3, '合同到期支付30%', NOW(), NOW(), 0),
    (108, '预付款', 1, 480000.00, '2026-07-15', 1, 1, '合同签订后支付40%', NOW(), NOW(), 0),
    (108, '验收款', 4, 720000.00, '2026-10-15', 0, 2, '项目验收后支付60%', NOW(), NOW(), 0)
ON CONFLICT DO NOTHING;

-- ============================================================
-- 9. 补全回款（为合同5,6,107,108创建回款记录）
-- 注：mxx_sale_payment表无approval_status字段
-- ============================================================
INSERT INTO mxx_sale_payment (id, payment_no, contract_id, order_id, customer_id, customer_name, amount, applied_amount, unapplied_amount, currency, payment_method, payment_date, payer, bank_flow_no, status, owner_user_id, dept_id, create_by, create_time, update_by, update_time, confirm_time, confirm_by, deleted)
VALUES
    (104, 'PAY-20260701-0104', 6, 105, 1, '阿里巴巴集团', 500000.00, 0.00, 500000.00, 1, 1, '2026-07-15', '阿里巴巴集团', 'BF20260701001', 2, 5, 111, '3', NOW(), '3', NOW(), NOW(), 3, 0),
    (105, 'PAY-20260709-0105', 108, 109, 9, '百度', 480000.00, 0.00, 480000.00, 1, 1, '2026-07-15', '百度', 'BF20260701002', 2, 5, 111, '3', NOW(), '3', NOW(), NOW(), 3, 0),
    (106, 'PAY-20260707-0106', 107, 108, 7, '美团', 240000.00, 0.00, 240000.00, 1, 1, '2026-07-15', '美团', 'BF20260701003', 2, 6, 111, '3', NOW(), '3', NOW(), NOW(), 3, 0),
    (107, 'PAY-20260705-0107', 5, 107, 5, '小米科技', 100000.00, 0.00, 100000.00, 1, 1, '2026-07-25', '小米科技', 'BF20260701004', 1, 5, 111, '3', NOW(), '3', NOW(), NOW(), 3, 0)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 10. 创建发票（原0条，为所有已审批合同创建）
-- ============================================================
INSERT INTO mxx_sale_invoice (id, invoice_no, title, invoice_type, contract_id, order_id, customer_id, customer_name, tax_no, invoice_date, due_date, amount, tax_rate, tax_amount, currency, status, buyer_name, buyer_tax_no, buyer_address, buyer_bank, owner_user_id, dept_id, create_by, create_time, update_by, update_time, deleted)
VALUES
    (100, 'INV-20260701-0100', '阿里巴巴年度采购发票', 1, 1, 1, 1, '阿里巴巴集团', '91330100123456789X', '2026-07-10', '2026-08-10', 800000.00, 0.13, 104000.00, 1, 2, '阿里巴巴集团', '91330100123456789X', '杭州市余杭区文一西路969号', '招商银行杭州分行 571905123410701', 5, 111, '3', NOW(), '3', NOW(), 0),
    (101, 'INV-20260702-0101', '腾讯云服务发票', 1, 2, 3, 2, '腾讯科技', '914403007090234567X', '2026-07-12', '2026-08-12', 500000.00, 0.13, 65000.00, 1, 2, '腾讯科技（深圳）有限公司', '914403007090234567X', '深圳市南山区科技中一路腾讯大厦', '招商银行深圳分行 755905123410702', 6, 111, '3', NOW(), '3', NOW(), 0),
    (102, 'INV-20260703-0102', '华为设备采购发票', 1, 3, 4, 3, '华为技术有限公司', '91440300MA5FL23456X', '2026-07-15', '2026-08-15', 1200000.00, 0.13, 156000.00, 1, 2, '华为技术有限公司', '91440300MA5FL23456X', '深圳市龙岗区坂田华为基地', '中国银行深圳分行 755905123410703', 5, 111, '3', NOW(), '3', NOW(), 0),
    (103, 'INV-20260704-0103', '字节跳动广告服务发票', 2, 4, 106, 4, '字节跳动', '91110108552345678X', '2026-07-20', '2026-08-20', 500000.00, 0.06, 30000.00, 1, 2, '北京字节跳动科技有限公司', '91110108552345678X', '北京市海淀区中关村东路1号院', '工商银行北京分行 0200096119200034567', 6, 111, '3', NOW(), '3', NOW(), 0),
    (104, 'INV-20260705-0104', '小米硬件采购发票', 1, 5, 107, 5, '小米科技', '91110108551345678X', '2026-07-25', '2026-08-25', 300000.00, 0.13, 39000.00, 1, 1, '小米科技有限责任公司', '91110108551345678X', '北京市海淀区清河中街68号', '招商银行北京分行 0209006119200034568', 5, 111, '3', NOW(), '3', NOW(), 0),
    (105, 'INV-20260706-0105', '阿里巴巴战略合作发票', 1, 6, 105, 1, '阿里巴巴集团', '91330100123456789X', '2026-07-25', '2026-08-25', 2212389.38, 0.13, 287610.62, 1, 2, '阿里巴巴集团', '91330100123456789X', '杭州市余杭区文一西路969号', '招商银行杭州分行 571905123410701', 5, 111, '3', NOW(), '3', NOW(), 0),
    (106, 'INV-20260707-0106', '美团IT服务发票', 2, 107, 108, 7, '美团', '91110108553345678X', '2026-07-20', '2026-08-20', 708000.00, 0.06, 92000.00, 1, 2, '北京三快科技有限公司', '91110108553345678X', '北京市朝阳区望京东路6号院', '招商银行北京分行 0209006119200034569', 6, 111, '3', NOW(), '3', NOW(), 0),
    (107, 'INV-20260709-0107', '百度云计算服务发票', 1, 108, 109, 9, '百度', '91110108554345678X', '2026-07-15', '2026-08-15', 1062000.00, 0.13, 138000.00, 1, 2, '百度在线网络技术（北京）有限公司', '91110108554345678X', '北京市海淀区上地十街10号', '中国银行北京分行 0200096119200034570', 5, 111, '3', NOW(), '3', NOW(), 0)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 11. 创建发货单（原0条，为所有已审批订单创建）
-- ============================================================
INSERT INTO mxx_sale_shipment (id, shipment_no, order_id, customer_id, shipment_date, logistics_company, tracking_no, shipping_method, receiver_name, receiver_phone, shipping_address, total_quantity, status, remark, created_by, create_time, update_time, deleted)
VALUES
    (100, 'SHP-20260705-0100', 1, 1, '2026-07-05', '顺丰速运', 'SF1234567890', 1, '马云', '13800000001', '杭州市余杭区文一西路969号阿里巴巴', 100, 2, '阿里巴巴年度采购首批发货', 3, NOW(), NOW(), 0),
    (101, 'SHP-20260710-0101', 3, 2, '2026-07-10', '德邦物流', 'DB1234567891', 2, '马化腾', '13800000002', '深圳市南山区科技中一路腾讯大厦', 50, 2, '腾讯云服务设备发货', 3, NOW(), NOW(), 0),
    (102, 'SHP-20260715-0102', 4, 3, '2026-07-15', '京东物流', 'JD1234567892', 1, '任正非', '13800000003', '深圳市龙岗区坂田华为基地', 200, 2, '华为设备采购发货', 3, NOW(), NOW(), 0),
    (103, 'SHP-20260720-0103', 106, 4, '2026-07-20', '顺丰速运', 'SF1234567893', 3, '张一鸣', '13800000004', '北京市海淀区中关村东路1号院字节跳动', 1, 1, '字节跳动广告服务线上交付', 3, NOW(), NOW(), 0),
    (104, 'SHP-20260725-0104', 105, 1, '2026-07-25', '德邦物流', 'DB1234567894', 1, '马云', '13800000001', '杭州市余杭区文一西路969号阿里巴巴', 500, 2, '阿里巴巴战略合作首批发货', 3, NOW(), NOW(), 0),
    (105, 'SHP-20260801-0105', 107, 5, '2026-08-01', '顺丰速运', 'SF1234567895', 1, '雷军', '13800000005', '北京市海淀区清河中街68号小米科技', 1000, 1, '小米智能硬件发货', 3, NOW(), NOW(), 0),
    (106, 'SHP-20260715-0106', 108, 7, '2026-07-15', '线上交付', 'ONLINE-001', 3, '李总', '13800001007', '北京市朝阳区望京东路6号院美团', 1, 2, '美团IT服务线上交付', 3, NOW(), NOW(), 0),
    (107, 'SHP-20260720-0107', 109, 9, '2026-07-20', '京东物流', 'JD1234567896', 1, '孙总', '13800001009', '北京市海淀区上地十街10号百度', 10, 2, '百度云计算设备发货', 3, NOW(), NOW(), 0)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 12. 创建退货单（原0条，为部分订单创建退货场景）
-- ============================================================
INSERT INTO mxx_sale_refund (id, refund_no, title, order_id, customer_id, customer_name, refund_type, refund_reason, refund_status, approval_status, total_amount, restocking_fee, refund_amount, refunded_amount, receiver, receiver_phone, receiver_address, quality_check_result, quality_check_remark, owner_user_id, dept_id, remark, create_by, create_time, update_by, update_time, deleted)
VALUES
    (100, 'RFD-20260720-0100', '华为设备部分退货', 4, 3, '华为技术有限公司', 1, '部分设备型号不匹配，需退换20台', 2, 3, 50000.00, 1000.00, 49000.00, 49000.00, '任正非', '13800000003', '深圳市龙岗区坂田华为基地', 1, '质检通过，设备完好', 5, 111, '华为退回20台设备，已退款', 3, NOW(), 3, NOW(), 0),
    (101, 'RFD-20260805-0101', '字节跳动广告服务退款', 106, 4, '字节跳动', 2, '服务未达预期，协商退还部分费用', 1, 1, 50000.00, 0.00, 50000.00, 0.00, '张一鸣', '13800000004', '北京市海淀区中关村东路1号院字节跳动', NULL, NULL, 6, 111, '字节跳动广告费部分退款申请中', 3, NOW(), 3, NOW(), 0)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 13. 创建费用申请（关联客户/商机/订单）
-- 注：currency 字段为 integer 类型（1=CNY）
-- ============================================================
INSERT INTO mxx_finance_expense (id, expense_no, title, expense_type, applicant_id, dept_id, customer_id, opportunity_id, order_id, amount, currency, apply_date, status, approval_status, remark, create_by, create_time, update_time, deleted)
VALUES
    (100, 'EXP-20260705-0100', '阿里巴巴客户拜访差旅费', 1, 5, 111, 1, 1, 1, 3500.00, 1, '2026-07-05', 6, 3, '赴杭州拜访阿里巴巴客户，产生机票和住宿费用', 5, NOW(), NOW(), 0),
    (101, 'EXP-20260710-0101', '腾讯客户招待费', 2, 6, 111, 2, 2, 3, 5000.00, 1, '2026-07-10', 6, 3, '腾讯客户来访招待费用，含餐饮和接待', 6, NOW(), NOW(), 0),
    (102, 'EXP-20260715-0102', '华为项目市场推广费', 5, 5, 111, 3, 3, 4, 12000.00, 1, '2026-07-15', 4, 3, '华为企业定制项目市场推广活动费用', 5, NOW(), NOW(), 0),
    (103, 'EXP-20260720-0103', '字节跳动项目交通费', 3, 6, 111, 4, 4, 106, 2000.00, 1, '2026-07-20', 6, 3, '字节跳动项目日常交通费用', 6, NOW(), NOW(), 0),
    (104, 'EXP-20260725-0104', '小米项目差旅费', 1, 5, 111, 5, 100, 107, 4200.00, 1, '2026-07-25', 2, 1, '赴北京拜访小米客户，推进硬件采购项目', 5, NOW(), NOW(), 0),
    (105, 'EXP-20260707-0105', '美团项目办公费', 4, 6, 111, 7, 101, 108, 1500.00, 1, '2026-07-07', 6, 3, '美团IT服务项目办公耗材费用', 6, NOW(), NOW(), 0),
    (106, 'EXP-20260709-0106', '百度云计算项目差旅费', 1, 5, 111, 9, 103, 109, 3800.00, 1, '2026-07-09', 6, 3, '赴北京拜访百度客户，洽谈云计算合作', 5, NOW(), NOW(), 0)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- 14. 更新商机状态镜像字段（反映下游单据状态）
-- ============================================================
UPDATE mxx_crm_opportunity SET
    quote_status = 2, order_status = 2, contract_status = 2, shipment_status = 2, payment_status = 2, invoice_status = 2
WHERE id = 1 AND deleted = 0;

UPDATE mxx_crm_opportunity SET
    quote_status = 2, order_status = 2, contract_status = 2, shipment_status = 2, payment_status = 2, invoice_status = 2
WHERE id = 2 AND deleted = 0;

UPDATE mxx_crm_opportunity SET
    quote_status = 2, order_status = 2, contract_status = 2, shipment_status = 2, payment_status = 1, invoice_status = 2
WHERE id = 3 AND deleted = 0;

UPDATE mxx_crm_opportunity SET
    quote_status = 2, order_status = 2, contract_status = 2, shipment_status = 1, payment_status = 0, invoice_status = 2
WHERE id = 4 AND deleted = 0;

UPDATE mxx_crm_opportunity SET
    quote_status = 2, order_status = 2, contract_status = 2, shipment_status = 1, payment_status = 1, invoice_status = 1
WHERE id = 100 AND deleted = 0;

UPDATE mxx_crm_opportunity SET
    quote_status = 2, order_status = 2, contract_status = 2, shipment_status = 2, payment_status = 2, invoice_status = 2
WHERE id = 101 AND deleted = 0;

UPDATE mxx_crm_opportunity SET
    quote_status = 1, order_status = 0, contract_status = 0, shipment_status = 0, payment_status = 0, invoice_status = 0
WHERE id = 102 AND deleted = 0;

UPDATE mxx_crm_opportunity SET
    quote_status = 2, order_status = 2, contract_status = 2, shipment_status = 2, payment_status = 1, invoice_status = 2
WHERE id = 103 AND deleted = 0;

-- ============================================================
-- 15. 更新序列（确保自增ID不冲突）
-- ============================================================
SELECT setval('mxx_crm_lead_id_seq', GREATEST((SELECT MAX(id) FROM mxx_crm_lead), 101));
SELECT setval('mxx_crm_contact_id_seq', GREATEST((SELECT MAX(id) FROM mxx_crm_contact), 103));
SELECT setval('mxx_crm_opportunity_id_seq', GREATEST((SELECT MAX(id) FROM mxx_crm_opportunity), 103));
SELECT setval('mxx_sale_quotation_id_seq', GREATEST((SELECT MAX(id) FROM mxx_sale_quotation), 106));
SELECT setval('mxx_sale_order_id_seq', GREATEST((SELECT MAX(id) FROM mxx_sale_order), 109));
SELECT setval('mxx_crm_contract_id_seq', GREATEST((SELECT MAX(id) FROM mxx_crm_contract), 108));
SELECT setval('mxx_sale_payment_id_seq', GREATEST((SELECT MAX(id) FROM mxx_sale_payment), 107));
SELECT setval('mxx_sale_invoice_id_seq', GREATEST((SELECT MAX(id) FROM mxx_sale_invoice), 107));
SELECT setval('mxx_sale_shipment_id_seq', GREATEST((SELECT MAX(id) FROM mxx_sale_shipment), 107));
SELECT setval('mxx_sale_refund_id_seq', GREATEST((SELECT MAX(id) FROM mxx_sale_refund), 101));
SELECT setval('mxx_finance_expense_id_seq', GREATEST((SELECT MAX(id) FROM mxx_finance_expense), 106));
SELECT setval('mxx_finance_expense_type_id_seq', GREATEST((SELECT MAX(id) FROM mxx_finance_expense_type), 5));
