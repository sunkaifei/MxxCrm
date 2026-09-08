//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::{DbConn, DbErr, TransactionTrait, Set, EntityTrait};
use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::core::r#enum::lead_source_enum::LeadSource;
use crate::SNOWFLAKE;
use crate::modules::crm::entity::lead;
use crate::modules::website::entity::leave_msg;
use crate::modules::website::model::leave_msg::{
    LeaveMsgDetailVO, LeaveMsgModel, LeaveMsgSaveDTO, LeaveMsgSubmitRequest,
};
use crate::modules::website::model::website::SiteDetailVO;

/// 在事务内创建线索（直接使用 entity，支持 ConnectionTrait 事务传入）
///
/// 注意：返回 `std::result::Result<i64, DbErr>` 而非项目内 `error::Result<i64>`，
/// 以便在 `db.transaction::<_, i64, DbErr>` 闭包中通过 `?` 传播错误。
async fn create_lead_in_txn<C: sea_orm::ConnectionTrait>(
    txn: &C,
    contact_name: Option<String>,
    contact_phone: Option<String>,
    contact_email: Option<String>,
    content: Option<String>,
    site_id: i64,
    assigned_to: i64,
    created_by: i64,
) -> std::result::Result<i64, DbErr> {
    let now = chrono::Local::now().naive_local().to_owned();
    let payload = lead::ActiveModel {
        contact_name: Set(contact_name),
        email: Set(contact_email),
        phone: Set(contact_phone.clone()),
        mobile: Set(contact_phone),
        source: Set(Some(LeadSource::Website)),
        source_detail: Set(Some(format!("网站留言（站点ID={}）", site_id))),
        status: Set(Some(6)), // 6=未审查
        level: Set(Some(5)),  // 5=其他
        assigned_to: Set(Some(assigned_to)),
        description: Set(content),
        created_by: Set(Some(created_by)),
        create_time: Set(Some(now.clone())),
        updated_by: Set(Some(created_by)),
        update_time: Set(Some(now)),
        ..Default::default()
    };
    lead::Entity::insert(payload)
        .exec(txn)
        .await
        .map(|r| r.last_insert_id)
}

