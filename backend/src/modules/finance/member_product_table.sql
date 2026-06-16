-- ============================================================
-- 会员产品配置表
-- 用于管理各种会员产品的配置信息
-- ============================================================

CREATE TABLE `mxx_member_product` (
    `id` bigint UNSIGNED NOT NULL AUTO_INCREMENT COMMENT '产品ID',
    `product_id` varchar(64) NOT NULL COMMENT '产品标识ID（如：breeder_vip_1month）',
    `product_name` varchar(100) NOT NULL COMMENT '产品名称',
    `product_type` varchar(50) NOT NULL COMMENT '产品类型: breeder_vip=养殖户VIP, shop_vip=商户VIP, normal_vip=普通VIP',
    `duration_type` tinyint NOT NULL DEFAULT '1' COMMENT '时长类型: 1=月度, 2=季度, 3=年度, 4=永久',
    `duration_value` int NOT NULL DEFAULT '1' COMMENT '时长值（配合duration_type使用，如：duration_type=1, duration_value=3 表示3个月）',
    `price` decimal(10,2) NOT NULL DEFAULT '0.00' COMMENT '价格',
    `original_price` decimal(10,2) DEFAULT NULL COMMENT '原价',
    `discount` decimal(5,2) DEFAULT NULL COMMENT '折扣（如：0.95 表示95折）',
    `member_type` tinyint NOT NULL DEFAULT '2' COMMENT '会员类型: 1=普通会员, 2=养殖户, 3=商户',
    `status` tinyint NOT NULL DEFAULT '1' COMMENT '状态: 0=下架, 1=上架',
    `sort_order` int NOT NULL DEFAULT '0' COMMENT '排序值，越小越靠前',
    `description` text COMMENT '产品描述',
    `features` json DEFAULT NULL COMMENT '产品特性（JSON格式）',
    `create_time` datetime DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    `update_time` datetime DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_product_id` (`product_id`),
    KEY `idx_product_type` (`product_type`),
    KEY `idx_member_type` (`member_type`),
    KEY `idx_status` (`status`),
    KEY `idx_sort_order` (`sort_order`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='会员产品配置表';

-- ============================================================
-- 插入示例数据
-- ============================================================

-- 养殖户VIP产品
INSERT INTO `mxx_member_product` (`product_id`, `product_name`, `product_type`, `duration_type`, `duration_value`, `price`, `original_price`, `discount`, `member_type`, `status`, `sort_order`, `description`, `features`) VALUES
('breeder_vip_1month', '养殖户VIP月卡', 'breeder_vip', 1, 1, 29.00, 39.00, 0.74, 2, 1, 1, '养殖户VIP会员，享受1个月特权', '["无限添加宠物", "数据分析报表", "优先客服支持", "专属标识"]'),
('breeder_vip_3month', '养殖户VIP季卡', 'breeder_vip', 2, 1, 79.00, 117.00, 0.68, 2, 1, 2, '养殖户VIP会员，享受3个月特权', '["无限添加宠物", "数据分析报表", "优先客服支持", "专属标识"]'),
('breeder_vip_1year', '养殖户VIP年卡', 'breeder_vip', 3, 1, 299.00, 468.00, 0.64, 2, 1, 3, '养殖户VIP会员，享受12个月特权', '["无限添加宠物", "数据分析报表", "优先客服支持", "专属标识", "专属活动"]');

-- 商户VIP产品
INSERT INTO `mxx_member_product` (`product_id`, `product_name`, `product_type`, `duration_type`, `duration_value`, `price`, `original_price`, `discount`, `member_type`, `status`, `sort_order`, `description`, `features`) VALUES
('shop_vip_1month', '商户VIP月卡', 'shop_vip', 1, 1, 99.00, 129.00, 0.77, 3, 1, 10, '商户VIP会员，享受1个月特权', '["商品管理", "订单管理", "营销工具", "数据分析"]'),
('shop_vip_3month', '商户VIP季卡', 'shop_vip', 2, 1, 269.00, 387.00, 0.70, 3, 1, 11, '商户VIP会员，享受3个月特权', '["商品管理", "订单管理", "营销工具", "数据分析"]'),
('shop_vip_1year', '商户VIP年卡', 'shop_vip', 3, 1, 999.00, 1548.00, 0.65, 3, 1, 12, '商户VIP会员，享受12个月特权', '["商品管理", "订单管理", "营销工具", "数据分析", "专属客服"]');

-- 普通VIP产品
INSERT INTO `mxx_member_product` (`product_id`, `product_name`, `product_type`, `duration_type`, `duration_value`, `price`, `original_price`, `discount`, `member_type`, `status`, `sort_order`, `description`, `features`) VALUES
('normal_vip_1month', '普通VIP月卡', 'normal_vip', 1, 1, 9.90, 19.90, 0.50, 1, 1, 20, '普通VIP会员，享受1个月特权', '["专属标识", "优先客服"]'),
('normal_vip_1year', '普通VIP年卡', 'normal_vip', 3, 1, 99.00, 238.80, 0.41, 1, 1, 21, '普通VIP会员，享受12个月特权', '["专属标识", "优先客服", "专属活动"]');

-- 体验产品（特殊产品，价格1元，体验7天）
INSERT INTO `mxx_member_product` (`product_id`, `product_name`, `product_type`, `duration_type`, `duration_value`, `price`, `original_price`, `discount`, `member_type`, `status`, `sort_order`, `description`, `features`) VALUES
('breeder_vip_experience', '养殖户VIP体验', 'breeder_vip', 1, 0, 1.00, 29.00, 0.03, 2, 1, 0, '养殖户VIP体验7天', '["体验所有VIP功能", "7天有效期"]');
