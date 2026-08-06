-- 采购模块完整测试数据（修正版 v2）
-- 关联：供应商→申请→订单→收货→退货
-- 注意：admin表ID从5开始，created_by必须用存在的用户ID

-- ============================================================
-- 1. 补充3个供应商（industry为枚举类型）
-- ============================================================
INSERT INTO mxx_purchase_supplier (supplier_no, name, short_name, contact_person, phone, email, country, region, address, industry, level, currency, credit_limit, credit_days, bank_name, bank_account, tax_id, status, payment_terms, delivery_terms, notes, is_active, create_time, deleted)
VALUES
  ('SUP000004', '上海精密机械制造有限公司', '上海精密', '王建国', '138-0000-1001', 'wang@shjingmi.cn', '中国', '上海市', '上海市浦东新区张江高科技园区龙东大道300号', 'manufacturer', 2, 1, 500000.00, 45, '工商银行张江支行', '1001234567890123456', '91310000MA1FL33XXX', 1, '月结45天', '送货上门', '长期合作供应商', true, NOW(), 0),
  ('SUP000005', '苏州电子信息科技公司', '苏州电子', '李明华', '139-0000-1002', 'li@szelec.cn', '中国', '江苏省', '苏州市工业园区星湖街328号', 'manufacturer', 1, 1, 800000.00, 30, '建设银行苏州分行', '3200150123456789012', '91320500MA2XYZ123', 1, '月结30天', '快递发货', '核心电子元件供应商', true, NOW(), 0),
  ('SUP000006', '浙江包装材料有限公司', '浙江包装', '张小芳', '137-0000-1003', 'zhang@zjbz.cn', '中国', '浙江省', '杭州市余杭区仓前街道海创科技中心', 'wholesale', 3, 1, 200000.00, 15, '农业银行余杭支行', '1901234567890123456', '91330110MA3PACK567', 1, '货到付款', '物流配送', '包装材料定期采购', true, NOW(), 0);

-- ============================================================
-- 2. 采购申请（3条，不同状态）
-- urgency是int类型(1低/2中/3高), status是int(0草稿/1待审/2已批/3已驳回)
-- ============================================================
INSERT INTO mxx_purchase_requisition (pr_no, pr_type, title, department_id, requester_id, expected_date, urgency, total_amount, currency, status, reason, remark, created_by, create_time, deleted)
VALUES
  ('PR20260806001', 'routine', '8月电子元器件采购申请', 103, 1, '2026-08-15', 2, 158000.00, 'CNY', 2, '产线扩产所需电子元器件', '按季度备货计划申请', 5, NOW(), 0),
  ('PR20260806002', 'urgent', '紧急机械配件采购申请', 100, 1, '2026-08-12', 3, 86500.00, 'CNY', 1, '设备突发故障需紧急采购配件', '生产线停机等待维修', 5, NOW(), 0),
  ('PR20260806003', 'routine', '包装材料批量采购申请', 101, 1, '2026-08-20', 1, 32400.00, 'CNY', 3, '季度包装材料常规备货', '含礼盒及快递包装', 5, NOW(), 0);

-- 采购申请明细（字段: estimated_price, estimated_amount）
INSERT INTO mxx_purchase_requisition_item (pr_id, product_id, product_name, product_sku, spec, unit, quantity, estimated_price, estimated_amount, remark, create_time, deleted)
VALUES
  (1, 1, '智能手机 X1', 'PRD000001', '标准版', '台', 50, 2200.00, 110000.00, '主力产品备货', NOW(), 0),
  (1, 2, '笔记本电脑 Pro', 'PRD000002', '16G/512G', '台', 20, 2400.00, 48000.00, '高端型号补货', NOW(), 0),
  (2, 5, '不粘锅套装', 'PRD000005', '3件套', '套', 100, 86.50, 8650.00, '配件紧急采购', NOW(), 0),
  (3, 3, '运动T恤', 'PRD000003', 'M码黑色', '件', 500, 32.00, 16000.00, '秋季新品包装', NOW(), 0),
  (3, 4, '休闲运动鞋', 'PRD000004', '42码', '双', 200, 82.00, 16400.00, '季度备货', NOW(), 0);

-- ============================================================
-- 3. 采购订单（4条）
-- currency是int(1=CNY), status是枚举, payment_status是枚举
-- ============================================================
INSERT INTO mxx_purchase_po (purchase_no, supplier_id, purchase_date, expected_date, amount, currency, status, payment_status, notes, pr_id, pr_no, department_id, buyer_id, total_quantity, tax_total, discount_amount, freight_amount, delivery_address, delivery_terms, payment_terms, created_by, create_time, deleted)
VALUES
  ('PO20260806001', 4, '2026-08-06', '2026-08-15', 110000.00, 1, 'ordered', 'unpaid', '电子元器件批量采购', 1, 'PR20260806001', 103, 1, 50, 13000.00, 0, 500.00, '上海市浦东新区张江高科技园区', '送货上门', '月结45天', 5, NOW(), 0),
  ('PO20260806002', 5, '2026-08-06', '2026-08-12', 48000.00, 1, 'received', 'unpaid', '笔记本电脑补货', 1, 'PR20260806001', 103, 1, 20, 5760.00, 0, 300.00, '上海市浦东新区张江高科技园区', '快递发货', '月结30天', 5, NOW(), 0),
  ('PO20260806003', 4, '2026-08-06', '2026-08-12', 8650.00, 1, 'draft', 'unpaid', '紧急机械配件采购', 2, 'PR20260806002', 100, 1, 100, 1018.50, 0, 200.00, '上海市浦东新区张江高科技园区', '送货上门', '月结45天', 5, NOW(), 0),
  ('PO20260806004', 6, '2026-08-06', '2026-08-20', 32400.00, 1, 'completed', 'paid', '包装材料季度采购已完结', 3, 'PR20260806003', 101, 1, 700, 3840.00, 0, 800.00, '杭州市余杭区仓前街道', '物流配送', '货到付款', 5, NOW(), 0);

