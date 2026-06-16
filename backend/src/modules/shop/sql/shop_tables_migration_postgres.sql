-- =============================================
-- Shop Module Database Migration Script
-- PostgreSQL Version
-- Date: 2026-06-14
-- =============================================

-- 1. mxx_shop 表 - 修改布尔字段为 SMALLINT
ALTER TABLE mxx_shop ALTER COLUMN self_operated TYPE SMALLINT USING CASE WHEN self_operated THEN 1 ELSE 0 END;
ALTER TABLE mxx_shop ALTER COLUMN store_disable TYPE SMALLINT USING CASE WHEN store_disable THEN 1 ELSE 0 END;
ALTER TABLE mxx_shop ALTER COLUMN page_show TYPE SMALLINT USING CASE WHEN page_show THEN 1 ELSE 0 END;
ALTER TABLE mxx_shop ALTER COLUMN self_pick_flag TYPE SMALLINT USING CASE WHEN self_pick_flag THEN 1 ELSE 0 END;
ALTER TABLE mxx_shop ALTER COLUMN delete_flag TYPE SMALLINT USING CASE WHEN delete_flag THEN 1 ELSE 0 END;

-- 添加字段注释
COMMENT ON COLUMN mxx_shop.id IS '主键ID';
COMMENT ON COLUMN mxx_shop.store_logo IS '店铺LOGO';
COMMENT ON COLUMN mxx_shop.store_name IS '店铺名称';
COMMENT ON COLUMN mxx_shop.self_operated IS '是否自营: 0=否, 1=是';
COMMENT ON COLUMN mxx_shop.store_address_detail IS '店铺详细地址';
COMMENT ON COLUMN mxx_shop.store_address_id_path IS '店铺地址ID路径';
COMMENT ON COLUMN mxx_shop.store_address_path IS '店铺地址路径';
COMMENT ON COLUMN mxx_shop.store_end_time IS '店铺营业结束时间';
COMMENT ON COLUMN mxx_shop.store_disable IS '店铺状态: 0=正常, 1=关闭';
COMMENT ON COLUMN mxx_shop.store_center IS '店铺中心位置';
COMMENT ON COLUMN mxx_shop.store_desc IS '店铺简介';
COMMENT ON COLUMN mxx_shop.delivery_score IS '配送评分';
COMMENT ON COLUMN mxx_shop.description_score IS '描述评分';
COMMENT ON COLUMN mxx_shop.service_score IS '服务评分';
COMMENT ON COLUMN mxx_shop.goods_num IS '商品数量';
COMMENT ON COLUMN mxx_shop.collection_num IS '收藏数量';
COMMENT ON COLUMN mxx_shop.yzf_mp_sign IS '银豹小程序签名';
COMMENT ON COLUMN mxx_shop.yzf_sign IS '银豹签名';
COMMENT ON COLUMN mxx_shop.merchant_euid IS '商家EUID';
COMMENT ON COLUMN mxx_shop.page_show IS '是否页面展示: 0=否, 1=是';
COMMENT ON COLUMN mxx_shop.self_pick_flag IS '是否支持自提: 0=否, 1=是';
COMMENT ON COLUMN mxx_shop.create_by IS '创建人';
COMMENT ON COLUMN mxx_shop.create_time IS '创建时间';
COMMENT ON COLUMN mxx_shop.update_by IS '更新人';
COMMENT ON COLUMN mxx_shop.update_time IS '更新时间';
COMMENT ON COLUMN mxx_shop.delete_flag IS '删除标志: 0=未删除, 1=已删除';
COMMENT ON TABLE mxx_shop IS '店铺/供货商表';

-- 2. mxx_shop_category 表
ALTER TABLE mxx_shop_category ALTER COLUMN is_show TYPE SMALLINT USING CASE WHEN is_show THEN 1 ELSE 0 END;