/// 提交留言（前台访客调用）
///
/// 流程：
/// 1. 保存留言记录（status=0 待处理）
/// 2. 若站点配置了 lead_owner_id，则自动转线索：
///    - 创建线索（source=website, assigned_to=lead_owner_id, status=6 未审查）
///    - 回写留言的 lead_id / converted_to_lead=1 / status=1
/// 3. 全程事务包裹，保证原子性
pub async fn submit(
    db: &DbConn,
    site: &SiteDetailVO,
    req: LeaveMsgSubmitRequest,
    ip_address: Option<String>,
    user_agent: Option<String>,
) -> Result<i64> {
    let mut dto: LeaveMsgSaveDTO = req.into();
    dto.id = Some(SNOWFLAKE.generate() as i64);
    dto.website_id = site.id;
    dto.ip_address = ip_address;
    dto.user_agent = user_agent;

    let lead_owner_id = site.lead_owner_id;
    let leave_msg_id = dto.id.unwrap_or_default();
    let site_id = site.id.unwrap_or_default();

    // 站点配置了 lead_owner_id 时，留言 + 转线索原子执行
    if let Some(owner_id) = lead_owner_id {
        let dto_clone = dto.clone();
        let leave_msg_id_clone = leave_msg_id;
        let site_id_clone = site_id;

        db.transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move {
                // 1. 插入留言
                LeaveMsgModel::insert(txn, &dto_clone).await?;

                // 2. 创建线索
                let lead_id = create_lead_in_txn(
                    txn,
                    dto_clone.contact_name.clone(),
                    dto_clone.contact_phone.clone(),
                    dto_clone.contact_email.clone(),
                    dto_clone.content.clone(),
                    site_id_clone,
                    owner_id,
                    owner_id,
                ).await?;

                // 3. 回写留言的线索ID
                LeaveMsgModel::update_lead_info(txn, leave_msg_id_clone, lead_id).await?;

                Ok(leave_msg_id_clone)
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    } else {
        // 未配置 lead_owner_id，仅保存留言，待后台手动转线索
        LeaveMsgModel::insert(db, &dto).await?;
    }

    // G-2.8: 触发"新留言"邮件通知（失败不影响主流程）
    let notify_ctx = serde_json::json!({
        "contact_name": dto.contact_name.clone().unwrap_or_default(),
        "contact_phone": dto.contact_phone.clone().unwrap_or_default(),
        "contact_email": dto.contact_email.clone().unwrap_or_default(),
        "content": dto.content.clone().unwrap_or_default(),
        "site_name": site.site_name.clone().unwrap_or_default(),
        "leave_msg_id": leave_msg_id,
    });
    if let Err(e) = crate::modules::website::service::website_notification_config_service::send_notification(
        db,
        site_id,
        "new_leave_msg",
        notify_ctx,
    ).await {
        log::warn!("[通知触发失败] new_leave_msg, error={}", e);
    }

    Ok(leave_msg_id)
}

/// 手动转线索（后台管理员调用）
///
/// 将指定留言转为线索，分配给指定负责人。
/// 使用事务保证原子性，并做前置状态校验（已转线索的不允许重复转）。
pub async fn convert_to_lead(
    db: &DbConn,
    leave_msg_id: i64,
    assigned_to: i64,
    created_by: i64,
) -> Result<i64> {
    // 前置查询：校验留言存在且未转线索
    let msg = LeaveMsgModel::find_by_id(db, leave_msg_id)
        .await?
        .ok_or_else(|| Error::from("留言不存在"))?;

    if msg.converted_to_lead.unwrap_or(0) == 1 {
        return Err(Error::from("该留言已转线索，不能重复转换"));
    }

    let contact_name = msg.contact_name.clone();
    let contact_phone = msg.contact_phone.clone();
    let contact_email = msg.contact_email.clone();
    let content = msg.content.clone();
    let website_id = msg.website_id.unwrap_or_default();

    let leave_msg_id_clone = leave_msg_id;

    let lead_id = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move {
                // 1. 创建线索
                let new_lead_id = create_lead_in_txn(
                    txn,
                    contact_name,
                    contact_phone,
                    contact_email,
                    content,
                    website_id,
                    assigned_to,
                    created_by,
                ).await?;

                // 2. 回写留言的线索ID
                LeaveMsgModel::update_lead_info(txn, leave_msg_id_clone, new_lead_id).await?;

                Ok(new_lead_id)
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(lead_id)
}

/// 根据ID查询留言
pub async fn find_by_id(db: &DbConn, id: i64) -> Result<LeaveMsgDetailVO> {
    let msg = LeaveMsgModel::find_by_id(db, id)
        .await?
        .ok_or_else(|| Error::from("留言不存在"))?;
    Ok(msg.into())
}

/// 分页查询留言列表
pub async fn get_by_page(
    db: &DbConn,
    page: i64,
    page_size: i64,
    website_id: Option<i64>,
    status: Option<i32>,
) -> Result<ResultPage<Vec<LeaveMsgDetailVO>>> {
    let (list, total) = LeaveMsgModel::select_in_page(db, page, page_size, website_id, status).await?;
    let list_vo: Vec<LeaveMsgDetailVO> = list.into_iter().map(|m| m.into()).collect();
    Ok(ResultPage::new_simple(list_vo, total))
}

/// 批量软删除留言
pub async fn batch_delete(db: &DbConn, ids: Vec<i64>) -> Result<i64> {
    LeaveMsgModel::batch_soft_delete(db, ids).await.map_err(|e| Error::from(e.to_string()))
}

/// 更新留言状态（标记为已处理/已忽略）
pub async fn update_status(db: &DbConn, id: i64, status: i32) -> Result<i64> {
    LeaveMsgModel::update_status(db, id, status).await.map_err(|e| Error::from(e.to_string()))
}

/// 查询所有未转线索的留言（供后台批量处理）
#[allow(dead_code)]
pub async fn find_unconverted(db: &DbConn, website_id: Option<i64>) -> Result<Vec<leave_msg::Model>> {
    use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, QueryOrder};

    let mut query = leave_msg::Entity::find()
        .filter(leave_msg::Column::Deleted.eq(0))
        .filter(leave_msg::Column::ConvertedToLead.eq(0));

    if let Some(wid) = website_id {
        query = query.filter(leave_msg::Column::WebsiteId.eq(wid));
    }

    let list = query.order_by_desc(leave_msg::Column::CreateTime).all(db).await?;
    Ok(list)
}
