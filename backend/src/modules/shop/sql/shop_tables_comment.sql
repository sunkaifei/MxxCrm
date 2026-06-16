-- ============================================
-- Shop Module Table Field Comments
-- 商城模块数据库字段注释
-- ============================================

-- ----------------------------
-- mxx_shop_spu - 商品SPU表
-- ----------------------------
ALTER TABLE `mxx_shop_spu` MODIFY COLUMN `id` bigint NOT NULL AUTO_INCREMENT COMMENT '主键ID';
ALTER TABLE `mxx_shop_spu` MODIFY COLUMN `shop_id` bigint NOT NULL COMMENT '所属店铺ID';
ALTER TABLE `mxx_shop_spu` MODIFY COLUMN `category_id` bigint NOT NULL COMMENT '分类ID';
ALTER TABLE `mxx_shop_spu` MODIFY COLUMN `title` varchar(255) NOT NULL COMMENT '商品标题';
ALTER TABLE `mxx_shop_spu` MODIFY COLUMN `subtitle` varchar(500) NULL DEFAULT NULL COMMENT '副标题/卖点';
ALTER TABLE `mxx_shop_spu` MODIFY COLUMN `primary_image` varchar(500) NOT NULL COMMENT '主图';
ALTER TABLE `mxx_shop_spu` MODIFY COLUMN `images` json NULL DEFAULT NULL COMMENT '商品轮播图列表(JSON)';
ALTER TABLE `mxx_shop_spu` MODIFY COLUMN `video` varchar(500) NULL DEFAULT NULL COMMENT '商品视频';
ALTER TABLE `mxx_shop_spu` MODIFY COLUMN `desc_content` json NULL DEFAULT NULL COMMENT '商品详情(富文本/图片列表JSON)';
ALTER TABLE `mxx_shop_spu` MODIFY COLUMN `is_commission` tinyint(1) NOT NULL DEFAULT 0 COMMENT '是否参与佣金: 0=否, 1=是';
ALTER TABLE `mxx_shop_spu` MODIFY COLUMN `status` tinyint NOT NULL DEFAULT 0 COMMENT '状态: 0=待审核, 1=已上架, 2=下架, 3=审核驳回';
ALTER TABLE `mxx_shop_spu` MODIFY COLUMN `stock_total` int NOT NULL DEFAULT 0 COMMENT '总库存';
ALTER TABLE `mxx_shop_spu` MODIFY COLUMN `sold_num` int NOT NULL DEFAULT 0 COMMENT '已售数量';
ALTER TABLE `mxx_shop_spu` MODIFY COLUMN `min_sale_price` decimal(10,2) NOT NULL COMMENT '最低销售价(分)';
ALTER TABLE `mxx_shop_spu` MODIFY COLUMN `max_sale_price` decimal(10,2) NOT NULL COMMENT '最高销售价(分)';
ALTER TABLE `mxx_shop_spu` MODIFY COLUMN `min_line_price` decimal(10,2) NOT NULL COMMENT '最低划线价(分)';
ALTER TABLE `mxx_shop_spu` MODIFY COLUMN `max_line_price` decimal(10,2) NOT NULL COMMENT '最高划线价(分)';
ALTER TABLE `mxx_shop_spu` MODIFY COLUMN `audit_remark` varchar(500) NULL DEFAULT NULL COMMENT '审核备注';
ALTER TABLE `mxx_shop_spu` MODIFY COLUMN `freight_template_id` bigint NULL DEFAULT NULL COMMENT '运费模板ID';
ALTER TABLE `mxx_shop_spu` MODIFY COLUMN `create_time` datetime NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间';
ALTER TABLE `mxx_shop_spu` MODIFY COLUMN `update_time` datetime NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间';
ALTER TABLE `mxx_shop_spu` COMMENT = '商品SPU表';

