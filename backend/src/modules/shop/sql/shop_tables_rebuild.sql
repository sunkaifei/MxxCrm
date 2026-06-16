-- =============================================
-- Shop Module Database Tables Rebuild Script
-- Date: 2026-06-14
-- =============================================

-- 1. 店铺表
DROP TABLE IF EXISTS mxx_shop CASCADE;
CREATE TABLE mxx_shop (
    id BIGSERIAL PRIMARY KEY COMMENT '主键ID',
    store_logo VARCHAR(255) COMMENT '店铺LOGO',
    store_name VARCHAR(255) COMMENT '店铺名称',
    self_operated SMALLINT NOT NULL DEFAULT 0 COMMENT '是否自营: 0=否, 1=是',
    store_address_detail TEXT COMMENT '店铺详细地址',
    store_address_id_path VARCHAR(500) COMMENT '店铺地址ID路径',
    store_address_path VARCHAR(500) COMMENT '店铺地址路径',
    store_end_time TIMESTAMP COMMENT '店铺营业结束时间',
    store_disable SMALLINT NOT NULL DEFAULT 0 COMMENT '店铺状态: 0=正常, 1=关闭',
    store_center VARCHAR(255) COMMENT '店铺中心位置',
    store_desc TEXT COMMENT '店铺简介',
    delivery_score DECIMAL(5,2) COMMENT '配送评分',
    description_score DECIMAL(5,2) COMMENT '描述评分',
    service_score DECIMAL(5,2) COMMENT '服务评分',
    goods_num INT COMMENT '商品数量',
    collection_num INT COMMENT '收藏数量',
    yzf_mp_sign VARCHAR(500) COMMENT '银豹小程序签名',
    yzf_sign VARCHAR(500) COMMENT '银豹签名',
    merchant_euid VARCHAR(100) COMMENT '商家EUID',
    page_show SMALLINT DEFAULT 0 COMMENT '是否页面展示: 0=否, 1=是',
    self_pick_flag SMALLINT DEFAULT 0 COMMENT '是否支持自提: 0=否, 1=是',
    create_by VARCHAR(100) COMMENT '创建人',
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    update_by VARCHAR(100) COMMENT '更新人',
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    delete_flag SMALLINT DEFAULT 0 COMMENT '删除标志: 0=未删除, 1=已删除'
) COMMENT='店铺/供货商表';

-- 2. 商品分类表
DROP TABLE IF EXISTS mxx_shop_category CASCADE;
CREATE TABLE mxx_shop_category (
    id BIGSERIAL PRIMARY KEY COMMENT '主键ID',
    parent_id BIGINT NOT NULL DEFAULT 0 COMMENT '上级分类ID, 0=顶级',
    name VARCHAR(100) NOT NULL COMMENT '分类名称',
    icon VARCHAR(255) COMMENT '分类图标',
    sort_order INT NOT NULL DEFAULT 0 COMMENT '排序值(升序)',
    is_show SMALLINT NOT NULL DEFAULT 1 COMMENT '是否显示: 0=隐藏, 1=显示',
    level SMALLINT NOT NULL DEFAULT 1 COMMENT '层级: 1/2/3',
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间'
) COMMENT='商品分类表';

-- 3. 商品SPU表
DROP TABLE IF EXISTS mxx_shop_spu CASCADE;
CREATE TABLE mxx_shop_spu (
    id BIGSERIAL PRIMARY KEY COMMENT '主键ID',
    shop_id BIGINT NOT NULL COMMENT '所属店铺ID',
    category_id BIGINT NOT NULL COMMENT '分类ID',
    title VARCHAR(500) NOT NULL COMMENT '商品标题',
    subtitle VARCHAR(500) COMMENT '副标题/卖点',
    primary_image VARCHAR(500) NOT NULL COMMENT '主图',
    images JSON COMMENT '商品轮播图列表(JSON)',
    video VARCHAR(500) COMMENT '商品视频',
    desc_content JSON COMMENT '商品详情(富文本/图片列表JSON)',
    is_commission SMALLINT NOT NULL DEFAULT 0 COMMENT '是否参与佣金: 0=否, 1=是',
    status SMALLINT NOT NULL DEFAULT 0 COMMENT '状态: 0=待审核, 1=已上架, 2=下架, 3=审核驳回',
    stock_total INT NOT NULL DEFAULT 0 COMMENT '总库存',
    sold_num INT NOT NULL DEFAULT 0 COMMENT '已售数量',
    min_sale_price DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '最低销售价(分)',
    max_sale_price DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '最高销售价(分)',
    min_line_price DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '最低划线价(分)',
    max_line_price DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '最高划线价(分)',
    audit_remark VARCHAR(500) COMMENT '审核备注',
    freight_template_id BIGINT COMMENT '运费模板ID',
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间'
) COMMENT='商品SPU表';

