//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};

use crate::modules::product::entity::product_unit;
use crate::utils::string_utils::serialize_option_u64_to_string;

/// 产品单位保存请求（选择器内联新增用）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductUnitSaveRequest {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub decimal_places: Option<i16>,
    pub status: Option<i32>,
    pub remark: Option<String>,
}

/// 产品单位精度更新请求（管理面板内联修改；不支持改名——mxx_product.unit 按名称引用，改名会使存量产品失去关联）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductUnitUpdateRequest {
    pub id: Option<i64>,
    pub decimal_places: Option<i16>,
}

/// 产品单位删除请求（软删除）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductUnitDeleteRequest {
    pub id: Option<i64>,
}

/// 产品单位选项 VO（选择器下拉用）
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductUnitOptionVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub name: Option<String>,
    pub decimal_places: Option<i16>,
    pub is_default: Option<bool>,
}

impl From<product_unit::Model> for ProductUnitOptionVO {
    fn from(item: product_unit::Model) -> Self {
        ProductUnitOptionVO {
            id: Option::from(item.id),
            name: item.name,
            decimal_places: item.decimal_places,
            is_default: item.is_default,
        }
    }
}

/// 查询启用单位选项（默认单位排前，其余按 sort_order）
pub async fn find_options<C: ConnectionTrait>(db: &C) -> Result<Vec<product_unit::Model>, DbErr> {
    product_unit::Entity::find()
        .filter(product_unit::Column::Deleted.eq(0))
        .filter(product_unit::Column::Status.eq(1))
        .order_by_desc(product_unit::Column::IsDefault)
        .order_by_asc(product_unit::Column::SortOrder)
        .order_by_asc(product_unit::Column::Id)
        .all(db)
        .await
}

/// 按名称查询单位（未删除）
pub async fn find_by_name<C: ConnectionTrait>(db: &C, name: &str) -> Result<Option<product_unit::Model>, DbErr> {
    product_unit::Entity::find()
        .filter(product_unit::Column::Name.eq(name))
        .filter(product_unit::Column::Deleted.eq(0))
        .one(db)
        .await
}

/// 新增单位（默认整数精度、启用、排序靠后）
pub async fn insert<C: ConnectionTrait>(
    db: &C,
    name: &str,
    decimal_places: Option<i16>,
    created_by: i64,
) -> Result<i64, DbErr> {
    let now = chrono::Utc::now().naive_utc();
    let payload = product_unit::ActiveModel {
        name: Set(Some(name.to_string())),
        decimal_places: Set(decimal_places.or(Some(0))),
        is_default: Set(Some(false)),
        status: Set(Some(1)),
        sort_order: Set(Some(100)),
        remark: Set(None),
        deleted: Set(Some(0)),
        created_by: Set(Some(created_by)),
        updated_by: Set(Some(created_by)),
        create_time: Set(Some(now)),
        update_time: Set(Some(now)),
        ..Default::default()
    };
    let result = payload.insert(db).await?;
    Ok(result.id)
}

/// 按ID查询单位（未删除）
pub async fn find_by_id<C: ConnectionTrait>(
    db: &C,
    id: i64,
) -> Result<Option<product_unit::Model>, DbErr> {
    product_unit::Entity::find()
        .filter(product_unit::Column::Id.eq(id))
        .filter(product_unit::Column::Deleted.eq(0))
        .one(db)
        .await
}

/// 修改单位小数位精度（仅更新精度字段，不动名称与状态）
pub async fn update_decimal_places<C: ConnectionTrait>(
    db: &C,
    id: i64,
    decimal_places: i16,
    updated_by: i64,
) -> Result<(), DbErr> {
    let now = chrono::Utc::now().naive_utc();
    let payload = product_unit::ActiveModel {
        id: Set(id),
        decimal_places: Set(Some(decimal_places)),
        updated_by: Set(Some(updated_by)),
        update_time: Set(Some(now)),
        ..Default::default()
    };
    payload.update(db).await?;
    Ok(())
}

/// 软删除单位（deleted=1；唯一索引仅约束未删除行，之后可重建同名单位）
pub async fn soft_delete<C: ConnectionTrait>(db: &C, id: i64, updated_by: i64) -> Result<(), DbErr> {
    let now = chrono::Utc::now().naive_utc();
    let payload = product_unit::ActiveModel {
        id: Set(id),
        deleted: Set(Some(1)),
        updated_by: Set(Some(updated_by)),
        update_time: Set(Some(now)),
        ..Default::default()
    };
    payload.update(db).await?;
    Ok(())
}
