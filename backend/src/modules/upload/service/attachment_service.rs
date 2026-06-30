//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 附件服务层（新文件管理系统）
//! - 上传：按 entity_type 多层安全校验、SHA-256 去重、按业务类型分目录存储
//! - 删除：count_quote > 0 拒绝；软删除记录 + 删除物理文件
//! - 绑定/解绑：维护 entity_type/entity_id 与 count_quote 引用计数
//! - 下载：读取物理文件返回二进制
//!

use crate::core::errors::error::{Error, Result};
use crate::core::kit::config;
use crate::core::web::response::ResultPage;
use crate::modules::system::model::admin::AdminModel;
use crate::modules::system::service::config_service;
use crate::modules::upload::model::attachment::{
    AttachmentBindRequest, AttachmentByEntityQuery, AttachmentDetailVO, AttachmentListVO,
    AttachmentModel, AttachmentPageRequest, AttachmentSaveDTO, AttachmentUnbindRequest,
    AttachmentUpdateRequest, ImageFormRequest, PageWhere, StorageType, UploadFileResult,
};
use crate::modules::upload::service::file_validator;
use crate::utils::encryption_utils;
use crate::utils::time_utils::current_date;
use crate::SNOWFLAKE;
use actix_multipart::form::text::Text;
use chrono::Local;
use sea_orm::DbConn;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// # 上传文件
///
/// 新文件管理系统上传入口，按 entity_type 进行多层安全校验，
/// SHA-256 去重，按 `{entity_type}/{YYYY/MM/DD}/{uuid}.{ext}` 路径存储。
///
/// * `db` 数据库连接
/// * `form` multipart 表单（含 file / entity_type / entity_id）
/// * `uploaded_by` 上传者用户ID（由 controller 从 JWT 提取）
pub async fn upload_file(
    db: &DbConn,
    form: ImageFormRequest,
    uploaded_by: Option<i64>,
) -> Result<UploadFileResult> {
    // 1. 解析 entity_type（必填）
    let entity_type = form
        .entity_type
        .as_ref()
        .map(|t| t.0.clone())
        .unwrap_or_default();
    if entity_type.is_empty() {
        return Err(Error::from("entity_type 不能为空"));
    }

    // 2. 解析 entity_id（可选）
    let entity_id: Option<i64> = form
        .entity_id
        .as_ref()
        .and_then(|t| t.0.parse::<i64>().ok());

    // 3. 解析原始文件名
    let original_name = form.file.file_name.clone().unwrap_or_default();
    if original_name.is_empty() {
        return Err(Error::from("没有获取到文件名，上传失败"));
    }

    // 4. 读取文件二进制内容
    let file_data =
        fs::read(&form.file.file).map_err(|e| Error::from(format!("读取文件失败: {}", e)))?;

    // 5. 多层安全校验（扩展名白名单、大小、魔数、图片解码、SHA-256）
    let validated = file_validator::validate_upload(&entity_type, &original_name, &file_data)?;

    // 头像走专用的覆盖上传逻辑：同一用户只保留一条头像记录，重新上传覆盖物理文件。
    // 存储路径固定为 avatar/{用户注册日期(YYYY/MM/DD)}/{md5(user_id)}.ext，
    // 路径稳定 → 自然覆盖，不会每次新增文件/记录。不参与全局 SHA-256 去重。
    if entity_type == "avatar" {
        let result = upload_avatar(db, uploaded_by, validated, file_data).await;
        // 无论成功失败，都清理临时文件
        let _ = fs::remove_file(&form.file.file);
        return result;
    }

    // 6. SHA-256 去重：若已存在相同 hash 的未软删除记录，直接返回
    if let Some(existing) = AttachmentModel::select_by_hash(db, validated.file_hash.clone()).await? {
        // 删除临时文件
        let _ = fs::remove_file(&form.file.file);
        let storage_url = upload_storage_url(db, &Some(StorageType::Local as i32)).await?;
        let full_url = format!(
            "{}{}",
            storage_url,
            existing.upload_url.clone().unwrap_or_default()
        );
        return Ok(UploadFileResult {
            id: Some(existing.id),
            original_name: existing.original_name.or(existing.name),
            url: Some(full_url),
            ext: existing.ext,
            size: existing.size,
            mime_type: existing.mime_type,
            file_hash: existing.file_hash,
            is_public: existing.is_public,
            entity_type: existing.entity_type,
        });
    }

    // 7. 生成存储路径：{entity_type}/{YYYY/MM/DD}/{uuid}.{ext}
    let date_dir = current_date(); // 2024/01/01
    let new_name = format!("{}.{}", encryption_utils::uuid(), validated.ext);
    let path = build_storage_path(&entity_type, &date_dir, &new_name)?;
    let upload_url = build_upload_url(&entity_type, &date_dir, &new_name);

    // 8. 写入物理文件
    save_file_to_local(&file_data, &path)?;
    // 删除临时文件
    let _ = fs::remove_file(&form.file.file);

    // 9. 构造保存 DTO（填充新字段）
    let id = SNOWFLAKE.generate();
    let save_dto = AttachmentSaveDTO {
        id,
        type_id: parse_type_id(&form.type_id),
        name: Some(new_name.clone()),
        path: Some(path.clone()),
        upload_url: Some(upload_url.clone()),
        ext: Some(validated.ext.clone()),
        size: Some(validated.size),
        md5: None, // 新系统使用 SHA-256，不再写入 MD5
        storage_type: Some(StorageType::Local as i32),
        r#type: Some(if file_validator::is_image_ext(&validated.ext) {
            2
        } else {
            1
        }),
        status: Some(1),
        create_time: Some(Local::now().naive_local()),
        original_name: Some(validated.original_name.clone()),
        mime_type: Some(validated.mime_type.clone()),
        file_hash: Some(validated.file_hash.clone()),
        entity_type: Some(entity_type.clone()),
        entity_id,
        uploaded_by,
        is_public: Some(validated.is_public),
        deleted: Some(0),
    };

    let rows = AttachmentModel::insert(db, &save_dto).await?;
    if rows == 0 {
        return Err(Error::from("上传文件保存失败"));
    }

    // 10. 返回上传结果
    let storage_url = upload_storage_url(db, &Some(StorageType::Local as i32)).await?;
    let full_url = format!("{}{}", storage_url, upload_url);

    Ok(UploadFileResult {
        id: Some(id),
        original_name: Some(validated.original_name),
        url: Some(full_url),
        ext: Some(validated.ext),
        size: Some(validated.size),
        mime_type: Some(validated.mime_type),
        file_hash: Some(validated.file_hash),
        is_public: Some(validated.is_public),
        entity_type: Some(entity_type),
    })
}

