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
use crate::modules::system::entity::{dict, dict::Entity as Dict};
use crate::utils::string_utils::{serialize_option_u64_to_string, deserialize_string_to_u64};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DictSaveRequest {
    // 字典名称
    pub dict_name: Option<String>,
    // 字典编码
    pub dict_code: Option<String>,
    // 排序
    pub sort: Option<i32>,
    // 状态（0正常 1停用）
    pub status: Option<i32>,
    // 创建者
    pub create_by: Option<String>,
    // 更新者
    pub update_by: Option<String>,
    // 备注
    pub remark: Option<String>,
}

impl From<DictSaveRequest> for DictSaveDTO {
    fn from(d: DictSaveRequest) -> Self {
        Self {
            id: None,
            dict_name: d.dict_name,
            dict_code: d.dict_code,
            sort: d.sort,
            status: d.status,
            create_by: d.create_by,
            update_by: d.update_by,
            remark: d.remark,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DictUpdateRequest {
    // 字典主键
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    // 字典名称
    pub dict_name: Option<String>,
    // 字典编码
    pub dict_code: Option<String>,
    // 排序
    pub sort: Option<i32>,
    // 状态（0正常 1停用）
    pub status: Option<i32>,
    // 创建者
    pub create_by: Option<String>,
    // 更新者
    pub update_by: Option<String>,
    // 备注
    pub remark: Option<String>,
}

impl From<DictUpdateRequest> for DictSaveDTO {
    fn from(d: DictUpdateRequest) -> Self {
        Self {
            id: d.id,
            dict_name: d.dict_name,
            dict_code: d.dict_code,
            sort: d.sort,
            status: d.status,
            create_by: d.create_by,
            update_by: d.update_by,
            remark: d.remark,
        }
    }
}

///字典类型分页响应数据
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DictListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    // 字典名称
    pub dict_name: Option<String>,
    // 字典编码
    pub dict_code: Option<String>,
    // 排序
    pub sort: Option<i32>,
    // 状态（0正常 1停用）
    pub status: Option<i32>,
    // 备注
    pub remark: Option<String>,
    ///更新时间
    pub update_time: String,
}

impl From<dict::Model> for DictListVO {
    fn from(m: dict::Model) -> Self {
        Self {
            id: Option::from(m.id),
            dict_name: m.dict_name,
            dict_code: m.dict_code,
            sort: m.sort,
            status: m.status,
            remark: m.remark,
            update_time: m.update_time.unwrap_or_default().to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DictSaveDTO {
    // 字典主键
    pub id: Option<i64>,
    // 字典名称
    pub dict_name: Option<String>,
    // 字典编码
    pub dict_code: Option<String>,
    // 排序
    pub sort: Option<i32>,
    // 状态（0正常 1停用）
    pub status: Option<i32>,
    // 创建者
    pub create_by: Option<String>,
    // 更新者
    pub update_by: Option<String>,
    // 备注
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DictDetailVO {
    // 字典主键
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    // 字典名称
    pub dict_name: Option<String>,
    // 字典编码
    pub dict_code: Option<String>,
    // 排序
    pub sort: Option<i32>,
    // 状态（0正常 1停用）
    pub status: Option<i32>,
    // 备注
    pub remark: Option<String>,
}

impl From<dict::Model> for DictDetailVO {
    fn from(m: dict::Model) -> Self {
        Self {
            id: Option::from(m.id),
            dict_name: m.dict_name,
            dict_code: m.dict_code,
            sort: m.sort,
            status: m.status,
            remark: m.remark,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TypeListQuery {
    pub dict_name: Option<String>,
    // 字典编码
    pub dict_code: Option<String>,
    // 状态查询（0和空都是所有，1查询为0的数据，2查询为1的数据）
    pub status: Option<i32>,
    // 当前页码数
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
}


// 条件
#[derive(Clone)]
pub struct PageWhere {
    // 字典名称
    pub dict_name: Option<String>,
    // 字典编码
    pub dict_code: Option<String>,
    pub status: Option<i32>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut dict_name = None;
        if self.dict_name != Some("".to_string()) {
            dict_name = self.dict_name.clone();
        }

        let mut dict_code = None;
        if self.dict_code != Some("".to_string()) {
            dict_code = self.dict_code.clone();
        }
        
        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        Self {
            dict_name,
            dict_code,
            status,
        }
    }
}

pub struct DictModel;
impl DictModel {

    pub async fn insert(db: &DbConn, form_data: &DictSaveDTO) -> Result<i64, DbErr> {
        let entity = dict::ActiveModel {
            id:              Set(form_data.id.unwrap_or_default().to_owned()),
            dict_name:       Set(form_data.dict_name.to_owned()),
            dict_code:       Set(form_data.dict_code.to_owned()),
            sort:            Set(form_data.sort.to_owned()),
            status:          Set(form_data.status.to_owned()),
            create_by:       Set(form_data.create_by.to_owned()),
            update_by:       Set(form_data.update_by.to_owned()),
            remark:          Set(form_data.remark.to_owned()),
            create_time:     Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            update_time:     Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        Dict::insert(entity)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 批量删除
    pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        Dict::delete_many()
            .filter(dict::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn update(db: &DbConn, id: &Option<i64>, form_data: &DictSaveDTO) -> Result<i64, DbErr> {
        let entity = dict::ActiveModel {
            dict_name:       Set(form_data.dict_name.to_owned()),
            dict_code:       Set(form_data.dict_code.to_owned()),
            sort:            Set(form_data.sort.to_owned()),
            status:          Set(form_data.status.to_owned()),
            update_by:       Set(form_data.update_by.to_owned()),
            remark:          Set(form_data.remark.to_owned()),
            update_time:     Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let update_result: UpdateResult = Dict::update_many()
            .set(entity)
            .filter(dict::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db).await?;
        Ok(update_result.rows_affected as i64)
    }

    /// 根据字典名称查询是否被占用
    pub async fn find_by_name_unique(db: &DbConn, name: &Option<String>, id: &Option<i64>) -> Result<bool, DbErr> {
        let result_num = Dict::find()
            .filter(dict::Column::DictName.eq(name.clone().unwrap_or_default()))
            .apply_if(id.clone(), |query, v| {
                query.filter(dict::Column::Id.ne(v))
            })
            .count(db).await? as i64;
        Ok(result_num > 0)
    }

    /// 根据字典编码查询是否被占用
    pub async fn find_by_code_unique(db: &DbConn, dict_code: &Option<String>, id: &Option<i64>) -> Result<bool, DbErr> {
        let result_num = Dict::find()
            .filter(dict::Column::DictCode.eq(dict_code.clone().unwrap_or_default()))
            .apply_if(id.clone(), |query, v| {
                query.filter(dict::Column::Id.ne(v))
            })
            .count(db).await? as i64;
        Ok(result_num > 0)
    }

    /// 根据id查询
    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<dict::Model>, DbErr> {
        Dict::find_by_id(id.clone().unwrap_or_default())
            .one(db)
            .await
    }

    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        Dict::find()
            .apply_if(wheres.dict_name, |query, v| {
                query.filter(dict::Column::DictName.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.dict_code, |query, v| {
                query.filter(dict::Column::DictCode.eq(v))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(dict::Column::Status.eq(v))
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
    ) -> Result<(Vec<dict::Model>, i64), DbErr> {
        let paginator = Dict::find()
            .apply_if(wheres.dict_name, |query, v| {
                query.filter(dict::Column::DictName.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.dict_code, |query, v| {
                query.filter(dict::Column::DictCode.eq(v))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(dict::Column::Status.eq(v))
            })
            .order_by_desc(dict::Column::Sort)
            .paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }

}