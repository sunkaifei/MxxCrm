//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
use sea_orm::*;
use sea_orm::prelude::{DateTime, Date};
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::crm::entity::{followup, followup::Entity as Followup};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

/// 跟进记录新增请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct FollowupSaveRequest {
    pub lead_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub opportunity_id: Option<i64>,
    pub activity_type: Option<i32>,
    /// 跟进来源类型：1=线索跟进, 2=客户跟进, 3=商机跟进
    /// 若未传，则按 lead_id/customer_id 自动判断
    pub source_type: Option<i16>,
    pub content: Option<String>,
    pub next_follow_date: Option<Date>,
    pub duration_minutes: Option<i32>,
    pub result: Option<String>,
    pub assigned_to: Option<i64>,
    pub lead_status: Option<i32>,
    /// 签到地址
    pub visit_address: Option<String>,
    /// 纬度
    pub visit_latitude: Option<f64>,
    /// 经度
    pub visit_longitude: Option<f64>,
    /// 定位精度(米)
    pub visit_accuracy: Option<f64>,
    /// 现场照片(URL数组)
    pub visit_photos: Option<serde_json::Value>,
    /// 距客户距离(米)
    pub visit_distance: Option<f64>,
    /// 签到时间
    pub check_in_time: Option<DateTime>,
    /// 签退时间
    pub check_out_time: Option<DateTime>,
    /// 客户坐标(签到时快照)-纬度
    pub visit_customer_lat: Option<f64>,
    /// 客户坐标(签到时快照)-经度
    pub visit_customer_lng: Option<f64>,
}

impl From<FollowupSaveRequest> for FollowupSaveDTO {
    fn from(item: FollowupSaveRequest) -> Self {
        // 自动推断 source_type：未传则按 customer_id/lead_id 判断
        let source_type: i16 = item.source_type.unwrap_or_else(|| {
            if item.customer_id.is_some() {
                2
            } else if item.lead_id.is_some() {
                1
            } else if item.opportunity_id.is_some() {
                3
            } else {
                1
            }
        });
        FollowupSaveDTO {
            id: None,
            lead_id: item.lead_id,
            customer_id: item.customer_id,
            opportunity_id: item.opportunity_id,
            activity_type: item.activity_type,
            source_type: Some(source_type),
            content: item.content,
            next_follow_date: item.next_follow_date,
            duration_minutes: item.duration_minutes,
            result: item.result,
            assigned_to: None,
            deleted: None,
            created_by: None,
            create_time: None,
            updated_by: None,
            update_time: None,
            visit_address: item.visit_address,
            visit_latitude: item.visit_latitude.map(|v| Decimal::from_f64(v).unwrap_or_default()),
            visit_longitude: item.visit_longitude.map(|v| Decimal::from_f64(v).unwrap_or_default()),
            visit_accuracy: item.visit_accuracy.map(|v| Decimal::from_f64(v).unwrap_or_default()),
            visit_photos: item.visit_photos,
            visit_distance: item.visit_distance.map(|v| Decimal::from_f64(v).unwrap_or_default()),
            check_in_time: item.check_in_time,
            check_out_time: item.check_out_time,
            visit_customer_lat: item.visit_customer_lat.map(|v| Decimal::from_f64(v).unwrap_or_default()),
            visit_customer_lng: item.visit_customer_lng.map(|v| Decimal::from_f64(v).unwrap_or_default()),
        }
    }
}