/// # 头像覆盖上传
///
/// 与通用上传的区别：
/// - 存储目录使用用户**注册时间**（create_time）的 `YYYY/MM/DD`，而非上传当天
/// - 文件名使用 `md5(user_id).ext`，对同一用户稳定 → 物理路径固定 → 重新上传直接覆盖
/// - 同一用户只保留一条头像记录：存在则更新文件字段，不存在则新增
/// - 不参与全局 SHA-256 去重（避免复用其他用户的头像记录）
///
/// * `db` 数据库连接
/// * `uploaded_by` 上传者用户ID（由 controller 从 JWT 提取）
/// * `validated` 校验后的文件元信息
/// * `file_data` 文件二进制内容
async fn upload_avatar(
    db: &DbConn,
    uploaded_by: Option<i64>,
    validated: file_validator::ValidatedFile,
    file_data: Vec<u8>,
) -> Result<UploadFileResult> {
    let user_id = uploaded_by
        .ok_or_else(|| Error::from("未获取到登录用户信息，无法上传头像"))?;

    // 查询用户注册时间作为存储目录
    let user = AdminModel::find_by_id(db, &Some(user_id))
        .await?
        .ok_or_else(|| Error::from("用户不存在，无法上传头像"))?;
    let reg_date = user
        .create_time
        .map(|t| t.format("%Y/%m/%d").to_string())
        .unwrap_or_else(current_date);

    // 文件名固定为 md5(user_id).ext，路径稳定 → 覆盖
    let new_name = format!(
        "{}.{}",
        encryption_utils::md5(&user_id.to_string()),
        validated.ext
    );
    let path = build_storage_path("avatar", &reg_date, &new_name)?;
    let upload_url = build_upload_url("avatar", &reg_date, &new_name);

    // 写入物理文件（同路径直接覆盖）
    save_file_to_local(&file_data, &path)?;

    let storage_url = upload_storage_url(db, &Some(StorageType::Local as i32)).await?;
    // 头像物理路径稳定（md5(user_id).jpg），覆盖上传后浏览器会命中旧缓存。
    // 追加 ?v=<毫秒时间戳> 作为缓存破坏版本号：每次上传 URL 变化，强制重新拉取；
    // 该版本号会随 admin.avatar 一起持久化，刷新后仍生效。静态文件服务忽略查询串。
    let full_url = format!(
        "{}{}?v={}",
        storage_url,
        upload_url,
        Local::now().timestamp_millis()
    );

    // 查询是否已有头像记录
    let existing = AttachmentModel::find_avatar_by_user(db, user_id).await?;

    let original_name = validated.original_name.clone();
    let ext = validated.ext.clone();
    let mime_type = validated.mime_type.clone();
    let file_hash = validated.file_hash.clone();
    let size = validated.size;
    let is_public = validated.is_public;

    let record_id = if let Some(old) = existing {
        // 旧物理路径若与新路径不同（理论上不会，仅扩展名变化时可能），删除旧文件
        let old_path = old.path.clone().unwrap_or_default();
        if !old_path.is_empty() && old_path != path && Path::new(&old_path).exists() {
            let _ = fs::remove_file(&old_path);
        }
        // 更新现有记录的文件字段
        AttachmentModel::update_avatar_file(
            db,
            old.id,
            Some(new_name.clone()),
            Some(path.clone()),
            Some(upload_url.clone()),
            Some(ext.clone()),
            Some(size),
            Some(mime_type.clone()),
            Some(file_hash.clone()),
            Some(original_name.clone()),
        )
        .await?;
        old.id
    } else {
        // 新增头像记录
        let id = SNOWFLAKE.generate();
        let save_dto = AttachmentSaveDTO {
            id,
            type_id: None,
            name: Some(new_name.clone()),
            path: Some(path.clone()),
            upload_url: Some(upload_url.clone()),
            ext: Some(ext.clone()),
            size: Some(size),
            md5: None,
            storage_type: Some(StorageType::Local as i32),
            r#type: Some(2),
            status: Some(1),
            create_time: Some(Local::now().naive_local()),
            original_name: Some(original_name.clone()),
            mime_type: Some(mime_type.clone()),
            file_hash: Some(file_hash.clone()),
            entity_type: Some("avatar".to_string()),
            entity_id: None,
            uploaded_by: Some(user_id),
            is_public: Some(is_public),
            deleted: Some(0),
        };
        let rows = AttachmentModel::insert(db, &save_dto).await?;
        if rows == 0 {
            return Err(Error::from("头像保存失败"));
        }
        id
    };

    Ok(UploadFileResult {
        id: Some(record_id),
        original_name: Some(original_name),
        url: Some(full_url),
        ext: Some(ext),
        size: Some(size),
        mime_type: Some(mime_type),
        file_hash: Some(file_hash),
        is_public: Some(is_public),
        entity_type: Some("avatar".to_string()),
    })
}

