use sea_orm::*;
use sea_orm::prelude::{Date, DateTime};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::purchase::entity::{supplier_brand, supplier_brand::Entity as SupplierBrand};
use crate::utils::string_utils::serialize_option_u64_to_string;

/// 供应商品牌DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SupplierBrandDTO {
    pub id: Option<i64>,
    pub supplier_id: Option<i64>,
    pub brand_id: Option<i64>,
    pub is_authorized: Option<i32>,
    pub authorization_no: Option<String>,
    pub authorization_start: Option<Date>,
    pub authorization_end: Option<Date>,
    pub authorization_file: Option<String>,
    pub remark: Option<String>,
}

/// 供应商品牌VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SupplierBrandVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub supplier_id: Option<i64>,
    pub brand_id: Option<i64>,
    pub is_authorized: Option<i32>,
    pub authorization_no: Option<String>,
    pub authorization_start: Option<Date>,
    pub authorization_end: Option<Date>,
    pub authorization_file: Option<String>,
    pub remark: Option<String>,
    pub deleted: Option<i32>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

impl From<supplier_brand::Model> for SupplierBrandVO {
    fn from(item: supplier_brand::Model) -> Self {
        SupplierBrandVO {
            id: Option::from(item.id),
            supplier_id: item.supplier_id,
            brand_id: item.brand_id,
            is_authorized: item.is_authorized,
            authorization_no: item.authorization_no,
            authorization_start: item.authorization_start,
            authorization_end: item.authorization_end,
            authorization_file: item.authorization_file,
            remark: item.remark,
            deleted: item.deleted,
            create_time: item.create_time,
            update_time: item.update_time,
        }
    }
}

/// 供应商品牌数据模型操作类
pub struct SupplierBrandModel;

impl SupplierBrandModel {
    /// 新增供应商品牌关联
    pub async fn insert<C: ConnectionTrait>(db: &C, req: &SupplierBrandDTO) -> Result<i64, DbErr> {
        let now = chrono::Utc::now().naive_utc();
        let payload = supplier_brand::ActiveModel {
            supplier_id: Set(req.supplier_id),
            brand_id: Set(req.brand_id),
            is_authorized: Set(req.is_authorized),
            authorization_no: Set(req.authorization_no.clone()),
            authorization_start: Set(req.authorization_start),
            authorization_end: Set(req.authorization_end),
            authorization_file: Set(req.authorization_file.clone()),
            remark: Set(req.remark.clone()),
            deleted: Set(Some(0)),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            ..Default::default()
        };

        SupplierBrand::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 更新供应商品牌关联
    pub async fn update<C: ConnectionTrait>(db: &C, id: i64, req: &SupplierBrandDTO) -> Result<i64, DbErr> {
        let now = chrono::Utc::now().naive_utc();
        let payload = supplier_brand::ActiveModel {
            supplier_id: Set(req.supplier_id),
            brand_id: Set(req.brand_id),
            is_authorized: Set(req.is_authorized),
            authorization_no: Set(req.authorization_no.clone()),
            authorization_start: Set(req.authorization_start),
            authorization_end: Set(req.authorization_end),
            authorization_file: Set(req.authorization_file.clone()),
            remark: Set(req.remark.clone()),
            update_time: Set(Some(now)),
            ..Default::default()
        };

        let update_result = SupplierBrand::update_many()
            .set(payload)
            .filter(supplier_brand::Column::Id.eq(id))
            .filter(supplier_brand::Column::Deleted.eq(0))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// 根据供应商ID查询品牌列表
    pub async fn find_by_supplier_id<C: ConnectionTrait>(db: &C, supplier_id: i64) -> Result<Vec<supplier_brand::Model>, DbErr> {
        SupplierBrand::find()
            .filter(supplier_brand::Column::SupplierId.eq(supplier_id))
            .filter(supplier_brand::Column::Deleted.eq(0))
            .all(db)
            .await
    }

    /// 根据品牌ID查询供应商列表
    pub async fn find_by_brand_id<C: ConnectionTrait>(db: &C, brand_id: i64) -> Result<Vec<supplier_brand::Model>, DbErr> {
        SupplierBrand::find()
            .filter(supplier_brand::Column::BrandId.eq(brand_id))
            .filter(supplier_brand::Column::Deleted.eq(0))
            .all(db)
            .await
    }

    /// 批量删除供应商品牌关联（软删除）
    pub async fn batch_delete<C: ConnectionTrait>(db: &C, ids: &Vec<i64>) -> Result<i64, DbErr> {
        SupplierBrand::update_many()
            .set(supplier_brand::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(supplier_brand::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
}