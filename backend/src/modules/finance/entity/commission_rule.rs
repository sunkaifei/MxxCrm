//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_finance_commission_rule")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 规则名称
    pub rule_name: Option<String>,

    /// 部门ID
    pub department_id: Option<i64>,

    /// 岗位ID
    pub post_id: Option<i64>,

    /// 规则类型/方案类型(旧字段,保留兼容): 1=个人提成 2=团队分成 3=部门经理 4=总监 5=团队长
    pub rule_type: Option<i32>,

    /// 提成性质(新): 1=个人提成 2=管理分润 3=团队激励奖金 4=团建资金池 5=总提成再分配 6=利润提成
    pub commission_category: i16,

    /// 受益岗位(新): 1=销售本人 2=直属主管 3=部门经理 4=总监 5=总经理 6=自定义岗位
    pub beneficiary_role: i16,

    /// 计算方式(新): 1=按比例 2=固定金额(达标后) 3=阶梯累进 4=超额递增
    pub calc_method: i16,

    /// 达标门槛(calc_method=2时使用)
    pub bonus_target: Option<Decimal>,

    /// 固定奖金金额(calc_method=2时使用)
    pub bonus_fixed_amount: Option<Decimal>,

    /// 单笔提成封顶(NULL=不封顶)
    pub commission_cap: Option<Decimal>,

    /// 月度提成保底(NULL=不保底)
    pub commission_floor: Option<Decimal>,

    /// 客户分类筛选: new=仅新客户 old=仅老客户 NULL=全部
    pub customer_category: Option<String>,

    /// 递延发放月数: 0=随当月发 N=分N个月递延
    pub defer_months: i32,

    /// 关联资金池ID(category=4时使用)
    pub pool_id: Option<i64>,

    /// 适用范围: 1=指定部门 2=全公司 3=指定岗位 4=指定人员
    pub apply_scope: Option<i32>,

    /// 提成目标岗位类型（经理/总监/团队长的岗位标识）
    pub commission_target_type: Option<i32>,

    /// 优先级（数字越小越先计算）
    pub priority: Option<i32>,

    /// 计算基准: 1=个人月累计 2=团队月累计 3=单笔合同 4=单笔回款
    pub calc_base_type: Option<i32>,

    /// 提成基数字段: payment_amount=回款额(默认), contract_amount=合同额, net_amount=净回款额, profit=毛利
    /// 当为 None 或 "payment_amount" 时按回款额计算
    pub calc_base_field: Option<String>,

    /// 阶梯模式: 0=单档命中(默认,命中最高的阶梯), 1=累进(分段累计), 2=超额递增(超额部分按高档率)
    pub tier_mode: Option<i32>,

    /// 触发条件: 1=完全回款 2=合同签订 3=部分回款 4=发货完成 5=客户验收
    pub trigger_condition: Option<i32>,

    /// P2-3: 产品线维度（可选，用于按产品线差异化提成）
    pub product_line: Option<String>,

    /// P2-3: 区域编码维度（可选，用于按区域差异化提成）
    pub region_code: Option<String>,

    /// P2-3: 客户类型维度（可选，如 VIP/普通/战略等）
    pub customer_type: Option<String>,

    /// 生效日期
    pub effective_date: Option<chrono::NaiveDate>,

    /// 失效日期
    pub expiry_date: Option<chrono::NaiveDate>,

    /// 是否默认方案: 0=否 1=是
    pub is_default: Option<i32>,

    /// 是否启用: 0=禁用 1=启用
    pub enabled: Option<i32>,

    /// 描述
    pub description: Option<String>,

    /// 创建人ID
    pub created_by: Option<i64>,

    /// 创建时间
    pub create_time: Option<DateTime>,

    /// 更新人ID
    pub updated_by: Option<i64>,

    /// 更新时间
    pub update_time: Option<DateTime>,

    /// 删除标识: 0=未删除 1=已删除
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