/// 解析 type_id（兼容旧字段，可为 None）
fn parse_type_id(type_id: &Option<Text<String>>) -> Option<i64> {
    type_id.as_ref().and_then(|t| t.0.parse::<i64>().ok())
}

/// 构造物理存储绝对路径：`{upload_path}/{entity_type}/{YYYY/MM/DD}/{file_name}`
fn build_storage_path(entity_type: &str, date_dir: &str, file_name: &str) -> Result<String> {
    let base = config::section::<String>("attach", "upload_path", "./storage/upload".to_string());
    let base = base.trim_end_matches('/');
    Ok(format!(
        "{}/{}/{}/{}",
        base, entity_type, date_dir, file_name
    ))
}

/// 构造相对访问 URL：`{upload_url}/{entity_type}/{YYYY/MM/DD}/{file_name}`
fn build_upload_url(entity_type: &str, date_dir: &str, file_name: &str) -> String {
    let base = config::section::<String>("attach", "upload_url", "/upload/".to_string());
    let base = base.trim_start_matches('/').trim_end_matches('/');
    format!("/{}/{}/{}/{}", base, entity_type, date_dir, file_name)
}

/// 将文件二进制内容写入本地存储路径（自动创建父目录）
fn save_file_to_local(file_data: &[u8], full_path: &str) -> Result<()> {
    let path = Path::new(full_path);
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| Error::from(format!("创建目录失败: {}", e)))?;
        }
    }
    fs::write(path, file_data).map_err(|e| Error::from(format!("文件写入失败: {}", e)))?;
    if !path.exists() {
        return Err(Error::from("上传失败：文件未创建成功"));
    }
    // Linux 下设置文件权限与上传根目录一致
    if cfg!(target_os = "linux") {
        let target_path =
            config::section::<String>("attach", "upload_path", "".to_string());
        if let Ok(metadata) = fs::metadata(&target_path) {
            let permissions = metadata.permissions();
            let _ = fs::set_permissions(full_path, permissions);
        }
    }
    Ok(())
}

