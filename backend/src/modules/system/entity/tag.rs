use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_tag")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub group_id: Option<i64>,
    pub tag_name: Option<String>,
    pub tag_color: Option<String>,
    pub description: Option<String>,
    pub is_global: Option<bool>,
    pub created_by: Option<i64>,
    pub status: Option<i32>,
    pub create_time: Option<DateTime>,
    pub updated_by: Option<i64>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tag_group::Entity",
        from = "Column::GroupId",
        to = "super::tag_group::Column::Id"
    )]
    TagGroup,
}

impl ActiveModelBehavior for ActiveModel {}
