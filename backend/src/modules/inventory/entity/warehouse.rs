use sea_orm::entity::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_inventory_warehouse")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    /// 仓库ID
    pub id: i64,
    /// 仓库名称
    pub name: Option<String>,
    /// 仓库编码
    pub code: Option<String>,
    /// 仓库地址
    pub address: Option<String>,
    /// 联系人
    pub contact_person: Option<String>,
    /// 联系电话
    pub contact_phone: Option<String>,
    /// 是否启用
    pub is_active: Option<bool>,
    /// 创建时间
    pub created_at: Option<DateTime<Utc>>,
    /// 更新时间
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