-- 4. 商品SKU表
DROP TABLE IF EXISTS mxx_shop_sku CASCADE;
CREATE TABLE mxx_shop_sku (
    id BIGSERIAL PRIMARY KEY COMMENT '主键ID',
    spu_id BIGINT NOT NULL COMMENT '所属SPU ID',
    spec_desc VARCHAR(200) NOT NULL COMMENT '规格描述(如"颜色:白色 尺码:M")',
    sku_code VARCHAR(100) COMMENT 'SKU编码',
    image VARCHAR(500) COMMENT 'SKU图片',
    base_price DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '供货底价(分)',
    retail_price DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '零售价(分)',
    sale_price DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '实际售价(分)',
    line_price DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '划线价(分)',
    stock INT NOT NULL DEFAULT 0 COMMENT '库存',
    safe_stock INT NOT NULL DEFAULT 0 COMMENT '安全库存',
    sold_num INT NOT NULL DEFAULT 0 COMMENT '已售数量',
    weight DECIMAL(10,3) COMMENT '重量(kg)',
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间'
) COMMENT='商品SKU表';

-- 5. 购物车表
DROP TABLE IF EXISTS mxx_shop_cart CASCADE;
CREATE TABLE mxx_shop_cart (
    id BIGSERIAL PRIMARY KEY COMMENT '主键ID',
    user_id BIGINT NOT NULL COMMENT '买家用户ID',
    shop_id BIGINT NOT NULL COMMENT '店铺ID',
    spu_id BIGINT NOT NULL COMMENT '商品ID',
    sku_id BIGINT NOT NULL COMMENT 'SKU ID',
    quantity INT NOT NULL DEFAULT 1 COMMENT '数量',
    selected SMALLINT NOT NULL DEFAULT 1 COMMENT '是否选中: 0=否, 1=是',
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间'
) COMMENT='购物车表';

-- 6. 订单表
DROP TABLE IF EXISTS mxx_shop_order CASCADE;
CREATE TABLE mxx_shop_order (
    id BIGSERIAL PRIMARY KEY COMMENT '主键ID',
    order_no VARCHAR(50) NOT NULL UNIQUE COMMENT '订单编号',
    user_id BIGINT NOT NULL COMMENT '买家用户ID',
    shop_id BIGINT NOT NULL COMMENT '店铺ID',
    total_amount DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '订单总金额(分)',
    freight_amount DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '运费(分)',
    commission_amount DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '平台佣金(分)',
    commission_rate DECIMAL(5,2) NOT NULL DEFAULT 0 COMMENT '佣金比例',
    settlement_amount DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '供货商结算金额(分)',
    refund_amount DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '退款金额(分)',
    goods_count INT NOT NULL DEFAULT 0 COMMENT '商品数量',
    status SMALLINT NOT NULL DEFAULT 0 COMMENT '状态: 0=待付款, 1=待发货, 2=已发货, 3=已签收, 4=已完成, 5=已取消, 6=退款中, 7=已退款',
    receiver_name VARCHAR(100) NOT NULL COMMENT '收货人',
    receiver_phone VARCHAR(20) NOT NULL COMMENT '收货电话',
    receiver_address TEXT NOT NULL COMMENT '收货地址',
    buyer_remark VARCHAR(500) COMMENT '买家留言',
    cancel_reason VARCHAR(500) COMMENT '取消原因',
    pay_method VARCHAR(50) COMMENT '支付方式',
    pay_time TIMESTAMP COMMENT '支付时间',
    delivery_time TIMESTAMP COMMENT '发货时间',
    receive_time TIMESTAMP COMMENT '签收/确认收货时间',
    finish_time TIMESTAMP COMMENT '完成(可评价)时间',
    cancel_time TIMESTAMP COMMENT '取消时间',
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间'
) COMMENT='订单表';

