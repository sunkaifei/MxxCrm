//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_multipart::form::*;
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::text::Text;
use chrono::Local;
use sea_orm::*;
use sea_orm::prelude::DateTime;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::upload::entity::{attachment, attachment::Entity as Attach};
use crate::utils::string_utils::{serialize_option_u64_to_string, deserialize_string_to_u64};


#[derive(Debug, MultipartForm)]
pub struct ImageFormRequest {
    #[multipart(rename = "file")]
    pub file: TempFile,
    pub type_id: Option<Text<String>>,
    /// 业务类型（product/contract/invoice/quotation/avatar/payment/common）
    pub entity_type: Option<Text<String>>,
    /// 关联业务ID（可选，上传时可不绑定）
    pub entity_id: Option<Text<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UploadResult {
    ///图片地址
    pub src: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UploadImageResult {
    pub file_name: String,
    pub url: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BatchMoveRequest {
    ///需要移动的id
    pub ids: Option<Vec<Option<String>>>,
    /// 移动到所属的目录id
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub type_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttachmentUpdateRequest {
    pub id: Option<i64>,
    pub type_id: Option<i64>,
    pub name: Option<String>,
    pub path: Option<String>,
    pub upload_url: Option<String>,
    pub ext: Option<String>,
    pub size: Option<i64>,
    pub md5: Option<String>,
    pub r#type: Option<i32>,
    pub status: Option<i32>,
}

impl From<AttachmentUpdateRequest> for AttachmentSaveDTO {
    fn from(req: AttachmentUpdateRequest) -> Self {
        Self {
            id: 0,
            type_id: req.type_id,
            name: req.name,
            path: req.path,
            upload_url: req.upload_url,
            ext: req.ext,
            size: req.size,
            md5: req.md5,
            storage_type: None,
            r#type: req.r#type,
            status: req.status,
            create_time: Some(Local::now().naive_local()),
            original_name: None,
            mime_type: None,
            file_hash: None,
            entity_type: None,
            entity_id: None,
            uploaded_by: None,
            is_public: None,
            deleted: None,
        }
    }

}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UploadBase64Data {
    pub base64: Option<String>,
}

/// 上传文件响应（JSON 返回）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UploadFileResult {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 原始文件名
    pub original_name: Option<String>,
    /// 访问URL
    pub url: Option<String>,
    /// 扩展名
    pub ext: Option<String>,
    /// 文件大小
    pub size: Option<i64>,
    /// MIME类型
    pub mime_type: Option<String>,
    /// SHA-256
    pub file_hash: Option<String>,
    /// 是否公开
    pub is_public: Option<bool>,
    /// 业务类型
    pub entity_type: Option<String>,
}

/// 附件绑定请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentBindRequest {
    /// 附件ID列表
    pub ids: Vec<i64>,
    /// 业务类型
    pub entity_type: String,
    /// 关联业务ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub entity_id: Option<i64>,
}

/// 附件解绑请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentUnbindRequest {
    /// 附件ID列表
    pub ids: Vec<i64>,
}

/// 按业务实体查询附件
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentByEntityQuery {
    /// 业务类型
    pub entity_type: String,
    /// 关联业务ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub entity_id: Option<i64>,
}




#[derive(Debug)]
pub enum StorageType {
    Local = 1,
    Qiniu = 2,
    Aliyun = 3,
    Tencent = 4,
}

impl StorageType {
    pub fn from_i16(value: i32) -> Option<Self> {
        match value {
            1 => Some(StorageType::Local),
            2 => Some(StorageType::Qiniu),
            3 => Some(StorageType::Aliyun),
            4 => Some(StorageType::Tencent),
            _ => None,
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentPageRequest {
    pub name: Option<String>,
    /// 分类id
    pub type_id: Option<i64>,
    // 状态查询（0和空都是所有，1查询为0的数据，2查询为1的数据）
    pub r#type: Option<i32>,
    // 当前页码数（通过 camelCase 自动转换为 pageNum）
    pub page_num: Option<i64>,
    // 每页条数（通过 camelCase 自动转换为 pageSize）
    pub page_size: Option<i64>,
    pub status: Option<i32>,
    /// 业务类型筛选（product/contract/invoice/quotation/avatar/payment/common）
    pub entity_type: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentSaveDTO {
    pub id: i64,
    pub type_id: Option<i64>,
    pub name: Option<String>,
    pub path: Option<String>,
    pub upload_url: Option<String>,
    pub ext: Option<String>,
    pub size: Option<i64>,
    pub md5: Option<String>,
    /// 图片上传类型 1本地 2七牛云 3阿里云 4腾讯云
    pub storage_type: Option<i32>,
    pub r#type: Option<i32>,
    pub status: Option<i32>,
    pub create_time: Option<DateTime>,
    /// 原始文件名
    pub original_name: Option<String>,
    /// MIME类型
    pub mime_type: Option<String>,
    /// SHA-256哈希
    pub file_hash: Option<String>,
    /// 业务类型（product/contract/invoice/quotation/avatar/payment/common）
    pub entity_type: Option<String>,
    /// 关联业务ID
    pub entity_id: Option<i64>,
    /// 上传者用户ID
    pub uploaded_by: Option<i64>,
    /// 是否公开访问
    pub is_public: Option<bool>,
    /// 软删除：0未删除 1已删除
    pub deleted: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub type_id: Option<i64>,
    pub name: Option<String>,
    pub path: Option<String>,
    pub upload_url: Option<String>,
    pub ext: Option<String>,
    pub size: Option<i64>,
    pub md5: Option<String>,
    /// 图片上传类型 1本地 2七牛云 3阿里云 4腾讯云
    pub storage_type: Option<i32>,
    pub r#type: Option<i32>,
    pub status: Option<i32>,
    pub create_time: Option<String>,
    /// 原始文件名
    pub original_name: Option<String>,
    /// MIME类型
    pub mime_type: Option<String>,
    /// 业务类型（product/contract/invoice/quotation/avatar/payment/common）
    pub entity_type: Option<String>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub entity_id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub uploaded_by: Option<i64>,
    /// 上传人名称（关联 mxx_system_admin 查询得到，列表接口填充）
    pub uploaded_name: Option<String>,
    /// 是否公开访问
    pub is_public: Option<bool>,
    /// 软删除：0未删除 1已删除
    pub deleted: Option<i32>,
}

impl From<attachment::Model> for AttachmentListVO {
    fn from(dto: attachment::Model) -> Self {
        Self {
            id: Option::from(dto.id),
            type_id: dto.type_id,
            name: dto.name,
            path: dto.path,
            upload_url: dto.upload_url,
            ext: dto.ext,
            size: dto.size,
            md5: dto.md5,
            storage_type: dto.storage_type,
            r#type: dto.r#type,
            status: dto.status,
            create_time: dto.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            original_name: dto.original_name,
            mime_type: dto.mime_type,
            entity_type: dto.entity_type,
            entity_id: dto.entity_id,
            uploaded_by: dto.uploaded_by,
            uploaded_name: None,
            is_public: dto.is_public,
            deleted: dto.deleted,
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentPageVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub type_id: Option<i64>,
    pub name: Option<String>,
    pub upload_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub type_id: Option<i64>,
    pub name: Option<String>,
    pub path: Option<String>,
    pub upload_url: Option<String>,
    pub ext: Option<String>,
    pub size: Option<i64>,
    pub md5: Option<String>,
    pub storage_type: Option<i32>,
    pub r#type: Option<i32>,
    pub status: Option<i32>,
}

impl From<attachment::Model> for AttachmentDetailVO {
    fn from(model: attachment::Model) -> Self {
        Self {
            id: Some(model.id),
            type_id: model.type_id,
            name: model.name,
            path: model.path,
            upload_url: model.upload_url,
            ext: model.ext,
            size: model.size,
            md5: model.md5,
            storage_type: model.storage_type,
            r#type: model.r#type,
            status: model.status,
        }
    }
}


/// 条件
#[derive(Clone)]
pub struct PageWhere {
    pub name: Option<String>,
    /// 分类id
    pub type_id: Option<i64>,
    pub r#type: Option<i32>,
    pub status: Option<i32>,
    /// 业务类型筛选（product/contract/invoice/quotation/avatar/payment/common）
    pub entity_type: Option<String>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut name = None;
        if self.name != Some("".to_string()) {
            name = self.name.clone();
        }


        let mut type_id = None;
        if self.type_id != Some(0) {
            type_id = self.type_id;
        }

        let mut r#type = None;
        if self.r#type == Some(1) || self.r#type == Some(0) {
              r#type = self.r#type;
        }

        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        let mut entity_type = None;
        if let Some(et) = self.entity_type.as_ref() {
            if !et.is_empty() {
                entity_type = self.entity_type.clone();
            }
        }

        Self {
            name,
            type_id,
            r#type,
            status,
            entity_type,
        }
    }
}

pub struct AttachmentModel;

impl AttachmentModel {
    pub async fn insert(db: &DbConn, form_data: &AttachmentSaveDTO) -> Result<i64, DbErr> {
        let payload = attachment::ActiveModel {
            type_id:       Set(form_data.type_id.to_owned()),
            name:          Set(form_data.name.to_owned()),
            path:          Set(form_data.path.to_owned()),
            upload_url:    Set(form_data.upload_url.to_owned()),
            ext:           Set(form_data.ext.to_owned()),
            size:          Set(form_data.size.to_owned()),
            md5:           Set(form_data.md5.to_owned()),
            r#type:        Set(form_data.r#type.to_owned()),
            status:        Set(form_data.status.to_owned()),
            create_time:   Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            original_name: Set(form_data.original_name.to_owned()),
            mime_type:     Set(form_data.mime_type.to_owned()),
            file_hash:     Set(form_data.file_hash.to_owned()),
            entity_type:   Set(form_data.entity_type.to_owned()),
            entity_id:     Set(form_data.entity_id.to_owned()),
            uploaded_by:   Set(form_data.uploaded_by.to_owned()),
            is_public:     Set(form_data.is_public.to_owned()),
            deleted:       Set(form_data.deleted.to_owned().or(Some(0))),
            ..Default::default()
        };

        Attach::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id as i64)
    }
    
    
    pub async fn update(db: &DbConn, form_data: &AttachmentSaveDTO) -> Result<i64, DbErr> {
         let update_result = Attach::update_many().set(
            attachment::ActiveModel {
                type_id:       Set(form_data.type_id.to_owned()),
                name:          Set(form_data.name.to_owned()),
                path:          Set(form_data.path.to_owned()),
                upload_url:    Set(form_data.upload_url.to_owned()),
                ext:           Set(form_data.ext.to_owned()),
                size:          Set(form_data.size.to_owned()),
                md5:           Set(form_data.md5.to_owned()),
                r#type:        Set(form_data.r#type.to_owned()),
                status:        Set(form_data.status.to_owned()),
                original_name: Set(form_data.original_name.to_owned()),
                mime_type:     Set(form_data.mime_type.to_owned()),
                file_hash:     Set(form_data.file_hash.to_owned()),
                entity_type:   Set(form_data.entity_type.to_owned()),
                entity_id:     Set(form_data.entity_id.to_owned()),
                uploaded_by:   Set(form_data.uploaded_by.to_owned()),
                is_public:     Set(form_data.is_public.to_owned()),
                deleted:       Set(form_data.deleted.to_owned()),
                ..Default::default()
            }
        ).filter(attachment::Column::Id.eq(form_data.id))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// # 批量修改分类
    /// * `db` 数据库连接
    /// * `type_id` 分类id
    /// * `ids` 附件id
    pub async fn batch_update(db: &DbConn, type_id: Option<i64>, ids: Vec<i64>) -> Result<i64, DbErr> {
        let update_result = Attach::update_many().set(
            attachment::ActiveModel {
                type_id:      Set(type_id.to_owned()),
                ..Default::default()
            }
        )
        .filter(attachment::Column::Id.is_in(ids))
        .exec(db)
        .await?;

        Ok(update_result.rows_affected as i64)
    }
    
    
    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        let delete_result = Attach::delete_many()
            .filter(attachment::Column::Id.is_in(ids))
            .exec(db)
            .await?;
        
        Ok(delete_result.rows_affected as i64)
    }
    
    
    pub async fn delete_by_id(db: &DbConn, id: &Option<i64>) -> Result<i64, DbErr> {
        let delete_result = Attach::delete_by_id(id.clone().unwrap_or_default()).exec(db).await?;
        Ok(delete_result.rows_affected as i64)
    }
    
    /// 根据id查询
    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<attachment::Model>, DbErr> {
        let result_data = Attach::find_by_id(id.clone().unwrap_or_default()).one(db).await?;
        Ok(result_data)
    }

    /// 根据id查询未软删除的记录
    pub async fn find_active_by_id(db: &DbConn, id: i64) -> Result<Option<attachment::Model>, DbErr> {
        let find_result = Attach::find()
            .filter(attachment::Column::Id.eq(id))
            .filter(attachment::Column::Deleted.eq(0))
            .one(db)
            .await?;
        Ok(find_result)
    }

    /// 根据md5查询（admin端使用）
    pub async fn select_by_md5(db: &DbConn, md5: String) -> Result<Option<attachment::Model>, DbErr> {
        let find_result = Attach::find()
            .filter(attachment::Column::Md5.eq(md5))
            .one(db)
            .await?;
        Ok(find_result)
    }

    /// 根据文件SHA-256哈希查询（基于 file_hash 上传去重，仅未软删除）
    pub async fn select_by_hash(db: &DbConn, file_hash: String) -> Result<Option<attachment::Model>, DbErr> {
        let find_result = Attach::find()
            .filter(attachment::Column::FileHash.eq(file_hash))
            .filter(attachment::Column::Deleted.eq(0))
            .one(db)
            .await?;
        Ok(find_result)
    }
    
    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        Attach::find()
            .filter(attachment::Column::Deleted.eq(0))
            .apply_if(wheres.name, |query, v| {
                query.filter(attachment::Column::Name.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.type_id, |query, v| {
                query.filter(attachment::Column::TypeId.eq(v))
            })
            .apply_if(wheres.r#type, |query, v| {
                match v {
                    1 => query.filter(attachment::Column::Type.eq(0)),
                    2 => query.filter(attachment::Column::Type.eq(1)),
                    3 => query.filter(attachment::Column::Type.eq(2)),
                    _ => query, // 默认情况下不应用任何过滤条件
                }
            })
            .apply_if(wheres.status, |query, v| {
                match v {
                    1 => query.filter(attachment::Column::Status.eq(0)),
                    2 => query.filter(attachment::Column::Status.eq(1)),
                    3 => query.filter(attachment::Column::Status.eq(2)),
                    _ => query, // 默认情况下不应用任何过滤条件
                }
            })
            .apply_if(wheres.entity_type, |query, v| {
                query.filter(attachment::Column::EntityType.eq(v))
            })
            .count(db)
            .await
            .map(|c| c as i64)
    }


    pub async fn list_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        wheres: PageWhere,
    ) -> Result<(Vec<attachment::Model>, i64), DbErr> {
        let paginator = Attach::find()
            .filter(attachment::Column::Deleted.eq(0))
            .apply_if(wheres.name, |query, v| {
                query.filter(attachment::Column::Name.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.type_id, |query, v| {
                query.filter(attachment::Column::TypeId.eq(v))
            })
            .apply_if(wheres.r#type, |query, v| {
                match v {
                    1 => query.filter(attachment::Column::Type.eq(0)),
                    2 => query.filter(attachment::Column::Type.eq(1)),
                    3 => query.filter(attachment::Column::Type.eq(2)),
                    _ => query, // 默认情况下不应用任何过滤条件
                }
            })
            .apply_if(wheres.status, |query, v| {
                match v {
                    1 => query.filter(attachment::Column::Status.eq(0)),
                    2 => query.filter(attachment::Column::Status.eq(1)),
                    3 => query.filter(attachment::Column::Status.eq(2)),
                    _ => query, // 默认情况下不应用任何过滤条件
                }
            })
            .apply_if(wheres.entity_type, |query, v| {
                query.filter(attachment::Column::EntityType.eq(v))
            })
            .order_by_desc(attachment::Column::Id)
            .paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;
        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }

    /// 软删除：将 deleted 字段置为 1
    pub async fn soft_delete_by_id(db: &DbConn, id: i64) -> Result<i64, DbErr> {
        let update_result = Attach::update_many()
            .set(attachment::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(attachment::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }

    /// 绑定附件到业务实体：更新 entity_type/entity_id，count_quote += 1
    pub async fn bind_attachment(
        db: &DbConn,
        id: i64,
        entity_type: String,
        entity_id: i64,
    ) -> Result<i64, DbErr> {
        // 先查当前 count_quote
        let current = Attach::find_by_id(id).one(db).await?;
        let Some(model) = current else {
            return Ok(0);
        };
        let new_count = model.count_quote.unwrap_or(0) + 1;
        let update_result = Attach::update_many()
            .set(attachment::ActiveModel {
                entity_type: Set(Some(entity_type)),
                entity_id: Set(Some(entity_id)),
                count_quote: Set(Some(new_count)),
                ..Default::default()
            })
            .filter(attachment::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }

    /// 解绑附件：count_quote -= 1（最小为 0）
    pub async fn unbind_attachment(db: &DbConn, id: i64) -> Result<i64, DbErr> {
        let current = Attach::find_by_id(id).one(db).await?;
        let Some(model) = current else {
            return Ok(0);
        };
        let cur = model.count_quote.unwrap_or(0);
        let new_count = if cur > 0 { cur - 1 } else { 0 };
        let update_result = Attach::update_many()
            .set(attachment::ActiveModel {
                count_quote: Set(Some(new_count)),
                ..Default::default()
            })
            .filter(attachment::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }

    /// 按 entity_type + entity_id 查询附件列表
    pub async fn find_by_entity(
        db: &DbConn,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<Vec<attachment::Model>, DbErr> {
        Attach::find()
            .filter(attachment::Column::EntityType.eq(entity_type))
            .filter(attachment::Column::EntityId.eq(entity_id))
            .filter(attachment::Column::Deleted.eq(0))
            .order_by_desc(attachment::Column::Id)
            .all(db)
            .await
    }

    /// 查询某用户的头像记录（未软删除，取最新一条）
    /// 用于头像覆盖上传：同一用户只保留一条头像记录，重新上传时更新该记录。
    pub async fn find_avatar_by_user(
        db: &DbConn,
        user_id: i64,
    ) -> Result<Option<attachment::Model>, DbErr> {
        Attach::find()
            .filter(attachment::Column::EntityType.eq("avatar"))
            .filter(attachment::Column::UploadedBy.eq(user_id))
            .filter(attachment::Column::Deleted.eq(0))
            .order_by_desc(attachment::Column::Id)
            .one(db)
            .await
    }

    /// 头像覆盖上传：仅更新文件相关字段（不动 type_id/storage_type/entity_type/uploaded_by 等归属字段）
    pub async fn update_avatar_file(
        db: &DbConn,
        id: i64,
        name: Option<String>,
        path: Option<String>,
        upload_url: Option<String>,
        ext: Option<String>,
        size: Option<i64>,
        mime_type: Option<String>,
        file_hash: Option<String>,
        original_name: Option<String>,
    ) -> Result<i64, DbErr> {
        let update_result = Attach::update_many()
            .set(attachment::ActiveModel {
                name: Set(name),
                path: Set(path),
                upload_url: Set(upload_url),
                ext: Set(ext),
                size: Set(size),
                mime_type: Set(mime_type),
                file_hash: Set(file_hash),
                original_name: Set(original_name),
                ..Default::default()
            })
            .filter(attachment::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }
}

