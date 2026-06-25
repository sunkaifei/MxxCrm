use sea_orm::*;
use sea_orm::prelude::DateTime;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::product::entity::{category, category::Entity as Category};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

/// 产品分类新增请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CategorySaveRequest {
    /// 父级分类ID
    pub parent_id: Option<i64>,
    /// 分类名称
    pub name: Option<String>,
    /// 分类图片
    pub image: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 排序号
    pub sort_order: Option<i32>,
}

impl From<CategorySaveRequest> for CategorySaveDTO {
    fn from(item: CategorySaveRequest) -> Self {
        CategorySaveDTO {
            id: None,
            parent_id: item.parent_id,
            name: item.name,
            image: item.image,
            description: item.description,
            sort_order: item.sort_order,
            deleted: None,
            create_time: None,
            update_time: None,
        }
    }
}

/// 产品分类更新请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CategoryUpdateRequest {
    /// 分类ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 父级分类ID
    pub parent_id: Option<i64>,
    /// 分类名称
    pub name: Option<String>,
    /// 分类图片
    pub image: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 排序号
    pub sort_order: Option<i32>,
}

impl From<CategoryUpdateRequest> for CategorySaveDTO {
    fn from(item: CategoryUpdateRequest) -> Self {
        CategorySaveDTO {
            id: item.id,
            parent_id: item.parent_id,
            name: item.name,
            image: item.image,
            description: item.description,
            sort_order: item.sort_order,
            deleted: None,
            create_time: None,
            update_time: None,
        }
    }
}

/// 产品分类保存DTO（包含新增和更新的所有字段）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct CategorySaveDTO {
    /// 分类ID
    pub id: Option<i64>,
    /// 父级分类ID
    pub parent_id: Option<i64>,
    /// 分类名称
    pub name: Option<String>,
    /// 分类图片
    pub image: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 排序号
    pub sort_order: Option<i32>,
    /// 软删除标记
    pub deleted: Option<i32>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}

/// 产品分类详情VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CategoryDetailVO {
    /// 分类ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 父级分类ID
    pub parent_id: Option<i64>,
    /// 分类名称
    pub name: Option<String>,
    /// 分类图片
    pub image: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 排序号
    pub sort_order: Option<i32>,
}

impl From<category::Model> for CategoryDetailVO {
    fn from(item: category::Model) -> Self {
        CategoryDetailVO {
            id: Option::from(item.id),
            parent_id: item.parent_id,
            name: item.name,
            image: item.image,
            description: item.description,
            sort_order: item.sort_order,
        }
    }
}

/// 产品分类列表VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CategoryListVO {
    /// 分类ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 父级分类ID
    pub parent_id: Option<i64>,
    /// 分类名称
    pub name: Option<String>,
    /// 分类图片
    pub image: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 排序号
    pub sort_order: Option<i32>,
}

impl From<category::Model> for CategoryListVO {
    fn from(item: category::Model) -> Self {
        CategoryListVO {
            id: Option::from(item.id),
            parent_id: item.parent_id,
            name: item.name,
            image: item.image,
            description: item.description,
            sort_order: item.sort_order,
        }
    }
}

/// 产品分类列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CategoryListQuery {
    /// 页码
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    /// 每页大小
    pub page_size: Option<i64>,
    /// 父级分类ID
    pub parent_id: Option<i64>,
    /// 关键词（搜索分类名称）
    pub keywords: Option<String>,
}

/// 产品分类数据模型操作类
pub struct CategoryModel;

impl CategoryModel {
    /// 新增产品分类
    pub async fn insert(db: &DbConn, req: &CategorySaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Utc::now().naive_utc();
        let payload = category::ActiveModel {
            parent_id: Set(req.parent_id),
            name: Set(req.name.clone()),
            image: Set(req.image.clone()),
            description: Set(req.description.clone()),
            sort_order: Set(req.sort_order),
            create_time: Set(Option::from(now)),
            update_time: Set(Option::from(now)),
            ..Default::default()
        };

        Category::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 批量删除产品分类（软删除）
    pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        Category::update_many()
            .set(category::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(category::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    /// 更新产品分类信息
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, req: &CategorySaveDTO) -> Result<i64, DbErr> {
        let payload = category::ActiveModel {
            parent_id: Set(req.parent_id),
            name: Set(req.name.clone()),
            image: Set(req.image.clone()),
            description: Set(req.description.clone()),
            sort_order: Set(req.sort_order),
            update_time: Set(Option::from(chrono::Utc::now().naive_utc())),
            ..Default::default()
        };

        let update_result: UpdateResult = Category::update_many()
            .set(payload)
            .filter(category::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// 根据ID查询产品分类详情
    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<category::Model>, DbErr> {
        Category::find_by_id(id)
            .filter(category::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 分页查询产品分类列表
    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        parent_id: Option<i64>,
    ) -> Result<(Vec<category::Model>, i64), DbErr> {
        let mut query = Category::find()
            .filter(category::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            query = query.filter(category::Column::Name.contains(k));
        }
        if let Some(p) = parent_id {
            query = query.filter(category::Column::ParentId.eq(p));
        }

        let paginator = query.order_by_desc(category::Column::CreateTime).paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;
        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}
