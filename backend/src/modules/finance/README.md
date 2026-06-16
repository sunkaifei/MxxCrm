# 财务模块 (Finance Module)

## 模块概述

财务模块负责处理在线支付、会员费用、退款记录和财务统计等核心财务功能。

## 目录结构

```
src/modules/finance/
├── entity/                         # 数据库实体
│   ├── mod.rs
│   ├── payment_record.rs          # 支付记录表实体
│   ├── member_fee.rs              # 会员费用记录表实体
│   ├── refund_record.rs           # 退款记录表实体
│   └── finance_statistics.rs      # 财务统计表实体
├── model/                          # 数据模型（DTO）
│   ├── mod.rs
│   ├── payment_record.rs          # 支付记录数据模型
│   ├── member_fee.rs              # 会员费用数据模型
│   ├── refund_record.rs           # 退款记录数据模型
│   └── finance_statistics.rs      # 财务统计数据模型
├── service/                        # 业务逻辑服务
│   ├── mod.rs
│   ├── payment_record_service.rs      # 支付记录服务
│   ├── member_fee_service.rs          # 会员费用服务
│   ├── refund_record_service.rs       # 退款记录服务
│   └── finance_statistics_service.rs  # 财务统计服务
├── controller/                     # 控制器（API接口）
│   ├── mod.rs
│   ├── admin/                     # 管理员端接口
│   │   ├── mod.rs
│   │   ├── payment_admin_controller.rs      # 支付记录管理
│   │   ├── member_fee_admin_controller.rs   # 会员费用管理
│   │   ├── refund_admin_controller.rs       # 退款记录管理
│   │   └── statistics_admin_controller.rs   # 财务统计
│   └── user/                      # 用户端接口
│       ├── mod.rs
│       ├── payment_user_controller.rs      # 用户支付接口
│       └── member_fee_user_controller.rs   # 用户会员接口
└── mod.rs                         # 模块入口
```

## 数据库表设计

### 1. 支付记录表 (mxx_payment_record)

| 字段名 | 类型 | 说明 |
|-------|------|------|
| id | BIGINT | 主键，自增 |
| user_id | BIGINT | 用户ID |
| order_id | VARCHAR(64) | 订单ID |
| payment_type | TINYINT | 支付类型: 1=在线支付, 2=会员费用, 3=充值, 4=其他 |
| amount | DECIMAL(10,2) | 支付金额 |
| pay_method | TINYINT | 支付方式: 1=微信支付, 2=支付宝, 3=银行卡 |
| status | TINYINT | 支付状态: 0=待支付, 1=支付成功, 2=支付失败, 3=已退款 |
| transaction_id | VARCHAR(128) | 第三方支付订单号 |
| pay_time | DATETIME | 支付时间 |
| remark | VARCHAR(255) | 备注 |
| create_time | DATETIME | 创建时间 |
| update_time | DATETIME | 更新时间 |

### 2. 会员费用表 (mxx_member_fee)

| 字段名 | 类型 | 说明 |
|-------|------|------|
| id | BIGINT | 主键，自增 |
| user_id | BIGINT | 用户ID |
| member_type | TINYINT | 会员类型: 1=普通会员, 2=养殖户, 3=商户 |
| amount | DECIMAL(10,2) | 费用金额 |
| valid_start_time | DATETIME | 会员有效期开始时间 |
| valid_end_time | DATETIME | 会员有效期结束时间 |
| status | TINYINT | 支付状态: 0=待支付, 1=已支付, 2=已过期 |
| payment_record_id | BIGINT | 关联支付记录ID |
| remark | VARCHAR(255) | 备注 |
| create_time | DATETIME | 创建时间 |
| update_time | DATETIME | 更新时间 |

### 3. 退款记录表 (mxx_refund_record)

| 字段名 | 类型 | 说明 |
|-------|------|------|
| id | BIGINT | 主键，自增 |
| user_id | BIGINT | 用户ID |
| payment_record_id | BIGINT | 关联支付记录ID |
| amount | DECIMAL(10,2) | 退款金额 |
| status | TINYINT | 退款状态: 0=待审核, 1=审核通过, 2=退款成功, 3=退款失败, 4=审核拒绝 |
| transaction_id | VARCHAR(128) | 第三方退款订单号 |
| refund_time | DATETIME | 退款时间 |
| reason | VARCHAR(255) | 退款原因 |
| remark | VARCHAR(255) | 备注 |
| create_time | DATETIME | 创建时间 |
| update_time | DATETIME | 更新时间 |