-- ----------------------------
-- mxx_shop_sku - 商品SKU表
-- ----------------------------
ALTER TABLE `mxx_shop_sku` MODIFY COLUMN `id` bigint NOT NULL AUTO_INCREMENT COMMENT '主键ID';
ALTER TABLE `mxx_shop_sku` MODIFY COLUMN `spu_id` bigint NOT NULL COMMENT '所属SPU ID';
ALTER TABLE `mxx_shop_sku` MODIFY COLUMN `spec_desc` varchar(255) NOT NULL COMMENT '规格描述(如"颜色:白色 尺码:M")';
ALTER TABLE `mxx_shop_sku` MODIFY COLUMN `sku_code` varchar(100) NULL DEFAULT NULL COMMENT 'SKU编码';
ALTER TABLE `mxx_shop_sku` MODIFY COLUMN `image` varchar(500) NULL DEFAULT NULL COMMENT 'SKU图片';
ALTER TABLE `mxx_shop_sku` MODIFY COLUMN `base_price` decimal(10,2) NOT NULL COMMENT '供货底价(分)';
ALTER TABLE `mxx_shop_sku` MODIFY COLUMN `retail_price` decimal(10,2) NOT NULL COMMENT '零售价(分)';
ALTER TABLE `mxx_shop_sku` MODIFY COLUMN `sale_price` decimal(10,2) NOT NULL COMMENT '实际售价(分)';
ALTER TABLE `mxx_shop_sku` MODIFY COLUMN `line_price` decimal(10,2) NOT NULL COMMENT '划线价(分)';
ALTER TABLE `mxx_shop_sku` MODIFY COLUMN `stock` int NOT NULL DEFAULT 0 COMMENT '库存';
ALTER TABLE `mxx_shop_sku` MODIFY COLUMN `safe_stock` int NOT NULL DEFAULT 0 COMMENT '安全库存';
ALTER TABLE `mxx_shop_sku` MODIFY COLUMN `sold_num` int NOT NULL DEFAULT 0 COMMENT '已售数量';
ALTER TABLE `mxx_shop_sku` MODIFY COLUMN `weight` decimal(8,2) NULL DEFAULT NULL COMMENT '重量(kg)';
ALTER TABLE `mxx_shop_sku` MODIFY COLUMN `create_time` datetime NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间';
ALTER TABLE `mxx_shop_sku` MODIFY COLUMN `update_time` datetime NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间';
ALTER TABLE `mxx_shop_sku` COMMENT = '商品SKU表';

-- ----------------------------
-- mxx_shop_cart - 购物车表
-- ----------------------------
ALTER TABLE `mxx_shop_cart` MODIFY COLUMN `id` bigint NOT NULL AUTO_INCREMENT COMMENT '主键ID';
ALTER TABLE `mxx_shop_cart` MODIFY COLUMN `user_id` bigint NOT NULL COMMENT '买家用户ID';
ALTER TABLE `mxx_shop_cart` MODIFY COLUMN `shop_id` bigint NOT NULL COMMENT '店铺ID';
ALTER TABLE `mxx_shop_cart` MODIFY COLUMN `spu_id` bigint NOT NULL COMMENT '商品ID';
ALTER TABLE `mxx_shop_cart` MODIFY COLUMN `sku_id` bigint NOT NULL COMMENT 'SKU ID';
ALTER TABLE `mxx_shop_cart` MODIFY COLUMN `quantity` int NOT NULL DEFAULT 1 COMMENT '数量';
ALTER TABLE `mxx_shop_cart` MODIFY COLUMN `selected` tinyint(1) NOT NULL DEFAULT 1 COMMENT '是否选中: 0=否, 1=是';
ALTER TABLE `mxx_shop_cart` MODIFY COLUMN `create_time` datetime NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间';
ALTER TABLE `mxx_shop_cart` MODIFY COLUMN `update_time` datetime NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间';
ALTER TABLE `mxx_shop_cart` COMMENT = '购物车表';

