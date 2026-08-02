//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::modules::website::entity::{content_model_field, content_model_field::Entity as ContentModelField};
use sea_orm::prelude::DateTime;
use sea_orm::*;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

/// 模型字段新增请求
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct FieldSaveRequest {
    // 模型ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub model_id: Option<i64>,
    // 字段名称
    pub field_name: Option<String>,
    // 字段标签
    pub field_label: Option<String>,
    // 字段类型
    pub field_type: Option<i32>,
    // 字段选项
    pub field_options: Option<String>,
    // 默认值
    pub default_value: Option<String>,
    // 占位提示
    pub placeholder: Option<String>,
    // 是否必填
    pub is_required: Option<i32>,
    // 是否可搜索
    pub is_searchable: Option<i32>,
    // 列表是否显示
    pub is_list_show: Option<i32>,
    // 详情是否显示
    pub is_detail_show: Option<i32>,
    // 排序
    pub sort: Option<i32>,
    // 状态：0停用，1正常
    pub status: Option<i32>,
}

impl From<FieldSaveRequest> for FieldSaveDTO {
    fn from(form_data: FieldSaveRequest) -> Self {
        FieldSaveDTO {
            id: None,
            model_id: form_data.model_id,
            field_name: form_data.field_name,
            field_label: form_data.field_label,
            field_type: form_data.field_type,
            field_options: form_data.field_options,
            default_value: form_data.default_value,
            placeholder: form_data.placeholder,
            is_required: form_data.is_required,
            is_searchable: form_data.is_searchable,
            is_list_show: form_data.is_list_show,
            is_detail_show: form_data.is_detail_show,
            sort: form_data.sort,
            status: form_data.status,
            create_time: None,
        }
    }
}

/// 模型字段更新请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FieldUpdateRequest {
    // 字段ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    // 模型ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub model_id: Option<i64>,
    // 字段名称
    pub field_name: Option<String>,
    // 字段标签
    pub field_label: Option<String>,
    // 字段类型
    pub field_type: Option<i32>,
    // 字段选项
    pub field_options: Option<String>,
    // 默认值
    pub default_value: Option<String>,
    // 占位提示
    pub placeholder: Option<String>,
    // 是否必填
    pub is_required: Option<i32>,
    // 是否可搜索
    pub is_searchable: Option<i32>,
    // 列表是否显示
    pub is_list_show: Option<i32>,
    // 详情是否显示
    pub is_detail_show: Option<i32>,
    // 排序
    pub sort: Option<i32>,
    // 状态：0停用，1正常
    pub status: Option<i32>,
}

impl From<FieldUpdateRequest> for FieldSaveDTO {
    fn from(form_data: FieldUpdateRequest) -> Self {
        FieldSaveDTO {
            id: form_data.id,
            model_id: form_data.model_id,
            field_name: form_data.field_name,
            field_label: form_data.field_label,
            field_type: form_data.field_type,
            field_options: form_data.field_options,
            default_value: form_data.default_value,
            placeholder: form_data.placeholder,
            is_required: form_data.is_required,
            is_searchable: form_data.is_searchable,
            is_list_show: form_data.is_list_show,
            is_detail_show: form_data.is_detail_show,
            sort: form_data.sort,
            status: form_data.status,
            create_time: None,
        }
    }
}

/// 模型字段内部传输DTO
pub struct FieldSaveDTO {
    // 字段ID
    pub id: Option<i64>,
    // 模型ID
    pub model_id: Option<i64>,
    // 字段名称
    pub field_name: Option<String>,
    // 字段标签
    pub field_label: Option<String>,
    // 字段类型
    pub field_type: Option<i32>,
    // 字段选项
    pub field_options: Option<String>,
    // 默认值
    pub default_value: Option<String>,
    // 占位提示
    pub placeholder: Option<String>,
    // 是否必填
    pub is_required: Option<i32>,
    // 是否可搜索
    pub is_searchable: Option<i32>,
    // 列表是否显示
    pub is_list_show: Option<i32>,
    // 详情是否显示
    pub is_detail_show: Option<i32>,
    // 排序
    pub sort: Option<i32>,
    // 状态：0停用，1正常
    pub status: Option<i32>,
    // 创建时间
    pub create_time: Option<DateTime>,
}