/// 跟进记录更新请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct FollowupUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    pub lead_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub opportunity_id: Option<i64>,
    pub activity_type: Option<i32>,
    pub source_type: Option<i16>,
    pub content: Option<String>,
    pub next_follow_date: Option<Date>,
    pub duration_minutes: Option<i32>,
    pub result: Option<String>,
    pub assigned_to: Option<i64>,
    /// 签到地址
    pub visit_address: Option<String>,
    /// 纬度
    pub visit_latitude: Option<f64>,
    /// 经度
    pub visit_longitude: Option<f64>,
    /// 定位精度(米)
    pub visit_accuracy: Option<f64>,
    /// 现场照片(URL数组)
    pub visit_photos: Option<serde_json::Value>,
    /// 距客户距离(米)
    pub visit_distance: Option<f64>,
    /// 签到时间
    pub check_in_time: Option<DateTime>,
    /// 签退时间
    pub check_out_time: Option<DateTime>,
    /// 客户坐标(签到时快照)-纬度
    pub visit_customer_lat: Option<f64>,
    /// 客户坐标(签到时快照)-经度
    pub visit_customer_lng: Option<f64>,
}

impl From<FollowupUpdateRequest> for FollowupSaveDTO {
    fn from(item: FollowupUpdateRequest) -> Self {
        FollowupSaveDTO {
            id: item.id,
            lead_id: item.lead_id,
            customer_id: item.customer_id,
            opportunity_id: item.opportunity_id,
            activity_type: item.activity_type,
            source_type: item.source_type,
            content: item.content,
            next_follow_date: item.next_follow_date,
            duration_minutes: item.duration_minutes,
            result: item.result,
            assigned_to: None,
            deleted: None,
            created_by: None,
            create_time: None,
            updated_by: None,
            update_time: None,
            visit_address: item.visit_address,
            visit_latitude: item.visit_latitude.map(|v| Decimal::from_f64(v).unwrap_or_default()),
            visit_longitude: item.visit_longitude.map(|v| Decimal::from_f64(v).unwrap_or_default()),
            visit_accuracy: item.visit_accuracy.map(|v| Decimal::from_f64(v).unwrap_or_default()),
            visit_photos: item.visit_photos,
            visit_distance: item.visit_distance.map(|v| Decimal::from_f64(v).unwrap_or_default()),
            check_in_time: item.check_in_time,
            check_out_time: item.check_out_time,
            visit_customer_lat: item.visit_customer_lat.map(|v| Decimal::from_f64(v).unwrap_or_default()),
            visit_customer_lng: item.visit_customer_lng.map(|v| Decimal::from_f64(v).unwrap_or_default()),
        }
    }
}

/// 跟进记录保存DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct FollowupSaveDTO {
    pub id: Option<i64>,
    pub lead_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub opportunity_id: Option<i64>,
    pub activity_type: Option<i32>,
    pub source_type: Option<i16>,
    pub content: Option<String>,
    pub next_follow_date: Option<Date>,
    pub duration_minutes: Option<i32>,
    pub result: Option<String>,
    pub assigned_to: Option<i64>,
    pub deleted: Option<i32>,
    pub created_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub updated_by: Option<i64>,
    pub update_time: Option<DateTime>,
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

/// 跟进记录详情VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct FollowupDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub lead_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub opportunity_id: Option<i64>,
    pub activity_type: Option<i32>,
    pub source_type: Option<i16>,
    pub content: Option<String>,
    pub next_follow_date: Option<Date>,
    pub duration_minutes: Option<i32>,
    pub result: Option<String>,
    pub assigned_to: Option<i64>,
    pub created_by: Option<i64>,
    pub created_by_name: Option<String>,
    pub create_time: Option<DateTime>,
    pub follow_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
    pub customer_name: Option<String>,
    pub assignee_name: Option<String>,
    /// 签到地址
    pub visit_address: Option<String>,
    /// 纬度
    pub visit_latitude: Option<f64>,
    /// 经度
    pub visit_longitude: Option<f64>,
    /// 定位精度(米)
    pub visit_accuracy: Option<f64>,
    /// 现场照片(URL数组)
    pub visit_photos: Option<serde_json::Value>,
    /// 距客户距离(米)
    pub visit_distance: Option<f64>,
    /// 签到时间
    pub check_in_time: Option<DateTime>,
    /// 签退时间
    pub check_out_time: Option<DateTime>,
    /// 客户坐标(签到时快照)-纬度
    pub visit_customer_lat: Option<f64>,
    /// 客户坐标(签到时快照)-经度
    pub visit_customer_lng: Option<f64>,
}