-- 添加字段注释
COMMENT ON COLUMN mxx_shop_category.id IS '主键ID';
COMMENT ON COLUMN mxx_shop_category.parent_id IS '上级分类ID, 0=顶级';
COMMENT ON COLUMN mxx_shop_category.name IS '分类名称';
COMMENT ON COLUMN mxx_shop_category.icon IS '分类图标';
COMMENT ON COLUMN mxx_shop_category.sort_order IS '排序值(升序)';
COMMENT ON COLUMN mxx_shop_category.is_show IS '是否显示: 0=隐藏, 1=显示';
COMMENT ON COLUMN mxx_shop_category.level IS '层级: 1/2/3';
COMMENT ON COLUMN mxx_shop_category.create_time IS '创建时间';
COMMENT ON COLUMN mxx_shop_category.update_time IS '更新时间';
COMMENT ON TABLE mxx_shop_category IS '商品分类表';

-- 3. mxx_shop_spu 表
ALTER TABLE mxx_shop_spu ALTER COLUMN is_commission TYPE SMALLINT USING CASE WHEN is_commission THEN 1 ELSE 0 END;

-- 添加字段注释
COMMENT ON COLUMN mxx_shop_spu.id IS '主键ID';
COMMENT ON COLUMN mxx_shop_spu.shop_id IS '所属店铺ID';
COMMENT ON COLUMN mxx_shop_spu.category_id IS '分类ID';
COMMENT ON COLUMN mxx_shop_spu.title IS '商品标题';
COMMENT ON COLUMN mxx_shop_spu.subtitle IS '副标题/卖点';
COMMENT ON COLUMN mxx_shop_spu.primary_image IS '主图';
COMMENT ON COLUMN mxx_shop_spu.images IS '商品轮播图列表(JSON)';
COMMENT ON COLUMN mxx_shop_spu.video IS '商品视频';
COMMENT ON COLUMN mxx_shop_spu.desc_content IS '商品详情(富文本/图片列表JSON)';
COMMENT ON COLUMN mxx_shop_spu.is_commission IS '是否参与佣金: 0=否, 1=是';
COMMENT ON COLUMN mxx_shop_spu.status IS '状态: 0=待审核, 1=已上架, 2=下架, 3=审核驳回';
COMMENT ON COLUMN mxx_shop_spu.stock_total IS '总库存';
COMMENT ON COLUMN mxx_shop_spu.sold_num IS '已售数量';
COMMENT ON COLUMN mxx_shop_spu.min_sale_price IS '最低销售价(分)';
COMMENT ON COLUMN mxx_shop_spu.max_sale_price IS '最高销售价(分)';
COMMENT ON COLUMN mxx_shop_spu.min_line_price IS '最低划线价(分)';
COMMENT ON COLUMN mxx_shop_spu.max_line_price IS '最高划线价(分)';
COMMENT ON COLUMN mxx_shop_spu.audit_remark IS '审核备注';
COMMENT ON COLUMN mxx_shop_spu.freight_template_id IS '运费模板ID';
COMMENT ON COLUMN mxx_shop_spu.create_time IS '创建时间';
COMMENT ON COLUMN mxx_shop_spu.update_time IS '更新时间';
COMMENT ON TABLE mxx_shop_spu IS '商品SPU表';

-- 4. mxx_shop_sku 表 - 添加字段注释
COMMENT ON COLUMN mxx_shop_sku.id IS '主键ID';
COMMENT ON COLUMN mxx_shop_sku.spu_id IS '所属SPU ID';
COMMENT ON COLUMN mxx_shop_sku.spec_desc IS '规格描述(如"颜色:白色 尺码:M")';
COMMENT ON COLUMN mxx_shop_sku.sku_code IS 'SKU编码';
COMMENT ON COLUMN mxx_shop_sku.image IS 'SKU图片';
COMMENT ON COLUMN mxx_shop_sku.base_price IS '供货底价(分)';
COMMENT ON COLUMN mxx_shop_sku.retail_price IS '零售价(分)';
COMMENT ON COLUMN mxx_shop_sku.sale_price IS '实际售价(分)';
COMMENT ON COLUMN mxx_shop_sku.line_price IS '划线价(分)';
COMMENT ON COLUMN mxx_shop_sku.stock IS '库存';
COMMENT ON COLUMN mxx_shop_sku.safe_stock IS '安全库存';
COMMENT ON COLUMN mxx_shop_sku.sold_num IS '已售数量';
COMMENT ON COLUMN mxx_shop_sku.weight IS '重量(kg)';
COMMENT ON COLUMN mxx_shop_sku.create_time IS '创建时间';
COMMENT ON COLUMN mxx_shop_sku.update_time IS '更新时间';
COMMENT ON TABLE mxx_shop_sku IS '商品SKU表';

