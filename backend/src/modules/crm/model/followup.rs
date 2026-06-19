use sea_orm::*;
use sea_orm::prelude::{DateTime, Date};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::crm::entity::{followup, followup::Entity as Followup};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};
use crate::core::r#enum::activity_type_enum::ActivityType;

/// 跟进记录新增请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct FollowupSaveRequest {
    pub lead_id: Option<i64>,
    pub customer_id: Option<i64>,
    pub opportunity_id: Option<i64>,
    pub activity_type: Option<ActivityType>,
    pub subject: Option<String>,
    pub content: Option<String>,
    pub next_follow_date: Option<Date>,
    pub duration_minutes: Option<i32>,
    pub result: Option<String>,
    pub assigned_to: Option<i64>,
}

impl From<FollowupSaveRequest> for FollowupSaveDTO {
    fn from(item: FollowupSaveRequest) -> Self {
        FollowupSaveDTO {
            id: None,
            lead_id: item.lead_id,
            customer_id: item.customer_id,
            opportunity_id: item.opportunity_id,
            activity_type: item.activity_type,
            subject: item.subject,
            content: item.content,
            next_follow_date: item.next_follow_date,
            duration_minutes: item.duration_minutes,
            result: item.result,
            assigned_to: None,
            deleted: None,
            created_by: None,
            created_at: None,
            updated_by: None,
            updated_at: None,
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
    pub activity_type: Option<ActivityType>,
    pub subject: Option<String>,
    pub content: Option<String>,
    pub next_follow_date: Option<Date>,
    pub duration_minutes: Option<i32>,
    pub result: Option<String>,
    pub assigned_to: Option<i64>,
}

impl From<FollowupUpdateRequest> for FollowupSaveDTO {
    fn from(item: FollowupUpdateRequest) -> Self {
        FollowupSaveDTO {
            id: item.id,
            lead_id: item.lead_id,
            customer_id: item.customer_id,
            opportunity_id: item.opportunity_id,
            activity_type: item.activity_type,
            subject: item.subject,
            content: item.content,
            next_follow_date: item.next_follow_date,
            duration_minutes: item.duration_minutes,
            result: item.result,
            assigned_to: None,
            deleted: None,
            created_by: None,
            created_at: None,
            updated_by: None,
            updated_at: None,
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
    pub activity_type: Option<ActivityType>,
    pub subject: Option<String>,
    pub content: Option<String>,
    pub next_follow_date: Option<Date>,
    pub duration_minutes: Option<i32>,
    pub result: Option<String>,
    pub assigned_to: Option<i64>,
    pub deleted: Option<i32>,
    pub created_by: Option<i64>,
    pub created_at: Option<DateTime>,
    pub updated_by: Option<i64>,
    pub updated_at: Option<DateTime>,
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
    pub activity_type: Option<String>,
    pub subject: Option<String>,
    pub content: Option<String>,
    pub next_follow_date: Option<Date>,
    pub duration_minutes: Option<i32>,
    pub result: Option<String>,
    pub assigned_to: Option<i64>,
}

impl From<followup::Model> for FollowupDetailVO {
    fn from(item: followup::Model) -> Self {
        FollowupDetailVO {
            id: Option::from(item.id),
            lead_id: item.lead_id,
            customer_id: item.customer_id,
            opportunity_id: item.opportunity_id,
            activity_type: item.activity_type.map(|v| v.to_string()),
            subject: item.subject,
            content: item.content,
            next_follow_date: item.next_follow_date,
            duration_minutes: item.duration_minutes,
            result: item.result,
            assigned_to: item.assigned_to,
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
    pub activity_type: Option<String>,
    pub subject: Option<String>,
    pub content: Option<String>,
    pub next_follow_date: Option<Date>,
    pub result: Option<String>,
    pub assigned_to: Option<i64>,
}

impl From<followup::Model> for FollowupListVO {
    fn from(item: followup::Model) -> Self {
        FollowupListVO {
            id: Option::from(item.id),
            lead_id: item.lead_id,
            customer_id: item.customer_id,
            opportunity_id: item.opportunity_id,
            activity_type: item.activity_type.map(|v| v.to_string()),
            subject: item.subject,
            content: item.content,
            next_follow_date: item.next_follow_date,
            result: item.result,
            assigned_to: item.assigned_to,
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
}

/// 跟进记录数据模型操作类
pub struct FollowupModel;

impl FollowupModel {
    pub async fn insert(db: &DbConn, req: &FollowupSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = followup::ActiveModel {
            lead_id: Set(req.lead_id.clone()),
            customer_id: Set(req.customer_id.clone()),
            opportunity_id: Set(req.opportunity_id.clone()),
            activity_type: Set(req.activity_type.clone()),
            subject: Set(req.subject.clone()),
            content: Set(req.content.clone()),
            next_follow_date: Set(req.next_follow_date.clone()),
            duration_minutes: Set(req.duration_minutes.clone()),
            result: Set(req.result.clone()),
            assigned_to: Set(req.assigned_to.clone()),
            created_by: Set(req.created_by.clone()),
            created_at: Set(Option::from(now)),
            updated_by: Set(req.updated_by.clone()),
            updated_at: Set(Option::from(now)),
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
            subject: Set(req.subject.clone()),
            content: Set(req.content.clone()),
            next_follow_date: Set(req.next_follow_date.clone()),
            duration_minutes: Set(req.duration_minutes.clone()),
            result: Set(req.result.clone()),
            updated_by: Set(req.updated_by.clone()),
            updated_at: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        let update_result: UpdateResult = Followup::update_many()
            .set(payload)
            .filter(followup::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<followup::Model>, DbErr> {
        Followup::find_by_id(id)
            .filter(followup::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        customer_id: Option<i64>,
        lead_id: Option<i64>,
        opportunity_id: Option<i64>,
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

        let paginator = query.order_by_desc(followup::Column::CreatedAt).paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}