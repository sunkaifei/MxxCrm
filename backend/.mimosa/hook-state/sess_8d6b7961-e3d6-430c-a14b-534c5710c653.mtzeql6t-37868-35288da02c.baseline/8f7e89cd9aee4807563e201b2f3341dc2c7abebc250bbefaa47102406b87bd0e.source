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
use crate::modules::system::entity::{salary_band, salary_band::Entity as SalaryBand};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};
use chrono::Local;
use rust_decimal::Decimal;
use sea_orm::prelude::DateTime;
use sea_orm::*;

/// 岗位薪资带宽新增请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SalaryBandSaveRequest {
    /// 岗位ID
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub post_id: Option<i64>,
    /// 带宽下限
    pub min_salary: Option<Decimal>,
    /// 带宽上限
    pub max_salary: Option<Decimal>,
    /// 1启用 0停用
    pub status: Option<i32>,
    /// 备注
    pub remark: Option<String>,
}

impl From<SalaryBandSaveRequest> for SalaryBandSaveDTO {
    fn from(value: SalaryBandSaveRequest) -> Self {
        Self {
            id: None,
            post_id: value.post_id,
            min_salary: value.min_salary,
            max_salary: value.max_salary,
            status: value.status,
            remark: value.remark,
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
            deleted: None,
        }
    }
}

/// 岗位薪资带宽更新请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SalaryBandUpdateRequest {
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 岗位ID
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub post_id: Option<i64>,
    /// 带宽下限
    pub min_salary: Option<Decimal>,
    /// 带宽上限
    pub max_salary: Option<Decimal>,
    /// 1启用 0停用
    pub status: Option<i32>,
    /// 备注
    pub remark: Option<String>,
}

impl From<SalaryBandUpdateRequest> for SalaryBandSaveDTO {
    fn from(value: SalaryBandUpdateRequest) -> Self {
        Self {
            id: value.id,
            post_id: value.post_id,
            min_salary: value.min_salary,
            max_salary: value.max_salary,
            status: value.status,
            remark: value.remark,
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
            deleted: None,
        }
    }
}

/// 岗位薪资带宽数据操作DTO
pub struct SalaryBandSaveDTO {
    pub id: Option<i64>,
    pub post_id: Option<i64>,
    pub min_salary: Option<Decimal>,
    pub max_salary: Option<Decimal>,
    pub status: Option<i32>,
    pub remark: Option<String>,
    pub create_by: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
    pub deleted: Option<i32>,
}

/// 岗位薪资带宽列表VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SalaryBandListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 岗位ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub post_id: Option<i64>,
    /// 岗位名称
    pub post_name: Option<String>,
    /// 带宽下限
    pub min_salary: Option<Decimal>,
    /// 带宽上限
    pub max_salary: Option<Decimal>,
    /// 1启用 0停用
    pub status: Option<i32>,
    /// 备注
    pub remark: Option<String>,
    /// 创建时间
    pub create_time: Option<String>,
}

impl From<salary_band::Model> for SalaryBandListVO {
    fn from(value: salary_band::Model) -> Self {
        Self {
            id: Some(value.id),
            post_id: Some(value.post_id),
            post_name: None,
            min_salary: Some(value.min_salary),
            max_salary: Some(value.max_salary),
            status: value.status,
            remark: value.remark,
            create_time: value
                .create_time
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// 岗位薪资带宽详情VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SalaryBandDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub post_id: Option<i64>,
    pub post_name: Option<String>,
    pub min_salary: Option<Decimal>,
    pub max_salary: Option<Decimal>,
    pub status: Option<i32>,
    pub remark: Option<String>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}

/// 岗位薪资带宽分页查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    /// 岗位ID过滤
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub post_id: Option<i64>,
    /// 状态过滤
    pub status: Option<i32>,
}

/// 查询条件
#[derive(Clone)]
pub struct PageWhere {
    pub post_id: Option<i64>,
    pub status: Option<i32>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut post_id = None;
        if self.post_id.is_some() && self.post_id != Some(0) {
            post_id = self.post_id;
        }

        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        Self { post_id, status }
    }
}