-- 5. mxx_shop_cart 表
ALTER TABLE mxx_shop_cart ALTER COLUMN selected TYPE SMALLINT USING CASE WHEN selected THEN 1 ELSE 0 END;

-- 添加字段注释
COMMENT ON COLUMN mxx_shop_cart.id IS '主键ID';
COMMENT ON COLUMN mxx_shop_cart.user_id IS '买家用户ID';
COMMENT ON COLUMN mxx_shop_cart.shop_id IS '店铺ID';
COMMENT ON COLUMN mxx_shop_cart.spu_id IS '商品ID';
COMMENT ON COLUMN mxx_shop_cart.sku_id IS 'SKU ID';
COMMENT ON COLUMN mxx_shop_cart.quantity IS '数量';
COMMENT ON COLUMN mxx_shop_cart.selected IS '是否选中: 0=否, 1=是';
COMMENT ON COLUMN mxx_shop_cart.create_time IS '创建时间';
COMMENT ON COLUMN mxx_shop_cart.update_time IS '更新时间';
COMMENT ON TABLE mxx_shop_cart IS '购物车表';

-- 6. mxx_shop_order 表 - 添加字段注释
COMMENT ON COLUMN mxx_shop_order.id IS '主键ID';
COMMENT ON COLUMN mxx_shop_order.order_no IS '订单编号';
COMMENT ON COLUMN mxx_shop_order.user_id IS '买家用户ID';
COMMENT ON COLUMN mxx_shop_order.shop_id IS '店铺ID';
COMMENT ON COLUMN mxx_shop_order.total_amount IS '订单总金额(分)';
COMMENT ON COLUMN mxx_shop_order.freight_amount IS '运费(分)';
COMMENT ON COLUMN mxx_shop_order.commission_amount IS '平台佣金(分)';
COMMENT ON COLUMN mxx_shop_order.commission_rate IS '佣金比例';
COMMENT ON COLUMN mxx_shop_order.settlement_amount IS '供货商结算金额(分)';
COMMENT ON COLUMN mxx_shop_order.refund_amount IS '退款金额(分)';
COMMENT ON COLUMN mxx_shop_order.goods_count IS '商品数量';
COMMENT ON COLUMN mxx_shop_order.status IS '状态: 0=待付款, 1=待发货, 2=已发货, 3=已签收, 4=已完成, 5=已取消, 6=退款中, 7=已退款';
COMMENT ON COLUMN mxx_shop_order.receiver_name IS '收货人';
COMMENT ON COLUMN mxx_shop_order.receiver_phone IS '收货电话';
COMMENT ON COLUMN mxx_shop_order.receiver_address IS '收货地址';
COMMENT ON COLUMN mxx_shop_order.buyer_remark IS '买家留言';
COMMENT ON COLUMN mxx_shop_order.cancel_reason IS '取消原因';
COMMENT ON COLUMN mxx_shop_order.pay_method IS '支付方式';
COMMENT ON COLUMN mxx_shop_order.pay_time IS '支付时间';
COMMENT ON COLUMN mxx_shop_order.delivery_time IS '发货时间';
COMMENT ON COLUMN mxx_shop_order.receive_time IS '签收/确认收货时间';
COMMENT ON COLUMN mxx_shop_order.finish_time IS '完成(可评价)时间';
COMMENT ON COLUMN mxx_shop_order.cancel_time IS '取消时间';
COMMENT ON COLUMN mxx_shop_order.create_time IS '创建时间';
COMMENT ON COLUMN mxx_shop_order.update_time IS '更新时间';
COMMENT ON TABLE mxx_shop_order IS '订单表';

