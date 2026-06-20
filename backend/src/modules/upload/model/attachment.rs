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
            user_id: None,
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
        }
    }
    
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UploadBase64Data {
    pub base64: Option<String>,
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
    // 当前页码数
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    // 每页条数
    pub page_size: Option<i64>,
    pub status: Option<i32>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct AttachmentSaveDTO {
    pub id: i64,
    pub user_id: Option<i64>,
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

        Self {
            name,
            type_id,
            r#type,
            status,
        }
    }
}

pub struct AttachmentModel;

impl AttachmentModel {
    pub async fn insert(db: &DbConn, form_data: &AttachmentSaveDTO) -> Result<i64, DbErr> {
        let payload = attachment::ActiveModel {
            user_id:      Set(form_data.user_id.to_owned()),
            type_id:      Set(form_data.type_id.to_owned()),
            name:         Set(form_data.name.to_owned()),
            path:         Set(form_data.path.to_owned()),
            upload_url:   Set(form_data.upload_url.to_owned()),
            ext:          Set(form_data.ext.to_owned()),
            size:         Set(form_data.size.to_owned()),
            md5:          Set(form_data.md5.to_owned()),
            r#type:       Set(form_data.r#type.to_owned()),
            status:       Set(form_data.status.to_owned()),
            create_time:  Set(Option::from(chrono::Local::now().naive_local().to_owned())),
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
                type_id:      Set(form_data.type_id.to_owned()),
                name:         Set(form_data.name.to_owned()),
                path:         Set(form_data.path.to_owned()),
                upload_url:   Set(form_data.upload_url.to_owned()),
                ext:          Set(form_data.ext.to_owned()),
                size:         Set(form_data.size.to_owned()),
                md5:          Set(form_data.md5.to_owned()),
                r#type:       Set(form_data.r#type.to_owned()),
                status:       Set(form_data.status.to_owned()),
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
    
    /// 根据md5查询（admin端使用）
    pub async fn select_by_md5(db: &DbConn, md5: String) -> Result<Option<attachment::Model>, DbErr> {
        let find_result = Attach::find()
            .filter(attachment::Column::Md5.eq(md5))
            .one(db)
            .await?;
        Ok(find_result)
    }
    
    /// 根据用户ID和md5查询（用户端上传去重）
    pub async fn select_by_user_id_and_md5(db: &DbConn, user_id: i64, md5: String) -> Result<Option<attachment::Model>, DbErr> {
        let find_result = Attach::find()
            .filter(attachment::Column::UserId.eq(user_id))
            .filter(attachment::Column::Md5.eq(md5))
            .one(db)
            .await?;
        Ok(find_result)
    }
    
    
    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        Attach::find()
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
            .order_by_desc(attachment::Column::Id)
            .paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;
        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}

