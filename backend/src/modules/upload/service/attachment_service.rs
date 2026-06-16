//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::{Error, Result};
use crate::core::kit::config;
use crate::core::web::response::{MetaResp, ResultPage};
use crate::modules::system::service::config_service;
use crate::modules::upload::model::attachment::{AttachmentDetailVO, AttachmentModel, AttachmentPageRequest, AttachmentPageVO, AttachmentSaveDTO, AttachmentUpdateRequest, ImageFormRequest, PageWhere, StorageType};
use crate::modules::upload::service::attachment_category_service;
use crate::utils::encryption_utils;
use crate::utils::string_utils::text_to_u64;
use crate::utils::time_utils::current_date;
use crate::SNOWFLAKE;
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::text::Text;
use actix_web::HttpResponse;
use sea_orm::DbConn;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

pub async fn upload_image(db: &DbConn, form: ImageFormRequest) -> Result<HttpResponse> {
    // 获取分类id如果失败默认为1
    let type_id = &form.type_id.unwrap_or(Text("1".to_string()));
    let type_id_u64: i64 = type_id.0.parse().unwrap_or(1);
    // 根据分类id获取分类信息
    if type_id_u64 == 0 {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "未获取到分类id", "local")));
    }
    let category_vo = attachment_category_service::find_by_id(&db, &Some(type_id_u64)).await?;

    if category_vo.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "未获取到分类信息", "local")));
    }
    let file_name = &form.file.file_name.clone().unwrap_or_else(|| "".to_string());

    if file_name.as_str() == "" {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "没有获取到文件，上传失败", "local")));
    }
    
    // 获取当前时间做目录
    let directory = Some(current_date());
    // 设置上传目录
    let module_dir = category_vo.unwrap_or_default().en_name.unwrap_or_default().to_string();
    // 设置上传路径
    let directory_url = upload_path(&module_dir, &directory, &"".to_string())?;

    let ext = get_extension(file_name.clone().as_str());
    if !is_image(ext.clone()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "请上传正确图片类型", "local")));
    }
    let name = format!("{}.{}", encryption_utils::uuid(), ext);
    
    let path = upload_path(&module_dir, &directory, &name)?;
    let url = upload_url(&module_dir,&directory, &name);

    let buffer = &fs::read(&form.file.file).expect("读取文件失败");
    let contents = String::from_utf8_lossy(&buffer).to_string();
    let md5 = encryption_utils::md5(contents.as_str());
    let size = buffer.len() as i64;
    
    let mut result_map: HashMap<String, String> = HashMap::new();
    
    // 获取上传服务器类型
    let config_detail = config_service::select_by_key(&db, &"uploadType".to_string()).await?;
    
    // 获取上传服务器类型
    let storage_type = config_detail.config_value
        .as_deref() // 将 Option<String> 转换为 Option<&str>
        .and_then(|s| s.parse::<i32>().ok()) // 解析字符串为 i32
        .and_then(StorageType::from_i16) // 将 i32 转换为 StorageType
        .ok_or("Invalid storage type")?; // 如果任何步骤失败，返回错误

    // 获取上传服务器地址
    let storage_url = upload_storage_url(&db, &config_detail.config_value.as_deref().and_then(|s| s.parse::<i32>().ok())).await?;

    // 根据类型上传到对应服务器
    match &storage_type {
        StorageType::Local => {
            // 判断是否有相同
            let attach_data = AttachmentModel::select_by_md5(&db, md5.clone()).await;
            match attach_data {
                Ok(attach_option) => {
                    match attach_option {
                        Some(attach) => {
                            result_map.insert("fileName".to_string(), attach.name.unwrap_or_default());

                            let img_url = format!(
                                "{}{}",
                                &storage_url,
                                attach.upload_url.unwrap_or_default()
                            ).to_string();
                            result_map.insert("url".to_string(), img_url);
                            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result_map, "local")));
                        }
                        _ => {
                            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "获取上传服务器地址失败", "local")));
                        }
                    }
                }
                Err(err) => {
                    let _ = Error::from(format!(
                        "{}：{}",
                        "查询图片信息错误".to_string(),
                        err.to_string()
                    ));
                }
            };
            // 上传到本地服务器
            upload_to_local_storage(&form.file, &Some(directory_url)).await
        },
        StorageType::Qiniu => upload_to_qiniu_storage().await,
        StorageType::Aliyun => upload_to_aliyun_storage().await,
        StorageType::Tencent => upload_to_tencent_storage().await,
    }?;
    
    let id = SNOWFLAKE.generate();
    
    let type_id_u64 = match text_to_u64(&type_id) {
        Ok(number) => number,
        Err(_) => {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "请选择正确的类型", "local")));
        }
    };
    let create_data = AttachmentSaveDTO {
        id,
        user_id: None,
        type_id: Some(type_id_u64),
        name: Some(file_name.clone()),
        path: Some(path.clone()),
        upload_url: Some(url.clone()),
        ext: Some(ext.clone()),
        size: Some(size),
        md5: Some(md5),
        storage_type: config_detail.config_value
            .as_deref()
            .and_then(|s| s.parse::<i32>().ok()).clone(),
        r#type: Some(2),
        status: Some(1i32),
        create_time: None,
    };
    
    let rows = AttachmentModel::insert(&db, &create_data.clone()).await?;
    if rows > 0 {
        result_map.insert("fileName".to_string(), file_name.clone().to_string());
        let img_url = format!(
            "{}{}",
            &storage_url,
            &url.to_string()
        ).to_string();
        result_map.insert("url".to_string(), img_url);

        if fs::remove_file(&path).is_err() {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "保存失败，图片删除错误", "local")));
        }
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result_map, "local")));
    }
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "上传图片保存失败", "local")))
}