-- ----------------------------
-- mxx_shop_category - 商品分类表
-- ----------------------------
ALTER TABLE `mxx_shop_category` MODIFY COLUMN `id` bigint NOT NULL AUTO_INCREMENT COMMENT '主键ID';
ALTER TABLE `mxx_shop_category` MODIFY COLUMN `parent_id` bigint NOT NULL DEFAULT 0 COMMENT '上级分类ID, 0=顶级';
ALTER TABLE `mxx_shop_category` MODIFY COLUMN `name` varchar(100) NOT NULL COMMENT '分类名称';
ALTER TABLE `mxx_shop_category` MODIFY COLUMN `icon` varchar(255) NULL DEFAULT NULL COMMENT '分类图标';
ALTER TABLE `mxx_shop_category` MODIFY COLUMN `sort_order` int NOT NULL DEFAULT 0 COMMENT '排序值(升序)';
ALTER TABLE `mxx_shop_category` MODIFY COLUMN `is_show` tinyint(1) NOT NULL DEFAULT 1 COMMENT '是否显示: 0=隐藏, 1=显示';
ALTER TABLE `mxx_shop_category` MODIFY COLUMN `level` tinyint NOT NULL DEFAULT 1 COMMENT '层级: 1/2/3';
ALTER TABLE `mxx_shop_category` MODIFY COLUMN `create_time` datetime NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间';
ALTER TABLE `mxx_shop_category` MODIFY COLUMN `update_time` datetime NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间';
ALTER TABLE `mxx_shop_category` COMMENT = '商品分类表';

-- ----------------------------
-- mxx_shop_order - 订单表
-- ----------------------------
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `id` bigint NOT NULL AUTO_INCREMENT COMMENT '主键ID';
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `order_no` varchar(50) NOT NULL COMMENT '订单编号';
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `user_id` bigint NOT NULL COMMENT '买家用户ID';
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `shop_id` bigint NOT NULL COMMENT '店铺ID';
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `total_amount` decimal(12,2) NOT NULL COMMENT '订单总金额(分)';
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `freight_amount` decimal(10,2) NOT NULL COMMENT '运费(分)';
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `commission_amount` decimal(10,2) NOT NULL COMMENT '平台佣金(分)';
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `commission_rate` decimal(5,2) NOT NULL COMMENT '佣金比例';
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `settlement_amount` decimal(12,2) NOT NULL COMMENT '供货商结算金额(分)';
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `refund_amount` decimal(12,2) NOT NULL COMMENT '退款金额(分)';
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `goods_count` int NOT NULL DEFAULT 0 COMMENT '商品数量';
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `status` tinyint NOT NULL DEFAULT 0 COMMENT '状态: 0=待付款, 1=待发货, 2=已发货, 3=已签收, 4=已完成, 5=已取消, 6=退款中, 7=已退款';
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `receiver_name` varchar(50) NOT NULL COMMENT '收货人';
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `receiver_phone` varchar(20) NOT NULL COMMENT '收货电话';
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `receiver_address` varchar(500) NOT NULL COMMENT '收货地址';
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `buyer_remark` varchar(500) NULL DEFAULT NULL COMMENT '买家留言';
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `cancel_reason` varchar(500) NULL DEFAULT NULL COMMENT '取消原因';
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `pay_method` varchar(50) NULL DEFAULT NULL COMMENT '支付方式';
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `pay_time` datetime NULL DEFAULT NULL COMMENT '支付时间';
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `delivery_time` datetime NULL DEFAULT NULL COMMENT '发货时间';
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `receive_time` datetime NULL DEFAULT NULL COMMENT '签收/确认收货时间';
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `finish_time` datetime NULL DEFAULT NULL COMMENT '完成(可评价)时间';
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `cancel_time` datetime NULL DEFAULT NULL COMMENT '取消时间';
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `create_time` datetime NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间';
ALTER TABLE `mxx_shop_order` MODIFY COLUMN `update_time` datetime NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间';
ALTER TABLE `mxx_shop_order` COMMENT = '订单表';

