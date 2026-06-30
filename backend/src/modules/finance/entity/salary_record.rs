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
#[sea_orm(table_name = "mxx_finance_salary_record")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 员工ID
    pub employee_id: i64,

    /// 员工姓名
    pub employee_name: Option<String>,

    /// 部门名称
    pub department_name: Option<String>,

    /// 年份
    pub year: i32,

    /// 月份
    pub month: i32,

    /// 基本工资
    pub base_salary: Decimal,

    /// 提成金额
    pub commission_amount: Decimal,

    /// 绩效奖金
    pub performance_bonus: Decimal,

    /// 扣款金额
    pub deduction_amount: Decimal,

    /// 应发工资
    pub total_salary: Decimal,

    /// 状态: 0=待审核 1=已审核 2=已发放
    pub status: Option<i32>,

    /// 备注
    pub remark: Option<String>,

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
