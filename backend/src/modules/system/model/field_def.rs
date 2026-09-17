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
use crate::modules::system::entity::{field_def, field_def::Entity as FieldDef};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

/// 新增字段定义请求（管理侧 save）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct FieldDefSaveRequest {
    /// 业务模块标识
    pub module: Option<String>,
    /// 字段键
    pub field_key: Option<String>,
    /// 显示名
    pub field_label: Option<String>,
    /// 类型：1文本 2多行文本 3数字 4日期 5日期时间 6单选 7多选 8布尔 9附件 10成员 11金额
    pub field_type: Option<i32>,
    /// 类型配置
    pub options: Option<serde_json::Value>,
    /// 必填：0否 1是
    pub required: Option<i32>,
    /// 可见角色 role_key 数组
    pub visible_roles: Option<serde_json::Value>,
    /// 可编辑角色 role_key 数组
    pub editable_roles: Option<serde_json::Value>,
    /// 列表显示：0否 1是
    pub list_visible: Option<i32>,
    /// 列表内列顺序
    pub list_sort: Option<i32>,
    /// 参与筛选：0否 1是
    pub filterable: Option<i32>,
    /// 表单内排序
    pub sort: Option<i32>,
    /// 备注
    pub remark: Option<String>,
}

impl From<FieldDefSaveRequest> for FieldDefSaveDTO {
    fn from(item: FieldDefSaveRequest) -> Self {
        FieldDefSaveDTO {
            id: None,
            module: item.module,
            field_key: item.field_key,
            field_label: item.field_label,
            field_type: item.field_type,
            options: item.options,
            required: item.required,
            visible_roles: item.visible_roles,
            editable_roles: item.editable_roles,
            list_visible: item.list_visible,
            list_sort: item.list_sort,
            filterable: item.filterable,
            sort: item.sort,
            remark: item.remark,
            create_by: None,
            update_by: None,
        }
    }
}

/// 更新字段定义请求（管理侧 update；module/field_key/field_type 提交后与旧值比对做锁定校验，见 P0-12/P0-17）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct FieldDefUpdateRequest {
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    pub module: Option<String>,
    pub field_key: Option<String>,
    pub field_label: Option<String>,
    pub field_type: Option<i32>,
    pub options: Option<serde_json::Value>,
    pub required: Option<i32>,
    pub visible_roles: Option<serde_json::Value>,
    pub editable_roles: Option<serde_json::Value>,
    pub list_visible: Option<i32>,
    pub list_sort: Option<i32>,
    pub filterable: Option<i32>,
    pub sort: Option<i32>,
    pub remark: Option<String>,
}

impl From<FieldDefUpdateRequest> for FieldDefSaveDTO {
    fn from(item: FieldDefUpdateRequest) -> Self {
        FieldDefSaveDTO {
            id: item.id,
            module: item.module,
            field_key: item.field_key,
            field_label: item.field_label,
            field_type: item.field_type,
            options: item.options,
            required: item.required,
            visible_roles: item.visible_roles,
            editable_roles: item.editable_roles,
            list_visible: item.list_visible,
            list_sort: item.list_sort,
            filterable: item.filterable,
            sort: item.sort,
            remark: item.remark,
            create_by: None,
            update_by: None,
        }
    }
}

/// 停用/启用请求（管理侧 status）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct FieldDefStatusRequest {
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 状态：1启用 0停用
    pub status: Option<i32>,
}

/// 字段定义传输对象（service/model 层）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct FieldDefSaveDTO {
    pub id: Option<i64>,
    pub module: Option<String>,
    pub field_key: Option<String>,
    pub field_label: Option<String>,
    pub field_type: Option<i32>,
    pub options: Option<serde_json::Value>,
    pub required: Option<i32>,
    pub visible_roles: Option<serde_json::Value>,
    pub editable_roles: Option<serde_json::Value>,
    pub list_visible: Option<i32>,
    pub list_sort: Option<i32>,
    pub filterable: Option<i32>,
    pub sort: Option<i32>,
    pub remark: Option<String>,
    pub create_by: Option<String>,
    pub update_by: Option<String>,
}

