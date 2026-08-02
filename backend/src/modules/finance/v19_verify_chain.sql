-- ============================================================
-- 全链路数据完整性验证脚本
-- 验证 Lead→Customer→Contact→Opportunity→Quotation→Order→Contract→Payment→Invoice→Shipment
-- ============================================================

-- 1. 数据统计概览
\echo '========== 1. 各表数据统计 =========='
SELECT 'leads' as tbl, count(*) as total, count(*) FILTER (WHERE deleted=0) as active FROM mxx_crm_lead
UNION ALL SELECT 'customers', count(*), count(*) FILTER (WHERE deleted=0) FROM mxx_crm_customer
UNION ALL SELECT 'contacts', count(*), count(*) FILTER (WHERE deleted=0) FROM mxx_crm_contact
UNION ALL SELECT 'opportunities', count(*), count(*) FILTER (WHERE deleted=0) FROM mxx_crm_opportunity
UNION ALL SELECT 'quotations', count(*), count(*) FILTER (WHERE deleted=0) FROM mxx_sale_quotation
UNION ALL SELECT 'orders', count(*), count(*) FILTER (WHERE deleted=0) FROM mxx_sale_order
UNION ALL SELECT 'contracts', count(*), count(*) FILTER (WHERE deleted=0) FROM mxx_crm_contract
UNION ALL SELECT 'payments', count(*), count(*) FILTER (WHERE deleted=0) FROM mxx_sale_payment
UNION ALL SELECT 'invoices', count(*), count(*) FILTER (WHERE deleted=0) FROM mxx_sale_invoice
UNION ALL SELECT 'shipments', count(*), count(*) FILTER (WHERE deleted=0) FROM mxx_sale_shipment
UNION ALL SELECT 'refunds', count(*), count(*) FILTER (WHERE deleted=0) FROM mxx_sale_refund
UNION ALL SELECT 'expenses', count(*), count(*) FILTER (WHERE deleted=0) FROM mxx_finance_expense
ORDER BY 1;

-- 2. Lead → Customer 关联验证
\echo '========== 2. Lead → Customer 关联验证 =========='
SELECT 'lead_no_customer' as broken_chain, count(*) as cnt
FROM mxx_crm_lead l
WHERE l.deleted=0 AND l.status=3 AND l.converted_to_customer_id IS NULL;

SELECT 'lead_customer_invalid' as broken_chain, count(*) as cnt
FROM mxx_crm_lead l
LEFT JOIN mxx_crm_customer c ON c.id = l.converted_to_customer_id AND c.deleted=0
WHERE l.deleted=0 AND l.converted_to_customer_id IS NOT NULL AND c.id IS NULL;

-- 3. Customer → Contact 关联验证
\echo '========== 3. Customer → Contact 关联验证 =========='
SELECT 'customer_no_contact' as broken_chain, c.id as customer_id, c.company_name
FROM mxx_crm_customer c
LEFT JOIN mxx_crm_contact ct ON ct.customer_id = c.id AND ct.deleted=0
WHERE c.deleted=0 AND ct.id IS NULL;

-- 4. Opportunity → Lead/Customer 关联验证
\echo '========== 4. Opportunity → Lead/Customer 关联验证 =========='
SELECT 'opp_no_customer' as broken_chain, count(*) as cnt
FROM mxx_crm_opportunity o
WHERE o.deleted=0 AND o.customer_id IS NULL;

SELECT 'opp_customer_invalid' as broken_chain, count(*) as cnt
FROM mxx_crm_opportunity o
LEFT JOIN mxx_crm_customer c ON c.id = o.customer_id AND c.deleted=0
WHERE o.deleted=0 AND o.customer_id IS NOT NULL AND c.id IS NULL;

-- 5. Quotation → Opportunity/Customer 关联验证
\echo '========== 5. Quotation → Opportunity/Customer 关联验证 =========='
SELECT 'quo_no_opp' as broken_chain, q.id, q.quotation_no, q.customer_id
FROM mxx_sale_quotation q
WHERE q.deleted=0 AND (q.opportunity_id IS NULL OR q.opportunity_id = 0);

SELECT 'quo_opp_invalid' as broken_chain, count(*) as cnt
FROM mxx_sale_quotation q
LEFT JOIN mxx_crm_opportunity o ON o.id = q.opportunity_id AND o.deleted=0
WHERE q.deleted=0 AND q.opportunity_id IS NOT NULL AND q.opportunity_id > 0 AND o.id IS NULL;

-- 6. Order → Quotation/Opportunity 关联验证
\echo '========== 6. Order → Quotation/Opportunity 关联验证 =========='
SELECT 'order_no_quo' as broken_chain, o.id, o.order_no
FROM mxx_sale_order o
WHERE o.deleted=0 AND (o.quotation_id IS NULL OR o.quotation_id = 0);

SELECT 'order_quo_invalid' as broken_chain, count(*) as cnt
FROM mxx_sale_order o
LEFT JOIN mxx_sale_quotation q ON q.id = o.quotation_id AND q.deleted=0
WHERE o.deleted=0 AND o.quotation_id IS NOT NULL AND o.quotation_id > 0 AND q.id IS NULL;

SELECT 'order_no_opp' as broken_chain, count(*) as cnt
FROM mxx_sale_order o
WHERE o.deleted=0 AND (o.opportunity_id IS NULL OR o.opportunity_id = 0);