/// # 批量删除附件
///
/// 删除规则：
/// - count_quote > 0 拒绝删除
/// - 软删除（deleted=1）+ 删除物理文件
///
/// * `db` 数据库连接
/// * `ids` 附件ID集合
pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64> {
    if ids.is_empty() {
        return Err(Error::from("未获得id，删除失败"));
    }

    let mut rows = 0;
    for id in ids {
        let Some(attachment) = AttachmentModel::find_active_by_id(db, id).await? else {
            // 不存在或已软删除，跳过
            continue;
        };

        // count_quote > 0 拒绝删除
        if attachment.count_quote.unwrap_or(0) > 0 {
            return Err(Error::from(format!(
                "附件 [{}] 已被业务引用（count_quote > 0），无法删除",
                id
            )));
        }

        // 软删除数据库记录
        AttachmentModel::soft_delete_by_id(db, id).await?;

        // 删除物理文件
        let path = attachment.path.clone().unwrap_or_default();
        if !path.is_empty() && Path::new(&path).exists() {
            if let Err(e) = fs::remove_file(&path) {
                log::error!("删除物理文件失败: {}, err: {}", path, e);
            }
        }
        rows += 1;
    }
    Ok(rows)
}

/// # 修改附件信息
/// * `db` 数据库连接
/// * `req` 请求参数
pub async fn update(db: &DbConn, req: AttachmentUpdateRequest) -> Result<i64> {
    let save_dto = AttachmentSaveDTO::from(req);
    let rows = AttachmentModel::update(db, &save_dto).await?;
    if rows > 0 {
        Ok(rows)
    } else {
        Err(Error::from("修改失败"))
    }
}

/// # 批量修改分类
/// * `db` 数据库连接
/// * `type_id` 分类id
/// * `ids` 附件id
pub async fn batch_update(db: &DbConn, type_id: Option<i64>, ids: Vec<i64>) -> Result<i64> {
    let rows = AttachmentModel::batch_update(db, type_id, ids).await?;
    if rows > 0 {
        Ok(rows)
    } else {
        Err(Error::from("修改失败"))
    }
}

pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<AttachmentDetailVO> {
    let attachment = AttachmentModel::find_by_id(db, id)
        .await?
        .ok_or_else(|| Error::from("该附件不存在"))?;
    let attachment_vo = AttachmentDetailVO::from(attachment);
    Ok(attachment_vo)
}

/// 分页查询
///
/// - 支持 entity_type 筛选
/// - 返回类型保持 AttachmentListVO
pub async fn get_by_page(
    db: &DbConn,
    query: AttachmentPageRequest,
) -> Result<ResultPage<Vec<AttachmentListVO>>> {
    let page: i64 = query.page_num.unwrap_or(1);
    let per_page: i64 = query.page_size.unwrap_or(20);

    let search_where = PageWhere {
        name: query.name,
        type_id: query.type_id,
        r#type: query.r#type,
        status: query.status,
        entity_type: query.entity_type,
    };
    let search_where = search_where.format();

    let (list, _num_pages) =
        AttachmentModel::list_in_page(db, page, per_page, search_where.clone()).await?;

    // 批量查询上传人名称（关联 mxx_system_admin），避免在循环内逐条查询
    let uploader_ids: Vec<i64> = list
        .iter()
        .filter_map(|m| m.uploaded_by)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let uploader_map: HashMap<i64, String> = if uploader_ids.is_empty() {
        HashMap::new()
    } else {
        AdminModel::find_by_id_in(db, uploader_ids)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|u| (u.id, u.nick_name.or(u.user_name).unwrap_or_default()))
            .collect()
    };

    let mut list_data: Vec<AttachmentListVO> = Vec::new();
    for item in list {
        let url = upload_storage_url(db, &item.storage_type).await?;
        let mut vo = AttachmentListVO::from(item);
        vo.upload_url = Option::from(format!("{}{}", url, vo.upload_url.unwrap_or_default()));
        if let Some(uid) = vo.uploaded_by {
            vo.uploaded_name = uploader_map.get(&uid).cloned();
        }
        list_data.push(vo);
    }
    let count = AttachmentModel::select_count(db, search_where.clone()).await?;

    let page_data = ResultPage::new_simple(list_data, count);

    Ok(page_data)
}