### 4. 财务统计表 (mxx_finance_statistics)

| 字段名 | 类型 | 说明 |
|-------|------|------|
| id | BIGINT | 主键，自增 |
| stat_date | DATE | 统计日期 |
| stat_type | TINYINT | 统计类型: 1=日报, 2=周报, 3=月报 |
| total_income | DECIMAL(15,2) | 总收入金额 |
| success_amount | DECIMAL(15,2) | 支付成功金额 |
| refund_amount | DECIMAL(15,2) | 退款金额 |
| member_fee_amount | DECIMAL(15,2) | 会员费用收入 |
| order_count | BIGINT | 订单数量 |
| success_count | BIGINT | 支付成功数量 |
| refund_count | BIGINT | 退款数量 |
| create_time | DATETIME | 创建时间 |
| update_time | DATETIME | 更新时间 |

## API 接口列表

### 管理员接口 (Admin APIs)

#### 支付记录管理

| 接口名称 | 请求路径 | 请求方法 | 说明 |
|---------|---------|---------|------|
| 支付记录列表 | /api/admin/payment/list | GET | 获取支付记录列表（支持分页、筛选） |
| 支付记录详情 | /api/admin/payment/detail/{id} | GET | 获取指定支付记录详情 |
| 创建支付记录 | /api/admin/payment/create | POST | 创建新的支付记录 |
| 更新支付记录 | /api/admin/payment/update/{id} | PUT | 更新指定支付记录 |
| 删除支付记录 | /api/admin/payment/delete/{id} | DELETE | 删除指定支付记录 |

#### 会员费用管理

| 接口名称 | 请求路径 | 请求方法 | 说明 |
|---------|---------|---------|------|
| 会员费用列表 | /api/admin/member-fee/list | GET | 获取会员费用记录列表 |
| 会员费用详情 | /api/admin/member-fee/detail/{id} | GET | 获取指定会员费用详情 |
| 创建会员费用 | /api/admin/member-fee/create | POST | 创建新的会员费用记录 |
| 更新会员费用 | /api/admin/member-fee/update/{id} | PUT | 更新指定会员费用记录 |
| 删除会员费用 | /api/admin/member-fee/delete/{id} | DELETE | 删除指定会员费用记录 |

#### 退款记录管理

| 接口名称 | 请求路径 | 请求方法 | 说明 |
|---------|---------|---------|------|
| 退款记录列表 | /api/admin/refund/list | GET | 获取退款记录列表 |
| 退款记录详情 | /api/admin/refund/detail/{id} | GET | 获取指定退款记录详情 |
| 创建退款记录 | /api/admin/refund/create | POST | 创建新的退款记录 |
| 更新退款记录 | /api/admin/refund/update/{id} | PUT | 更新指定退款记录 |
| 删除退款记录 | /api/admin/refund/delete/{id} | DELETE | 删除指定退款记录 |

#### 财务统计

| 接口名称 | 请求路径 | 请求方法 | 说明 |
|---------|---------|---------|------|
| 财务汇总 | /api/admin/statistics/summary | GET | 获取今日财务汇总数据 |
| 统计列表 | /api/admin/statistics/list | GET | 获取历史统计数据 |
| 生成日报 | /api/admin/statistics/generate-daily | GET | 手动生成昨日日报统计 |

### 用户接口 (User APIs)

#### 用户支付

| 接口名称 | 请求路径 | 请求方法 | 说明 |
|---------|---------|---------|------|
| 我的支付记录 | /api/user/payment/list | GET | 获取当前用户的支付记录 |
| 支付详情 | /api/user/payment/detail/{id} | GET | 获取指定支付记录详情 |
| 创建支付 | /api/user/payment/create | POST | 创建新的支付订单 |

#### 用户会员