-- 7. mxx_shop_order_item 表 - 添加字段注释
COMMENT ON COLUMN mxx_shop_order_item.id IS '主键ID';
COMMENT ON COLUMN mxx_shop_order_item.order_id IS '订单ID';
COMMENT ON COLUMN mxx_shop_order_item.spu_id IS '商品ID';
COMMENT ON COLUMN mxx_shop_order_item.sku_id IS 'SKU ID';
COMMENT ON COLUMN mxx_shop_order_item.goods_title IS '商品标题(快照)';
COMMENT ON COLUMN mxx_shop_order_item.goods_image IS '商品图片(快照)';
COMMENT ON COLUMN mxx_shop_order_item.spec_desc IS '规格描述(如"颜色:白色 尺码:M")';
COMMENT ON COLUMN mxx_shop_order_item.price IS '成交单价(分)';
COMMENT ON COLUMN mxx_shop_order_item.quantity IS '数量';
COMMENT ON COLUMN mxx_shop_order_item.base_price IS '供货底价(分)';
COMMENT ON COLUMN mxx_shop_order_item.commission_amount IS '该SKU佣金(分)';
COMMENT ON COLUMN mxx_shop_order_item.settlement_amount IS '该SKU结算金额(分)';
COMMENT ON COLUMN mxx_shop_order_item.create_time IS '创建时间';
COMMENT ON TABLE mxx_shop_order_item IS '订单明细表';

-- 8. mxx_shop_review 表
ALTER TABLE mxx_shop_review ALTER COLUMN is_anonymous TYPE SMALLINT USING CASE WHEN is_anonymous THEN 1 ELSE 0 END;

-- 添加字段注释
COMMENT ON COLUMN mxx_shop_review.id IS '主键ID';
COMMENT ON COLUMN mxx_shop_review.order_id IS '订单ID';
COMMENT ON COLUMN mxx_shop_review.spu_id IS '商品ID';
COMMENT ON COLUMN mxx_shop_review.sku_id IS 'SKU ID';
COMMENT ON COLUMN mxx_shop_review.user_id IS '买家用户ID';
COMMENT ON COLUMN mxx_shop_review.shop_id IS '店铺ID';
COMMENT ON COLUMN mxx_shop_review.score IS '评分: 1~5';
COMMENT ON COLUMN mxx_shop_review.content IS '评价内容';
COMMENT ON COLUMN mxx_shop_review.images IS '评价图片列表(JSON)';
COMMENT ON COLUMN mxx_shop_review.is_anonymous IS '是否匿名: 0=否, 1=是';
COMMENT ON COLUMN mxx_shop_review.reply_content IS '供货商回复';
COMMENT ON COLUMN mxx_shop_review.reply_time IS '回复时间';
COMMENT ON COLUMN mxx_shop_review.create_time IS '创建时间';
COMMENT ON COLUMN mxx_shop_review.update_time IS '更新时间';
COMMENT ON TABLE mxx_shop_review IS '商品评价表';

-- 9. mxx_shop_supplier_apply 表 - 添加字段注释
COMMENT ON COLUMN mxx_shop_supplier_apply.id IS '主键ID';
COMMENT ON COLUMN mxx_shop_supplier_apply.user_id IS '申请用户ID';
COMMENT ON COLUMN mxx_shop_supplier_apply.shop_name IS '店铺名称';
COMMENT ON COLUMN mxx_shop_supplier_apply.contact_name IS '联系人';
COMMENT ON COLUMN mxx_shop_supplier_apply.contact_phone IS '联系电话';
COMMENT ON COLUMN mxx_shop_supplier_apply.shop_logo IS '店铺LOGO';
COMMENT ON COLUMN mxx_shop_supplier_apply.shop_desc IS '店铺简介';
COMMENT ON COLUMN mxx_shop_supplier_apply.business_license IS '营业执照图片';
COMMENT ON COLUMN mxx_shop_supplier_apply.status IS '状态: 0=待审核, 1=通过, 2=驳回';
COMMENT ON COLUMN mxx_shop_supplier_apply.audit_remark IS '审核备注';
COMMENT ON COLUMN mxx_shop_supplier_apply.audit_time IS '审核时间';
COMMENT ON COLUMN mxx_shop_supplier_apply.create_time IS '创建时间';
COMMENT ON COLUMN mxx_shop_supplier_apply.update_time IS '更新时间';
COMMENT ON TABLE mxx_shop_supplier_apply IS '供货商入驻申请表';

-- 10. mxx_shop_spec 表 - 添加字段注释
COMMENT ON COLUMN mxx_shop_spec.id IS '主键ID';
COMMENT ON COLUMN mxx_shop_spec.spu_id IS '所属SPU ID';
COMMENT ON COLUMN mxx_shop_spec.spec_name IS '规格名称(如颜色、尺寸)';
COMMENT ON COLUMN mxx_shop_spec.sort_order IS '排序值';
COMMENT ON TABLE mxx_shop_spec IS '规格定义表';

