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
#[sea_orm(table_name = "mxx_crm_followup")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub lead_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub opportunity_id: Option<i64>,
    pub activity_type: Option<i32>,
    /// 跟进来源类型：1=线索跟进, 2=客户跟进, 3=商机跟进
    pub source_type: Option<i16>,
    pub content: Option<String>,
    pub next_follow_date: Option<Date>,
    pub duration_minutes: Option<i32>,
    pub result: Option<String>,
    pub assigned_to: Option<i64>,
    pub created_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub updated_by: Option<i64>,
    pub update_time: Option<DateTime>,
    /// 删除人ID（回收站）
    pub delete_by: Option<i64>,
    /// 删除时间（回收站保留期计算）
    pub delete_time: Option<DateTime>,
    pub deleted: Option<i32>,
    /// 签到地址
    pub visit_address: Option<String>,
    /// 纬度
    pub visit_latitude: Option<Decimal>,
    /// 经度
    pub visit_longitude: Option<Decimal>,
    /// 定位精度(米)
    pub visit_accuracy: Option<Decimal>,
    /// 现场照片(URL数组)
    pub visit_photos: Option<serde_json::Value>,
    /// 距客户距离(米)
    pub visit_distance: Option<Decimal>,
    /// 签到时间
    pub check_in_time: Option<DateTime>,
    /// 签退时间
    pub check_out_time: Option<DateTime>,
    /// 客户坐标(签到时快照)-纬度
    pub visit_customer_lat: Option<Decimal>,
    /// 客户坐标(签到时快照)-经度
    pub visit_customer_lng: Option<Decimal>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}