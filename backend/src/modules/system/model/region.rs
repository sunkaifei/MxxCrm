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
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::system::entity::{post, region, region::Entity as Region};
use crate::utils::string_utils::{serialize_option_u64_to_string,deserialize_string_to_u64};


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegionSaveRequest {
    /// 短网址
    pub short_url: Option<String>,
    /// 行政区域父ID，例如区县的pid指向市，市的pid指向省，省的pid则是0
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub parent_id: Option<i64>,
    /// 地区别名
    pub title: Option<String>,
    /// 行政区域名称
    pub region_name: Option<String>,
    /// 行政区域类型，如如1则是省， 如果是2则是市，如果是3则是区县
    pub region_type: Option<i32>,
    /// 行政区域编码
    pub region_code: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 显示状态
    pub status: Option<i32>,
}

impl From<RegionSaveRequest> for RegionSaveDTO {
    fn from(request: RegionSaveRequest) -> Self {
        Self {
            id: None,
            short_url: request.short_url,
            parent_id: request.parent_id,
            title: request.title,
            region_name: request.region_name,
            region_type: request.region_type,
            region_code: request.region_code,
            sort: request.sort,
            status: request.status,
       }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegionUpdateRequest {
    /// 主键
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 短网址
    pub short_url: Option<String>,
    /// 行政区域父ID，例如区县的pid指向市，市的pid指向省，省的pid则是0
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub parent_id: Option<i64>,
    /// 地区别名
    pub title: Option<String>,
    /// 行政区域名称
    pub region_name: Option<String>,
    /// 行政区域类型，如如1则是省， 如果是2则是市，如果是3则是区县
    pub region_type: Option<i32>,
    /// 行政区域编码
    pub region_code: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 显示状态
    pub status: Option<i32>,
}

impl From<RegionUpdateRequest> for RegionSaveDTO {
    fn from(request: RegionUpdateRequest) -> Self {
        Self {
            id: request.id,
            short_url: request.short_url,
            parent_id: request.parent_id,
            title: request.title,
            region_name: request.region_name,
            region_type: request.region_type,
            region_code: request.region_code,
            sort: request.sort,
            status: request.status,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegionSaveDTO {
    /// 主键
    pub id: Option<i64>,
    /// 短网址
    pub short_url: Option<String>,
    /// 行政区域父ID，例如区县的pid指向市，市的pid指向省，省的pid则是0
    pub parent_id: Option<i64>,
    /// 地区别名
    pub title: Option<String>,
    /// 行政区域名称
    pub region_name: Option<String>,
    /// 行政区域类型，如如1则是省， 如果是2则是市，如果是3则是区县
    pub region_type: Option<i32>,
    /// 行政区域编码
    pub region_code: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 显示状态
    pub status: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegionDetailVO {
    /// 主键
    pub id: Option<i64>,
    /// 短网址
    pub short_url: Option<String>,
    /// 行政区域父ID，例如区县的pid指向市，市的pid指向省，省的pid则是0
    pub parent_id: Option<i64>,
    /// 地区别名
    pub title: Option<String>,
    /// 行政区域名称
    pub region_name: Option<String>,
    /// 行政区域类型，如如1则是省， 如果是2则是市，如果是3则是区县
    pub region_type: Option<i32>,
    /// 行政区域编码
    pub region_code: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 显示状态
    pub status: Option<i32>,
}

impl From<region::Model> for RegionDetailVO {
    fn from(model: region::Model) -> Self {
        Self {
            id: Option::from(model.id),
            short_url: model.short_url,
            parent_id: model.parent_id,
            title: model.title,
            region_name: model.region_name,
            region_type: model.region_type,
            region_code: model.region_code,
            sort: model.sort,
            status: model.status,
        }
   }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegionListVO {
    /// 主键
    pub id: Option<i64>,
    /// 短网址
    pub short_url: Option<String>,
    /// 行政区域父ID，例如区县的pid指向市，市的pid指向省，省的pid则是0
    pub parent_id: Option<i64>,
    /// 地区别名
    pub title: Option<String>,
    /// 行政区域名称
    pub region_name: Option<String>,
    /// 行政区域类型，如如1则是省， 如果是2则是市，如果是3则是区县
    pub region_type: Option<i32>,
    /// 行政区域编码
    pub region_code: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 显示状态
    pub status: Option<i32>,
}

impl From<region::Model> for RegionListVO {
    fn from(model: region::Model) -> Self {
        Self {
            id: Option::from(model.id),
            short_url: model.short_url,
            parent_id: model.parent_id,
            title: model.title,
            region_name: model.region_name,
            region_type: model.region_type,
            region_code: model.region_code,
            sort: model.sort,
            status: model.status,
        }
    }
}


/// 行政区域树
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegionTreeVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub parent_id: Option<i64>,
    pub title: Option<String>,
    pub region_name: Option<String>,
    pub sort: Option<i32>,
    /// 显示状态
    pub status: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<RegionTreeVO>>,
}

/// 行政区域树
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegionShortTreeVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub parent_id: Option<i64>,
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<RegionShortTreeVO>>,
}

/// 用户端行政区域树VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegionUserTreeVO {
    pub id: Option<String>,
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<RegionUserTreeVO>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery{
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub title: Option<String>,
    pub status: Option<i32>,
}


/// 条件
#[derive(Clone)]
pub struct PageWhere {
    pub region_name: Option<String>,
    pub status: Option<i32>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut region_name = None;
        if self.region_name != Some("".to_string()) {
            region_name = self.region_name.clone();
        }

        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        Self {
            region_name,
            status,
        }
    }
}

pub struct RegionModel;

impl RegionModel {
    
    pub async fn insert(db: &DbConn, form_data: &RegionSaveDTO) -> Result<i64, DbErr> {
        let payload = region::ActiveModel {
            id:              Set(form_data.id.unwrap_or_default().to_owned()),
            short_url:       Set(form_data.short_url.to_owned()),
            parent_id:       Set(form_data.parent_id.to_owned()),
            title:           Set(form_data.title.to_owned()),
            region_name:     Set(form_data.region_name.to_owned()),
            region_type:     Set(form_data.region_type.to_owned()),
            region_code:     Set(form_data.region_code.to_owned()),
            sort:            Set(form_data.sort.to_owned()),
            status:          Set(form_data.status.to_owned()),
            ..Default::default()
        };
        Region::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        Region::delete_many()
            .filter(region::Column::Id.is_in(ids))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
    
    pub async fn update_by_id(db: &DbConn, id: i64, form_data: RegionSaveDTO) -> Result<i64, DbErr> {
        let payload = region::ActiveModel {
            short_url:       Set(form_data.short_url.to_owned()),
            parent_id:       Set(form_data.parent_id.to_owned()),
            title:           Set(form_data.title.to_owned()),
            region_name:     Set(form_data.region_name.to_owned()),
            region_type:     Set(form_data.region_type.to_owned()),
            region_code:     Set(form_data.region_code.to_owned()),
            sort:            Set(form_data.sort.to_owned()),
            status:          Set(form_data.status.to_owned()),
            ..Default::default()
        };
        
        let update_result: UpdateResult = Region::update_many()
            .set(payload)
            .filter(post::Column::Id.eq(id))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }
    
    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<region::Model>, DbErr> {
        Region::find_by_id(id.unwrap_or_default())
            .one(db)
            .await
    }
    
    /// 根据父级id查询
    pub async fn find_by_parent_ids(db: &DbConn, ids: &Vec<i64>) -> Result<Vec<region::Model>, DbErr> {
        Region::find()
            .filter(region::Column::ParentId.is_in(ids.clone()))
            .all(db)
            .await
    }

    pub async fn find_all(db: &DbConn) -> Result<Vec<region::Model>, DbErr> {
        Region::find()
            .all(db)
            .await
    }


    // 搜索
    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        Region::find()
            .apply_if(wheres.region_name, |query, v| {
                query.filter(region::Column::RegionName.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(region::Column::Status.eq(v))
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
    ) -> Result<(Vec<region::Model>, i64), DbErr> {
        let paginator = Region::find()
            .apply_if(wheres.region_name, |query, v| {
                query.filter(region::Column::RegionName.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(region::Column::Status.eq(v))
            })
            .order_by_asc(region::Column::Sort)
            .paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}