-- 11. mxx_shop_spec_value 表 - 添加字段注释
COMMENT ON COLUMN mxx_shop_spec_value.id IS '主键ID';
COMMENT ON COLUMN mxx_shop_spec_value.spec_id IS '所属规格ID';
COMMENT ON COLUMN mxx_shop_spec_value.spec_value IS '规格值(如红色、XL)';
COMMENT ON COLUMN mxx_shop_spec_value.sort_order IS '排序值';
COMMENT ON TABLE mxx_shop_spec_value IS '规格值表';

-- 12. mxx_shop_promotion 表 - 添加字段注释
COMMENT ON COLUMN mxx_shop_promotion.id IS '主键ID';
COMMENT ON COLUMN mxx_shop_promotion.shop_id IS '店铺ID(0=平台活动)';
COMMENT ON COLUMN mxx_shop_promotion.title IS '活动标题';
COMMENT ON COLUMN mxx_shop_promotion.promotion_type IS '类型: 1=满减, 2=折扣, 3=限时抢购, 4=新人专享';
COMMENT ON COLUMN mxx_shop_promotion.discount_value IS '优惠值(满减金额/折扣率/特价价格)';
COMMENT ON COLUMN mxx_shop_promotion.condition_value IS '条件值(满减条件金额)';
COMMENT ON COLUMN mxx_shop_promotion.start_time IS '开始时间';
COMMENT ON COLUMN mxx_shop_promotion.end_time IS '结束时间';
COMMENT ON COLUMN mxx_shop_promotion.status IS '状态: 0=未开始, 1=进行中, 2=已结束, 3=已关闭';
COMMENT ON COLUMN mxx_shop_promotion.create_time IS '创建时间';
COMMENT ON COLUMN mxx_shop_promotion.update_time IS '更新时间';
COMMENT ON TABLE mxx_shop_promotion IS '促销活动表';

-- 13. mxx_shop_promotion_spu 表 - 添加字段注释
COMMENT ON COLUMN mxx_shop_promotion_spu.id IS '主键ID';
COMMENT ON COLUMN mxx_shop_promotion_spu.promotion_id IS '活动ID';
COMMENT ON COLUMN mxx_shop_promotion_spu.spu_id IS '商品ID';
COMMENT ON COLUMN mxx_shop_promotion_spu.sku_id IS 'SKU ID(可选)';
COMMENT ON COLUMN mxx_shop_promotion_spu.create_time IS '创建时间';
COMMENT ON TABLE mxx_shop_promotion_spu IS '促销商品关联表';

-- 14. mxx_shop_settlement 表 - 添加字段注释
COMMENT ON COLUMN mxx_shop_settlement.id IS '主键ID';
COMMENT ON COLUMN mxx_shop_settlement.shop_id IS '店铺ID';
COMMENT ON COLUMN mxx_shop_settlement.settlement_no IS '结算单号';
COMMENT ON COLUMN mxx_shop_settlement.period_start IS '结算周期开始';
COMMENT ON COLUMN mxx_shop_settlement.period_end IS '结算周期结束';
COMMENT ON COLUMN mxx_shop_settlement.order_count IS '订单数量';
COMMENT ON COLUMN mxx_shop_settlement.total_amount IS '总交易额(分)';
COMMENT ON COLUMN mxx_shop_settlement.commission_amount IS '总佣金(分)';
COMMENT ON COLUMN mxx_shop_settlement.settlement_amount IS '结算金额(分)';
COMMENT ON COLUMN mxx_shop_settlement.settlement_status IS '状态: 0=待结算, 1=已结算, 2=已付款';
COMMENT ON COLUMN mxx_shop_settlement.settle_time IS '付款时间';
COMMENT ON COLUMN mxx_shop_settlement.remark IS '备注';
COMMENT ON COLUMN mxx_shop_settlement.create_time IS '创建时间';
COMMENT ON COLUMN mxx_shop_settlement.update_time IS '更新时间';
COMMENT ON TABLE mxx_shop_settlement IS '结算记录表';

