//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::system::entity::{admin_notice_merge, notice, notice::Entity as Notice};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string,deserialize_string_to_i32};
use chrono::Local;
use sea_orm::prelude::DateTime;
use sea_orm::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NoticeSaveRequest {
    /// 通知标题
    pub title: Option<String>,
    /// 通知内容
    pub content: Option<String>,
    /// 通知类型（关联字典编码：notice_type）
    #[serde(deserialize_with = "deserialize_string_to_i32")]
    pub r#type: Option<i32>,
    /// 通知等级（字典code：notice_level）
    pub level: Option<String>,
    /// 目标类型（1: 全体, 2: 指定）
    pub target_type: Option<i32>,
    /// 目标人ID集合（多个使用英文逗号,分割）
    pub target_user_ids: Option<String>,
    /// 发布人ID
    pub publisher_id: Option<i64>,
    /// 发布状态（0: 未发布, 1: 已发布, -1: 已撤回）
    pub publish_status: Option<i32>,
    /// 发布时间
    pub publish_time: Option<DateTime>,
    /// 撤回时间
    pub revoke_time: Option<DateTime>,
    /// 创建人ID
    pub create_by: Option<i64>,
}

impl From<NoticeSaveRequest> for NoticeSaveDTO {
    fn from(req: NoticeSaveRequest) -> Self {
        Self {
            id: None,
            title: req.title,
            content: req.content,
            r#type: req.r#type,
            level: req.level,
            target_type: req.target_type,
            target_user_ids: req.target_user_ids,
            publisher_id: req.publisher_id,
            publish_status: req.publish_status,
            publish_time: None,
            revoke_time: None,
            create_by: req.create_by,
            create_time: None,
            update_by: None,
            update_time: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NoticeUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 通知标题
    pub title: Option<String>,
    /// 通知内容
    pub content: Option<String>,
    /// 通知类型（关联字典编码：notice_type）
    #[serde(deserialize_with = "deserialize_string_to_i32")]
    pub r#type: Option<i32>,
    /// 通知等级（字典code：notice_level）
    pub level: Option<String>,
    /// 目标类型（1: 全体, 2: 指定）
    pub target_type: Option<i32>,
    /// 目标人ID集合（多个使用英文逗号,分割）
    pub target_user_ids: Option<String>,
    /// 发布人ID
    pub publisher_id: Option<i64>,
    /// 发布状态（0: 未发布, 1: 已发布, -1: 已撤回）
    pub publish_status: Option<i32>,
    /// 更新人ID
    pub update_by: Option<i64>,
}

impl From<NoticeUpdateRequest> for NoticeSaveDTO {
    fn from(req: NoticeUpdateRequest) -> Self {
        Self {
            id: req.id,
            title: req.title,
            content: req.content,
            r#type: req.r#type,
            level: req.level,
            target_type: req.target_type,
            target_user_ids: req.target_user_ids,
            publisher_id: req.publisher_id,
            publish_status: req.publish_status,
            publish_time: None,
            revoke_time: None,
            create_by: None,
            create_time: None,
            update_by: req.update_by,
            update_time: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NoticeSaveDTO {
    pub id: Option<i64>,
    /// 通知标题
    pub title: Option<String>,
    /// 通知内容
    pub content: Option<String>,
    /// 通知类型（关联字典编码：notice_type）
    pub r#type: Option<i32>,
    /// 通知等级（字典code：notice_level）
    pub level: Option<String>,
    /// 目标类型（1: 全体, 2: 指定）
    pub target_type: Option<i32>,
    /// 目标人ID集合（多个使用英文逗号,分割）
    pub target_user_ids: Option<String>,
    /// 发布人ID
    pub publisher_id: Option<i64>,
    /// 发布状态（0: 未发布, 1: 已发布, -1: 已撤回）
    pub publish_status: Option<i32>,
    /// 发布时间
    pub publish_time: Option<DateTime>,
    /// 撤回时间
    pub revoke_time: Option<DateTime>,
    /// 创建人ID
    pub create_by: Option<i64>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新人ID
    pub update_by: Option<i64>,
    /// 更新时间
    pub update_time: Option<DateTime>,

}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct NoticeListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 通知标题
    pub notice_title: Option<String>,
    /// 通知内容
    pub content: Option<String>,
    /// 通知类型（关联字典编码：notice_type）
    pub notice_type: Option<i32>,
    /// 通知等级（字典code：notice_level）
    pub level: Option<String>,
    /// 目标类型（1: 全体, 2: 指定）
    pub target_type: Option<i32>,
    /// 目标人ID集合（多个使用英文逗号,分割）
    pub target_user_ids: Option<String>,
    /// 发布人ID
    pub publisher_id: Option<i64>,
    /// 发布人名称
    pub publish_name: Option<String>,
    /// 发布状态（0: 未发布, 1: 已发布, -1: 已撤回）
    pub status: Option<i32>,
    /// 发布时间
    pub publish_time: Option<String>,
    /// 撤回时间
    pub revoke_time: Option<String>,
    /// 创建人ID
    pub create_by: Option<i64>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 更新人ID
    pub update_by: Option<i64>,
    /// 更新时间
    pub update_time: Option<String>,
}

impl From<notice::Model> for NoticeListVO {
    fn from(model: notice::Model) -> Self {
        Self {
            id: Option::from(model.id),
            notice_title: model.title,
            content: model.content,
            notice_type: model.r#type,
            level: model.level,
            target_type: model.target_type,
            target_user_ids: model.target_user_ids,
            publisher_id: model.publisher_id,
            publish_name: None,
            status: model.publish_status,
            publish_time: model.publish_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            revoke_time: model.revoke_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            create_by: model.create_by,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_by: model.update_by,
            update_time: model.update_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct MyNoticeListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 通知标题
    pub title: Option<String>,
    /// 通知内容
    pub content: Option<String>,
    /// 通知类型（关联字典编码：notice_type）
    pub r#type: Option<i32>,
    /// 通知等级（字典code：notice_level）
    pub level: Option<String>,
    /// 发布人ID
    pub publisher_id: Option<i64>,
    /// 发布人名称
    pub publish_name: Option<String>,
    /// 发布状态（0: 未发布, 1: 已发布, -1: 已撤回）
    pub publish_status: Option<i32>,
    /// 发布时间
    pub publish_time: Option<String>,
    /// 读取状态（0: 未读, 1: 已读）
    pub is_read: Option<i32>,
}

#[derive(FromQueryResult, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct MyNoticeModel {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 通知标题
    pub title: Option<String>,
    /// 通知内容
    pub content: Option<String>,
    /// 通知类型（关联字典编码：notice_type）
    pub r#type: Option<i32>,
    /// 通知等级（字典code：notice_level）
    pub level: Option<String>,
    /// 发布人ID
    pub publisher_id: Option<i64>,
    /// 发布状态（0: 未发布, 1: 已发布, -1: 已撤回）
    pub publish_status: Option<i32>,
    /// 发布时间
    pub publish_time: Option<DateTime>,
    /// 读取状态（0: 未读, 1: 已读）
    pub is_read: Option<i32>,
}

impl From<MyNoticeModel> for MyNoticeListVO {
    fn from(model: MyNoticeModel) -> Self {
        Self {
            id: Option::from(model.id),
            title: model.title,
            content: model.content,
            r#type: model.r#type,
            level: model.level,
            publisher_id: model.publisher_id,
            publish_name: None,
            publish_status: model.publish_status,
            publish_time: model.publish_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            is_read: model.is_read,
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct NoticeDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 通知标题
    pub notice_title: Option<String>,
    /// 通知内容
    pub content: Option<String>,
    /// 通知类型（关联字典编码：notice_type）
    pub notice_type: Option<i32>,
    /// 通知等级（字典code：notice_level）
    pub level: Option<String>,
    /// 目标类型（1: 全体, 2: 指定）
    pub target_type: Option<i32>,
    /// 目标人ID集合（多个使用英文逗号,分割）
    pub target_user_ids: Option<String>,
    /// 发布人ID
    pub publisher_id: Option<i64>,
    /// 发布状态（0: 未发布, 1: 已发布, -1: 已撤回）
    pub status: Option<i32>,
    /// 发布时间
    pub publish_time: Option<String>,
    /// 撤回时间
    pub revoke_time: Option<String>,
    /// 创建人ID
    pub create_by: Option<i64>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 更新人ID
    pub update_by: Option<i64>,
    /// 更新时间
    pub update_time: Option<String>,
}

impl From<notice::Model> for NoticeDetailVO {
    fn from(model: notice::Model) -> Self {
        Self {
            id: Option::from(model.id),
            notice_title: model.title,
            content: model.content,
            notice_type: model.r#type,
            level: model.level,
            target_type: model.target_type,
            target_user_ids: model.target_user_ids,
            publisher_id: model.publisher_id,
            status: model.publish_status,
            publish_time: model.publish_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            revoke_time: model.revoke_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            create_by: model.create_by,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_by: model.update_by,
            update_time: model.update_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}



#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub title: Option<String>,
    pub user_id: Option<i64>,
    /// 读取状态（0: 未读, 1: 已读）
    pub is_read: Option<i32>,
    pub status: Option<i32>,

    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
}


/// 条件
#[derive(Clone)]
pub struct PageWhere {
    pub title: Option<String>,
    pub user_id: Option<i64>,
    /// 读取状态（0: 未读, 1: 已读）
    pub is_read: Option<i32>,
    pub status: Option<i32>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut title = None;
        if self.title != Some("".to_string()) {
            title = self.title.clone();
        }

        let mut user_id = None;
        if let Some(id) = self.user_id {
            if id > 0 {
                user_id = Some(id);
            }
        }

        let mut is_read = None;
        if self.is_read == Some(1) || self.is_read == Some(0) {
            is_read = self.is_read;
        }
        
        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        Self {
            title,
            user_id,
            is_read,
            status,
        }
    }
}


pub struct NoticeModel;

impl NoticeModel {
    pub async fn insert(db: &DbConn, form_data: &NoticeSaveDTO) -> Result<i64, DbErr> {
        let payload = notice::ActiveModel {
            title:            Set(form_data.title.to_owned()),
            content:          Set(form_data.content.to_owned()),
            r#type:           Set(form_data.r#type.to_owned()),
            level:            Set(form_data.level.to_owned()),
            target_type:      Set(form_data.target_type.to_owned()),
            target_user_ids:  Set(form_data.target_user_ids.to_owned()),
            publisher_id:     Set(form_data.publisher_id.to_owned()),
            publish_status:   Set(form_data.publish_status.to_owned()),
            publish_time:     Set(form_data.publish_time.to_owned()),
            revoke_time:      Set(form_data.revoke_time.to_owned()),
            create_by:        Set(form_data.create_by.to_owned()),
            create_time:      Set(Option::from(Local::now().naive_utc())),
            update_by:        Set(form_data.update_by.to_owned()),
            update_time:      Set(Option::from(Local::now().naive_utc())),
            ..Default::default()
        };
        Notice::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 批量删除通知信息
    /// * `db`: 数据库连接对象
    /// * `ids`: 通知id集合
    ///
    /// 返回删除的记录数，返回值大于0表示删除成功
    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        let delete_result: DeleteResult = Notice::delete_many()
            .filter(notice::Column::Id.is_in(ids))
            .exec(db)
            .await?;
        Ok(delete_result.rows_affected as i64)
    }

    /// 按id更新通知信息
    /// * `db`: 数据库连接对象
    /// * `id`: 通知id
    /// * `form_data`: 通知信息对象
    ///
    /// 返回更新的记录数，返回值大于0表示更新成功
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, form_data: &NoticeSaveDTO) -> Result<i64, DbErr> {
        let payload = notice::ActiveModel {
            title:            Set(form_data.title.to_owned()),
            content:          Set(form_data.content.to_owned()),
            r#type:           Set(form_data.r#type.to_owned()),
            level:            Set(form_data.level.to_owned()),
            target_type:      Set(form_data.target_type.to_owned()),
            target_user_ids:  Set(form_data.target_user_ids.to_owned()),
            publisher_id:     Set(form_data.publisher_id.to_owned()),
            publish_status:   Set(form_data.publish_status.to_owned()),
            publish_time:     Set(form_data.publish_time.to_owned()),
            revoke_time:      Set(form_data.revoke_time.to_owned()),
            update_by:        Set(form_data.update_by.to_owned()),
            update_time:      Set(Option::from(Local::now().naive_utc())),
            ..Default::default()
        };
        let update_result: UpdateResult = Notice::update_many()
            .set(payload)
            .filter(notice::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// # 更新通知发布状态
    /// * `db` - 数据库连接
    /// * `id` - 需要修改的id
    /// * `publish_status` - 发布状态（0: 未发布, 1: 已发布, -1: 已撤回）
    /// * `user_id` - 操作人id
    ///
    /// 返回值: 更新数量
    pub async fn update_by_id_revoke(db: &DbConn, id: &Option<i64>, user_id: &Option<i64>) -> Result<i64, DbErr> {
        let payload = notice::ActiveModel {
            revoke_time:      Set(Some(Local::now().naive_local()).to_owned()),
            publish_status:   Set(Some(-1).to_owned()),
            update_by:        Set(user_id.to_owned()),
            update_time:      Set(Option::from(Local::now().naive_local())),
            ..Default::default()
        };
        let update_result: UpdateResult = Notice::update_many()
            .set(payload)
            .filter(notice::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }


    /// # 更新通知发布状态
    /// * `db` - 数据库连接
    /// * `id` - 需要修改的id
    /// * `publish_status` - 发布状态（0: 未发布, 1: 已发布, -1: 已撤回）
    /// * `user_id` - 操作人id
    ///
    /// 返回值: 更新数量
    pub async fn update_by_id_publish(db: &DbConn, id: &Option<i64>, user_id: &Option<i64>) -> Result<i64, DbErr> {
        let payload = notice::ActiveModel {
            publisher_id:     Set(user_id.to_owned()),
            publish_status:   Set(Some(1).to_owned()),
            publish_time:     Set(Some(Local::now().naive_local()).to_owned()),
            update_by:        Set(user_id.to_owned()),
            update_time:      Set(Option::from(Local::now().naive_local())),
            ..Default::default()
        };
        let update_result: UpdateResult = Notice::update_many()
            .set(payload)
            .filter(notice::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// ### 根据id查询通知信息
    /// * `db` - 数据库连接
    /// * `id` - 通知id
    ///
    /// 返回值: Option<notice::Model>
    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<notice::Model>, DbErr> {
        let model = Notice::find_by_id(id.clone().unwrap_or_default())
            .filter(notice::Column::Deleted.eq(Some(0)))
            .one(db)
            .await?;
        Ok(model)
    }

    /// ### 用户查询通知数量
    /// * `db` - 数据库连接
    /// * `wheres` - 查询条件
    ///
    /// 返回值: i64
    pub async fn select_my_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        Notice::find()
            .distinct()
            .join_rev(
                JoinType::LeftJoin,
                admin_notice_merge::Relation::Notice.def(),
            )
            .apply_if(wheres.title, |query, v| {
                query.filter(notice::Column::Title.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.user_id, |query, v| {
                query.filter(admin_notice_merge::Column::UserId.eq(v))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(notice::Column::PublishStatus.eq(v))
            })
            .apply_if(wheres.is_read, |query, v| {
                query.filter(admin_notice_merge::Column::IsRead.eq(v))
            })
            .filter(notice::Column::TargetType.eq(Some(1)))
            .filter(notice::Column::Deleted.eq(Some(0)))
            .count(db)
            .await
            .map(|c| c as i64)
    }

    /// ### 用户查询通知列表
    ///
    /// * 联合查询时用`.into_model::<MyNoticeModel>();`方式将结果集转换成自定义结构体，结构体中的字段需要与数据库字段保持一致，结构体上需要加 <br />
    /// `#[derive(FromQueryResult)]`
    ///
    /// * `db` - 数据库连接
    /// * `page` - 页码
    /// * `per_page` - 每页数量
    /// * `wheres` - 查询条件
    ///
    /// 返回值: Result<(Vec<MyNoticeModel>, i64), DbErr>
    pub async fn select_my_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        wheres: PageWhere,
    ) -> Result<(Vec<MyNoticeModel>, i64), DbErr> {
        let paginator = Notice::find()
            .distinct()
            .select_only()
            .column(notice::Column::Id)
            .column(notice::Column::Title)
            .column(notice::Column::Content)
            .column(notice::Column::Type)
            .column(notice::Column::Level)
            .column(notice::Column::TargetType)
            .column(notice::Column::TargetUserIds)
            .column(notice::Column::PublisherId)
            .column(notice::Column::PublishStatus)
            .column(notice::Column::PublishTime)
            .column(notice::Column::RevokeTime)
            .column(admin_notice_merge::Column::IsRead)
            .join_rev(
                JoinType::LeftJoin,
                admin_notice_merge::Relation::Notice.def(),
            )
            .apply_if(wheres.title, |query, v| {
                query.filter(notice::Column::Title.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.user_id, |query, v| {
                query.filter(admin_notice_merge::Column::UserId.eq(v))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(notice::Column::PublishStatus.eq(v))
            })
            .apply_if(wheres.is_read, |query, v| {
                query.filter(admin_notice_merge::Column::IsRead.eq(v))
            })
            .filter(notice::Column::TargetType.eq(Some(1)))
            .filter(notice::Column::Deleted.eq(Some(0)))
            .order_by_desc(notice::Column::Id)
            .into_model::<MyNoticeModel>(); // 映射为 MyNoticeListVO
        let paginator = paginator.paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;
        // 获取指定页的数据
        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
    
    /// ### 查询通知数量
    /// * `db` - 数据库连接
    /// * `wheres` - 查询条件
    ///
    /// 返回值: i64
    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        Notice::find()
            .apply_if(wheres.title, |query, v| {
                query.filter(notice::Column::Title.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(notice::Column::PublishStatus.eq(v))
            })
            .count(db)
            .await
            .map(|c| c as i64)
    }

    /// ### 查询通知列表
    /// * `db` - 数据库连接
    /// * `page` - 页码
    /// * `per_page` - 每页数量
    /// * `wheres` - 查询条件
    ///
    /// 返回值: Result<(Vec<notice::Model>, i64), DbErr>
    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        wheres: PageWhere,
    ) -> Result<(Vec<notice::Model>, i64), DbErr> {
        let paginator = Notice::find()
            .apply_if(wheres.title, |query, v| {
                query.filter(notice::Column::Title.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(notice::Column::PublishStatus.eq(v))
            })
            .order_by_desc(notice::Column::Id);
        
        //let sql = paginator.build(DbBackend::MySql);
        //log::info!("===============Generated SQL: {}", sql.to_string());
        let paginator = paginator.paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
    
}