| 接口名称 | 请求路径 | 请求方法 | 说明 |
|---------|---------|---------|------|
| 我的会员信息 | /api/user/member-fee/info | GET | 获取当前用户的会员信息 |
| 购买会员 | /api/user/member-fee/purchase | POST | 购买/续费会员 |

## 状态码说明

### 支付状态 (status)
- `0` - 待支付
- `1` - 支付成功
- `2` - 支付失败
- `3` - 已退款

### 支付类型 (payment_type)
- `1` - 在线支付
- `2` - 会员费用
- `3` - 充值
- `4` - 其他

### 支付方式 (pay_method)
- `1` - 微信支付
- `2` - 支付宝
- `3` - 银行卡

### 退款状态 (status)
- `0` - 待审核
- `1` - 审核通过
- `2` - 退款成功
- `3` - 退款失败
- `4` - 审核拒绝

### 会员类型 (member_type)
- `1` - 普通会员
- `2` - 养殖户
- `3` - 商户

### 会员状态 (status)
- `0` - 待支付
- `1` - 已支付
- `2` - 已过期

### 统计类型 (stat_type)
- `1` - 日报
- `2` - 周报
- `3` - 月报

## 服务层功能说明

### PaymentRecordService (支付记录服务)

| 方法名 | 说明 |
|-------|------|
| get_list | 获取支付记录列表（支持分页、筛选） |
| get_by_id | 根据ID获取支付记录详情 |
| insert | 创建新的支付记录 |
| update | 更新支付记录 |
| delete | 删除支付记录 |

### MemberFeeService (会员费用服务)

| 方法名 | 说明 |
|-------|------|
| get_list | 获取会员费用记录列表 |
| get_by_id | 根据ID获取会员费用详情 |
| get_by_user_id | 根据用户ID获取会员信息 |
| insert | 创建新的会员费用记录 |
| update | 更新会员费用记录 |
| delete | 删除会员费用记录 |

### RefundRecordService (退款记录服务)

| 方法名 | 说明 |
|-------|------|
| get_list | 获取退款记录列表 |
| get_by_id | 根据ID获取退款记录详情 |
| insert | 创建新的退款记录 |
| update | 更新退款记录 |
| delete | 删除退款记录 |

### FinanceStatisticsService (财务统计服务)

| 方法名 | 说明 |
|-------|------|
| get_list | 获取历史统计数据列表 |
| get_summary | 获取今日财务汇总数据 |
| generate_daily_statistics | 生成昨日日报统计数据 |

## 使用示例

### 创建支付记录

```rust
use crate::modules::finance::model::payment_record::PaymentRecordSaveRequest;
use crate::modules::finance::service::payment_record_service;

let request = PaymentRecordSaveRequest {
    user_id: 1,
    order_id: Some("ORDER202501010001".to_string()),
    payment_type: Some(1),
    amount: 199.99,
    pay_method: Some(1),
    status: Some(1),
    transaction_id: Some("WX123456789".to_string()),
    pay_time: Some("2025-01-01 10:30:00".to_string()),
    remark: Some("订单支付".to_string()),
};

let result = payment_record_service::insert(&db, request).await;
```

### 购买会员

```rust
use crate::modules::finance::model::member_fee::MemberFeeSaveRequest;
use crate::modules::finance::service::member_fee_service;

let request = MemberFeeSaveRequest {
    user_id: 1,
    member_type: Some(2),  // VIP会员
    amount: 299.00,
    valid_start_time: Some("2025-01-01 00:00:00".to_string()),
    valid_end_time: Some("2026-01-01 00:00:00".to_string()),
    status: Some(1),
    payment_record_id: Some(1001),
    remark: Some("VIP会员年费".to_string()),
};

let result = member_fee_service::insert(&db, request).await;
```

## 注意事项

1. 所有金额字段使用 `DECIMAL` 类型，避免浮点数精度问题
2. 支付记录创建后，应通过第三方支付接口完成实际支付流程
3. 会员费用有效期过期后，需要将状态更新为 `2=已过期`
4. 退款记录创建后，需要审核通过才能执行退款操作
5. 财务统计数据建议每日凌晨通过定时任务自动生成
