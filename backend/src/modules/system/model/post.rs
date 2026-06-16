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
use crate::modules::system::entity::{post, post::Entity as Post};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};
use sea_orm::prelude::DateTime;
use sea_orm::*;


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostSaveRequest {
    /// 岗位编码，权限控制的时候使用
    pub post_code: Option<String>,
    /// 岗位名称
    pub post_name: Option<String>,
    /// 岗位状态
    pub status: Option<i32>,
    /// 岗位排序
    pub sort: Option<i32>,
    /// 备注
    pub remark: Option<String>,
}

impl From<PostSaveRequest> for PostSaveDTO {
    fn from(value: PostSaveRequest) -> Self {
        Self {
            id: None,
            post_code: value.post_code,
            post_name: value.post_name,
            status: value.status,
            sort: value.sort,
            remark: value.remark,
            create_time: None,
            update_time: None,
            is_del: None,
        }
    }
    
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 岗位编码，权限控制的时候使用
    pub post_code: Option<String>,
    /// 岗位名称
    pub post_name: Option<String>,
    /// 岗位状态
    pub status: Option<i32>,
    /// 岗位排序
    pub sort: Option<i32>,
    /// 备注
    pub remark: Option<String>,
}

impl From<PostUpdateRequest> for PostSaveDTO {
    fn from(value: PostUpdateRequest) -> Self {
        Self {
            id: value.id,
            post_code: value.post_code,
            post_name: value.post_name,
            status: value.status,
            sort: value.sort,
            remark: value.remark,
            create_time: None,
            update_time: None,
            is_del: None,
        }
    }
}


pub struct PostSaveDTO {
    /// ID
    pub id: Option<i64>,
    /// 岗位编码，权限控制的时候使用
    pub post_code: Option<String>,
    /// 岗位名称
    pub post_name: Option<String>,
    /// 岗位状态
    pub status: Option<i32>,
    /// 岗位排序
    pub sort: Option<i32>,
    /// 备注
    pub remark: Option<String>,
    /// 创建日期
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 是否删除
    pub is_del: Option<i32>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostDetailVO {
    pub id: Option<i64>,
    /// 岗位编码，权限控制的时候使用
    pub post_code: Option<String>,
    /// 岗位名称
    pub post_name: Option<String>,
    /// 岗位状态
    pub status: Option<i32>,
    /// 岗位排序
    pub sort: Option<i32>,
    /// 备注
    pub remark: Option<String>,
    /// 创建日期
    pub create_time: Option<String>,
    /// 更新时间
    pub update_time: Option<String>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id:  Option<i64>,
    /// 岗位编码，权限控制的时候使用
    pub post_code: Option<String>,
    pub post_name: Option<String>,
    /// 岗位状态
    pub status: Option<i32>,
    /// 岗位排序
    pub sort:  Option<i32>,
    
    /// 备注
    pub remark: Option<String>,
    pub create_time:Option<String>,
}

impl From<post::Model> for PostListVO {
    fn from(value: post::Model) -> Self {
        Self {
            id: Option::from(value.id),
            post_code: value.post_code,
            post_name: value.post_name,
            status: value.status,
            sort: value.sort,
            remark: value.remark,
            create_time: value.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}



#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub post_code: Option<String>,
    pub post_name: Option<String>,
    pub status: Option<i32>,
}


/// 条件
#[derive(Clone)]
pub struct PageWhere {
    pub post_code: Option<String>,
    pub post_name: Option<String>,
    pub status: Option<i32>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut post_code = None;
        if self.post_code != Some("".to_string()) {
            post_code = self.post_code.clone();
        }

        let mut post_name = None;
        if self.post_name != Some("".to_string()) {
            post_name = self.post_name.clone();
        }

        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        Self {
            post_code,
            post_name,
            status,
        }
    }
}

pub struct PostModel;

impl PostModel {
    pub async fn insert(db: &DbConn, form_data: PostSaveDTO) -> Result<i64, DbErr> {
        let payload = post::ActiveModel {
            post_code:    Set(form_data.post_code.to_owned()),
            post_name:    Set(form_data.post_name.to_owned()),
            status:       Set(form_data.status.to_owned()),
            sort:         Set(form_data.sort.to_owned()),
            remark:       Set(form_data.remark.to_owned()),
            ..Default::default()
        };

        Post::insert(payload)
        .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        Post::delete_many()
            .filter(post::Column::Id.is_in(ids))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn update_by_id(db: &DbConn, id: i64, form_data: PostSaveDTO) -> Result<i64, DbErr> {
        let payload = post::ActiveModel {
            post_code:   Set(form_data.post_code.to_owned()),
            post_name:   Set(form_data.post_name.to_owned()),
            status:      Set(form_data.status.to_owned()),
            sort:        Set(form_data.sort.to_owned()),
            remark:      Set(form_data.remark).to_owned(),
            ..Default::default()
        };
        
        let update_result: UpdateResult = Post::update_many()
            .set(payload)
            .filter(post::Column::Id.eq(id))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// 根据岗位名称查询岗位数量
    pub async fn find_by_post_name_unique(db: &DbConn, post_name: &Option<String>, id: &Option<i64>) -> Result<i64, DbErr> {
        Post::find()
            .filter(post::Column::PostName.eq(post_name.clone().unwrap_or_default()))
            .apply_if(id.clone(), |query, v| {
                query.filter(post::Column::Id.ne(v))
            })
            .count(db)
            .await
            .map(|c| c as i64)
    }

    /// 根据岗位编码查询岗位数量
    pub async fn find_by_post_code_unique(db: &DbConn, post_code: &Option<String>, id: &Option<i64>) -> Result<i64, DbErr> {
        Post::find()
            .filter(post::Column::PostCode.eq(post_code.clone().unwrap_or_default()))
            .apply_if(id.clone(), |query, v| {
                query.filter(post::Column::Id.ne(v))
            })
            .count(db)
            .await
            .map(|c| c as i64)
    }

    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<post::Model>, DbErr> {
        Post::find_by_id(id.unwrap_or_default())
            .one(db)
            .await
    }

    pub async fn find_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<Vec<post::Model>, DbErr> {
        Post::find()
            .filter(post::Column::Id.is_in(ids))
            .all(db)
            .await
    }

    pub async fn find_all(db: &DbConn) -> Result<Vec<post::Model>, DbErr> {
        Post::find()
            .all(db)
            .await
    }


    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        Post::find()
            .apply_if(wheres.post_code, |query, v| {
                query.filter(post::Column::PostCode.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.post_name, |query, v| {
                query.filter(post::Column::PostName.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(post::Column::Status.eq(v))
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
    ) -> Result<(Vec<post::Model>, i64), DbErr> {
        let paginator = Post::find()
            .apply_if(wheres.post_code, |query, v| {
                query.filter(post::Column::PostCode.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.post_name, |query, v| {
                query.filter(post::Column::PostName.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(post::Column::Status.eq(v))
            })
            .order_by_asc(post::Column::Id)
            .paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}