-- 15. mxx_shop_delivery 表 - 添加字段注释
COMMENT ON COLUMN mxx_shop_delivery.id IS '主键ID';
COMMENT ON COLUMN mxx_shop_delivery.order_id IS '订单ID';
COMMENT ON COLUMN mxx_shop_delivery.logistics_company IS '快递公司';
COMMENT ON COLUMN mxx_shop_delivery.logistics_no IS '快递单号';
COMMENT ON COLUMN mxx_shop_delivery.receiver_name IS '收货人';
COMMENT ON COLUMN mxx_shop_delivery.receiver_phone IS '收货电话';
COMMENT ON COLUMN mxx_shop_delivery.receiver_address IS '收货地址';
COMMENT ON COLUMN mxx_shop_delivery.delivery_remark IS '发货备注';
COMMENT ON COLUMN mxx_shop_delivery.delivery_status IS '物流状态: 0=待揽收, 1=运输中, 2=已签收';
COMMENT ON COLUMN mxx_shop_delivery.create_time IS '创建时间';
COMMENT ON COLUMN mxx_shop_delivery.update_time IS '更新时间';
COMMENT ON TABLE mxx_shop_delivery IS '发货物流表';

-- 16. mxx_shop_refund 表 - 添加字段注释
COMMENT ON COLUMN mxx_shop_refund.id IS '主键ID';
COMMENT ON COLUMN mxx_shop_refund.order_id IS '订单ID';
COMMENT ON COLUMN mxx_shop_refund.order_item_id IS '订单明细ID';
COMMENT ON COLUMN mxx_shop_refund.refund_no IS '退款单号';
COMMENT ON COLUMN mxx_shop_refund.user_id IS '买家用户ID';
COMMENT ON COLUMN mxx_shop_refund.shop_id IS '店铺ID';
COMMENT ON COLUMN mxx_shop_refund.refund_type IS '退款类型: 1=退货退款, 2=仅退款';
COMMENT ON COLUMN mxx_shop_refund.refund_reason IS '退款原因';
COMMENT ON COLUMN mxx_shop_refund.refund_amount IS '退款金额(分)';
COMMENT ON COLUMN mxx_shop_refund.refund_status IS '退款状态: 0=待审核, 1=已同意, 2=已驳回, 3=退款中, 4=已退款';
COMMENT ON COLUMN mxx_shop_refund.audit_remark IS '审核备注';
COMMENT ON COLUMN mxx_shop_refund.audit_time IS '审核时间';
COMMENT ON COLUMN mxx_shop_refund.reject_reason IS '驳回原因';
COMMENT ON COLUMN mxx_shop_refund.create_time IS '创建时间';
COMMENT ON COLUMN mxx_shop_refund.update_time IS '更新时间';
COMMENT ON TABLE mxx_shop_refund IS '退款申请表';

-- 17. mxx_shop_user_merge 表 - 添加字段注释
COMMENT ON COLUMN mxx_shop_user_merge.id IS '主键ID';
COMMENT ON COLUMN mxx_shop_user_merge.shop_id IS '店铺ID';
COMMENT ON COLUMN mxx_shop_user_merge.user_id IS '用户ID';
COMMENT ON COLUMN mxx_shop_user_merge.is_admin IS '是否管理员: 0=否, 1=是';
COMMENT ON COLUMN mxx_shop_user_merge.create_time IS '创建时间';
COMMENT ON TABLE mxx_shop_user_merge IS '店铺与用户关联表';

-- 创建缺失的索引
CREATE INDEX IF NOT EXISTS idx_shop_store_name ON mxx_shop(store_name);
CREATE INDEX IF NOT EXISTS idx_shop_self_operated ON mxx_shop(self_operated);
CREATE INDEX IF NOT EXISTS idx_shop_store_disable ON mxx_shop(store_disable);

CREATE INDEX IF NOT EXISTS idx_category_parent_id ON mxx_shop_category(parent_id);
CREATE INDEX IF NOT EXISTS idx_category_level ON mxx_shop_category(level);

CREATE INDEX IF NOT EXISTS idx_spu_shop_id ON mxx_shop_spu(shop_id);
CREATE INDEX IF NOT EXISTS idx_spu_category_id ON mxx_shop_spu(category_id);
CREATE INDEX IF NOT EXISTS idx_spu_status ON mxx_shop_spu(status);

CREATE INDEX IF NOT EXISTS idx_sku_spu_id ON mxx_shop_sku(spu_id);