/// 模型字段列表VO
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FieldListVO {
    // 字段ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    // 模型ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub model_id: Option<i64>,
    // 字段名称
    pub field_name: Option<String>,
    // 字段标签
    pub field_label: Option<String>,
    // 字段类型
    pub field_type: Option<i32>,
    // 字段选项
    pub field_options: Option<String>,
    // 默认值
    pub default_value: Option<String>,
    // 占位提示
    pub placeholder: Option<String>,
    // 是否必填
    pub is_required: Option<i32>,
    // 是否可搜索
    pub is_searchable: Option<i32>,
    // 列表是否显示
    pub is_list_show: Option<i32>,
    // 详情是否显示
    pub is_detail_show: Option<i32>,
    // 排序
    pub sort: Option<i32>,
    // 状态：0停用，1正常
    pub status: Option<i32>,
    // 创建时间
    pub create_time: Option<String>,
}

impl From<content_model_field::Model> for FieldListVO {
    fn from(model: content_model_field::Model) -> Self {
        FieldListVO {
            id: Option::from(model.id),
            model_id: model.model_id,
            field_name: model.field_name,
            field_label: model.field_label,
            field_type: model.field_type,
            field_options: model.field_options,
            default_value: model.default_value,
            placeholder: model.placeholder,
            is_required: model.is_required,
            is_searchable: model.is_searchable,
            is_list_show: model.is_list_show,
            is_detail_show: model.is_detail_show,
            sort: model.sort,
            status: model.status,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// 模型字段详情VO
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FieldDetailVO {
    // 字段ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    // 模型ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub model_id: Option<i64>,
    // 字段名称
    pub field_name: Option<String>,
    // 字段标签
    pub field_label: Option<String>,
    // 字段类型
    pub field_type: Option<i32>,
    // 字段选项
    pub field_options: Option<String>,
    // 默认值
    pub default_value: Option<String>,
    // 占位提示
    pub placeholder: Option<String>,
    // 是否必填
    pub is_required: Option<i32>,
    // 是否可搜索
    pub is_searchable: Option<i32>,
    // 列表是否显示
    pub is_list_show: Option<i32>,
    // 详情是否显示
    pub is_detail_show: Option<i32>,
    // 排序
    pub sort: Option<i32>,
    // 状态：0停用，1正常
    pub status: Option<i32>,
    // 创建时间
    pub create_time: Option<String>,
}

impl From<content_model_field::Model> for FieldDetailVO {
    fn from(model: content_model_field::Model) -> Self {
        FieldDetailVO {
            id: Option::from(model.id),
            model_id: model.model_id,
            field_name: model.field_name,
            field_label: model.field_label,
            field_type: model.field_type,
            field_options: model.field_options,
            default_value: model.default_value,
            placeholder: model.placeholder,
            is_required: model.is_required,
            is_searchable: model.is_searchable,
            is_list_show: model.is_list_show,
            is_detail_show: model.is_detail_show,
            sort: model.sort,
            status: model.status,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// 查询条件
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub model_id: Option<i64>,
    pub field_name: Option<String>,
    pub status: Option<i32>,
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
}

/// 条件
#[derive(Clone)]
pub struct PageWhere {
    pub model_id: Option<i64>,
    pub field_name: Option<String>,
    pub status: Option<i32>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut model_id = None;
        if self.model_id >= Some(0) {
            model_id = self.model_id.clone();
        }

        let mut field_name = None;
        if self.field_name != Some("".to_string()) {
            field_name = self.field_name.clone();
        }

        let mut status = None;
        if self.status == Some(0) || self.status == Some(1) {
            status = self.status;
        }

        Self {
            model_id,
            field_name,
            status,
        }
    }
}

pub struct ContentModelFieldModel;

impl ContentModelFieldModel {
    pub async fn insert(db: &DbConn, dto: &FieldSaveDTO) -> Result<i64, DbErr> {
        let model = content_model_field::ActiveModel {
            model_id: Set(dto.model_id.to_owned()),
            field_name: Set(dto.field_name.to_owned()),
            field_label: Set(dto.field_label.to_owned()),
            field_type: Set(dto.field_type.to_owned()),
            field_options: Set(dto.field_options.to_owned()),
            default_value: Set(dto.default_value.to_owned()),
            placeholder: Set(dto.placeholder.to_owned()),
            is_required: Set(dto.is_required.to_owned()),
            is_searchable: Set(dto.is_searchable.to_owned()),
            is_list_show: Set(dto.is_list_show.to_owned()),
            is_detail_show: Set(dto.is_detail_show.to_owned()),
            sort: Set(dto.sort.to_owned()),
            status: Set(dto.status.to_owned()),
            create_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let res = ContentModelField::insert(model).exec(db).await?;
        Ok(res.last_insert_id)
    }

    /// 按id批量软删除
    /// * `db` 数据库链接
    /// * `ids` id数组
    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        let update_result: UpdateResult = ContentModelField::update_many()
            .set(content_model_field::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(content_model_field::Column::Id.is_in(ids))
            .exec(db).await?;
        Ok(update_result.rows_affected as i64)
    }

    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, dto: &FieldSaveDTO) -> Result<i64, DbErr> {
        let model = content_model_field::ActiveModel {
            model_id: Set(dto.model_id.to_owned()),
            field_name: Set(dto.field_name.to_owned()),
            field_label: Set(dto.field_label.to_owned()),
            field_type: Set(dto.field_type.to_owned()),
            field_options: Set(dto.field_options.to_owned()),
            default_value: Set(dto.default_value.to_owned()),
            placeholder: Set(dto.placeholder.to_owned()),
            is_required: Set(dto.is_required.to_owned()),
            is_searchable: Set(dto.is_searchable.to_owned()),
            is_list_show: Set(dto.is_list_show.to_owned()),
            is_detail_show: Set(dto.is_detail_show.to_owned()),
            sort: Set(dto.sort.to_owned()),
            status: Set(dto.status.to_owned()),
            ..Default::default()
        };
        let result: UpdateResult = ContentModelField::update_many()
            .set(model)
            .filter(content_model_field::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db).await?;
        Ok(result.rows_affected as i64)
    }

    /// 按id查询
    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<content_model_field::Model>, DbErr> {
        let result = ContentModelField::find_by_id(id.clone().unwrap_or_default())
            .filter(content_model_field::Column::Deleted.eq(0))
            .one(db).await?;
        Ok(result)
    }

    /// 按模型ID查询所有字段
    pub async fn find_by_model_id(db: &DbConn, model_id: &Option<i64>) -> Result<Vec<content_model_field::Model>, DbErr> {
        let result = ContentModelField::find()
            .filter(content_model_field::Column::Deleted.eq(0))
            .filter(content_model_field::Column::ModelId.eq(model_id.clone().unwrap_or_default()))
            .order_by_asc(content_model_field::Column::Sort)
            .all(db).await?;
        Ok(result)
    }

    pub async fn select_in_page(db: &DbConn, page_num: i64, page_size: i64, where_case: PageWhere) -> Result<(Vec<content_model_field::Model>, u64), DbErr> {
        let paginator = ContentModelField::find()
            .filter(content_model_field::Column::Deleted.eq(0))
            .apply_if(where_case.model_id.clone(), |query, v| {
                query.filter(content_model_field::Column::ModelId.eq(v))
            })
            .apply_if(where_case.field_name.clone(), |query, v| {
                query.filter(content_model_field::Column::FieldName.contains(v))
            })
            .apply_if(where_case.status.clone(), |query, v| {
                query.filter(content_model_field::Column::Status.eq(v))
            })
            .order_by_asc(content_model_field::Column::Sort)
            .paginate(db, page_size as u64);
        let total = paginator.num_items().await?;
        let list = paginator.fetch_page(page_num as u64).await?;
        Ok((list, total))
    }

    pub async fn select_count(db: &DbConn, where_case: PageWhere) -> Result<i64, DbErr> {
        let count = ContentModelField::find()
            .filter(content_model_field::Column::Deleted.eq(0))
            .apply_if(where_case.model_id.clone(), |query, v| {
                query.filter(content_model_field::Column::ModelId.eq(v))
            })
            .apply_if(where_case.field_name.clone(), |query, v| {
                query.filter(content_model_field::Column::FieldName.contains(v))
            })
            .apply_if(where_case.status.clone(), |query, v| {
                query.filter(content_model_field::Column::Status.eq(v))
            })
            .count(db).await? as i64;
        Ok(count)
    }
}