-- 7. 订单明细表
DROP TABLE IF EXISTS mxx_shop_order_item CASCADE;
CREATE TABLE mxx_shop_order_item (
    id BIGSERIAL PRIMARY KEY COMMENT '主键ID',
    order_id BIGINT NOT NULL COMMENT '订单ID',
    spu_id BIGINT NOT NULL COMMENT '商品ID',
    sku_id BIGINT NOT NULL COMMENT 'SKU ID',
    goods_title VARCHAR(500) NOT NULL COMMENT '商品标题(快照)',
    goods_image VARCHAR(500) NOT NULL COMMENT '商品图片(快照)',
    spec_desc VARCHAR(200) COMMENT '规格描述(如"颜色:白色 尺码:M")',
    price DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '成交单价(分)',
    quantity INT NOT NULL DEFAULT 0 COMMENT '数量',
    base_price DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '供货底价(分)',
    commission_amount DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '该SKU佣金(分)',
    settlement_amount DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '该SKU结算金额(分)',
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间'
) COMMENT='订单明细表';

-- 8. 商品评价表
DROP TABLE IF EXISTS mxx_shop_review CASCADE;
CREATE TABLE mxx_shop_review (
    id BIGSERIAL PRIMARY KEY COMMENT '主键ID',
    order_id BIGINT NOT NULL COMMENT '订单ID',
    spu_id BIGINT NOT NULL COMMENT '商品ID',
    sku_id BIGINT NOT NULL COMMENT 'SKU ID',
    user_id BIGINT NOT NULL COMMENT '买家用户ID',
    shop_id BIGINT NOT NULL COMMENT '店铺ID',
    score SMALLINT NOT NULL DEFAULT 5 COMMENT '评分: 1~5',
    content TEXT COMMENT '评价内容',
    images JSON COMMENT '评价图片列表(JSON)',
    is_anonymous SMALLINT NOT NULL DEFAULT 0 COMMENT '是否匿名: 0=否, 1=是',
    reply_content TEXT COMMENT '供货商回复',
    reply_time TIMESTAMP COMMENT '回复时间',
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间'
) COMMENT='商品评价表';

-- 9. 供货商入驻申请表
DROP TABLE IF EXISTS mxx_shop_supplier_apply CASCADE;
CREATE TABLE mxx_shop_supplier_apply (
    id BIGSERIAL PRIMARY KEY COMMENT '主键ID',
    user_id BIGINT NOT NULL COMMENT '申请用户ID',
    shop_name VARCHAR(255) NOT NULL COMMENT '店铺名称',
    contact_name VARCHAR(100) NOT NULL COMMENT '联系人',
    contact_phone VARCHAR(20) NOT NULL COMMENT '联系电话',
    shop_logo VARCHAR(500) COMMENT '店铺LOGO',
    shop_desc TEXT COMMENT '店铺简介',
    business_license VARCHAR(500) COMMENT '营业执照图片',
    status SMALLINT NOT NULL DEFAULT 0 COMMENT '状态: 0=待审核, 1=通过, 2=驳回',
    audit_remark VARCHAR(500) COMMENT '审核备注',
    audit_time TIMESTAMP COMMENT '审核时间',
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间'
) COMMENT='供货商入驻申请表';

-- 10. 规格定义表
DROP TABLE IF EXISTS mxx_shop_spec CASCADE;
CREATE TABLE mxx_shop_spec (
    id BIGSERIAL PRIMARY KEY COMMENT '主键ID',
    spu_id BIGINT NOT NULL COMMENT '所属SPU ID',
    spec_name VARCHAR(100) NOT NULL COMMENT '规格名称(如颜色、尺寸)',
    sort_order INT NOT NULL DEFAULT 0 COMMENT '排序值'
) COMMENT='规格定义表';

-- 11. 规格值表
DROP TABLE IF EXISTS mxx_shop_spec_value CASCADE;
CREATE TABLE mxx_shop_spec_value (
    id BIGSERIAL PRIMARY KEY COMMENT '主键ID',
    spec_id BIGINT NOT NULL COMMENT '所属规格ID',
    spec_value VARCHAR(100) NOT NULL COMMENT '规格值(如红色、XL)',
    sort_order INT NOT NULL DEFAULT 0 COMMENT '排序值'
) COMMENT='规格值表';

