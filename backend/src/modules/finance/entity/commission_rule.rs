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

    /// 规则类型/方案类型: 1=个人提成 2=团队分成 3=部门经理 4=总监 5=团队长
    pub rule_type: Option<i32>,

    /// 适用范围: 1=指定部门 2=全公司 3=指定岗位 4=指定人员
    pub apply_scope: Option<i32>,

    /// 提成目标岗位类型（经理/总监/团队长的岗位标识）
    pub commission_target_type: Option<i32>,

    /// 优先级（数字越小越先计算）
    pub priority: Option<i32>,

    /// 计算基准: 1=个人月累计 2=团队月累计 3=单笔合同 4=单笔回款
    pub calc_base_type: Option<i32>,

    /// 触发条件: 1=完全回款 2=合同签订 3=部分回款 4=发货完成 5=客户验收
    pub trigger_condition: Option<i32>,

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
