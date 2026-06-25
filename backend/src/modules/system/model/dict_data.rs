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
use crate::modules::system::entity::{dict, dict_data, dict_data::Entity as DictData};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};
use sea_orm::*;
use sea_orm::{DbConn, DbErr, EntityTrait};


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DictDataSaveRequest {
    // 字典标签
    pub dict_label: Option<String>,
    // 字典键值
    pub dict_value: Option<String>,
    // 字典编码
    pub dict_code: Option<String>,
    // 字典排序
    pub dict_sort: Option<i32>,
    // 样式属性（其他样式扩展）
    pub css_class: Option<String>,
    // 表格回显样式
    pub list_class: Option<String>,
    // 状态（0正常 1停用）
    pub status: Option<i32>,
    // 备注
    pub remark: Option<String>,
}

impl From<DictDataSaveRequest> for DictDataSaveDTO {
    fn from(data: DictDataSaveRequest) -> Self {
        Self {
            id: None,
            dict_label: data.dict_label,
            dict_value: data.dict_value,
            dict_code: data.dict_code,
            dict_sort: data.dict_sort,
            css_class: data.css_class,
            list_class: data.list_class,
            is_default: Some(0),
            status: data.status,
            create_by: None,
            update_by: None,
            remark: data.remark,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DictDataUpdateRequest {
    // 字典编码
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    // 字典标签
    pub dict_label: Option<String>,
    // 字典键值
    pub dict_value: Option<String>,
    // 字典编码
    pub dict_code: Option<String>,
    // 字典排序
    pub dict_sort: Option<i32>,
    // 样式属性（其他样式扩展）
    pub css_class: Option<String>,
    // 表格回显样式
    pub list_class: Option<String>,
    // 状态（0正常 1停用）
    pub status: Option<i32>,
    // 备注
    pub remark: Option<String>,
}

impl From<DictDataUpdateRequest> for DictDataSaveDTO {
    fn from(data: DictDataUpdateRequest) -> Self {
        Self {
            id: Option::from(data.id),
            dict_label: data.dict_label,
            dict_value: data.dict_value,
            dict_code: data.dict_code,
            dict_sort: data.dict_sort,
            css_class: data.css_class,
            list_class: data.list_class,
            is_default: None,
            status: data.status,
            create_by: None,
            update_by: None,
            remark: data.remark,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DictDataSaveDTO {
    pub id: Option<i64>,
    // 字典标签
    pub dict_label: Option<String>,
    // 字典键值
    pub dict_value: Option<String>,
    // 字典编码
    pub dict_code: Option<String>,
    // 字典排序
    pub dict_sort: Option<i32>,
    // 样式属性（其他样式扩展）
    pub css_class: Option<String>,
    // 表格回显样式
    pub list_class: Option<String>,
    // 是否默认（Y是 N否）
    pub is_default: Option<i32>,
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
pub struct DictDataDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    // 字典标签
    pub dict_label: Option<String>,
    // 字典键值
    pub dict_value: Option<String>,
    // 字典编码
    pub dict_code: Option<String>,
    // 字典排序
    pub dict_sort: Option<i32>,
    // 样式属性（其他样式扩展）
    pub css_class: Option<String>,
    // 表格回显样式
    pub list_class: Option<String>,
    // 是否默认（Y是 N否）
    pub is_default: Option<i32>,
    // 状态（0正常 1停用）
    pub status: Option<i32>,
    // 创建者
    pub create_by: Option<String>,
    // 更新者
    pub update_by: Option<String>,
    // 备注
    pub remark: Option<String>,
}

impl From<dict_data::Model> for DictDataDetailVO {
    fn from(data: dict_data::Model) -> Self {
        Self {
            id: Option::from(data.id),
            dict_label: data.dict_label,
            dict_value: data.dict_value,
            dict_code: data.dict_code,
            dict_sort: data.dict_sort,
            css_class: data.css_class,
            list_class: data.list_class,
            is_default: data.is_default,
            status: data.status,
            create_by: data.create_by,
            update_by: data.update_by,
            remark: data.remark,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DictDataListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    // 字典标签
    pub dict_label: Option<String>,
    // 字典键值
    pub dict_value: Option<String>,
    // 字典编码
    pub dict_code: Option<String>,
    // 字典排序
    pub dict_sort: Option<i32>,
    // 样式属性（其他样式扩展）
    pub css_class: Option<String>,
    // 表格回显样式
    pub list_class: Option<String>,
    // 是否默认（Y是 N否）
    pub is_default: Option<i32>,
    // 状态（0正常 1停用）
    pub status: Option<i32>,
    // 创建者
    pub create_by: Option<String>,
    // 更新者
    pub update_by: Option<String>,
    // 备注
    pub remark: Option<String>,
}

impl From<dict_data::Model> for DictDataListVO {
    fn from(data: dict_data::Model) -> Self {
        Self {
            id: Option::from(data.id),
            dict_label: data.dict_label,
            dict_value: data.dict_value,
            dict_code: data.dict_code,
            dict_sort: data.dict_sort,
            css_class: data.css_class,
            list_class: data.list_class,
            is_default: data.is_default,
            status: data.status,
            create_by: data.create_by,
            update_by: data.update_by,
            remark: data.remark,
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DictDataOptionVO {
    // 字典标签
    pub label: Option<String>,
    // 字典键值
    pub value: Option<String>,
    pub tag_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataListQuery {
    pub dict_label: Option<String>,
    // 字典编码
    pub dict_code: Option<String>,
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
    pub dict_label: Option<String>,
    // 字典编码
    pub dict_code: Option<String>,
    pub status: Option<i32>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut dict_label = None;
        if self.dict_label != Some("".to_string()) {
            dict_label = self.dict_label.clone();
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
            dict_label,
            dict_code,
            status,
        }
    }
}


pub struct DictDataModel;

impl DictDataModel {
    
    pub async fn insert(db: &DbConn, form_data: &DictDataSaveDTO) -> Result<i64, DbErr> {
        let payload = dict_data::ActiveModel{
            dict_sort:       Set(form_data.dict_sort.to_owned()),
            dict_label:      Set(form_data.dict_label.to_owned()),
            dict_value:      Set(form_data.dict_value.to_owned()),
            dict_code:       Set(form_data.dict_code.to_owned()),
            css_class:       Set(form_data.css_class.to_owned()),
            list_class:      Set(form_data.list_class.to_owned()),
            is_default: match form_data.is_default {
                Some(value) => Set(Option::from(value)),
                None => NotSet,
            },
            status:          Set(form_data.status.to_owned()),
            create_by:       Set(form_data.create_by.to_owned()),
            create_time:     Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            update_by:       Set(form_data.update_by.to_owned()),
            update_time:     Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            remark:          Set(form_data.remark.to_owned()),
            ..Default::default()
        };
        DictData::insert(payload).exec(db).await.map(|r| r.last_insert_id)
    }

    /// ### 批量删除
    /// * db 数据库连接
    /// * ids: 批量删除的id
    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        DictData::delete_many()
            .filter(dict_data::Column::Id.is_in(ids))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
    
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, form_data: &DictDataSaveDTO) -> Result<i64, DbErr> {
        let payload = dict_data::ActiveModel {
            dict_sort:       Set(form_data.dict_sort.to_owned()),
            dict_label:      Set(form_data.dict_label.to_owned()),
            dict_value:      Set(form_data.dict_value.to_owned()),
            dict_code:       Set(form_data.dict_code.to_owned()),
            css_class:       Set(form_data.css_class.to_owned()),
            list_class:      Set(form_data.list_class.to_owned()),
            is_default: match form_data.is_default {
                Some(value) => Set(Option::from(value)),
                None => NotSet,
            },
            status:          Set(form_data.status.to_owned()),
            remark:          Set(form_data.remark.to_owned()),
            update_by:       Set(form_data.update_by.to_owned()),
            update_time:     Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        let update_result: UpdateResult = DictData::update_many()
            .set(payload)
            .filter(dict_data::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db).await?;
        Ok(update_result.rows_affected as i64)
    }

    /// 查询字典数据是否存在
    pub async fn find_by_code_unique(db: &DbConn, dict_code: &Option<String>) -> Result<bool, DbErr> {
        let result_num = DictData::find()
            .filter(dict_data::Column::DictCode.eq(dict_code.clone().unwrap_or_default()))
            .count(db).await? as i64;
        Ok(result_num > 0)
    }
    

    /// 查询字典标签名称是否存在
    pub async fn find_by_label_unique(db: &DbConn, dict_code: &Option<String>, dict_label: &Option<String>, id: &Option<i64>) -> Result<bool, DbErr> {
        let result_num = DictData::find()
            .filter(dict_data::Column::DictCode.eq(dict_code.clone().unwrap_or_default()))
            .filter(dict_data::Column::DictLabel.eq(dict_label.clone().unwrap_or_default()))
            .apply_if(id.clone(), |query, v| {
                query.filter(dict_data::Column::Id.ne(v))
            })
            .count(db).await? as i64;
        Ok(result_num > 0)
    }
    
    /// 根据id查询字典数据
    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<dict_data::Model>, DbErr> {
        DictData::find_by_id(id).one(db).await
    }
    
    pub async fn find_list_by_code(db: &DbConn, dict_code : &Option<String>) -> Result<Vec<dict_data::Model>, DbErr> {
        let query_data = DictData::find()
            .filter(dict_data::Column::DictCode.eq(dict_code.clone().unwrap_or_default()))
            .filter(dict_data::Column::Status.eq(1))
            .order_by_asc(dict_data::Column::DictSort)
            .all(db).await?;
        Ok(query_data)
    }

    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        DictData::find()
            .apply_if(wheres.dict_label, |query, v| {
                query.filter(dict_data::Column::DictLabel.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.dict_code, |query, v| {
                query.filter(dict_data::Column::DictCode.eq(v))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(dict_data::Column::Status.eq(v))
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
    ) -> Result<(Vec<dict_data::Model>, i64), DbErr> {
        let paginator = DictData::find()
            .apply_if(wheres.dict_label, |query, v| {
                query.filter(dict_data::Column::DictLabel.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.dict_code, |query, v| {
                query.filter(dict_data::Column::DictCode.eq(v))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(dict_data::Column::Status.eq(v))
            })
            .order_by_desc(dict_data::Column::Id)
            .paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }

}