pub struct SalaryBandModel;

impl SalaryBandModel {
    /// 新增带宽
    pub async fn insert(db: &DbConn, form_data: SalaryBandSaveDTO) -> Result<i64, DbErr> {
        let payload = salary_band::ActiveModel {
            post_id: Set(form_data.post_id.unwrap_or_default()),
            min_salary: Set(form_data.min_salary.unwrap_or_default()),
            max_salary: Set(form_data.max_salary.unwrap_or_default()),
            status: Set(form_data.status),
            remark: Set(form_data.remark),
            create_by: Set(form_data.create_by),
            create_time: Set(Option::from(Local::now().naive_local())),
            ..Default::default()
        };

        SalaryBand::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 批量删除带宽（逻辑删除）
    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        let now = Local::now().naive_local();
        SalaryBand::update_many()
            .col_expr(salary_band::Column::Deleted, sea_orm::sea_query::Expr::value(1))
            .col_expr(
                salary_band::Column::UpdateTime,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(salary_band::Column::Id.is_in(ids))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    /// 更新带宽
    pub async fn update_by_id(db: &DbConn, id: i64, form_data: SalaryBandSaveDTO) -> Result<i64, DbErr> {
        let payload = salary_band::ActiveModel {
            post_id: Set(form_data.post_id.unwrap_or_default()),
            min_salary: Set(form_data.min_salary.unwrap_or_default()),
            max_salary: Set(form_data.max_salary.unwrap_or_default()),
            status: Set(form_data.status),
            remark: Set(form_data.remark),
            update_time: Set(Option::from(Local::now().naive_local())),
            ..Default::default()
        };

        let update_result: UpdateResult = SalaryBand::update_many()
            .set(payload)
            .filter(salary_band::Column::Id.eq(id))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// 查询带宽详情
    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<salary_band::Model>, DbErr> {
        SalaryBand::find_by_id(id.unwrap_or_default())
            .one(db)
            .await
    }

    /// 按岗位ID查询启用的带宽（一个岗位一条，覆盖式维护）
    pub async fn find_by_post_id(db: &DbConn, post_id: i64) -> Result<Option<salary_band::Model>, DbErr> {
        SalaryBand::find()
            .filter(salary_band::Column::PostId.eq(post_id))
            .filter(salary_band::Column::Deleted.eq(0))
            .order_by_desc(salary_band::Column::Id)
            .one(db)
            .await
    }

    /// 按岗位ID集合批量查询启用的带宽
    pub async fn find_by_post_ids(db: &DbConn, post_ids: Vec<i64>) -> Result<Vec<salary_band::Model>, DbErr> {
        if post_ids.is_empty() {
            return Ok(vec![]);
        }
        SalaryBand::find()
            .filter(salary_band::Column::PostId.is_in(post_ids))
            .filter(salary_band::Column::Deleted.eq(0))
            .all(db)
            .await
    }

    /// 统计带宽数量
    pub async fn select_count(db: &DbConn, wheres: PageWhere) -> Result<i64, DbErr> {
        SalaryBand::find()
            .filter(salary_band::Column::Deleted.eq(0))
            .apply_if(wheres.post_id, |query, v| {
                query.filter(salary_band::Column::PostId.eq(v))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(salary_band::Column::Status.eq(v))
            })
            .count(db)
            .await
            .map(|c| c as i64)
    }

    /// 分页查询带宽
    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        wheres: PageWhere,
    ) -> Result<(Vec<salary_band::Model>, i64), DbErr> {
        let paginator = SalaryBand::find()
            .filter(salary_band::Column::Deleted.eq(0))
            .apply_if(wheres.post_id, |query, v| {
                query.filter(salary_band::Column::PostId.eq(v))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(salary_band::Column::Status.eq(v))
            })
            .order_by_desc(salary_band::Column::Id)
            .paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator
            .fetch_page((page - 1) as u64)
            .await
            .map(|p| (p, num_pages))
    }
}