-- ----------------------------
-- mxx_shop_order_item - 订单明细表
-- ----------------------------
ALTER TABLE `mxx_shop_order_item` MODIFY COLUMN `id` bigint NOT NULL AUTO_INCREMENT COMMENT '主键ID';
ALTER TABLE `mxx_shop_order_item` MODIFY COLUMN `order_id` bigint NOT NULL COMMENT '订单ID';
ALTER TABLE `mxx_shop_order_item` MODIFY COLUMN `spu_id` bigint NOT NULL COMMENT '商品ID';
ALTER TABLE `mxx_shop_order_item` MODIFY COLUMN `sku_id` bigint NOT NULL COMMENT 'SKU ID';
ALTER TABLE `mxx_shop_order_item` MODIFY COLUMN `goods_title` varchar(255) NOT NULL COMMENT '商品标题(快照)';
ALTER TABLE `mxx_shop_order_item` MODIFY COLUMN `goods_image` varchar(500) NOT NULL COMMENT '商品图片(快照)';
ALTER TABLE `mxx_shop_order_item` MODIFY COLUMN `spec_desc` varchar(255) NULL DEFAULT NULL COMMENT '规格描述(如"颜色:白色 尺码:M")';
ALTER TABLE `mxx_shop_order_item` MODIFY COLUMN `price` decimal(10,2) NOT NULL COMMENT '成交单价(分)';
ALTER TABLE `mxx_shop_order_item` MODIFY COLUMN `quantity` int NOT NULL DEFAULT 1 COMMENT '数量';
ALTER TABLE `mxx_shop_order_item` MODIFY COLUMN `base_price` decimal(10,2) NOT NULL COMMENT '供货底价(分)';
ALTER TABLE `mxx_shop_order_item` MODIFY COLUMN `commission_amount` decimal(10,2) NOT NULL COMMENT '该SKU佣金(分)';
ALTER TABLE `mxx_shop_order_item` MODIFY COLUMN `settlement_amount` decimal(10,2) NOT NULL COMMENT '该SKU结算金额(分)';
ALTER TABLE `mxx_shop_order_item` MODIFY COLUMN `create_time` datetime NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间';
ALTER TABLE `mxx_shop_order_item` COMMENT = '订单明细表';

-- ----------------------------
-- mxx_shop_review - 商品评价表
-- ----------------------------
ALTER TABLE `mxx_shop_review` MODIFY COLUMN `id` bigint NOT NULL AUTO_INCREMENT COMMENT '主键ID';
ALTER TABLE `mxx_shop_review` MODIFY COLUMN `order_id` bigint NOT NULL COMMENT '订单ID';
ALTER TABLE `mxx_shop_review` MODIFY COLUMN `spu_id` bigint NOT NULL COMMENT '商品ID';
ALTER TABLE `mxx_shop_review` MODIFY COLUMN `sku_id` bigint NOT NULL COMMENT 'SKU ID';
ALTER TABLE `mxx_shop_review` MODIFY COLUMN `user_id` bigint NOT NULL COMMENT '买家用户ID';
ALTER TABLE `mxx_shop_review` MODIFY COLUMN `shop_id` bigint NOT NULL COMMENT '店铺ID';
ALTER TABLE `mxx_shop_review` MODIFY COLUMN `score` tinyint NOT NULL DEFAULT 5 COMMENT '评分: 1~5';
ALTER TABLE `mxx_shop_review` MODIFY COLUMN `content` text NULL DEFAULT NULL COMMENT '评价内容';
ALTER TABLE `mxx_shop_review` MODIFY COLUMN `images` json NULL DEFAULT NULL COMMENT '评价图片列表(JSON)';
ALTER TABLE `mxx_shop_review` MODIFY COLUMN `is_anonymous` tinyint(1) NOT NULL DEFAULT 0 COMMENT '是否匿名: 0=否, 1=是';
ALTER TABLE `mxx_shop_review` MODIFY COLUMN `reply_content` text NULL DEFAULT NULL COMMENT '供货商回复';
ALTER TABLE `mxx_shop_review` MODIFY COLUMN `reply_time` datetime NULL DEFAULT NULL COMMENT '回复时间';
ALTER TABLE `mxx_shop_review` MODIFY COLUMN `create_time` datetime NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间';
ALTER TABLE `mxx_shop_review` MODIFY COLUMN `update_time` datetime NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间';
ALTER TABLE `mxx_shop_review` COMMENT = '商品评价表';