CREATE INDEX IF NOT EXISTS idx_cart_user_id ON mxx_shop_cart(user_id);
CREATE INDEX IF NOT EXISTS idx_cart_sku_id ON mxx_shop_cart(sku_id);

CREATE INDEX IF NOT EXISTS idx_order_order_no ON mxx_shop_order(order_no);
CREATE INDEX IF NOT EXISTS idx_order_user_id ON mxx_shop_order(user_id);
CREATE INDEX IF NOT EXISTS idx_order_shop_id ON mxx_shop_order(shop_id);
CREATE INDEX IF NOT EXISTS idx_order_status ON mxx_shop_order(status);

CREATE INDEX IF NOT EXISTS idx_order_item_order_id ON mxx_shop_order_item(order_id);
CREATE INDEX IF NOT EXISTS idx_order_item_spu_id ON mxx_shop_order_item(spu_id);

CREATE INDEX IF NOT EXISTS idx_review_spu_id ON mxx_shop_review(spu_id);
CREATE INDEX IF NOT EXISTS idx_review_order_id ON mxx_shop_review(order_id);
CREATE INDEX IF NOT EXISTS idx_review_user_id ON mxx_shop_review(user_id);

CREATE INDEX IF NOT EXISTS idx_supplier_apply_user_id ON mxx_shop_supplier_apply(user_id);
CREATE INDEX IF NOT EXISTS idx_supplier_apply_status ON mxx_shop_supplier_apply(status);

CREATE INDEX IF NOT EXISTS idx_spec_spu_id ON mxx_shop_spec(spu_id);
CREATE INDEX IF NOT EXISTS idx_spec_value_spec_id ON mxx_shop_spec_value(spec_id);

CREATE INDEX IF NOT EXISTS idx_promotion_shop_id ON mxx_shop_promotion(shop_id);
CREATE INDEX IF NOT EXISTS idx_promotion_status ON mxx_shop_promotion(status);

CREATE INDEX IF NOT EXISTS idx_promotion_spu_promotion_id ON mxx_shop_promotion_spu(promotion_id);
CREATE INDEX IF NOT EXISTS idx_promotion_spu_spu_id ON mxx_shop_promotion_spu(spu_id);

CREATE INDEX IF NOT EXISTS idx_settlement_shop_id ON mxx_shop_settlement(shop_id);
CREATE INDEX IF NOT EXISTS idx_settlement_status ON mxx_shop_settlement(settlement_status);

CREATE INDEX IF NOT EXISTS idx_delivery_order_id ON mxx_shop_delivery(order_id);

CREATE INDEX IF NOT EXISTS idx_refund_order_id ON mxx_shop_refund(order_id);
CREATE INDEX IF NOT EXISTS idx_refund_user_id ON mxx_shop_refund(user_id);
CREATE INDEX IF NOT EXISTS idx_refund_status ON mxx_shop_refund(refund_status);

CREATE INDEX IF NOT EXISTS idx_shop_user_merge_shop_id ON mxx_shop_user_merge(shop_id);
CREATE INDEX IF NOT EXISTS idx_shop_user_merge_user_id ON mxx_shop_user_merge(user_id);

-- 验证更新
SELECT 'mxx_shop' AS table_name, column_name, data_type 
FROM information_schema.columns 
WHERE table_name = 'mxx_shop' 
AND column_name IN ('self_operated', 'store_disable', 'page_show', 'self_pick_flag', 'delete_flag');

SELECT 'mxx_shop_category' AS table_name, column_name, data_type 
FROM information_schema.columns 
WHERE table_name = 'mxx_shop_category' 
AND column_name = 'is_show';

SELECT 'mxx_shop_spu' AS table_name, column_name, data_type 
FROM information_schema.columns 
WHERE table_name = 'mxx_shop_spu' 
AND column_name = 'is_commission';

SELECT 'mxx_shop_cart' AS table_name, column_name, data_type 
FROM information_schema.columns 
WHERE table_name = 'mxx_shop_cart' 
AND column_name = 'selected';

SELECT 'mxx_shop_review' AS table_name, column_name, data_type 
FROM information_schema.columns 
WHERE table_name = 'mxx_shop_review' 
AND column_name = 'is_anonymous';

SELECT '字段类型更新完成' AS result;