impl From<followup::Model> for FollowupDetailVO {
    fn from(item: followup::Model) -> Self {
        FollowupDetailVO {
            id: Option::from(item.id),
            lead_id: item.lead_id,
            customer_id: item.customer_id,
            opportunity_id: item.opportunity_id,
            activity_type: item.activity_type,
            source_type: item.source_type,
            content: item.content,
            next_follow_date: item.next_follow_date,
            duration_minutes: item.duration_minutes,
            result: item.result,
            assigned_to: item.assigned_to,
            created_by: item.created_by,
            created_by_name: None,
            create_time: item.create_time,
            follow_time: item.create_time,
            update_time: item.update_time,
            customer_name: None,
            assignee_name: None,
            visit_address: item.visit_address,
            visit_latitude: item.visit_latitude.and_then(|d| d.to_f64()),
            visit_longitude: item.visit_longitude.and_then(|d| d.to_f64()),
            visit_accuracy: item.visit_accuracy.and_then(|d| d.to_f64()),
            visit_photos: item.visit_photos,
            visit_distance: item.visit_distance.and_then(|d| d.to_f64()),
            check_in_time: item.check_in_time,
            check_out_time: item.check_out_time,
            visit_customer_lat: item.visit_customer_lat.and_then(|d| d.to_f64()),
            visit_customer_lng: item.visit_customer_lng.and_then(|d| d.to_f64()),
        }
    }
}

/// 跟进记录列表VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct FollowupListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub lead_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub opportunity_id: Option<i64>,
    pub activity_type: Option<i32>,
    pub source_type: Option<i16>,
    pub content: Option<String>,
    pub next_follow_date: Option<Date>,
    pub result: Option<String>,
    pub assigned_to: Option<i64>,
    pub created_by: Option<i64>,
    pub created_by_name: Option<String>,
    pub create_time: Option<DateTime>,
    pub follow_time: Option<DateTime>,
    pub customer_name: Option<String>,
    pub assignee_name: Option<String>,
    /// 线索名称（source_type=1 时使用，优先 company_name，其次 contact_name）
    pub lead_name: Option<String>,
    /// 签到地址
    pub visit_address: Option<String>,
    /// 纬度
    pub visit_latitude: Option<f64>,
    /// 经度
    pub visit_longitude: Option<f64>,
    /// 距客户距离(米)
    pub visit_distance: Option<f64>,
    /// 现场照片(URL数组)
    pub visit_photos: Option<serde_json::Value>,
    /// 签到时间
    pub check_in_time: Option<DateTime>,
    /// 签退时间
    pub check_out_time: Option<DateTime>,
}

impl From<followup::Model> for FollowupListVO {
    fn from(item: followup::Model) -> Self {
        FollowupListVO {
            id: Option::from(item.id),
            lead_id: item.lead_id,
            customer_id: item.customer_id,
            opportunity_id: item.opportunity_id,
            activity_type: item.activity_type,
            source_type: item.source_type,
            content: item.content,
            next_follow_date: item.next_follow_date,
            result: item.result,
            assigned_to: item.assigned_to,
            created_by: item.created_by,
            created_by_name: None,
            create_time: item.create_time,
            follow_time: item.create_time,
            customer_name: None,
            assignee_name: None,
            lead_name: None,
            visit_address: item.visit_address,
            visit_latitude: item.visit_latitude.and_then(|d| d.to_f64()),
            visit_longitude: item.visit_longitude.and_then(|d| d.to_f64()),
            visit_distance: item.visit_distance.and_then(|d| d.to_f64()),
            visit_photos: item.visit_photos,
            check_in_time: item.check_in_time,
            check_out_time: item.check_out_time,
        }
    }
}