/// 字段定义管理侧 VO（列表/详情共用）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct FieldDefListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub module: Option<String>,
    /// 系统字段标记：1=模块固定列字段（前端禁删/禁停用/禁改结构，仅可改显示名）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_system: Option<i32>,
    pub field_key: Option<String>,
    pub field_label: Option<String>,
    pub field_type: Option<i32>,
    pub options: Option<serde_json::Value>,
    pub required: Option<i32>,
    pub visible_roles: Option<serde_json::Value>,
    pub editable_roles: Option<serde_json::Value>,
    pub list_visible: Option<i32>,
    pub list_sort: Option<i32>,
    pub filterable: Option<i32>,
    pub indexed: Option<i32>,
    pub storage_type: Option<i32>,
    pub status: Option<i32>,
    pub sort: Option<i32>,
    pub remark: Option<String>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}

impl From<field_def::Model> for FieldDefListVO {
    fn from(item: field_def::Model) -> Self {
        FieldDefListVO {
            id: Option::from(item.id),
            module: item.module,
            is_system: item.is_system,
            field_key: item.field_key,
            field_label: item.field_label,
            field_type: item.field_type,
            options: item.options,
            required: item.required,
            visible_roles: item.visible_roles,
            editable_roles: item.editable_roles,
            list_visible: item.list_visible,
            list_sort: item.list_sort,
            filterable: item.filterable,
            indexed: item.indexed,
            storage_type: item.storage_type,
            status: item.status,
            sort: item.sort,
            remark: item.remark,
            create_time: item.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: item.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// 运行侧 schema 项（仅返回 status=1 且未删除的字段定义）
/// rename_all 双向：缓存走 JSON 字符串回读，serialize-only 会导致缓存命中时反序列化全 None
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SchemaItem {
    pub field_key: Option<String>,
    /// 系统字段标记：1=模块固定列字段（仅可改显示名，布局编排用）；0/None=自定义字段
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_system: Option<i32>,
    pub field_label: Option<String>,
    pub field_type: Option<i32>,
    pub options: Option<serde_json::Value>,
    pub required: Option<i32>,
    pub visible_roles: Option<serde_json::Value>,
    pub editable_roles: Option<serde_json::Value>,
    pub list_visible: Option<i32>,
    pub list_sort: Option<i32>,
    pub filterable: Option<i32>,
}

impl From<field_def::Model> for SchemaItem {
    fn from(item: field_def::Model) -> Self {
        SchemaItem {
            field_key: item.field_key,
            is_system: item.is_system,
            field_label: item.field_label,
            field_type: item.field_type,
            options: item.options,
            required: item.required,
            visible_roles: item.visible_roles,
            editable_roles: item.editable_roles,
            list_visible: item.list_visible,
            list_sort: item.list_sort,
            filterable: item.filterable,
        }
    }
}

/// 运行侧 schema 响应包装
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SchemaListVO {
    pub list: Vec<SchemaItem>,
}

/// 已接入模块下拉项（管理页 module 下拉唯一数据源，前后端同源）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct FieldModuleVO {
    pub module: Option<String>,
    pub label: Option<String>,
}

/// 一键加速请求（管理侧 accelerate，P1-4）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct FieldIndexRequest {
    /// 业务模块标识
    pub module: Option<String>,
}

/// 一键加速失败项（field_key + 中文原因，供管理页逐项展示）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct FieldIndexFailVO {
    pub field_key: Option<String>,
    pub error: Option<String>,
}

/// 一键加速报告（P1-4：created/skipped/failed + 模块级 GIN 状态）
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct FieldIndexReportVO {
    pub module: Option<String>,
    /// 本次新建成功的表达式索引对应 field_key
    pub created: Vec<String>,
    /// 已存在且有效、本次幂等跳过的 field_key
    pub skipped: Vec<String>,
    /// 创建失败的字段与原因（INVALID 清理重试后仍失败等）
    pub failed: Vec<FieldIndexFailVO>,
    /// 模块级 GIN 索引状态：created=本次新建 skipped=已存在有效 failed:xxx=失败原因
    pub gin_status: Option<String>,
}

