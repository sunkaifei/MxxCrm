//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 网站展示产品管理控制器：前台"产品中心"上架清单（选品/上架/下架/排序/移除）
//!

use actix_web::{web, HttpResponse};

use crate::core::errors::error::Result;
use crate::core::kit::global::AppState;
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::website::model::website_product::{
    WebsiteProductAddRequest, WebsiteProductCategoryRequest, WebsiteProductDeleteRequest,
    WebsiteProductLimitBuyRequest, WebsiteProductListQuery, WebsiteProductQuantityRequest,
    WebsiteProductRecommendRequest, WebsiteProductRelatedRequest, WebsiteProductScheduleRequest,
    WebsiteProductSeoRequest, WebsiteProductShelfRequest, WebsiteProductSkuPricesRequest,
    WebsiteProductSkusRequest, WebsiteProductSortRequest,
};
use crate::modules::website::service::website_product_service;

/// GET /website/product/list - 展示产品清单（分页，含产品信息富化）
pub async fn list(
    state: web::Data<AppState>,
    query: web::Query<WebsiteProductListQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let page_num = query.page_num.unwrap_or(1).max(1);
    let page_size = std::cmp::min(std::cmp::max(query.page_size.unwrap_or(10), 1), 100);
    let (list, total, _total_pages) =
        website_product_service::list(db, query.into_inner()).await?;
    // ResultPage 序列化为 {items, total, ...}，与前端 vxe 适配器的分页映射对齐
    let result = crate::core::web::response::ResultPage::new(list, total, page_num, page_size);
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::success(result, "local")))
}

/// POST /website/product/add - 从产品库选择产品加入展示清单（自动上架）
pub async fn add(
    state: web::Data<AppState>,
    body: web::Json<WebsiteProductAddRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let added = website_product_service::add(db, &body).await?;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<i64>::success(added, "local")))
}

/// PUT /website/product/shelf - 批量上架/下架（status: 1=上架 0=下架）
pub async fn shelf(
    state: web::Data<AppState>,
    body: web::Json<WebsiteProductShelfRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    website_product_service::shelf(db, &body).await?;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<String>::success("操作成功".to_string(), "local")))
}

/// PUT /website/product/sort - 调整清单排序
pub async fn sort(
    state: web::Data<AppState>,
    body: web::Json<WebsiteProductSortRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    website_product_service::update_sort(db, body.id, body.sort).await?;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<String>::success("操作成功".to_string(), "local")))
}

/// PUT /website/product/quantity - 调整展示数量（防超卖，钳制为不超过当前库存）
pub async fn quantity(
    state: web::Data<AppState>,
    body: web::Json<WebsiteProductQuantityRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let effective = website_product_service::update_quantity(db, body.id, body.quantity).await?;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::success(effective, "local")))
}

/// PUT /website/product/category - 归属栏目分类（内容类型=产品的栏目；0=清除）
pub async fn category(
    state: web::Data<AppState>,
    body: web::Json<WebsiteProductCategoryRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    website_product_service::update_category(db, body.id, body.category_id).await?;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<String>::success("操作成功".to_string(), "local")))
}

/// GET /website/product/categories - 栏目分类选项（内容类型=产品的栏目）
pub async fn categories(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    let options = website_product_service::shelf_category_options(db).await?;
    let list: Vec<serde_json::Value> = options
        .into_iter()
        .map(|(id, name)| serde_json::json!({ "id": id, "name": name }))
        .collect();
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::success(list, "local")))
}

/// GET /website/product/skus?productId={id} - 产品的 SKU 列表（选品抽屉展开用）
pub async fn skus(
    state: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let product_id = query
        .get("productId")
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or_else(|| crate::core::errors::error::Error::from("productId 不能为空"))?;
    let list = website_product_service::product_skus(db, product_id).await?;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::success(list, "local")))
}

/// PUT /website/product/update_skus - 更新清单行展示的 SKU（空 = 全部）
pub async fn update_skus(
    state: web::Data<AppState>,
    body: web::Json<WebsiteProductSkusRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    website_product_service::update_skus(db, body.id, &body.sku_ids).await?;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<String>::success("操作成功".to_string(), "local")))
}