-- 12. 促销活动表
DROP TABLE IF EXISTS mxx_shop_promotion CASCADE;
CREATE TABLE mxx_shop_promotion (
    id BIGSERIAL PRIMARY KEY COMMENT '主键ID',
    shop_id BIGINT DEFAULT 0 COMMENT '店铺ID(0=平台活动)',
    title VARCHAR(200) NOT NULL COMMENT '活动标题',
    promotion_type SMALLINT NOT NULL COMMENT '类型: 1=满减, 2=折扣, 3=限时抢购, 4=新人专享',
    discount_value DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '优惠值(满减金额/折扣率/特价价格)',
    condition_value DECIMAL(12,2) COMMENT '条件值(满减条件金额)',
    start_time TIMESTAMP NOT NULL COMMENT '开始时间',
    end_time TIMESTAMP NOT NULL COMMENT '结束时间',
    status SMALLINT NOT NULL DEFAULT 0 COMMENT '状态: 0=未开始, 1=进行中, 2=已结束, 3=已关闭',
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间'
) COMMENT='促销活动表';

-- 13. 促销商品关联表
DROP TABLE IF EXISTS mxx_shop_promotion_spu CASCADE;
CREATE TABLE mxx_shop_promotion_spu (
    id BIGSERIAL PRIMARY KEY COMMENT '主键ID',
    promotion_id BIGINT NOT NULL COMMENT '活动ID',
    spu_id BIGINT NOT NULL COMMENT '商品ID',
    sku_id BIGINT COMMENT 'SKU ID(可选)',
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间'
) COMMENT='促销商品关联表';

-- 14. 结算记录表
DROP TABLE IF EXISTS mxx_shop_settlement CASCADE;
CREATE TABLE mxx_shop_settlement (
    id BIGSERIAL PRIMARY KEY COMMENT '主键ID',
    shop_id BIGINT NOT NULL COMMENT '店铺ID',
    settlement_no VARCHAR(50) NOT NULL UNIQUE COMMENT '结算单号',
    period_start TIMESTAMP NOT NULL COMMENT '结算周期开始',
    period_end TIMESTAMP NOT NULL COMMENT '结算周期结束',
    order_count INT NOT NULL DEFAULT 0 COMMENT '订单数量',
    total_amount DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '总交易额(分)',
    commission_amount DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '总佣金(分)',
    settlement_amount DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '结算金额(分)',
    settlement_status SMALLINT NOT NULL DEFAULT 0 COMMENT '状态: 0=待结算, 1=已结算, 2=已付款',
    settle_time TIMESTAMP COMMENT '付款时间',
    remark VARCHAR(500) COMMENT '备注',
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间'
) COMMENT='结算记录表';

-- 15. 发货物流表
DROP TABLE IF EXISTS mxx_shop_delivery CASCADE;
CREATE TABLE mxx_shop_delivery (
    id BIGSERIAL PRIMARY KEY COMMENT '主键ID',
    order_id BIGINT NOT NULL COMMENT '订单ID',
    logistics_company VARCHAR(100) NOT NULL COMMENT '快递公司',
    logistics_no VARCHAR(100) NOT NULL COMMENT '快递单号',
    receiver_name VARCHAR(100) NOT NULL COMMENT '收货人',
    receiver_phone VARCHAR(20) NOT NULL COMMENT '收货电话',
    receiver_address TEXT NOT NULL COMMENT '收货地址',
    delivery_remark VARCHAR(500) COMMENT '发货备注',
    delivery_status SMALLINT NOT NULL DEFAULT 0 COMMENT '物流状态: 0=待揽收, 1=运输中, 2=已签收',
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间'
) COMMENT='发货物流表';

-- 16. 退款申请表
DROP TABLE IF EXISTS mxx_shop_refund CASCADE;
CREATE TABLE mxx_shop_refund (
    id BIGSERIAL PRIMARY KEY COMMENT '主键ID',
    order_id BIGINT NOT NULL COMMENT '订单ID',
    order_item_id BIGINT NOT NULL COMMENT '订单明细ID',
    refund_no VARCHAR(50) NOT NULL UNIQUE COMMENT '退款单号',
    user_id BIGINT NOT NULL COMMENT '买家用户ID',
    shop_id BIGINT NOT NULL COMMENT '店铺ID',
    refund_type SMALLINT NOT NULL COMMENT '退款类型: 1=退货退款, 2=仅退款',
    refund_reason VARCHAR(500) NOT NULL COMMENT '退款原因',
    refund_amount DECIMAL(12,2) NOT NULL DEFAULT 0 COMMENT '退款金额(分)',
    refund_status SMALLINT NOT NULL DEFAULT 0 COMMENT '退款状态: 0=待审核, 1=已同意, 2=已驳回, 3=退款中, 4=已退款',
    audit_remark VARCHAR(500) COMMENT '审核备注',
    audit_time TIMESTAMP COMMENT '审核时间',
    reject_reason VARCHAR(500) COMMENT '驳回原因',
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间'
) COMMENT='退款申请表';