async fn upload_to_local_storage(file: &TempFile, save_directory: &Option<String>) -> Result<bool> {
    if save_directory.as_ref().map_or(true, |directory_name| directory_name.trim().is_empty()) {
        return Err(Error::from("目录不能为空"))
    }
    
    let directory_url = save_directory.clone().unwrap_or_default();
    if !Path::new(directory_url.as_str()).exists() {
        log::info!("创建目录: {}", directory_url.as_str());
        fs::create_dir_all(directory_url.as_str()).unwrap_or_else(|err| eprintln!("创建目录失败: {}", err));
    }

    fs::copy(&file.file, &directory_url).unwrap_or_default();
    //log::info!("=============1=======logo:{:?}  ======{:?}", &form.file.file.path(),&path);
    if !Path::new(&directory_url).exists(){
        return Err(Error::from("上传失败"));
    }
    //log::info!("==========2==========logo:{:?}", &path);
    if cfg!(target_os = "linux") {
        //附件根目录路径
        let target_path = &config::section::<String>("attach", "upload_path", "".to_string());
        // 获取父级目录的权限
        let parent_permissions = match fs::metadata(&target_path) {
            Ok(metadata) => metadata.permissions(),
            Err(err) => {
                log::error!("linux下获取父级权限失败: {:?}", err);
                return Err(Error::from("linux下获取父级权限失败"));
            }
        };
        // 设置新文件的权限
        fs::set_permissions(&directory_url, parent_permissions).expect("父级目录权限设置给新文件失败");
    }
    Ok(true)
}

async fn upload_to_qiniu_storage() -> Result<bool> {
    println!("Uploading to Qiniu Storage...");
    Ok(true)
}

async fn upload_to_aliyun_storage() -> Result<bool> {
    println!("Uploading to Aliyun Storage...");
    Ok(true)
}

async fn upload_to_tencent_storage() -> Result<bool> {
    println!("Uploading to Tencent Storage...");
    Ok(true)
}

/// # 上传路径
/// 
/// * `module` 模块名称  如：article 
/// * `directory` 指定目录名称 如：current当前时间：2024/8/29，
/// * `file_name` 文件名称 如：123.jpg 
#[allow(dead_code)]
pub fn upload_path(module: &String, directory: &Option<String>, file_name: &String) -> Result<String> {
    // 检查 module 是否为空
    if module.is_empty() {
        return Err(Error::from("Module cannot be empty"));
    }
    
    let path = &config::section::<String>("attach", "upload_path", "".to_string());

    let directory_url: String = match directory {
        Some(dir) => format!("/{}/", dir),
        None => format!("/{}/", current_date()),
    };

    Ok(format!("{}{}{}{}", path, module, directory_url, file_name))
}

