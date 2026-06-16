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
    pub category_name: Option<String>,
    /// 分类编码
    pub category_code: Option<String>,
    /// 排序号
    pub sort_order: Option<i32>,
    /// 状态
    pub status: Option<String>,
}

impl From<CategorySaveRequest> for CategorySaveDTO {
    fn from(item: CategorySaveRequest) -> Self {
        CategorySaveDTO {
            id: None,
            parent_id: item.parent_id,
            category_name: item.category_name,
            category_code: item.category_code,
            sort_order: item.sort_order,
            status: item.status,
            deleted: None,
            created_by: None,
            created_at: None,
            updated_by: None,
            updated_at: None,
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
    pub category_name: Option<String>,
    /// 分类编码
    pub category_code: Option<String>,
    /// 排序号
    pub sort_order: Option<i32>,
    /// 状态
    pub status: Option<String>,
}

impl From<CategoryUpdateRequest> for CategorySaveDTO {
    fn from(item: CategoryUpdateRequest) -> Self {
        CategorySaveDTO {
            id: item.id,
            parent_id: item.parent_id,
            category_name: item.category_name,
            category_code: item.category_code,
            sort_order: item.sort_order,
            status: item.status,
            deleted: None,
            created_by: None,
            created_at: None,
            updated_by: None,
            updated_at: None,
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
    pub category_name: Option<String>,
    /// 分类编码
    pub category_code: Option<String>,
    /// 排序号
    pub sort_order: Option<i32>,
    /// 状态
    pub status: Option<String>,
    /// 软删除标记
    pub deleted: Option<i32>,
    /// 创建人ID
    pub created_by: Option<i64>,
    /// 创建时间
    pub created_at: Option<DateTime>,
    /// 更新人ID
    pub updated_by: Option<i64>,
    /// 更新时间
    pub updated_at: Option<DateTime>,
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
    pub category_name: Option<String>,
    /// 分类编码
    pub category_code: Option<String>,
    /// 排序号
    pub sort_order: Option<i32>,
    /// 状态
    pub status: Option<String>,
}

impl From<category::Model> for CategoryDetailVO {
    fn from(item: category::Model) -> Self {
        CategoryDetailVO {
            id: Option::from(item.id),
            parent_id: item.parent_id,
            category_name: item.category_name,
            category_code: item.category_code,
            sort_order: item.sort_order,
            status: item.status,
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
    pub category_name: Option<String>,
    /// 分类编码
    pub category_code: Option<String>,
    /// 排序号
    pub sort_order: Option<i32>,
    /// 状态
    pub status: Option<String>,
}

impl From<category::Model> for CategoryListVO {
    fn from(item: category::Model) -> Self {
        CategoryListVO {
            id: Option::from(item.id),
            parent_id: item.parent_id,
            category_name: item.category_name,
            category_code: item.category_code,
            sort_order: item.sort_order,
            status: item.status,
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
    /// 关键词（搜索分类名称、编码等）
    pub keywords: Option<String>,
    /// 状态
    pub status: Option<String>,
}

/// 产品分类数据模型操作类
pub struct CategoryModel;

impl CategoryModel {
    /// 新增产品分类
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `req` - 产品分类保存DTO
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 新增记录的ID
    pub async fn insert(db: &DbConn, req: &CategorySaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = category::ActiveModel {
            parent_id: Set(req.parent_id.clone()),
            category_name: Set(req.category_name.clone()),
            category_code: Set(req.category_code.clone()),
            sort_order: Set(req.sort_order.clone()),
            status: Set(req.status.clone()),
            created_by: Set(req.created_by.clone()),
            created_at: Set(Option::from(now)),
            updated_by: Set(req.updated_by.clone()),
            updated_at: Set(Option::from(now)),
            ..Default::default()
        };

        Category::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 批量删除产品分类（软删除）
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `ids` - 要删除的分类ID列表
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 删除的记录数
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
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `id` - 分类ID
    /// * `req` - 产品分类保存DTO
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 更新的记录数
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, req: &CategorySaveDTO) -> Result<i64, DbErr> {
        let payload = category::ActiveModel {
            parent_id: Set(req.parent_id.clone()),
            category_name: Set(req.category_name.clone()),
            category_code: Set(req.category_code.clone()),
            sort_order: Set(req.sort_order.clone()),
            status: Set(req.status.clone()),
            updated_by: Set(req.updated_by.clone()),
            updated_at: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
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
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `id` - 分类ID
    ///
    /// # 返回
    /// * `Result<Option<category::Model>, DbErr>` - 产品分类模型（未删除）
    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<category::Model>, DbErr> {
        Category::find_by_id(id)
            .filter(category::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 分页查询产品分类列表
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `page` - 页码
    /// * `per_page` - 每页大小
    /// * `keywords` - 关键词
    /// * `parent_id` - 父级分类ID
    /// * `status` - 状态
    ///
    /// # 返回
    /// * `Result<(Vec<category::Model>, i64), DbErr>` - (分类列表, 总页数)
    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        parent_id: Option<i64>,
        status: Option<String>,
    ) -> Result<(Vec<category::Model>, i64), DbErr> {
        let mut query = Category::find()
            .filter(category::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            query = query.filter(
                Condition::any()
                    .add(category::Column::CategoryName.contains(k.clone()))
                    .add(category::Column::CategoryCode.contains(k)),
            );
        }
        if let Some(p) = parent_id {
            query = query.filter(category::Column::ParentId.eq(p));
        }
        if let Some(s) = status {
            query = query.filter(category::Column::Status.eq(s));
        }

        let paginator = query.order_by_desc(category::Column::CreatedAt).paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}