-- 17. 店铺与用户关联表
DROP TABLE IF EXISTS mxx_shop_user_merge CASCADE;
CREATE TABLE mxx_shop_user_merge (
    id BIGSERIAL PRIMARY KEY COMMENT '主键ID',
    shop_id BIGINT NOT NULL COMMENT '店铺ID',
    user_id BIGINT NOT NULL COMMENT '用户ID',
    is_admin INT DEFAULT 0 COMMENT '是否管理员: 0=否, 1=是',
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间'
) COMMENT='店铺与用户关联表';

-- 创建索引
CREATE INDEX idx_shop_store_name ON mxx_shop(store_name);
CREATE INDEX idx_shop_self_operated ON mxx_shop(self_operated);
CREATE INDEX idx_shop_store_disable ON mxx_shop(store_disable);

CREATE INDEX idx_category_parent_id ON mxx_shop_category(parent_id);
CREATE INDEX idx_category_level ON mxx_shop_category(level);

CREATE INDEX idx_spu_shop_id ON mxx_shop_spu(shop_id);
CREATE INDEX idx_spu_category_id ON mxx_shop_spu(category_id);
CREATE INDEX idx_spu_status ON mxx_shop_spu(status);

CREATE INDEX idx_sku_spu_id ON mxx_shop_sku(spu_id);

CREATE INDEX idx_cart_user_id ON mxx_shop_cart(user_id);
CREATE INDEX idx_cart_sku_id ON mxx_shop_cart(sku_id);

CREATE INDEX idx_order_order_no ON mxx_shop_order(order_no);
CREATE INDEX idx_order_user_id ON mxx_shop_order(user_id);
CREATE INDEX idx_order_shop_id ON mxx_shop_order(shop_id);
CREATE INDEX idx_order_status ON mxx_shop_order(status);

CREATE INDEX idx_order_item_order_id ON mxx_shop_order_item(order_id);
CREATE INDEX idx_order_item_spu_id ON mxx_shop_order_item(spu_id);

CREATE INDEX idx_review_spu_id ON mxx_shop_review(spu_id);
CREATE INDEX idx_review_order_id ON mxx_shop_review(order_id);
CREATE INDEX idx_review_user_id ON mxx_shop_review(user_id);

CREATE INDEX idx_supplier_apply_user_id ON mxx_shop_supplier_apply(user_id);
CREATE INDEX idx_supplier_apply_status ON mxx_shop_supplier_apply(status);

CREATE INDEX idx_spec_spu_id ON mxx_shop_spec(spu_id);
CREATE INDEX idx_spec_value_spec_id ON mxx_shop_spec_value(spec_id);

CREATE INDEX idx_promotion_shop_id ON mxx_shop_promotion(shop_id);
CREATE INDEX idx_promotion_status ON mxx_shop_promotion(status);

CREATE INDEX idx_promotion_spu_promotion_id ON mxx_shop_promotion_spu(promotion_id);
CREATE INDEX idx_promotion_spu_spu_id ON mxx_shop_promotion_spu(spu_id);

CREATE INDEX idx_settlement_shop_id ON mxx_shop_settlement(shop_id);
CREATE INDEX idx_settlement_status ON mxx_shop_settlement(settlement_status);

CREATE INDEX idx_delivery_order_id ON mxx_shop_delivery(order_id);

CREATE INDEX idx_refund_order_id ON mxx_shop_refund(order_id);
CREATE INDEX idx_refund_user_id ON mxx_shop_refund(user_id);
CREATE INDEX idx_refund_status ON mxx_shop_refund(refund_status);

CREATE INDEX idx_shop_user_merge_shop_id ON mxx_shop_user_merge(shop_id);
CREATE INDEX idx_shop_user_merge_user_id ON mxx_shop_user_merge(user_id);