/// 跟进记录列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FollowupListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub customer_id: Option<i64>,
    pub lead_id: Option<i64>,
    pub opportunity_id: Option<i64>,
    /// 仅查询客户跟进记录（customer_id IS NOT NULL）
    pub only_customer: Option<bool>,
    /// 跟进来源类型筛选：1=线索跟进, 2=客户跟进, 3=商机跟进
    pub source_type: Option<i16>,
    /// 列表类型筛选：all=全部, my=我的, subordinate=下属, todayFollow=今日跟进
    pub list_type: Option<String>,
}

// ==================== 外勤拜访相关模型 ====================

/// 外勤拜访签到请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct VisitCheckInRequest {
    pub customer_id: Option<i64>,
    pub lead_id: Option<i64>,
    pub opportunity_id: Option<i64>,
    /// 拜访内容
    pub content: Option<String>,
    /// 签到地址
    pub visit_address: Option<String>,
    /// 纬度
    pub visit_latitude: Option<f64>,
    /// 经度
    pub visit_longitude: Option<f64>,
    /// 定位精度
    pub visit_accuracy: Option<f64>,
    /// 照片URL数组
    pub visit_photos: Option<serde_json::Value>,
    /// 距客户距离
    pub visit_distance: Option<f64>,
    /// 下次跟进日期
    pub next_follow_date: Option<Date>,
    /// 负责人
    pub assigned_to: Option<i64>,
}

/// 外勤拜访列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VisitListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub customer_id: Option<i64>,
    pub lead_id: Option<i64>,
    /// 列表类型筛选：all=全部, my=我的, subordinate=下属
    pub list_type: Option<String>,
}

/// 外勤拜访统计VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct VisitStatisticsVO {
    /// 总拜访次数
    pub total_visits: i64,
    /// 今日拜访
    pub today_visits: i64,
    /// 本周拜访
    pub week_visits: i64,
    /// 本月拜访
    pub month_visits: i64,
    /// 拜访客户数
    pub unique_customers: i64,
}

/// 跟进记录数据模型操作类
pub struct FollowupModel;