/// # 网站调用显示路径
/// * `module` 模块名称  如：article
/// * `directory` 目录名称 如：2024/8/29
/// * `file_name` 文件名称 如：123.jpg 
#[allow(dead_code)]
pub fn upload_url(module: &String, directory: &Option<String>, file_name: &String) -> String {
    let path = &config::section::<String>("attach", "upload_url", "".to_string());
    let directory_url: String = match directory {
        Some(dir) => format!("/{}/", dir),
        None => format!("/{}/", current_date()),
    };
    format!("{}{}{}{}", path, module, directory_url, file_name)
}

pub fn get_extension(filename: &str) -> String {
    let extension = Path::new(filename)
        .extension()
        .and_then(OsStr::to_str);

    if let Some(ext) = extension {
        return ext.to_string();
    }

    "".to_string()
}

pub fn is_image(extension: String) -> bool {
    extension.eq("png")
        || extension.eq("jpg")
        || extension.eq("jpeg")
        || extension.eq("ico")
        || extension.eq("gif")
        || extension.eq("bmp")
        || extension.eq("svg")
}

/// # 批量删除
/// * `db` 数据库连接
/// * `ids` id集合
pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64> {
    if ids.is_empty() {
        return Err(Error::from("未获得id，删除失败"));
    }

    let mut rows = 0;
    for id in ids {
        if let Some(attachment) = AttachmentModel::find_by_id(db, &Some(id)).await? {
            if attachment.count_quote.unwrap_or_default() == 0 {
                let path = attachment.path.clone().unwrap_or_default();
                log::info!("=======删除图片: {}", &path);
                if Path::new(&path).exists() {
                    if fs::remove_file(&path).is_err() {
                        log::error!("图片删除失败: {}", path);
                    }
                    log::warn!("文件不存在: {}", path);
                }
                AttachmentModel::delete_by_id(db, &Some(attachment.id)).await?;
                rows += 1;
            }
        }
    }
    Ok(rows)
}

/// # 修改附件信息
/// * `db` 数据库连接
/// * `req` 请求参数
pub async fn update(db: &DbConn, req: AttachmentUpdateRequest) -> Result<i64> {
    let seve_dto= AttachmentSaveDTO::from(req);
    let rows = AttachmentModel::update(db, &seve_dto).await?;
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
    let attachment = AttachmentModel::find_by_id(db, id).await?.ok_or_else(|| { Error::from("该附件不存在")})?;
    let attachment_vo = AttachmentDetailVO::from(attachment);
    Ok(attachment_vo)
}

/// 分页查询
pub async fn get_by_page(db: &DbConn, query: AttachmentPageRequest) ->  Result<ResultPage<Vec<AttachmentPageVO>>> {
    let page: i64 = query.page_num.unwrap();
    let per_page: i64 = query.page_size.unwrap();
    
    let search_where = PageWhere{
        name: query.name,
        type_id: query.type_id,
        r#type: query.r#type,
        status: query.status,
    };
    let search_where = search_where.format();

    let (list, _num_pages)  = AttachmentModel::list_in_page(&db,page,per_page, search_where.clone()).await?;
    //let list_data: Vec<AttachmentListVO> = list.into_iter().map(|item| AttachmentListVO::from(item)).collect();
    let mut list_data: Vec<AttachmentPageVO> = Vec::new();
    for item in list {
        let url = upload_storage_url(db, &item.storage_type).await?;
        list_data.push(AttachmentPageVO {
            id: Option::from(item.id),
            type_id: item.type_id,
            name: item.name,
            upload_url: Option::from(format!("{}{}", url, item.upload_url.unwrap_or_default())),
        });
    }
    let count = AttachmentModel::select_count(&db,search_where.clone()).await?;  
    
    let page_data = ResultPage::new_simple(list_data, count);
    
    Ok(page_data)
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

    // 获取上传服务器类型
    let config = config_service::select_by_key(db, &storage_key).await?;
    Ok(config.config_value.unwrap_or_default())
}
