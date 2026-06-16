//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::utils::string_utils::{serialize_option_u64_to_string, deserialize_string_to_u64};
use sea_orm::*;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::articles::entity::{label, label::Entity as Label};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LabelSaveRequest {
    // 短网址
    pub short_url: Option<String>,
    // 标签名称
    pub title: Option<String>,
    // 话题描述
    pub content: Option<String>,
    // 话题图片
    pub label_pic: Option<String>,
    // 标签使用数量
    pub count_topic: Option<i32>,
    //1为推荐
    pub isrecommend: Option<i32>,
    // 0未审核，1：正常显示；2：隐藏不显示
    pub status: Option<i32>,
}

impl From<LabelSaveRequest> for LabelSaveDTO {
    fn from(arg: LabelSaveRequest) -> Self {
        Self {
            id: None,
            short_url: arg.short_url,
            title: arg.title,
            content: arg.content,
            label_pic: arg.label_pic,
            count_view: Some(0),
            label_lock: Some(0),
            count_topic: arg.count_topic,
            count_follow: Some(0),
            isrecommend: arg.isrecommend,
            create_time: None,
            status: arg.status,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LabelUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    // 短网址
    pub short_url: Option<String>,
    // 标签名称
    pub title: Option<String>,
    // 话题描述
    pub content: Option<String>,
    // 话题图片
    pub label_pic: Option<String>,
    // 标签使用数量
    pub count_topic: Option<i32>,
    //1为推荐
    pub isrecommend: Option<i32>,
    // 0未审核，1：正常显示；2：隐藏不显示
    pub status: Option<i32>,
}

impl From<LabelUpdateRequest> for LabelSaveDTO {
    fn from(arg: LabelUpdateRequest) -> Self {
        Self {
            id: arg.id,
            short_url: arg.short_url,
            title: arg.title,
            content: arg.content,
            label_pic: arg.label_pic,
            count_view: Some(0),
            label_lock: Some(0),
            count_topic: arg.count_topic,
            count_follow: Some(0),
            isrecommend: arg.isrecommend,
            create_time: None,
            status: arg.status,
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct LabelSaveDTO {
    //文章ID
    pub id: Option<i64>,
    /// 短网址
    pub short_url: Option<String>,
    /// 标签名称
    pub title: Option<String>,
    /// 话题描述
    pub content: Option<String>,
    /// 话题图片
    pub label_pic: Option<String>,
    /// 访问统计
    pub count_view: Option<i32>,
    /// 话题是否锁定 1 锁定 0 未锁定
    pub label_lock: Option<i32>,
    /// 标签使用数量
    pub count_topic: Option<i32>,
    /// 关注数
    pub count_follow: Option<i32>,
    /// 1为推荐
    pub isrecommend: Option<i32>,
    ///创建时间
    pub create_time: Option<prelude::DateTime>,
    /// 0未审核，1：正常显示；2：隐藏不显示
    pub status: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LabelListVO {
    /// 标签ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 短网址
    pub short_url: Option<String>,
    /// 标签名称
    pub title: Option<String>,
    /// 话题描述
    pub content: Option<String>,
    /// 话题图片
    pub label_pic: Option<String>,
    /// 访问统计
    pub count_view: Option<i32>,
    /// 话题是否锁定 1 锁定 0 未锁定
    pub label_lock: Option<i32>,
    /// 标签使用数量
    pub count_topic: Option<i32>,
    /// 关注数
    pub count_follow: Option<i32>,
    /// 1为推荐
    pub isrecommend: Option<i32>,
    ///创建时间
    pub create_time: Option<prelude::DateTime>,
    /// 0未审核，1：正常显示；2：隐藏不显示
    pub status: Option<i32>,
}

impl From<label::Model> for LabelListVO {
    fn from(arg: label::Model) -> Self {
        Self {
            id: Option::from(arg.id),
            short_url: arg.short_url,
            title: arg.title,
            content: arg.content,
            label_pic: arg.label_pic,
            count_view: arg.count_view,
            label_lock: arg.label_lock,
            count_topic: arg.count_topic,
            count_follow: arg.count_follow,
            isrecommend: arg.isrecommend,
            create_time: arg.create_time,
            status: arg.status,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LabelDetailVO {
    /// 标签ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 短网址
    pub short_url: Option<String>,
    /// 标签名称
    pub title: Option<String>,
    /// 话题描述
    pub content: Option<String>,
    /// 话题图片
    pub label_pic: Option<String>,
    /// 访问统计
    pub count_view: Option<i32>,
    /// 话题是否锁定 1 锁定 0 未锁定
    pub label_lock: Option<i32>,
    /// 标签使用数量
    pub count_topic: Option<i32>,
    /// 关注数
    pub count_follow: Option<i32>,
    /// 1为推荐
    pub isrecommend: Option<i32>,
    ///创建时间
    pub create_time: Option<String>,
    /// 0未审核，1：正常显示；2：隐藏不显示
    pub status: Option<i32>,
}

impl From<label::Model> for LabelDetailVO {
    fn from(arg: label::Model) -> Self {
        Self {
            id: Option::from(arg.id),
            short_url: arg.short_url,
            title: arg.title,
            content: arg.content,
            label_pic: arg.label_pic,
            count_view: arg.count_view,
            label_lock: arg.label_lock,
            count_topic: arg.count_topic,
            count_follow: arg.count_follow,
            isrecommend: arg.isrecommend,
            create_time: arg.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            status: arg.status,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub title: Option<String>,
    // 状态查询（0所有，1查询为0的数据，2查询为1的数据）
    pub status: Option<i32>,
}


/// 条件
#[derive(Clone)]
pub struct PageWhere {
    pub title: Option<String>,
    // 状态查询（0所有，1查询为0的数据，2查询为1的数据）
    pub status: Option<i32>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut title = None;
        if self.title != Some("".to_string()) {
            title = self.title.clone();
        }
        
        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        Self {
            title,
            status,
        }
    }
}

pub struct LabelModel;

impl LabelModel {
    pub async fn insert(db: &DbConn, form_data: &LabelSaveDTO)  -> Result<i64, DbErr> {
        let payload = label::ActiveModel {
            id:              Set(form_data.id.unwrap_or_default().to_owned()),
            short_url:       Set(form_data.short_url.to_owned()),
            title:           Set(form_data.title.to_owned()),
            content:         Set(form_data.content.to_owned()),
            label_pic:       Set(form_data.label_pic.to_owned()),
            isrecommend:     Set(form_data.isrecommend.to_owned()),
            status:          Set(form_data.status.to_owned()),
            create_time:     Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        Label::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id as i64)
    }

    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        label::Entity::delete_many()
            .filter(label::Column::Id.is_in(ids))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, form_data: &LabelSaveDTO) -> Result<i64, DbErr> {
        let payload = label::ActiveModel {
            short_url:       Set(form_data.short_url.to_owned()),
            title:           Set(form_data.title.to_owned()),
            content:         Set(form_data.content.to_owned()),
            label_pic:       Set(form_data.label_pic.to_owned()),
            isrecommend:     Set(form_data.isrecommend.to_owned()),
            status:          Set(form_data.status.to_owned()),
            ..Default::default()
        };

        let update_result: UpdateResult = Label::update_many()
            .set(payload)
            .filter(label::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// 根据指定多个id批量假删除数据
    pub async fn batch_update_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        let update_result: UpdateResult = Label::update_many()
            .set(label::ActiveModel {
                deleted: Set(Some(1).to_owned()),
                ..Default::default()
            })
            .filter(label::Column::Id.is_in(ids))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// 标签短网址唯一性校验
    pub async fn find_by_short_url_unique(db: &DbConn, short_url: &Option<String>, id: &Option<i64>) -> Result<i64, DbErr> {
        let res = Label::find()
            .filter(label::Column::ShortUrl.eq(short_url.clone().unwrap_or_default()))
            .apply_if(id.clone(), |query, v| {
                query.filter(label::Column::Id.ne(v))
            })
            .count(db).await? as i64;
        Ok(res)
    }

    /// 标签名称唯一性校验
    pub async fn find_by_title_unique(db: &DbConn, title: &Option<String>, id: &Option<i64>) -> Result<i64, DbErr> {
        let res = Label::find()
            .filter(label::Column::Title.eq(title.clone().unwrap_or_default()))
            .apply_if(id.clone(), |query, v| {
                query.filter(label::Column::Id.ne(v))
            })
            .count(db).await? as i64;
        Ok(res)
    }

    /// 根据id查询
    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<label::Model>, DbErr> {
        let res = Label::find_by_id(id.clone().unwrap_or_default())
            .one(db)
            .await?;
        Ok(res)
    }

    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        Label::find()
            .apply_if(wheres.title, |query, v| {
                query.filter(label::Column::Title.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(label::Column::Status.eq(v))
            })
            .count(db)
            .await
            .map(|c| c as i64)
    }

    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        wheres: PageWhere,
    ) -> Result<(Vec<label::Model>, i64), DbErr> {
        let paginator = Label::find()
            .apply_if(wheres.title, |query, v| {
                query.filter(label::Column::Title.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(label::Column::Status.eq(v))
            })
            .order_by_desc(label::Column::Id)
            .paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}