impl FollowupModel {
    pub async fn insert(db: &impl ConnectionTrait, req: &FollowupSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = followup::ActiveModel {
            lead_id: Set(req.lead_id.clone()),
            customer_id: Set(req.customer_id.clone()),
            opportunity_id: Set(req.opportunity_id.clone()),
            activity_type: Set(req.activity_type.clone()),
            source_type: Set(req.source_type.clone()),
            content: Set(req.content.clone()),
            next_follow_date: Set(req.next_follow_date.clone()),
            duration_minutes: Set(req.duration_minutes.clone()),
            result: Set(req.result.clone()),
            assigned_to: Set(req.assigned_to.clone()),
            created_by: Set(req.created_by.clone()),
            create_time: Set(Option::from(now)),
            updated_by: Set(req.updated_by.clone()),
            update_time: Set(Option::from(now)),
            visit_address: Set(req.visit_address.clone()),
            visit_latitude: Set(req.visit_latitude.clone()),
            visit_longitude: Set(req.visit_longitude.clone()),
            visit_accuracy: Set(req.visit_accuracy.clone()),
            visit_photos: Set(req.visit_photos.clone()),
            visit_distance: Set(req.visit_distance.clone()),
            check_in_time: Set(req.check_in_time.clone()),
            check_out_time: Set(req.check_out_time.clone()),
            visit_customer_lat: Set(req.visit_customer_lat.clone()),
            visit_customer_lng: Set(req.visit_customer_lng.clone()),
            ..Default::default()
        };

        Followup::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        Followup::update_many()
            .set(followup::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(followup::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, req: &FollowupSaveDTO) -> Result<i64, DbErr> {
        let payload = followup::ActiveModel {
            lead_id: Set(req.lead_id.clone()),
            customer_id: Set(req.customer_id.clone()),
            opportunity_id: Set(req.opportunity_id.clone()),
            activity_type: Set(req.activity_type.clone()),
            source_type: Set(req.source_type.clone()),
            content: Set(req.content.clone()),
            next_follow_date: Set(req.next_follow_date.clone()),
            duration_minutes: Set(req.duration_minutes.clone()),
            result: Set(req.result.clone()),
            updated_by: Set(req.updated_by.clone()),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            visit_address: Set(req.visit_address.clone()),
            visit_latitude: Set(req.visit_latitude.clone()),
            visit_longitude: Set(req.visit_longitude.clone()),
            visit_accuracy: Set(req.visit_accuracy.clone()),
            visit_photos: Set(req.visit_photos.clone()),
            visit_distance: Set(req.visit_distance.clone()),
            check_in_time: Set(req.check_in_time.clone()),
            check_out_time: Set(req.check_out_time.clone()),
            visit_customer_lat: Set(req.visit_customer_lat.clone()),
            visit_customer_lng: Set(req.visit_customer_lng.clone()),
            ..Default::default()
        };

        let update_result: UpdateResult = Followup::update_many()
            .set(payload)
            .filter(followup::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// 签退：更新指定拜访记录的 check_out_time
    pub async fn update_check_out_time(db: &DbConn, id: i64, check_out_time: DateTime) -> Result<i64, DbErr> {
        let update_result: UpdateResult = Followup::update_many()
            .set(followup::ActiveModel {
                check_out_time: Set(Option::from(check_out_time)),
                update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
                ..Default::default()
            })
            .filter(followup::Column::Id.eq(id))
            .filter(followup::Column::Deleted.eq(0))
            .filter(followup::Column::ActivityType.eq(2))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// 拜访记录分页查询：筛选 activity_type=2，支持按创建人ID列表过滤
    pub async fn select_visit_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        customer_id: Option<i64>,
        lead_id: Option<i64>,
        creator_ids: Option<Vec<i64>>,
        time_range: Option<(chrono::NaiveDateTime, chrono::NaiveDateTime)>,
    ) -> Result<(Vec<followup::Model>, i64), DbErr> {
        let mut query = Followup::find()
            .filter(followup::Column::Deleted.eq(0))
            .filter(followup::Column::ActivityType.eq(2));

        if let Some(c) = customer_id {
            query = query.filter(followup::Column::CustomerId.eq(c));
        }
        if let Some(l) = lead_id {
            query = query.filter(followup::Column::LeadId.eq(l));
        }
        if let Some(ids) = creator_ids {
            if ids.is_empty() {
                return Ok((vec![], 0));
            }
            query = query.filter(followup::Column::CreatedBy.is_in(ids));
        }
        if let Some((start, end)) = time_range {
            query = query.filter(followup::Column::CreateTime.gte(start))
                .filter(followup::Column::CreateTime.lte(end));
        }

        let paginator = query.order_by_desc(followup::Column::CheckInTime).paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, total))
    }

    /// 统计拜访次数：按创建人ID列表 + 时间范围筛选
    pub async fn count_visits(
        db: &DbConn,
        creator_ids: Option<&Vec<i64>>,
        time_range: Option<(chrono::NaiveDateTime, chrono::NaiveDateTime)>,
    ) -> Result<i64, DbErr> {
        let mut query = Followup::find()
            .filter(followup::Column::Deleted.eq(0))
            .filter(followup::Column::ActivityType.eq(2));

        if let Some(ids) = creator_ids {
            if ids.is_empty() {
                return Ok(0);
            }
            query = query.filter(followup::Column::CreatedBy.is_in(ids.clone()));
        }
        if let Some((start, end)) = time_range {
            query = query.filter(followup::Column::CreateTime.gte(start))
                .filter(followup::Column::CreateTime.lte(end));
        }

        let count = query.count(db).await?;
        Ok(count as i64)
    }

    /// 统计拜访客户数（去重 customer_id）：按创建人ID列表筛选
    pub async fn count_unique_visit_customers(
        db: &DbConn,
        creator_ids: Option<&Vec<i64>>,
    ) -> Result<i64, DbErr> {
        let mut query = Followup::find()
            .filter(followup::Column::Deleted.eq(0))
            .filter(followup::Column::ActivityType.eq(2))
            .filter(followup::Column::CustomerId.is_not_null());

        if let Some(ids) = creator_ids {
            if ids.is_empty() {
                return Ok(0);
            }
            query = query.filter(followup::Column::CreatedBy.is_in(ids.clone()));
        }

        // 统计去重 customer_id 数量
        let count = query
            .select_only()
            .column(followup::Column::CustomerId)
            .distinct()
            .count(db)
            .await?;

        Ok(count as i64)
    }

    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<followup::Model>, DbErr> {
        Followup::find_by_id(id)
            .filter(followup::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    pub async fn select_by_lead_id(db: &DbConn, lead_id: i64) -> Result<Vec<followup::Model>, DbErr> {
        Followup::find()
            .filter(followup::Column::Deleted.eq(0))
            .filter(followup::Column::LeadId.eq(lead_id))
            .order_by_desc(followup::Column::CreateTime)
            .all(db)
            .await
    }

    pub async fn select_by_customer_id(db: &DbConn, customer_id: i64) -> Result<Vec<followup::Model>, DbErr> {
        Followup::find()
            .filter(followup::Column::Deleted.eq(0))
            .filter(followup::Column::CustomerId.eq(customer_id))
            .order_by_desc(followup::Column::CreateTime)
            .all(db)
            .await
    }

    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        customer_id: Option<i64>,
        lead_id: Option<i64>,
        opportunity_id: Option<i64>,
        only_customer: Option<bool>,
        source_type: Option<i16>,
    ) -> Result<(Vec<followup::Model>, i64), DbErr> {
        Self::select_in_page_internal(
            db, page, per_page,
            customer_id, lead_id, opportunity_id,
            only_customer, source_type,
            None, None,
        ).await
    }

    /// 分页查询跟进记录（按创建人ID列表过滤）
    /// creator_ids: None 表示不过滤；Some(vec) 表示按这些创建人过滤
    pub async fn select_in_page_by_creator_ids(
        db: &DbConn,
        page: i64,
        per_page: i64,
        customer_id: Option<i64>,
        lead_id: Option<i64>,
        opportunity_id: Option<i64>,
        only_customer: Option<bool>,
        source_type: Option<i16>,
        creator_ids: Option<Vec<i64>>,
    ) -> Result<(Vec<followup::Model>, i64), DbErr> {
        Self::select_in_page_internal(
            db, page, per_page,
            customer_id, lead_id, opportunity_id,
            only_customer, source_type,
            creator_ids, None,
        ).await
    }

    /// 分页查询今日跟进记录（按创建时间过滤为今日）
    /// creator_ids: None 表示不过滤；Some(vec) 表示按这些创建人过滤
    pub async fn select_today_follow_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        customer_id: Option<i64>,
        lead_id: Option<i64>,
        opportunity_id: Option<i64>,
        only_customer: Option<bool>,
        source_type: Option<i16>,
        creator_ids: Option<Vec<i64>>,
    ) -> Result<(Vec<followup::Model>, i64), DbErr> {
        let today = chrono::Local::now().naive_local().date();
        let today_start = chrono::NaiveDateTime::new(today, chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        let today_end = chrono::NaiveDateTime::new(today, chrono::NaiveTime::from_hms_opt(23, 59, 59).unwrap());
        Self::select_in_page_internal(
            db, page, per_page,
            customer_id, lead_id, opportunity_id,
            only_customer, source_type,
            creator_ids, Some((today_start, today_end)),
        ).await
    }

    /// 内部统一查询方法：支持按创建人 + 时间范围过滤
    async fn select_in_page_internal(
        db: &DbConn,
        page: i64,
        per_page: i64,
        customer_id: Option<i64>,
        lead_id: Option<i64>,
        opportunity_id: Option<i64>,
        only_customer: Option<bool>,
        source_type: Option<i16>,
        creator_ids: Option<Vec<i64>>,
        time_range: Option<(chrono::NaiveDateTime, chrono::NaiveDateTime)>,
    ) -> Result<(Vec<followup::Model>, i64), DbErr> {
        let mut query = Followup::find()
            .filter(followup::Column::Deleted.eq(0));

        if let Some(c) = customer_id {
            query = query.filter(followup::Column::CustomerId.eq(c));
        }
        if let Some(l) = lead_id {
            query = query.filter(followup::Column::LeadId.eq(l));
        }
        if let Some(o) = opportunity_id {
            query = query.filter(followup::Column::OpportunityId.eq(o));
        }
        if let Some(true) = only_customer {
            query = query.filter(followup::Column::CustomerId.is_not_null());
        }
        if let Some(s) = source_type {
            query = query.filter(followup::Column::SourceType.eq(s));
        }
        if let Some(ids) = creator_ids {
            if ids.is_empty() {
                return Ok((vec![], 0));
            }
            query = query.filter(followup::Column::CreatedBy.is_in(ids));
        }
        if let Some((start, end)) = time_range {
            query = query.filter(followup::Column::CreateTime.gte(start))
                .filter(followup::Column::CreateTime.lte(end));
        }

        let paginator = query.order_by_desc(followup::Column::CreateTime).paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, total))
    }

    /// 查询全部跟进记录（不分页），用于分组去重场景
    /// 筛选条件与 select_in_page_internal 一致，但不分页
    pub async fn select_all_internal(
        db: &DbConn,
        customer_id: Option<i64>,
        lead_id: Option<i64>,
        opportunity_id: Option<i64>,
        only_customer: Option<bool>,
        source_type: Option<i16>,
        creator_ids: Option<Vec<i64>>,
        time_range: Option<(chrono::NaiveDateTime, chrono::NaiveDateTime)>,
    ) -> Result<Vec<followup::Model>, DbErr> {
        let mut query = Followup::find()
            .filter(followup::Column::Deleted.eq(0));

        if let Some(c) = customer_id {
            query = query.filter(followup::Column::CustomerId.eq(c));
        }
        if let Some(l) = lead_id {
            query = query.filter(followup::Column::LeadId.eq(l));
        }
        if let Some(o) = opportunity_id {
            query = query.filter(followup::Column::OpportunityId.eq(o));
        }
        if let Some(true) = only_customer {
            query = query.filter(followup::Column::CustomerId.is_not_null());
        }
        if let Some(s) = source_type {
            query = query.filter(followup::Column::SourceType.eq(s));
        }
        if let Some(ids) = creator_ids {
            if ids.is_empty() {
                return Ok(vec![]);
            }
            query = query.filter(followup::Column::CreatedBy.is_in(ids));
        }
        if let Some((start, end)) = time_range {
            query = query.filter(followup::Column::CreateTime.gte(start))
                .filter(followup::Column::CreateTime.lte(end));
        }

        query.order_by_desc(followup::Column::CreateTime).all(db).await
    }

    /// 线索转客户时，把该线索的跟进记录继承到新客户
    /// - 设置 customer_id 为新客户ID
    /// - 保留 lead_id（便于追溯原线索）
    /// - source_type 升级为 2（客户跟进），但保留 lead_id 仍可识别线索来源
    pub async fn inherit_to_customer(
        db: &impl ConnectionTrait,
        lead_id: i64,
        customer_id: i64,
    ) -> Result<u64, DbErr> {
        let result = Followup::update_many()
            .set(followup::ActiveModel {
                customer_id: Set(Some(customer_id)),
                source_type: Set(Some(2i16)),
                update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
                ..Default::default()
            })
            .filter(followup::Column::LeadId.eq(lead_id))
            .filter(followup::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }
}