/// PUT /website/product/sku_prices - 保存 SKU 前台零售价（仅作用于前台在线销售）
pub async fn sku_prices(
    state: web::Data<AppState>,
    body: web::Json<WebsiteProductSkuPricesRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    website_product_service::update_sku_prices(db, body.id, &body.prices).await?;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<String>::success("保存成功".to_string(), "local")))
}

/// PUT /website/product/recommend - 批量设置/取消推荐
pub async fn recommend(
    state: web::Data<AppState>,
    body: web::Json<WebsiteProductRecommendRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    website_product_service::update_recommend(db, &body.ids, body.is_recommend).await?;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<String>::success("操作成功".to_string(), "local")))
}

/// PUT /website/product/seo - 保存 SEO 设置
pub async fn seo(
    state: web::Data<AppState>,
    body: web::Json<WebsiteProductSeoRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    website_product_service::update_seo(db, body.id, body.seo_title.clone(), body.seo_keywords.clone(), body.seo_description.clone()).await?;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<String>::success("保存成功".to_string(), "local")))
}

/// PUT /website/product/limit_buy - 设置每人限购数量
pub async fn limit_buy(
    state: web::Data<AppState>,
    body: web::Json<WebsiteProductLimitBuyRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    website_product_service::update_limit_buy(db, body.id, body.limit_buy).await?;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<String>::success("保存成功".to_string(), "local")))
}

/// PUT /website/product/schedule - 设置定时上架/下架
pub async fn schedule(
    state: web::Data<AppState>,
    body: web::Json<WebsiteProductScheduleRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let parse = |s: &Option<String>| -> Result<Option<chrono::NaiveDateTime>> {
        match s.as_deref().map(str::trim) {
            Some(v) if !v.is_empty() => Ok(Some(
                chrono::NaiveDateTime::parse_from_str(v, "%Y-%m-%d %H:%M:%S")
                    .map_err(|e| crate::core::errors::error::Error::from(format!("时间格式非法: {}", e)))?,
            )),
            _ => Ok(None),
        }
    };
    let list_at = parse(&body.list_at)?;
    let unlist_at = parse(&body.unlist_at)?;
    website_product_service::update_schedule(db, body.id, list_at, unlist_at).await?;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<String>::success("保存成功".to_string(), "local")))
}

/// PUT /website/product/related - 保存相关产品/相关文章
pub async fn related(
    state: web::Data<AppState>,
    body: web::Json<WebsiteProductRelatedRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    website_product_service::update_related(db, body.id, &body.related_product_ids, &body.related_article_ids).await?;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<String>::success("保存成功".to_string(), "local")))
}

/// DELETE /website/product/delete - 批量移除出展示清单
pub async fn delete(
    state: web::Data<AppState>,
    body: web::Json<WebsiteProductDeleteRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    website_product_service::delete(db, &body.ids).await?;
    Ok(HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<String>::success("移除成功".to_string(), "local")))
}

/// 注册路由（单站模式，后端自动定位默认站点）
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/website/product")
            .route("/list", web::get().to(list).wrap(require_permission("website:product:list")))
            .route("/add", web::post().to(add).wrap(require_permission("website:product:add")))
            .route("/shelf", web::put().to(shelf).wrap(require_permission("website:product:update")))
            .route("/sort", web::put().to(sort).wrap(require_permission("website:product:update")))
            .route("/quantity", web::put().to(quantity).wrap(require_permission("website:product:update")))
            .route("/category", web::put().to(category).wrap(require_permission("website:product:update")))
            .route("/categories", web::get().to(categories).wrap(require_permission("website:product:list")))
            .route("/skus", web::get().to(skus).wrap(require_permission("website:product:list")))
            .route("/update_skus", web::put().to(update_skus).wrap(require_permission("website:product:update")))
            .route("/sku_prices", web::put().to(sku_prices).wrap(require_permission("website:product:update")))
            .route("/recommend", web::put().to(recommend).wrap(require_permission("website:product:update")))
            .route("/seo", web::put().to(seo).wrap(require_permission("website:product:update")))
            .route("/limit_buy", web::put().to(limit_buy).wrap(require_permission("website:product:update")))
            .route("/schedule", web::put().to(schedule).wrap(require_permission("website:product:update")))
            .route("/related", web::put().to(related).wrap(require_permission("website:product:update")))
            .route("/delete", web::delete().to(delete).wrap(require_permission("website:product:delete"))),
    );
}