/// # 绑定附件到业务实体
///
/// 更新附件的 entity_type / entity_id，并将 count_quote += 1。
pub async fn bind_attachments(db: &DbConn, req: AttachmentBindRequest) -> Result<i64> {
    if req.ids.is_empty() {
        return Err(Error::from("ids 不能为空"));
    }
    let entity_id = req
        .entity_id
        .ok_or_else(|| Error::from("entity_id 不能为空"))?;
    if req.entity_type.is_empty() {
        return Err(Error::from("entity_type 不能为空"));
    }

    let mut rows = 0;
    for id in req.ids {
        // 校验附件存在且未软删除
        let exists = AttachmentModel::find_active_by_id(db, id).await?;
        if exists.is_none() {
            return Err(Error::from(format!("附件 [{}] 不存在", id)));
        }
        rows += AttachmentModel::bind_attachment(db, id, req.entity_type.clone(), entity_id)
            .await?;
    }
    Ok(rows)
}

/// # 解绑附件
///
/// 将 count_quote -= 1（最小为 0）。
pub async fn unbind_attachments(db: &DbConn, req: AttachmentUnbindRequest) -> Result<i64> {
    if req.ids.is_empty() {
        return Err(Error::from("ids 不能为空"));
    }

    let mut rows = 0;
    for id in req.ids {
        let exists = AttachmentModel::find_active_by_id(db, id).await?;
        if exists.is_none() {
            return Err(Error::from(format!("附件 [{}] 不存在", id)));
        }
        rows += AttachmentModel::unbind_attachment(db, id).await?;
    }
    Ok(rows)
}

/// # 按业务实体查询附件
///
/// 按 entity_type + entity_id 查询附件列表。
pub async fn get_by_entity(
    db: &DbConn,
    query: AttachmentByEntityQuery,
) -> Result<Vec<AttachmentListVO>> {
    let entity_id = query
        .entity_id
        .ok_or_else(|| Error::from("entity_id 不能为空"))?;
    if query.entity_type.is_empty() {
        return Err(Error::from("entity_type 不能为空"));
    }

    let list = AttachmentModel::find_by_entity(db, &query.entity_type, entity_id).await?;
    let storage_url = upload_storage_url(db, &Some(StorageType::Local as i32)).await?;
    let mut result = Vec::with_capacity(list.len());
    for item in list {
        let mut vo = AttachmentListVO::from(item);
        vo.upload_url = Some(format!(
            "{}{}",
            storage_url,
            vo.upload_url.unwrap_or_default()
        ));
        result.push(vo);
    }
    Ok(result)
}

/// # 下载附件
///
/// 读取物理文件返回二进制内容。
///
/// 返回：`(文件内容, 文件名, MIME 类型)`
pub async fn download_file(
    db: &DbConn,
    id: i64,
) -> Result<(Vec<u8>, String, String)> {
    let attachment = AttachmentModel::find_active_by_id(db, id)
        .await?
        .ok_or_else(|| Error::from("附件不存在"))?;

    let path = attachment.path.clone().unwrap_or_default();
    if path.is_empty() || !Path::new(&path).exists() {
        return Err(Error::from("物理文件不存在"));
    }

    let data = fs::read(&path).map_err(|e| Error::from(format!("读取文件失败: {}", e)))?;
    let file_name = attachment
        .original_name
        .clone()
        .or(attachment.name.clone())
        .unwrap_or_default();
    let mime_type = attachment
        .mime_type
        .clone()
        .unwrap_or_else(|| "application/octet-stream".to_string());
    Ok((data, file_name, mime_type))
}

/// # 获取上传服务器网址
pub async fn upload_storage_url(db: &DbConn, storage_type: &Option<i32>) -> Result<String> {
    // 使用枚举代替魔法数字
    let storage_key = storage_type
        .and_then(StorageType::from_i16)
        .map(|st| match st {
            StorageType::Local => "localStorage".to_string(),
            StorageType::Qiniu => "qiniuStorage".to_string(),
            StorageType::Aliyun => "aliyunStorage".to_string(),
            StorageType::Tencent => "tencentStorage".to_string(),
        })
        .unwrap_or_else(|| "localStorage".to_string());

    // 先尝试从数据库获取配置
    if let Ok(config) = config_service::select_by_key(db, &storage_key).await {
        if let Some(value) = config.config_value {
            if !value.is_empty() {
                return Ok(value);
            }
        }
    }

    // 数据库配置不存在或为空时，回退到 config.ini 配置
    match storage_key.as_str() {
        "localStorage" => {
            // 本地存储使用 static_url + upload_url 拼接
            let static_url = config::section::<String>(
                "attach",
                "static_url",
                "http://localhost:8080".to_string(),
            );
            let upload_url =
                config::section::<String>("attach", "upload_url", "/upload/".to_string());
            Ok(format!("{}{}", static_url.trim_end_matches('/'), upload_url))
        }
        _ => Err(Error::from(format!("未配置 {} 存储地址", storage_key))),
    }
}
