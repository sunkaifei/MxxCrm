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
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web::http::header::ContentType;
use minijinja::context;
use serde::{Deserialize, Serialize};
use crate::core::kit::global::AppState;
use crate::core::kit::template::get_template_a_with_cms;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp};
use crate::core::web::tags::cms_tags::CmsTagData;
use crate::modules::website::model::template_data::{ListQuery, TemplateDataSaveDTO, TemplateDataSaveRequest, TemplateDataUpdateRequest};
use crate::modules::website::service::{template_data_service, template_user_data_service, website_service};

/// TPL-6: 模板预览请求
/// 不保存模板内容，直接渲染返回 HTML，便于编辑时实时查看效果
#[derive(Debug, Serialize, Deserialize)]
pub struct PreviewTemplateRequest {
    /// 模板内容字符串
    pub temptext: String,
    /// 模板类型：1首页 2列表 3内容 4标签 5专题
    #[serde(rename = "typeId")]
    pub type_id: Option<i32>,
}

/// TPL-6: 模板预览
/// 注入站点 + CMS 数据 + 模拟分类，渲染模板字符串返回 HTML
pub async fn preview(
    state: web::Data<AppState>,
    _req: HttpRequest,
    body: web::Json<PreviewTemplateRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = body.into_inner();

    // 1. 加载默认站点
    let site = website_service::find_default(db).await?;
    let site_id = site.id.unwrap_or_default();
    let site_mode = site.site_mode.unwrap_or(1);

    // 2. 预取 CMS 标签数据（注入 site_mode）
    let mut cms_data = CmsTagData::fetch(db).await.unwrap_or_default();
    cms_data.site_mode = site_mode;

    // 3. 构建上下文（与 cms_index 保持一致，保证预览效果与正式渲染一致）
    let site_name = site.site_name.clone().unwrap_or_default();
    let site_domain = site.domain.clone().unwrap_or_default();
    let keywords = site.keywords.clone().unwrap_or_default();
    let description = site.description.clone().unwrap_or_default();

    let ctx = context!(
        site => &site,
        site_name => site_name,
        site_domain => site_domain,
        keywords => keywords,
        description => description,
        site_mode => site_mode,
        site_id => site_id,
        preview_mode => true,
    );

    // 4. 渲染模板字符串
    let html = get_template_a_with_cms(&payload.temptext, ctx, &cms_data)?;
    Ok(HttpResponse::Ok().content_type(ContentType::html()).body(html))
}

/// 新增
pub async fn add(state: web::Data<AppState>, req: HttpRequest, item: web::Json<TemplateDataSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.into_inner();

    let form_data = TemplateDataSaveDTO::from(payload);

    let website_id = req.headers().get("website_id")
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);


    let result = template_data_service::insert(&db, &form_data).await?;

    if result > 0 {
        template_user_data_service::save_website_template_merge(&db, &Some(website_id), &Some(result)).await?;
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::success("添加成功".to_string(), "local")))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "添加失败", "local")))
    }
}

/// 批量删除
pub async fn batch_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }

        let result = template_data_service::batch_delete_by_ids(&db, &ids_vec).await?;
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(Ok(result))))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

/// 修改
pub async fn update_by_id(state: web::Data<AppState>, _req: HttpRequest, id: web::Path<i64>, item: web::Json<TemplateDataUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = item.into_inner();

    let mut form_data = TemplateDataSaveDTO::from(payload);
    form_data.id = Some(id.into_inner());

    let result = template_data_service::update_by_id(&db, &form_data).await?;
    if result > 0 {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::success("修改成功".to_string(), "local")))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "修改失败", "local")))
    }
}

/// 详情
pub async fn get_by_detail(state: web::Data<AppState>, item: web::Path<InfoId>) ->Result<HttpResponse> {
    let db = &state.db;
    let result = template_data_service::get_by_detail(&db, &item.id).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
}

/// 根据模板ID查询所有模板数据（不分页，用于前端页面列表抽屉）
#[derive(Debug, Serialize, Deserialize)]
pub struct ListByTemplateQuery {
    #[serde(rename = "templateId")]
    pub template_id: i64,
}

pub async fn list_by_template(state: web::Data<AppState>, query: web::Query<ListByTemplateQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    let q = query.into_inner();
    let result = template_data_service::select_by_template_id(&db, &Some(q.template_id)).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(result, "local")))
}

/// 分页
pub async fn get_by_page(state: web::Data<AppState>, _req: HttpRequest, query: web::Query<ListQuery>) -> Result<HttpResponse> {
    let db = &state.db;
    template_data_service::get_by_page(&db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local"))
    })
}

/// 导出模板方案
///
/// 返回常规 JSON（非 MsgPack），便于前端直接下载保存为 .json 文件。
pub async fn export_scheme(state: web::Data<AppState>, item: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let template_id = item.into_inner();
    let export_data = template_data_service::export_template_scheme(&db, template_id).await?;
    Ok(HttpResponse::Ok().json(export_data))
}

/// 导入模板方案
///
/// 接收 JSON body：`{ "template": {...}, "template_data": [...], "overwrite": false }`
/// 返回 MsgPack，data 为导入后的 template_id。
pub async fn import_scheme(state: web::Data<AppState>, body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let db = &state.db;
    let payload = body.into_inner();
    let overwrite = payload.get("overwrite")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let result = template_data_service::import_template_scheme(&db, payload, overwrite).await?;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::success(result, "local")))
}

// ==================== 路由注册（单点维护）====================

/// 注册模板数据模块所有路由
///
/// 修改路径、权限码、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(template_data_admin_controller::register)` 注册。
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/template/data")
            // POST /template/data/add - 新增模板数据
            .route(
                "/add",
                web::post()
                    .to(add)
                    .wrap(require_permission("template:data:add")),
            )
            // DELETE /template/data/batch_delete - 批量删除模板数据
            .route(
                "/batch_delete",
                web::delete()
                    .to(batch_delete)
                    .wrap(require_permission("template:data:delete")),
            )
            // PUT /template/data/update/{id} - 修改模板数据
            .route(
                "/update/{id}",
                web::put()
                    .to(update_by_id)
                    .wrap(require_permission("template:data:update")),
            )
            // GET /template/data/detail/{id} - 模板数据详情
            .route(
                "/detail/{id}",
                web::get()
                    .to(get_by_detail)
                    .wrap(require_permission("template:data:view")),
            )
            // GET /template/data/list - 模板数据分页
            .route(
                "/list",
                web::get()
                    .to(get_by_page)
                    .wrap(require_permission("template:data:list")),
            )
            // GET /template/data/list_by_template - 按模板ID查询（不分页）
            .route(
                "/list_by_template",
                web::get()
                    .to(list_by_template)
                    .wrap(require_permission("template:data:list")),
            )
            // GET /template/data/export/{template_id} - 导出模板方案（JSON 下载）
            .route(
                "/export/{template_id}",
                web::get()
                    .to(export_scheme)
                    .wrap(require_permission("template:data:list")),
            )
            // POST /template/data/import - 导入模板方案
            .route(
                "/import",
                web::post()
                    .to(import_scheme)
                    .wrap(require_permission("template:data:add")),
            )
            // POST /template/data/preview - 模板预览（不保存，直接渲染返回 HTML）
            .route(
                "/preview",
                web::post()
                    .to(preview)
                    .wrap(require_permission("template:data:list")),
            ),
    );
}