-- ----------------------------
-- mxx_shop_delivery - 发货物流表
-- ----------------------------
ALTER TABLE `mxx_shop_delivery` MODIFY COLUMN `id` bigint NOT NULL AUTO_INCREMENT COMMENT '主键ID';
ALTER TABLE `mxx_shop_delivery` MODIFY COLUMN `order_id` bigint NOT NULL COMMENT '订单ID';
ALTER TABLE `mxx_shop_delivery` MODIFY COLUMN `logistics_company` varchar(100) NOT NULL COMMENT '快递公司';
ALTER TABLE `mxx_shop_delivery` MODIFY COLUMN `logistics_no` varchar(100) NOT NULL COMMENT '快递单号';
ALTER TABLE `mxx_shop_delivery` MODIFY COLUMN `receiver_name` varchar(50) NOT NULL COMMENT '收货人';
ALTER TABLE `mxx_shop_delivery` MODIFY COLUMN `receiver_phone` varchar(20) NOT NULL COMMENT '收货电话';
ALTER TABLE `mxx_shop_delivery` MODIFY COLUMN `receiver_address` varchar(500) NOT NULL COMMENT '收货地址';
ALTER TABLE `mxx_shop_delivery` MODIFY COLUMN `delivery_remark` varchar(500) NULL DEFAULT NULL COMMENT '发货备注';
ALTER TABLE `mxx_shop_delivery` MODIFY COLUMN `delivery_status` tinyint NOT NULL DEFAULT 0 COMMENT '物流状态: 0=待揽收, 1=运输中, 2=已签收';
ALTER TABLE `mxx_shop_delivery` MODIFY COLUMN `create_time` datetime NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间';
ALTER TABLE `mxx_shop_delivery` MODIFY COLUMN `update_time` datetime NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间';
ALTER TABLE `mxx_shop_delivery` COMMENT = '发货物流表';

-- ----------------------------
-- mxx_shop_settlement - 结算记录表
-- ----------------------------
ALTER TABLE `mxx_shop_settlement` MODIFY COLUMN `id` bigint NOT NULL AUTO_INCREMENT COMMENT '主键ID';
ALTER TABLE `mxx_shop_settlement` MODIFY COLUMN `shop_id` bigint NOT NULL COMMENT '店铺ID';
ALTER TABLE `mxx_shop_settlement` MODIFY COLUMN `settlement_no` varchar(50) NOT NULL COMMENT '结算单号';
ALTER TABLE `mxx_shop_settlement` MODIFY COLUMN `period_start` datetime NOT NULL COMMENT '结算周期开始';
ALTER TABLE `mxx_shop_settlement` MODIFY COLUMN `period_end` datetime NOT NULL COMMENT '结算周期结束';
ALTER TABLE `mxx_shop_settlement` MODIFY COLUMN `order_count` int NOT NULL DEFAULT 0 COMMENT '订单数量';
ALTER TABLE `mxx_shop_settlement` MODIFY COLUMN `total_amount` decimal(12,2) NOT NULL COMMENT '总交易额(分)';
ALTER TABLE `mxx_shop_settlement` MODIFY COLUMN `commission_amount` decimal(12,2) NOT NULL COMMENT '总佣金(分)';
ALTER TABLE `mxx_shop_settlement` MODIFY COLUMN `settlement_amount` decimal(12,2) NOT NULL COMMENT '结算金额(分)';
ALTER TABLE `mxx_shop_settlement` MODIFY COLUMN `settlement_status` tinyint NOT NULL DEFAULT 0 COMMENT '状态: 0=待结算, 1=已结算, 2=已付款';
ALTER TABLE `mxx_shop_settlement` MODIFY COLUMN `settle_time` datetime NULL DEFAULT NULL COMMENT '付款时间';
ALTER TABLE `mxx_shop_settlement` MODIFY COLUMN `remark` varchar(500) NULL DEFAULT NULL COMMENT '备注';
ALTER TABLE `mxx_shop_settlement` MODIFY COLUMN `create_time` datetime NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间';
ALTER TABLE `mxx_shop_settlement` MODIFY COLUMN `update_time` datetime NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间';
ALTER TABLE `mxx_shop_settlement` COMMENT = '结算记录表';