/// 列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    /// 业务模块标识（必填）
    pub module: Option<String>,
    /// field_label/field_key 模糊
    pub keyword: Option<String>,
    pub status: Option<i32>,
    /// 系统字段过滤：1=仅系统 0=仅自定义
    pub is_system: Option<i32>,
}

/// 运行侧 schema 查询参数（GET /field/schema?module=xxx）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SchemaQuery {
    pub module: Option<String>,
}

/// 列表查询条件
#[derive(Clone)]
pub struct PageWhere {
    pub module: Option<String>,
    pub keyword: Option<String>,
    pub status: Option<i32>,
    /// 系统字段标记过滤：None=不过滤；Some(0)=仅自定义；Some(1)=仅系统
    pub is_system: Option<i32>,
}

impl PageWhere {
    /// 格式化空值
    pub fn format(&self) -> Self {
        let mut keyword = None;
        if self.keyword.is_some() && self.keyword != Some("".to_string()) {
            keyword = self.keyword.clone();
        }

        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        let mut is_system = None;
        if self.is_system == Some(1) || self.is_system == Some(0) {
            is_system = self.is_system;
        }

        Self {
            module: self.module.clone(),
            keyword,
            status,
            is_system,
        }
    }
}

pub struct FieldDefModel;

impl FieldDefModel {
    /// 新增字段定义
    pub async fn insert<C: ConnectionTrait>(db: &C, req: &FieldDefSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = field_def::ActiveModel {
            module: Set(req.module.clone()),
            field_key: Set(req.field_key.clone()),
            field_label: Set(req.field_label.clone()),
            field_type: Set(req.field_type.clone()),
            options: Set(req.options.clone()),
            required: Set(req.required.clone()),
            visible_roles: Set(req.visible_roles.clone()),
            editable_roles: Set(req.editable_roles.clone()),
            list_visible: Set(req.list_visible.clone()),
            list_sort: Set(req.list_sort.clone()),
            filterable: Set(req.filterable.clone()),
            sort: Set(req.sort.clone()),
            remark: Set(req.remark.clone()),
            status: Set(Some(1)),
            deleted: Set(Some(0)),
            create_by: Set(req.create_by.clone()),
            update_by: Set(req.update_by.clone()),
            create_time: Set(Option::from(now.clone())),
            update_time: Set(Option::from(now)),
            ..Default::default()
        };

        FieldDef::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 更新字段定义
    pub async fn update_by_id<C: ConnectionTrait>(db: &C, id: &Option<i64>, req: &FieldDefSaveDTO) -> Result<i64, DbErr> {
        let mut payload = field_def::ActiveModel {
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        if let Some(v) = req.field_label.clone() { payload.field_label = Set(Some(v)); }
        if let Some(v) = req.options.clone() { payload.options = Set(Some(v)); }
        if let Some(v) = req.required { payload.required = Set(Some(v)); }
        if let Some(v) = req.visible_roles.clone() { payload.visible_roles = Set(Some(v)); }
        if let Some(v) = req.editable_roles.clone() { payload.editable_roles = Set(Some(v)); }
        if let Some(v) = req.list_visible { payload.list_visible = Set(Some(v)); }
        if let Some(v) = req.list_sort { payload.list_sort = Set(Some(v)); }
        if let Some(v) = req.filterable { payload.filterable = Set(Some(v)); }
        if let Some(v) = req.sort { payload.sort = Set(Some(v)); }
        if let Some(v) = req.remark.clone() { payload.remark = Set(Some(v)); }
        if let Some(v) = req.update_by.clone() { payload.update_by = Set(Some(v)); }

        let update_result: UpdateResult = FieldDef::update_many()
            .set(payload)
            .filter(field_def::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// 停用/启用字段定义
    pub async fn update_status<C: ConnectionTrait>(db: &C, id: i64, status: i32) -> Result<i64, DbErr> {
        let payload = field_def::ActiveModel {
            status: Set(Some(status)),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let update_result: UpdateResult = FieldDef::update_many()
            .set(payload)
            .filter(field_def::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }

    /// 逻辑删除字段定义（deleted=1，不清理业务表数据）
    pub async fn soft_delete_by_ids<C: ConnectionTrait>(db: &C, ids: &Vec<i64>) -> Result<i64, DbErr> {
        let payload = field_def::ActiveModel {
            deleted: Set(Some(1)),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let update_result: UpdateResult = FieldDef::update_many()
            .set(payload)
            .filter(field_def::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }

    /// 回写表达式索引标记（一键加速成功后 indexed=1，失败项保持原状；6.3 元数据可追溯）
    pub async fn set_indexed_flag<C: ConnectionTrait>(db: &C, ids: &Vec<i64>, indexed: i32) -> Result<i64, DbErr> {
        if ids.is_empty() {
            return Ok(0);
        }
        let payload = field_def::ActiveModel {
            indexed: Set(Some(indexed)),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let update_result: UpdateResult = FieldDef::update_many()
            .set(payload)
            .filter(field_def::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }

    /// 查询单个
    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<field_def::Model>, DbErr> {
        let result = FieldDef::find_by_id(id)
            .filter(field_def::Column::Deleted.eq(0))
            .one(db)
            .await?;
        Ok(result)
    }

    /// 按模块查询（deleted=0；only_active=true 时仅启用中，schema 下发与校验器用）
    /// 泛型签名：校验器在业务方事务内执行时传入 DatabaseTransaction
    pub async fn find_by_module<C: ConnectionTrait>(db: &C, module: &str, only_active: bool) -> Result<Vec<field_def::Model>, DbErr> {
        let mut query = FieldDef::find()
            .filter(field_def::Column::Module.eq(module))
            .filter(field_def::Column::Deleted.eq(0));
        if only_active {
            query = query.filter(field_def::Column::Status.eq(1));
        }
        let result = query
            .order_by_asc(field_def::Column::Sort)
            .order_by_asc(field_def::Column::Id)
            .all(db)
            .await?;
        Ok(result)
    }

    /// 同模块字段键唯一校验（deleted=0 范围内；exclude_id 用于更新排除自身）
    /// 泛型签名：save 接口在事务内查重，防并发重复键
    pub async fn count_by_key<C: ConnectionTrait>(db: &C, module: &str, key: &str, exclude_id: &Option<i64>) -> Result<i64, DbErr> {
        let result = FieldDef::find()
            .filter(field_def::Column::Module.eq(module))
            .filter(field_def::Column::FieldKey.eq(key))
            .filter(field_def::Column::Deleted.eq(0))
            .apply_if(exclude_id.clone(), |query, v| {
                query.filter(field_def::Column::Id.ne(v))
            })
            .count(db)
            .await
            .map(|c| c as i64)?;
        Ok(result)
    }

    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        FieldDef::find()
            .apply_if(wheres.module, |query, v| {
                query.filter(field_def::Column::Module.eq(v))
            })
            .apply_if(wheres.keyword.clone(), |query, v| {
                query.filter(field_def::Column::FieldKey.contains(v.as_str()).or(field_def::Column::FieldLabel.contains(v.as_str())))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(field_def::Column::Status.eq(v))
            })
            .apply_if(wheres.is_system, |query, v| {
                query.filter(field_def::Column::IsSystem.eq(v))
            })
            .filter(field_def::Column::Deleted.eq(0))
            .count(db)
            .await
            .map(|c| c as i64)
    }

    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        wheres: PageWhere,
    ) -> Result<(Vec<field_def::Model>, i64), DbErr> {
        let paginator = FieldDef::find()
            .apply_if(wheres.module, |query, v| {
                query.filter(field_def::Column::Module.eq(v))
            })
            .apply_if(wheres.keyword.clone(), |query, v| {
                query.filter(field_def::Column::FieldKey.contains(v.as_str()).or(field_def::Column::FieldLabel.contains(v.as_str())))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(field_def::Column::Status.eq(v))
            })
            .apply_if(wheres.is_system, |query, v| {
                query.filter(field_def::Column::IsSystem.eq(v))
            })
            .filter(field_def::Column::Deleted.eq(0))
            .order_by_asc(field_def::Column::Module)
            .order_by_asc(field_def::Column::Sort)
            .order_by_asc(field_def::Column::Id)
            .paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}
