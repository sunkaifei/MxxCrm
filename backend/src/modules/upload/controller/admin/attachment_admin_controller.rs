//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::entity::common::{BathDeleteIdRequest, BathIdRequest, InfoId};
use crate::core::web::response::MetaResp;
use crate::modules::upload::model::attachment::{AttachmentPageRequest, AttachmentUpdateRequest, BatchMoveRequest, ImageFormRequest};
use crate::modules::upload::service::{attachment_category_service, attachment_service};
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;
use actix_multipart::form::MultipartForm;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;
use crate::validate;

/// # 上传图片
#[post("/attachment/upload")]
pub async fn upload_attachment(state: web::Data<AppState>, _req: HttpRequest, MultipartForm(form): MultipartForm<ImageFormRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    
    Ok(attachment_service::upload_image(&db, form).await?)
}

/// # 删除附件
#[delete("/attachment/batch_delete")]
pub async fn delete_attachment(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }

        let ids = convert_vec_option_string_to_vec_u64(ids_vec);
        let result = attachment_service::batch_delete_by_ids(&db, ids).await?;
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(Ok(result))))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

/// # 修改附件信息
#[put("/attachment/update/{id}")]
pub async fn update(state: web::Data<AppState>, id: web::Path<i64>, item: web::Json<AttachmentUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let mut data = item.into_inner();
    data.id = Some(id.into_inner());
    validate!(data.name.is_none(), t!("attachment.category.id_empty", locale = "zh-CN").to_string());
    let result = attachment_service::update(&db, data).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(Ok(result))))
}

#[put("/attachment/batchmove")]
pub async fn batch_move(state: web::Data<AppState>, item: web::Json<BatchMoveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    validate!(item.type_id.is_none(), t!("attachment.category.id_empty", locale = "zh-CN").to_string());

    let category_info= attachment_category_service::find_by_id(db, &item.type_id).await?;
    if category_info.is_none() {
        validate!(true, t!("attachment.category.info_empty", locale = "zh-CN").to_string());
    }
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
        } else {
            
            let ids = convert_vec_option_string_to_vec_u64(ids_vec);
            let result = attachment_service::batch_update(db, item.type_id, ids).await;

            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
        }
    }else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
    
}


/// # 获取附件详情
#[get("/attachment/detail/{id}")]
pub async fn get_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "ID不能为空", "local")));
    }
    let result = attachment_service::get_by_detail(&db, &item.id).await?;
    
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
}

/// # 分页获取附件列表
#[get("/attachment/list")]
#[protect("attachment:list")]
pub async fn get_page_list(state: web::Data<AppState>, query: web::Query<AttachmentPageRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = query.into_inner();

    attachment_service::get_by_page(&db, payload).await.map(|page_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local"))
    })
}