-- ----------------------------
-- mxx_shop_promotion - 促销活动表
-- ----------------------------
ALTER TABLE `mxx_shop_promotion` MODIFY COLUMN `id` bigint NOT NULL AUTO_INCREMENT COMMENT '主键ID';
ALTER TABLE `mxx_shop_promotion` MODIFY COLUMN `shop_id` bigint NOT NULL COMMENT '店铺ID(NULL=平台活动)';
ALTER TABLE `mxx_shop_promotion` MODIFY COLUMN `title` varchar(200) NOT NULL COMMENT '活动标题';
ALTER TABLE `mxx_shop_promotion` MODIFY COLUMN `promotion_type` tinyint NOT NULL DEFAULT 1 COMMENT '类型: 1=满减, 2=折扣, 3=限时抢购, 4=新人专享';
ALTER TABLE `mxx_shop_promotion` MODIFY COLUMN `discount_value` decimal(10,2) NOT NULL COMMENT '优惠值(满减金额/折扣率/特价价格)';
ALTER TABLE `mxx_shop_promotion` MODIFY COLUMN `condition_value` decimal(10,2) NULL DEFAULT NULL COMMENT '条件值(满减条件金额)';
ALTER TABLE `mxx_shop_promotion` MODIFY COLUMN `start_time` datetime NOT NULL COMMENT '开始时间';
ALTER TABLE `mxx_shop_promotion` MODIFY COLUMN `end_time` datetime NOT NULL COMMENT '结束时间';
ALTER TABLE `mxx_shop_promotion` MODIFY COLUMN `status` tinyint NOT NULL DEFAULT 0 COMMENT '状态: 0=未开始, 1=进行中, 2=已结束, 3=已关闭';
ALTER TABLE `mxx_shop_promotion` MODIFY COLUMN `create_time` datetime NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间';
ALTER TABLE `mxx_shop_promotion` MODIFY COLUMN `update_time` datetime NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间';
ALTER TABLE `mxx_shop_promotion` COMMENT = '促销活动表';

-- ----------------------------
-- mxx_shop_promotion_spu - 促销商品关联表
-- ----------------------------
ALTER TABLE `mxx_shop_promotion_spu` MODIFY COLUMN `id` bigint NOT NULL AUTO_INCREMENT COMMENT '主键ID';
ALTER TABLE `mxx_shop_promotion_spu` MODIFY COLUMN `promotion_id` bigint NOT NULL COMMENT '活动ID';
ALTER TABLE `mxx_shop_promotion_spu` MODIFY COLUMN `spu_id` bigint NOT NULL COMMENT '商品ID';
ALTER TABLE `mxx_shop_promotion_spu` MODIFY COLUMN `sku_id` bigint NULL DEFAULT NULL COMMENT 'SKU ID(可选)';
ALTER TABLE `mxx_shop_promotion_spu` MODIFY COLUMN `create_time` datetime NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间';
ALTER TABLE `mxx_shop_promotion_spu` COMMENT = '促销商品关联表';

