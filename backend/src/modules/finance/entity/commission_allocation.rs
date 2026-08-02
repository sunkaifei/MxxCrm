//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 提成分配记录实体
//!

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_finance_commission_allocation")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 关联的提成结果ID
    pub commission_result_id: i64,

    /// 分配人ID(管理者)
    pub allocator_id: i64,

    /// 被分配员工ID
    pub employee_id: i64,

    /// 被分配员工姓名
    pub employee_name: Option<String>,

    /// 分配金额
    pub amount: Decimal,

    /// 分配方式: 1=平均 2=按业绩比例 3=手动
    pub allocate_method: i16,

    /// 该员工当期业绩(回款额)
    pub employee_payment: Option<Decimal>,

    /// 团队总业绩(回款额)
    pub team_total_payment: Option<Decimal>,

    /// 关联工资记录ID
    pub salary_record_id: Option<i64>,

    /// 年
    pub year: i32,

    /// 月
    pub month: i32,

    /// 备注
    pub remark: Option<String>,

    /// 创建时间
    pub create_time: DateTime,

    /// 删除标识: 0=未删除 1=已删除
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
