-- ============================================================
-- 财务模块数据库表结构
-- 包含：支付记录表、会员费用表、退款记录表、财务统计表
-- ============================================================

-- 支付记录表
CREATE TABLE `mxx_payment_record` (
    `id` bigint UNSIGNED NOT NULL AUTO_INCREMENT COMMENT '支付记录ID',
    `user_id` bigint UNSIGNED NOT NULL DEFAULT '0' COMMENT '用户ID',
    `order_id` varchar(64) DEFAULT NULL COMMENT '订单ID',
    `payment_type` tinyint DEFAULT '1' COMMENT '支付类型: 1=在线支付, 2=会员费用, 3=充值, 4=其他',
    `amount` decimal(10,2) NOT NULL DEFAULT '0.00' COMMENT '支付金额',
    `pay_method` tinyint DEFAULT '1' COMMENT '支付方式: 1=微信支付, 2=支付宝, 3=银行卡',
    `status` tinyint DEFAULT '0' COMMENT '支付状态: 0=待支付, 1=支付成功, 2=支付失败, 3=已退款',
    `transaction_id` varchar(128) DEFAULT NULL COMMENT '第三方支付订单号',
    `pay_time` datetime DEFAULT NULL COMMENT '支付时间',
    `remark` varchar(255) DEFAULT NULL COMMENT '备注',
    `create_time` datetime DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    `update_time` datetime DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    PRIMARY KEY (`id`),
    KEY `idx_user_id` (`user_id`),
    KEY `idx_order_id` (`order_id`),
    KEY `idx_status` (`status`),
    KEY `idx_payment_type` (`payment_type`),
    KEY `idx_create_time` (`create_time`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='支付记录表';

-- 会员费用表
CREATE TABLE `mxx_member_fee` (
    `id` bigint UNSIGNED NOT NULL AUTO_INCREMENT COMMENT '会员费用记录ID',
    `user_id` bigint UNSIGNED NOT NULL DEFAULT '0' COMMENT '用户ID',
    `member_type` tinyint DEFAULT '1' COMMENT '会员类型: 1=普通会员, 2=养殖户, 3=商户',
    `amount` decimal(10,2) NOT NULL DEFAULT '0.00' COMMENT '费用金额',
    `valid_start_time` datetime DEFAULT NULL COMMENT '会员有效期开始时间',
    `valid_end_time` datetime DEFAULT NULL COMMENT '会员有效期结束时间',
    `status` tinyint DEFAULT '0' COMMENT '支付状态: 0=待支付, 1=已支付, 2=已过期',
    `payment_record_id` bigint UNSIGNED DEFAULT NULL COMMENT '关联支付记录ID',
    `remark` varchar(255) DEFAULT NULL COMMENT '备注',
    `create_time` datetime DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    `update_time` datetime DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    PRIMARY KEY (`id`),
    KEY `idx_user_id` (`user_id`),
    KEY `idx_status` (`status`),
    KEY `idx_member_type` (`member_type`),
    KEY `idx_payment_record_id` (`payment_record_id`),
    KEY `idx_create_time` (`create_time`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='会员费用记录表';

-- 退款记录表
CREATE TABLE `mxx_refund_record` (
    `id` bigint UNSIGNED NOT NULL AUTO_INCREMENT COMMENT '退款记录ID',
    `user_id` bigint UNSIGNED NOT NULL DEFAULT '0' COMMENT '用户ID',
    `payment_record_id` bigint UNSIGNED NOT NULL DEFAULT '0' COMMENT '关联支付记录ID',
    `amount` decimal(10,2) NOT NULL DEFAULT '0.00' COMMENT '退款金额',
    `status` tinyint DEFAULT '0' COMMENT '退款状态: 0=待审核, 1=审核通过, 2=退款成功, 3=退款失败, 4=审核拒绝',
    `transaction_id` varchar(128) DEFAULT NULL COMMENT '第三方退款订单号',
    `refund_time` datetime DEFAULT NULL COMMENT '退款时间',
    `reason` varchar(255) DEFAULT NULL COMMENT '退款原因',
    `remark` varchar(255) DEFAULT NULL COMMENT '备注',
    `create_time` datetime DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    `update_time` datetime DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    PRIMARY KEY (`id`),
    KEY `idx_user_id` (`user_id`),
    KEY `idx_payment_record_id` (`payment_record_id`),
    KEY `idx_status` (`status`),
    KEY `idx_create_time` (`create_time`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='退款记录表';

-- 财务统计表
CREATE TABLE `mxx_finance_statistics` (
    `id` bigint UNSIGNED NOT NULL AUTO_INCREMENT COMMENT '统计记录ID',
    `stat_date` date DEFAULT NULL COMMENT '统计日期',
    `stat_type` tinyint DEFAULT '1' COMMENT '统计类型: 1=日报, 2=周报, 3=月报',
    `total_income` decimal(15,2) NOT NULL DEFAULT '0.00' COMMENT '总收入金额',
    `success_amount` decimal(15,2) NOT NULL DEFAULT '0.00' COMMENT '支付成功金额',
    `refund_amount` decimal(15,2) NOT NULL DEFAULT '0.00' COMMENT '退款金额',
    `member_fee_amount` decimal(15,2) NOT NULL DEFAULT '0.00' COMMENT '会员费用收入',
    `order_count` bigint DEFAULT '0' COMMENT '订单数量',
    `success_count` bigint DEFAULT '0' COMMENT '支付成功数量',
    `refund_count` bigint DEFAULT '0' COMMENT '退款数量',
    `create_time` datetime DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    `update_time` datetime DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_stat_date_type` (`stat_date`, `stat_type`),
    KEY `idx_stat_type` (`stat_type`),
    KEY `idx_stat_date` (`stat_date`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='财务统计表';

-- ============================================================
-- 示例数据（可选）
-- ============================================================

-- 插入支付记录示例
-- INSERT INTO `mxx_payment_record` (`user_id`, `order_id`, `payment_type`, `amount`, `pay_method`, `status`, `transaction_id`, `pay_time`, `remark`) VALUES
-- (1, 'ORDER202501010001', 1, 199.99, 1, 1, 'WX123456789', '2025-01-01 10:30:00', '订单支付');

-- 插入会员费用示例
-- INSERT INTO `mxx_member_fee` (`user_id`, `member_type`, `amount`, `valid_start_time`, `valid_end_time`, `status`, `payment_record_id`, `remark`) VALUES
-- (1, 2, 299.00, '2025-01-01 00:00:00', '2026-01-01 00:00:00', 1, 1, 'VIP会员年费');

-- 插入退款记录示例
-- INSERT INTO `mxx_refund_record` (`user_id`, `payment_record_id`, `amount`, `status`, `transaction_id`, `refund_time`, `reason`, `remark`) VALUES
-- (1, 1, 199.99, 2, 'REFUND202501010001', '2025-01-02 14:30:00', '用户申请退款', '退款成功');

-- 插入财务统计示例
-- INSERT INTO `mxx_finance_statistics` (`stat_date`, `stat_type`, `total_income`, `success_amount`, `refund_amount`, `member_fee_amount`, `order_count`, `success_count`, `refund_count`) VALUES
-- ('2025-01-01', 1, 499.99, 499.99, 199.99, 299.00, 2, 2, 1);