-- 采购订单明细
INSERT INTO mxx_purchase_po_item (po_id, product_id, product_name, product_sku, spec, unit, quantity, received_quantity, unit_price, amount, tax_rate, tax_amount, expected_date, remark, create_time, deleted)
VALUES
  (1, 1, '智能手机 X1', 'PRD000001', '标准版', '台', 50, 0, 2200.00, 110000.00, 13.00, 13000.00, '2026-08-15', '主力产品', NOW(), 0),
  (2, 2, '笔记本电脑 Pro', 'PRD000002', '16G/512G', '台', 20, 20, 2400.00, 48000.00, 12.00, 5760.00, '2026-08-12', '高端型号', NOW(), 0),
  (3, 5, '不粘锅套装', 'PRD000005', '3件套', '套', 100, 0, 86.50, 8650.00, 11.78, 1018.50, '2026-08-12', '紧急配件', NOW(), 0),
  (4, 3, '运动T恤', 'PRD000003', 'M码黑色', '件', 500, 500, 32.00, 16000.00, 13.00, 2080.00, '2026-08-20', '秋季新品', NOW(), 0),
  (4, 4, '休闲运动鞋', 'PRD000004', '42码', '双', 200, 200, 82.00, 16400.00, 13.00, 1760.00, '2026-08-20', '季度备货', NOW(), 0);

-- ============================================================
-- 4. 采购收货（2条）
-- receipt_item表只有: receipt_id, po_item_id, product_id, quantity, remark
-- ============================================================
INSERT INTO mxx_purchase_receipt (receipt_no, po_id, po_no, supplier_id, warehouse_id, status, total_quantity, remark, created_by, create_time, deleted)
VALUES
  ('RC20260806001', 2, 'PO20260806002', 5, 1, 2, 20, '笔记本电脑20台全部到货验收合格', 5, NOW(), 0),
  ('RC20260806002', 4, 'PO20260806004', 6, 1, 2, 700, '包装材料全部到货', 5, NOW(), 0);

INSERT INTO mxx_purchase_receipt_item (receipt_id, po_item_id, product_id, quantity, remark, create_time, deleted)
VALUES
  (1, 2, 2, 20, '全部验收合格', NOW(), 0),
  (2, 4, 3, 500, '质量合格', NOW(), 0),
  (2, 5, 4, 200, '质量合格', NOW(), 0);

-- ============================================================
-- 5. 采购退货（1条）
-- return_item表: return_id, po_item_id, product_id, product_name, product_sku, unit, return_quantity, unit_price, amount, reason
-- ============================================================
INSERT INTO mxx_purchase_return (return_no, receipt_id, po_id, supplier_id, return_date, total_amount, reason, status, remark, created_by, create_time, deleted)
VALUES
  ('RT20260806001', 2, 4, 6, '2026-08-08', 1640.00, '部分休闲运动鞋尺码偏差客户拒收', 2, '退回运动鞋20双供应商已确认补发', 5, NOW(), 0);

INSERT INTO mxx_purchase_return_item (return_id, po_item_id, product_id, product_name, product_sku, unit, return_quantity, unit_price, amount, reason, deleted)
VALUES
  (1, 5, 4, '休闲运动鞋', 'PRD000004', '双', 20, 82.00, 1640.00, '尺码偏差客户拒收', 0);

-- ============================================================
-- 验证
-- ============================================================
SELECT '--- suppliers ---' as info;
SELECT id, supplier_no, name FROM mxx_purchase_supplier WHERE deleted = 0 ORDER BY id;
SELECT '--- requisitions ---' as info;
SELECT id, pr_no, title, status FROM mxx_purchase_requisition WHERE deleted = 0 ORDER BY id;
SELECT '--- purchase orders ---' as info;
SELECT id, purchase_no, supplier_id, amount, status, payment_status FROM mxx_purchase_po WHERE deleted = 0 ORDER BY id;
SELECT '--- receipts ---' as info;
SELECT id, receipt_no, po_no, status FROM mxx_purchase_receipt WHERE deleted = 0 ORDER BY id;
SELECT '--- returns ---' as info;
SELECT id, return_no, po_id, total_amount, status FROM mxx_purchase_return WHERE deleted = 0 ORDER BY id;