-- 7. Contract → Order/Opportunity/Customer 关联验证
\echo '========== 7. Contract → Order/Opportunity/Customer 关联验证 =========='
SELECT 'contract_no_order' as broken_chain, c.id, c.contract_no
FROM mxx_crm_contract c
WHERE c.deleted=0 AND (c.order_id IS NULL OR c.order_id = 0);

SELECT 'contract_order_invalid' as broken_chain, count(*) as cnt
FROM mxx_crm_contract c
LEFT JOIN mxx_sale_order o ON o.id = c.order_id AND o.deleted=0
WHERE c.deleted=0 AND c.order_id IS NOT NULL AND c.order_id > 0 AND o.id IS NULL;

SELECT 'contract_no_opp' as broken_chain, c.id, c.contract_no
FROM mxx_crm_contract c
WHERE c.deleted=0 AND (c.opportunity_id IS NULL OR c.opportunity_id = 0);

-- 8. Payment → Contract/Order 关联验证
\echo '========== 8. Payment → Contract/Order 关联验证 =========='
SELECT 'payment_no_contract' as broken_chain, count(*) as cnt
FROM mxx_sale_payment p
WHERE p.deleted=0 AND (p.contract_id IS NULL OR p.contract_id = 0);

SELECT 'payment_contract_invalid' as broken_chain, count(*) as cnt
FROM mxx_sale_payment p
LEFT JOIN mxx_crm_contract c ON c.id = p.contract_id AND c.deleted=0
WHERE p.deleted=0 AND p.contract_id IS NOT NULL AND p.contract_id > 0 AND c.id IS NULL;

-- 9. Invoice → Contract/Order 关联验证
\echo '========== 9. Invoice → Contract/Order 关联验证 =========='
SELECT 'invoice_no_contract' as broken_chain, count(*) as cnt
FROM mxx_sale_invoice i
WHERE i.deleted=0 AND (i.contract_id IS NULL OR i.contract_id = 0);

SELECT 'invoice_contract_invalid' as broken_chain, count(*) as cnt
FROM mxx_sale_invoice i
LEFT JOIN mxx_crm_contract c ON c.id = i.contract_id AND c.deleted=0
WHERE i.deleted=0 AND i.contract_id IS NOT NULL AND i.contract_id > 0 AND c.id IS NULL;

-- 10. Shipment → Order 关联验证
\echo '========== 10. Shipment → Order 关联验证 =========='
SELECT 'ship_no_order' as broken_chain, count(*) as cnt
FROM mxx_sale_shipment s
WHERE s.deleted=0 AND (s.order_id IS NULL OR s.order_id = 0);

SELECT 'ship_order_invalid' as broken_chain, count(*) as cnt
FROM mxx_sale_shipment s
LEFT JOIN mxx_sale_order o ON o.id = s.order_id AND o.deleted=0
WHERE s.deleted=0 AND s.order_id IS NOT NULL AND s.order_id > 0 AND o.id IS NULL;

-- 11. 完整链路验证：统计能完整走通全链路的记录数
\echo '========== 11. 完整链路验证（能打通全链路的合同数） =========='
SELECT 'full_chain_contracts' as metric, count(DISTINCT c.id) as cnt
FROM mxx_crm_contract c
JOIN mxx_sale_order o ON o.id = c.order_id AND o.deleted=0
JOIN mxx_sale_quotation q ON q.id = o.quotation_id AND q.deleted=0
JOIN mxx_crm_opportunity op ON op.id = q.opportunity_id AND op.deleted=0
JOIN mxx_crm_customer cu ON cu.id = c.customer_id AND cu.deleted=0
LEFT JOIN mxx_crm_lead l ON l.id = op.lead_id AND l.deleted=0
LEFT JOIN mxx_sale_payment p ON p.contract_id = c.id AND p.deleted=0
LEFT JOIN mxx_sale_invoice i ON i.contract_id = c.id AND i.deleted=0
LEFT JOIN mxx_sale_shipment s ON s.order_id = c.order_id AND s.deleted=0
WHERE c.deleted=0
  AND c.order_id IS NOT NULL AND c.order_id > 0
  AND c.opportunity_id IS NOT NULL AND c.opportunity_id > 0;

-- 12. 列出所有完整链路的合同详情
\echo '========== 12. 完整链路合同详情 =========='
SELECT
    c.id as contract_id, c.contract_no, c.name as contract_name, c.total_amount,
    cu.company_name as customer,
    op.id as opp_id, op.name as opp_name,
    q.id as quo_id, q.quotation_no,
    o.id as order_id, o.order_no,
    (SELECT count(*) FROM mxx_sale_payment p WHERE p.contract_id=c.id AND p.deleted=0) as payments,
    (SELECT count(*) FROM mxx_sale_invoice i WHERE i.contract_id=c.id AND i.deleted=0) as invoices,
    (SELECT count(*) FROM mxx_sale_shipment s WHERE s.order_id=c.order_id AND s.deleted=0) as shipments,
    (SELECT count(*) FROM mxx_sale_refund r WHERE r.order_id=c.order_id AND r.deleted=0) as refunds
FROM mxx_crm_contract c
JOIN mxx_crm_customer cu ON cu.id = c.customer_id AND cu.deleted=0
LEFT JOIN mxx_crm_opportunity op ON op.id = c.opportunity_id AND op.deleted=0
LEFT JOIN mxx_sale_order o ON o.id = c.order_id AND o.deleted=0
LEFT JOIN mxx_sale_quotation q ON q.id = o.quotation_id AND q.deleted=0
WHERE c.deleted=0
ORDER BY c.id;
