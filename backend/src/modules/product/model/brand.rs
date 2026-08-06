use sea_orm::*;
use sea_orm::prelude::DateTime;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::product::entity::{brand, brand::Entity as Brand};
use crate::utils::string_utils::serialize_option_u64_to_string;

/// 品牌保存请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BrandSaveRequest {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub name_en: Option<String>,
    pub logo: Option<String>,
    pub description: Option<String>,
    pub country: Option<String>,
    pub website: Option<String>,
    pub status: Option<i32>,
    pub sort_order: Option<i32>,
    pub remark: Option<String>,
}

/// 品牌列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BrandListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub keyword: Option<String>,
    pub status: Option<i32>,
}

/// 品牌详情VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct BrandDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub name: Option<String>,
    pub name_en: Option<String>,
    pub logo: Option<String>,
    pub description: Option<String>,
    pub country: Option<String>,
    pub website: Option<String>,
    pub status: Option<i32>,
    pub sort_order: Option<i32>,
    pub remark: Option<String>,
    pub deleted: Option<i32>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

impl From<brand::Model> for BrandDetailVO {
    fn from(item: brand::Model) -> Self {
        BrandDetailVO {
            id: Option::from(item.id),
            name: item.name,
            name_en: item.name_en,
            logo: item.logo,
            description: item.description,
            country: item.country,
            website: item.website,
            status: item.status,
            sort_order: item.sort_order,
            remark: item.remark,
            deleted: item.deleted,
            created_by: item.created_by,
            updated_by: item.updated_by,
            create_time: item.create_time,
            update_time: item.update_time,
        }
    }
}

/// 品牌列表VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct BrandListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub name: Option<String>,
    pub name_en: Option<String>,
    pub logo: Option<String>,
    pub description: Option<String>,
    pub country: Option<String>,
    pub website: Option<String>,
    pub status: Option<i32>,
    pub sort_order: Option<i32>,
    pub remark: Option<String>,
    pub deleted: Option<i32>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

impl From<brand::Model> for BrandListVO {
    fn from(item: brand::Model) -> Self {
        BrandListVO {
            id: Option::from(item.id),
            name: item.name,
            name_en: item.name_en,
            logo: item.logo,
            description: item.description,
            country: item.country,
            website: item.website,
            status: item.status,
            sort_order: item.sort_order,
            remark: item.remark,
            deleted: item.deleted,
            created_by: item.created_by,
            updated_by: item.updated_by,
            create_time: item.create_time,
            update_time: item.update_time,
        }
    }
}

/// 品牌数据模型操作类
pub struct BrandModel;

impl BrandModel {
    /// 新增品牌
    pub async fn insert<C: ConnectionTrait>(db: &C, req: &BrandSaveRequest, created_by: i64) -> Result<i64, DbErr> {
        let now = chrono::Utc::now().naive_utc();
        let payload = brand::ActiveModel {
            name: Set(req.name.clone()),
            name_en: Set(req.name_en.clone()),
            logo: Set(req.logo.clone()),
            description: Set(req.description.clone()),
            country: Set(req.country.clone()),
            website: Set(req.website.clone()),
            status: Set(req.status),
            sort_order: Set(req.sort_order),
            remark: Set(req.remark.clone()),
            deleted: Set(Some(0)),
            created_by: Set(Some(created_by)),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            ..Default::default()
        };

        Brand::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 更新品牌
    pub async fn update<C: ConnectionTrait>(db: &C, id: i64, req: &BrandSaveRequest, updated_by: i64) -> Result<i64, DbErr> {
        let now = chrono::Utc::now().naive_utc();
        let payload = brand::ActiveModel {
            name: Set(req.name.clone()),
            name_en: Set(req.name_en.clone()),
            logo: Set(req.logo.clone()),
            description: Set(req.description.clone()),
            country: Set(req.country.clone()),
            website: Set(req.website.clone()),
            status: Set(req.status),
            sort_order: Set(req.sort_order),
            remark: Set(req.remark.clone()),
            updated_by: Set(Some(updated_by)),
            update_time: Set(Some(now)),
            ..Default::default()
        };

        let update_result = Brand::update_many()
            .set(payload)
            .filter(brand::Column::Id.eq(id))
            .filter(brand::Column::Deleted.eq(0))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// 根据ID查询品牌
    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<brand::Model>, DbErr> {
        Brand::find_by_id(id)
            .filter(brand::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 批量删除品牌（软删除）
    pub async fn batch_delete<C: ConnectionTrait>(db: &C, ids: &Vec<i64>) -> Result<i64, DbErr> {
        Brand::update_many()
            .set(brand::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(brand::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    /// 分页查询品牌列表
    pub async fn find_list<C: ConnectionTrait>(
        db: &C,
        page: i64,
        per_page: i64,
        keyword: Option<String>,
        status: Option<i32>,
    ) -> Result<(Vec<brand::Model>, i64), DbErr> {
        let mut query = Brand::find()
            .filter(brand::Column::Deleted.eq(0));

        if let Some(k) = keyword {
            if !k.is_empty() {
                query = query.filter(
                    Condition::any()
                        .add(brand::Column::Name.contains(k.clone()))
                        .add(brand::Column::NameEn.contains(k.clone())),
                );
            }
        }
        if let Some(s) = status {
            query = query.filter(brand::Column::Status.eq(s));
        }

        let paginator = query.order_by_desc(brand::Column::CreateTime).paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, total))
    }

    /// 查询所有未删除的品牌（用于下拉选择）
    pub async fn find_all<C: ConnectionTrait>(db: &C) -> Result<Vec<brand::Model>, DbErr> {
        Brand::find()
            .filter(brand::Column::Deleted.eq(0))
            .order_by_asc(brand::Column::SortOrder)
            .all(db)
            .await
    }
}