//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_finance_commission_result")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 薪资记录ID
    pub salary_record_id: Option<i64>,

    /// 合同ID
    pub contract_id: Option<i64>,

    /// 合同名称
    pub contract_name: Option<String>,

    /// 规则ID
    pub rule_id: i64,

    /// 规则名称
    pub rule_name: Option<String>,

    /// 规则类型/方案类型(旧字段)
    pub rule_type: i32,

    /// 提成性质(新): 1=个人提成 2=管理分润 3=团队激励奖金 4=团建资金池 5=总提成再分配 6=利润提成
    pub commission_category: Option<i16>,

    /// 受益岗位(新): 1=销售本人 2=直属主管 3=部门经理 4=总监 5=总经理 6=自定义岗位
    pub beneficiary_role: Option<i16>,

    /// 管理者层级(1=主管 2=经理 3=总监)，仅 category=2 时有值
    pub manager_level: Option<i32>,

    /// 分配状态: 0=无需分配 1=待分配 2=已分配
    pub allocate_status: Option<i16>,

    /// 已分配金额(用于追踪分配进度)
    pub allocated_amount: Option<Decimal>,

    /// 关联资金池ID(仅 category=4 时有值)
    pub pool_id: Option<i64>,

    /// 成本金额(仅 category=6 利润提成时有值)
    pub cost_amount: Option<Decimal>,

    /// 用户ID
    pub user_id: i64,

    /// 用户名称
    pub user_name: Option<String>,

    /// 用户岗位ID
    pub user_post_id: Option<i64>,

    /// 部门ID
    pub department_id: Option<i64>,

    /// 计算基准金额
    pub calc_base_amount: Decimal,

    /// 档位最低金额
    pub tier_min_amount: Option<Decimal>,

    /// 档位最高金额
    pub tier_max_amount: Option<Decimal>,

    /// 提成比例
    pub commission_rate: Decimal,

    /// 分配比例
    pub share_ratio: Option<Decimal>,

    /// 提成金额
    pub commission_amount: Decimal,

    /// 触发条件
    pub trigger_condition: i32,

    /// 触发来源ID
    pub trigger_source_id: Option<i64>,

    /// 周期年
    pub period_year: i32,

    /// 周期月
    pub period_month: i32,

    /// 是否已结算
    pub settled: i32,

    /// 备注
    pub remark: Option<String>,

    /// 创建时间
    pub create_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