-- ----------------------------
-- mxx_shop_refund - 退款申请表
-- ----------------------------
ALTER TABLE `mxx_shop_refund` MODIFY COLUMN `id` bigint NOT NULL AUTO_INCREMENT COMMENT '主键ID';
ALTER TABLE `mxx_shop_refund` MODIFY COLUMN `order_id` bigint NOT NULL COMMENT '订单ID';
ALTER TABLE `mxx_shop_refund` MODIFY COLUMN `order_item_id` bigint NOT NULL COMMENT '订单明细ID';
ALTER TABLE `mxx_shop_refund` MODIFY COLUMN `refund_no` varchar(50) NOT NULL COMMENT '退款单号';
ALTER TABLE `mxx_shop_refund` MODIFY COLUMN `user_id` bigint NOT NULL COMMENT '买家用户ID';
ALTER TABLE `mxx_shop_refund` MODIFY COLUMN `shop_id` bigint NOT NULL COMMENT '店铺ID';
ALTER TABLE `mxx_shop_refund` MODIFY COLUMN `refund_type` tinyint NOT NULL DEFAULT 1 COMMENT '退款类型: 1=退货退款, 2=仅退款';
ALTER TABLE `mxx_shop_refund` MODIFY COLUMN `refund_reason` varchar(500) NOT NULL COMMENT '退款原因';
ALTER TABLE `mxx_shop_refund` MODIFY COLUMN `refund_amount` decimal(12,2) NOT NULL COMMENT '退款金额(分)';
ALTER TABLE `mxx_shop_refund` MODIFY COLUMN `refund_status` tinyint NOT NULL DEFAULT 0 COMMENT '退款状态: 0=待审核, 1=已同意, 2=已驳回, 3=退款中, 4=已退款';
ALTER TABLE `mxx_shop_refund` MODIFY COLUMN `audit_remark` varchar(500) NULL DEFAULT NULL COMMENT '审核备注';
ALTER TABLE `mxx_shop_refund` MODIFY COLUMN `audit_time` datetime NULL DEFAULT NULL COMMENT '审核时间';
ALTER TABLE `mxx_shop_refund` MODIFY COLUMN `reject_reason` varchar(500) NULL DEFAULT NULL COMMENT '驳回原因';
ALTER TABLE `mxx_shop_refund` MODIFY COLUMN `create_time` datetime NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间';
ALTER TABLE `mxx_shop_refund` MODIFY COLUMN `update_time` datetime NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间';
ALTER TABLE `mxx_shop_refund` COMMENT = '退款申请表';

-- ----------------------------
-- mxx_shop_spec - 规格定义表
-- ----------------------------
ALTER TABLE `mxx_shop_spec` MODIFY COLUMN `id` bigint NOT NULL AUTO_INCREMENT COMMENT '主键ID';
ALTER TABLE `mxx_shop_spec` MODIFY COLUMN `spu_id` bigint NOT NULL COMMENT '所属SPU ID';
ALTER TABLE `mxx_shop_spec` MODIFY COLUMN `spec_name` varchar(50) NOT NULL COMMENT '规格名称(如颜色、尺寸)';
ALTER TABLE `mxx_shop_spec` MODIFY COLUMN `sort_order` int NOT NULL DEFAULT 0 COMMENT '排序值';
ALTER TABLE `mxx_shop_spec` COMMENT = '规格定义表';

-- ----------------------------
-- mxx_shop_spec_value - 规格值表
-- ----------------------------
ALTER TABLE `mxx_shop_spec_value` MODIFY COLUMN `id` bigint NOT NULL AUTO_INCREMENT COMMENT '主键ID';
ALTER TABLE `mxx_shop_spec_value` MODIFY COLUMN `spec_id` bigint NOT NULL COMMENT '所属规格ID';
ALTER TABLE `mxx_shop_spec_value` MODIFY COLUMN `spec_value` varchar(50) NOT NULL COMMENT '规格值(如红色、XL)';
ALTER TABLE `mxx_shop_spec_value` MODIFY COLUMN `sort_order` int NOT NULL DEFAULT 0 COMMENT '排序值';
ALTER TABLE `mxx_shop_spec_value` COMMENT = '规格值表';

