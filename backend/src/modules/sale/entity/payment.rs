use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_sale_payment")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub payment_no: Option<String>,
    pub contract_id: Option<i64>,
    pub order_id: Option<i64>,
    pub plan_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub customer_name: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<i32>,
    pub payment_method: Option<i32>,
    pub payment_date: Option<NaiveDate>,
    pub payer: Option<String>,
    pub payer_account: Option<String>,
    pub bank_flow_no: Option<String>,
    pub attachment: Option<String>,
    pub status: Option<i32>,
    pub remark: Option<String>,
    pub owner_user_id: Option<i64>,
    pub dept_id: Option<i64>,
    pub create_by: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
    pub confirm_time: Option<DateTime>,
    pub confirm_by: Option<String>,
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::order::Entity",
        from = "Column::OrderId",
        to = "super::order::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Order,
}

impl Related<super::order::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Order.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
