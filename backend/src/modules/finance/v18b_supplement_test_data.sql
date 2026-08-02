-- ============================================================
-- 补全合同9/10的全链路测试数据（生产库专用）
-- 合同9/10是之前已有的"全链路测试合同"，缺少报价单/订单/回款/发票/发货单
-- ============================================================

-- 报价单
INSERT INTO mxx_sale_quotation (id, quotation_no, customer_id, customer_name, contact_id, opportunity_id, opportunity_title, title, total_amount, currency, tax_amount, discount_amount, grand_total, valid_until, quotation_date, status, approval_status, payment_terms, delivery_terms, delivery_date, owner_user_id, dept_id, create_by, create_time, update_by, update_time, deleted)
VALUES
    (109, 'QUO-20260731-0109', 17, '全链路测试公司_1785511370', NULL, 15, '全链路测试商机', '合同9全链路测试报价单', 113000.00, 1, 13000.00, 0.00, 113000.00, '2026-08-31', '2026-07-31', 2, 3, '签订后30天付款', '签订后60天交付', '2026-09-30', 1, 111, '1', NOW(), '1', NOW(), 0),
    (110, 'QUO-20260731-0110', 18, '全链路测试公司_1785511987', NULL, 16, '全链路测试商机', '合同10全链路测试报价单', 113000.00, 1, 13000.00, 0.00, 113000.00, '2026-08-31', '2026-07-31', 2, 3, '签订后30天付款', '签订后60天交付', '2026-09-30', 1, 111, '1', NOW(), '1', NOW(), 0)
ON CONFLICT (id) DO NOTHING;

-- 订单
INSERT INTO mxx_sale_order (id, order_no, title, order_type, status, customer_id, customer_name, opportunity_id, quotation_id, contract_id, order_date, delivery_date, currency, product_amount, discount_amount, shipping_fee, tax_amount, other_fee, total_amount, paid_amount, unpaid_amount, payment_status, payment_method, payment_due_date, shipping_method, owner_user_id, dept_id, approval_status, create_by, create_time, update_by, update_time, deleted)
VALUES
    (110, 'ORD-20260731-0110', '合同9全链路测试订单', 1, 2, 17, '全链路测试公司_1785511370', 15, 109, 9, '2026-07-31', '2026-09-30', 1, 100000.00, 0.00, 0.00, 13000.00, 0.00, 113000.00, 0.00, 113000.00, 0, 2, '2026-08-31', 1, 1, 111, 3, '1', NOW(), '1', NOW(), 0),
    (111, 'ORD-20260731-0111', '合同10全链路测试订单', 1, 2, 18, '全链路测试公司_1785511987', 16, 110, 10, '2026-07-31', '2026-09-30', 1, 100000.00, 0.00, 0.00, 13000.00, 0.00, 113000.00, 0.00, 113000.00, 0, 2, '2026-08-31', 1, 1, 111, 3, '1', NOW(), '1', NOW(), 0)
ON CONFLICT (id) DO NOTHING;

-- 合同关联订单
UPDATE mxx_crm_contract SET order_id = 110 WHERE id = 9 AND (order_id IS NULL OR order_id = 0) AND deleted = 0;
UPDATE mxx_crm_contract SET order_id = 111 WHERE id = 10 AND (order_id IS NULL OR order_id = 0) AND deleted = 0;

-- 回款
INSERT INTO mxx_sale_payment (id, payment_no, contract_id, order_id, customer_id, customer_name, amount, applied_amount, unapplied_amount, currency, payment_method, payment_date, payer, bank_flow_no, status, owner_user_id, dept_id, create_by, create_time, update_by, update_time, confirm_time, confirm_by, deleted)
VALUES
    (108, 'PAY-20260731-0108', 9, 110, 17, '全链路测试公司_1785511370', 56500.00, 0.00, 56500.00, 1, 1, '2026-07-31', '全链路测试公司_1785511370', 'BF20260731001', 2, 1, 111, '1', NOW(), '1', NOW(), NOW(), 1, 0),
    (109, 'PAY-20260731-0109', 10, 111, 18, '全链路测试公司_1785511987', 56500.00, 0.00, 56500.00, 1, 1, '2026-07-31', '全链路测试公司_1785511987', 'BF20260731002', 2, 1, 111, '1', NOW(), '1', NOW(), NOW(), 1, 0)
ON CONFLICT (id) DO NOTHING;

-- 发票
INSERT INTO mxx_sale_invoice (id, invoice_no, title, invoice_type, contract_id, order_id, customer_id, customer_name, invoice_date, due_date, amount, tax_rate, tax_amount, currency, status, owner_user_id, dept_id, create_by, create_time, update_by, update_time, deleted)
VALUES
    (108, 'INV-20260731-0108', '合同9全链路测试发票', 1, 9, 110, 17, '全链路测试公司_1785511370', '2026-07-31', '2026-08-31', 100000.00, 0.13, 13000.00, 1, 2, 1, 111, '1', NOW(), '1', NOW(), 0),
    (109, 'INV-20260731-0109', '合同10全链路测试发票', 1, 10, 111, 18, '全链路测试公司_1785511987', '2026-07-31', '2026-08-31', 100000.00, 0.13, 13000.00, 1, 2, 1, 111, '1', NOW(), '1', NOW(), 0)
ON CONFLICT (id) DO NOTHING;

-- 发货单
INSERT INTO mxx_sale_shipment (id, shipment_no, order_id, customer_id, shipment_date, logistics_company, tracking_no, shipping_method, receiver_name, receiver_phone, shipping_address, total_quantity, status, remark, created_by, create_time, update_time, deleted)
VALUES
    (108, 'SHP-20260731-0108', 110, 17, '2026-07-31', '顺丰速运', 'SF1234567898', 1, '测试联系人', '13800000017', '全链路测试地址', 10, 2, '合同9发货', 1, NOW(), NOW(), 0),
    (109, 'SHP-20260731-0109', 111, 18, '2026-07-31', '顺丰速运', 'SF1234567899', 1, '测试联系人', '13800000018', '全链路测试地址', 10, 2, '合同10发货', 1, NOW(), NOW(), 0)
ON CONFLICT (id) DO NOTHING;

-- 更新序列
SELECT setval('mxx_sale_quotation_id_seq', GREATEST((SELECT MAX(id) FROM mxx_sale_quotation), 110));
SELECT setval('mxx_sale_order_id_seq', GREATEST((SELECT MAX(id) FROM mxx_sale_order), 111));
SELECT setval('mxx_sale_payment_id_seq', GREATEST((SELECT MAX(id) FROM mxx_sale_payment), 109));
SELECT setval('mxx_sale_invoice_id_seq', GREATEST((SELECT MAX(id) FROM mxx_sale_invoice), 109));
SELECT setval('mxx_sale_shipment_id_seq', GREATEST((SELECT MAX(id) FROM mxx_sale_shipment), 109));