-- ----------------------------
-- mxx_shop_supplier_apply - 供货商入驻申请表
-- ----------------------------
ALTER TABLE `mxx_shop_supplier_apply` MODIFY COLUMN `id` bigint NOT NULL AUTO_INCREMENT COMMENT '主键ID';
ALTER TABLE `mxx_shop_supplier_apply` MODIFY COLUMN `user_id` bigint NOT NULL COMMENT '申请用户ID';
ALTER TABLE `mxx_shop_supplier_apply` MODIFY COLUMN `shop_name` varchar(200) NOT NULL COMMENT '店铺名称';
ALTER TABLE `mxx_shop_supplier_apply` MODIFY COLUMN `contact_name` varchar(50) NOT NULL COMMENT '联系人';
ALTER TABLE `mxx_shop_supplier_apply` MODIFY COLUMN `contact_phone` varchar(20) NOT NULL COMMENT '联系电话';
ALTER TABLE `mxx_shop_supplier_apply` MODIFY COLUMN `shop_logo` varchar(500) NULL DEFAULT NULL COMMENT '店铺LOGO';
ALTER TABLE `mxx_shop_supplier_apply` MODIFY COLUMN `shop_desc` varchar(500) NULL DEFAULT NULL COMMENT '店铺简介';
ALTER TABLE `mxx_shop_supplier_apply` MODIFY COLUMN `business_license` varchar(500) NULL DEFAULT NULL COMMENT '营业执照图片';
ALTER TABLE `mxx_shop_supplier_apply` MODIFY COLUMN `status` tinyint NOT NULL DEFAULT 0 COMMENT '状态: 0=待审核, 1=通过, 2=驳回';
ALTER TABLE `mxx_shop_supplier_apply` MODIFY COLUMN `audit_remark` varchar(500) NULL DEFAULT NULL COMMENT '审核备注';
ALTER TABLE `mxx_shop_supplier_apply` MODIFY COLUMN `audit_time` datetime NULL DEFAULT NULL COMMENT '审核时间';
ALTER TABLE `mxx_shop_supplier_apply` MODIFY COLUMN `create_time` datetime NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间';
ALTER TABLE `mxx_shop_supplier_apply` MODIFY COLUMN `update_time` datetime NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间';
ALTER TABLE `mxx_shop_supplier_apply` COMMENT = '供货商入驻申请表';

-- ----------------------------
-- mxx_shop - 店铺/供货商表(补充缺失注释)
-- ----------------------------
ALTER TABLE `mxx_shop` MODIFY COLUMN `yzf_mp_sign` varchar(255) NULL DEFAULT NULL COMMENT '银豹小程序签名';
ALTER TABLE `mxx_shop` MODIFY COLUMN `yzf_sign` varchar(255) NULL DEFAULT NULL COMMENT '银豹签名';
ALTER TABLE `mxx_shop` MODIFY COLUMN `merchant_euid` varchar(255) NULL DEFAULT NULL COMMENT '商家EUID';

-- ----------------------------
-- mxx_shop_user_merge - 店铺与用户关联表(补充缺失注释)
-- ----------------------------
ALTER TABLE `mxx_shop_user_merge` MODIFY COLUMN `id` bigint UNSIGNED NOT NULL AUTO_INCREMENT COMMENT '主键ID';
ALTER TABLE `mxx_shop_user_merge` MODIFY COLUMN `create_time` datetime NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间';
ALTER TABLE `mxx_shop_user_merge` COMMENT = '店铺与用户关联表';
