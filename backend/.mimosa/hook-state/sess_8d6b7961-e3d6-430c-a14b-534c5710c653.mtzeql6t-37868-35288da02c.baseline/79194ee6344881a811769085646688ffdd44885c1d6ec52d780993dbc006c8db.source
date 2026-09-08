//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 网站展示产品服务：前台"产品中心"上架清单管理（选品/上架/下架/排序/移除）
//! 与前台展示过滤（CMS 产品列表、详情、模板标签统一走本模块的上架口径）。
//!

use std::collections::HashMap;

use rust_decimal::Decimal;
use sea_orm::sea_query::Expr;
use sea_orm::*;

use crate::core::errors::error::Result;
use crate::modules::message::service::notification_service::NotificationService;
use crate::modules::articles::entity::{category as article_category, article as article_entity};
use crate::modules::product::entity::{category as product_category, product, sku as product_sku};
use crate::modules::product::model::product::{ProductListQuery, ProductListVO};
use crate::modules::product::service::product_service;
use crate::modules::website::entity::website_product;
use crate::modules::website::model::website_product::{
    WebsiteProductAddRequest, WebsiteProductCategoryRequest, WebsiteProductListQuery,
    WebsiteProductListVO, WebsiteProductQuantityRequest, WebsiteProductShelfRequest,
};
use crate::modules::website::model::website::SiteModel;

/// 栏目内容类型：产品（栏目管理 contentType=2）
pub const ARTICLE_CONTENT_TYPE_PRODUCT: i32 = 2;

/// 默认站点 ID（单站模式）
async fn default_site_id(db: &DatabaseConnection) -> i64 {
    SiteModel::find_default(db)
        .await
        .ok()
        .flatten()
        .map(|w| w.id)
        .unwrap_or(1)
}

/// 解析逗号分隔 ID 列表
fn parse_id_list(text: &Option<String>) -> Vec<i64> {
    text.as_deref()
        .unwrap_or_default()
        .split(',')
        .filter_map(|s| s.trim().parse::<i64>().ok())
        .collect()
}

/// 序列化为逗号分隔 ID 列表
fn format_id_list(ids: &[i64]) -> String {
    ids.iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

/// 批量查询产品可售库存基准（防超卖口径）：
/// 多规格产品 = SUM(mxx_product_sku.stock)（启用 SKU）；单规格 = 产品库存字段。
/// （12.5 需求：库存基准以多规格 SKU 库存为准，不再使用 mxx_inventory_stock）
async fn stock_base_map(db: &DatabaseConnection, pids: &[i64]) -> Result<HashMap<i64, i64>> {
    let mut map = HashMap::new();
    if pids.is_empty() {
        return Ok(map);
    }
    let mut sku_sum: HashMap<i64, i64> = HashMap::new();
    let mut has_sku: std::collections::HashSet<i64> = std::collections::HashSet::new();
    for s in product_sku::Entity::find()
        .filter(product_sku::Column::ProductId.is_in(pids.to_vec()))
        .filter(product_sku::Column::IsActive.eq(true))
        .all(db)
        .await?
    {
        has_sku.insert(s.product_id);
        *sku_sum.entry(s.product_id).or_insert(0) += s.stock.unwrap_or(0) as i64;
    }
    let missing: Vec<i64> = pids.iter().filter(|p| !has_sku.contains(*p)).copied().collect();
    let mut pstock: HashMap<i64, i64> = HashMap::new();
    if !missing.is_empty() {
        for p in product::Entity::find()
            .filter(product::Column::Id.is_in(missing))
            .all(db)
            .await?
        {
            pstock.insert(p.id, p.stock.unwrap_or(0) as i64);
        }
    }
    for pid in pids {
        let v = if has_sku.contains(pid) { sku_sum[pid] } else { pstock.get(pid).copied().unwrap_or(0) };
        map.insert(*pid, v);
    }
    Ok(map)
}

// ==================== 管理端 ====================

/// 逗号分隔 sku_ids 存取
fn parse_sku_ids(text: &Option<String>) -> Vec<i64> {
    text.as_deref()
        .unwrap_or_default()
        .split(',')
        .filter_map(|s| s.trim().parse::<i64>().ok())
        .collect()
}

fn format_sku_ids(ids: &[i64]) -> String {
    ids.iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

/// 展示产品清单（分页 + 产品信息/库存/栏目分类/SKU 富化）
pub async fn list(
    db: &DatabaseConnection,
    query: WebsiteProductListQuery,
) -> Result<(Vec<WebsiteProductListVO>, i64, i64)> {
    let website_id = default_site_id(db).await;
    let page_num = std::cmp::max(query.page_num.unwrap_or(1), 1);
    let page_size = std::cmp::min(std::cmp::max(query.page_size.unwrap_or(10), 1), 100);

    // 清单行（默认站点）
    let mut rows = website_product::Entity::find()
        .filter(website_product::Column::WebsiteId.eq(website_id))
        .filter(website_product::Column::Deleted.eq(0))
        .apply_if(query.status, |q, v| {
            q.filter(website_product::Column::Status.eq(v))
        })
        .apply_if(query.category_id.filter(|c| *c > 0), |q, v| {
            q.filter(website_product::Column::CategoryId.eq(v))
        })
        .all(db)
        .await?;

    // 批量取产品信息
    let pids: Vec<i64> = rows.iter().map(|r| r.product_id).collect();
    let mut product_map: HashMap<i64, product::Model> = HashMap::new();
    if !pids.is_empty() {
        for p in product::Entity::find()
            .filter(product::Column::Id.is_in(pids.clone()))
            .filter(product::Column::Deleted.eq(0))
            .all(db)
            .await?
        {
            product_map.insert(p.id, p);
        }
    }

    // 当前仓储库存（防超卖基准）
    let stock_map = stock_base_map(db, &pids).await?;

    // 产品库分类名称（参考信息）
    let pcat_ids: Vec<i64> = product_map
        .values()
        .filter_map(|p| p.category_id)
        .collect();
    let mut pcat_map: HashMap<i64, String> = HashMap::new();
    if !pcat_ids.is_empty() {
        for c in product_category::Entity::find()
            .filter(product_category::Column::Id.is_in(pcat_ids))
            .filter(product_category::Column::Deleted.eq(0))
            .all(db)
            .await?
        {
            if let Some(n) = c.name {
                pcat_map.insert(c.id, n);
            }
        }
    }

    // 栏目分类名称（栏目管理中内容类型=产品的栏目）
    let cat_ids: Vec<i64> = rows.iter().filter_map(|r| r.category_id).collect();
    let mut cat_map: HashMap<i64, String> = HashMap::new();
    if !cat_ids.is_empty() {
        for c in article_category::Entity::find()
            .filter(article_category::Column::Id.is_in(cat_ids))
            .filter(article_category::Column::Status.eq(1))
            .all(db)
            .await?
        {
            if let Some(n) = c.category_name {
                cat_map.insert(c.id, n);
            }
        }
    }

    // 关键词过滤（按产品名称，内存过滤——上架清单体量小）
    if let Some(kw) = query.keywords.as_deref() {
        let kw = kw.trim().to_lowercase();
        if !kw.is_empty() {
            rows.retain(|r| {
                product_map
                    .get(&r.product_id)
                    .and_then(|p| p.name.as_deref())
                    .map(|n| n.to_lowercase().contains(&kw))
                    .unwrap_or(false)
            });
        }
    }

    // 清单排序：sort 升序，其次加入时间倒序
    rows.sort_by(|a, b| {
        a.sort.unwrap_or(0)
            .cmp(&b.sort.unwrap_or(0))
            .then(b.id.cmp(&a.id))
    });

    let total = rows.len() as i64;
    let total_pages = if total == 0 { 1 } else { (total + page_size - 1) / page_size };
    let page_rows: Vec<website_product::Model> = rows
        .into_iter()
        .skip(((page_num - 1) * page_size) as usize)
        .take(page_size as usize)
        .collect();

    // 自愈钳制：存储数量 > 库存基准时自动写回钳制值（数据自愈，管理员无感知）
    let stale_ids: Vec<(i64, i64)> = page_rows
        .iter()
        .filter(|r| {
            let base = stock_map.get(&r.product_id).copied().unwrap_or(0);
            let stored = r.quantity.unwrap_or(0) as i64;
            stored > base
        })
        .map(|r| {
            let base = stock_map.get(&r.product_id).copied().unwrap_or(0);
            (r.id, base)
        })
        .collect();
    if !stale_ids.is_empty() {
        let now = chrono::Local::now().naive_local();
        for (row_id, clamped) in &stale_ids {
            website_product::Entity::update_many()
                .col_expr(website_product::Column::Quantity, Expr::value(*clamped as i32))
                .col_expr(website_product::Column::UpdateTime, Expr::value(now))
                .filter(website_product::Column::Id.eq(*row_id))
                .exec(db)
                .await?;
        }
    }

    let list: Vec<WebsiteProductListVO> = page_rows
        .into_iter()
        .map(|r| {
            let p = product_map.get(&r.product_id);
            let pcat_id = p.and_then(|p| p.category_id);
            let stock = stock_map.get(&r.product_id).copied();
            let stored = r.quantity.unwrap_or(0) as i64;
            // 生效展示数量 = min(存储数量, 当前库存)——库存减少时自动跟随下降
            let effective = stock.map(|s| std::cmp::min(stored, s)).unwrap_or(stored);
            let sku_ids = parse_sku_ids(&r.sku_ids);
            let related_pids = parse_id_list(&r.related_product_ids);
            let related_aids = parse_id_list(&r.related_article_ids);
            WebsiteProductListVO {
                id: r.id,
                product_id: Some(r.product_id),
                product_name: p.and_then(|p| p.name.clone()),
                product_image: p.and_then(|p| p.image_url.clone()),
                sale_price: p.and_then(|p| p.sale_price),
                quantity: Some(stored),
                total_stock: stock,
                effective_quantity: Some(effective),
                category_id: r.category_id,
                category_name: r.category_id.and_then(|cid| cat_map.get(&cid).cloned()),
                product_category_name: pcat_id.and_then(|cid| pcat_map.get(&cid).cloned()),
                is_active: p.and_then(|p| p.is_active),
                status: r.status,
                sort: r.sort,
                sku_ids: if sku_ids.is_empty() { None } else { Some(sku_ids.clone()) },
                sku_count: Some(sku_ids.len() as i64),
                sku_prices: r.sku_prices.as_deref().and_then(|s| serde_json::from_str(s).ok()),
                is_recommend: r.is_recommend,
                related_product_ids: if related_pids.is_empty() { None } else { Some(related_pids) },
                related_article_ids: if related_aids.is_empty() { None } else { Some(related_aids) },
                seo_title: r.seo_title,
                seo_keywords: r.seo_keywords,
                seo_description: r.seo_description,
                retail_price: r.retail_price,
                promo_price: r.promo_price,
                promo_start: r.promo_start.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
                promo_end: r.promo_end.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
                sku_promo_prices: r.sku_promo_prices.as_deref().and_then(|s| serde_json::from_str(s).ok()),
                limit_buy: r.limit_buy,
                stock_warn_threshold: r.stock_warn_threshold,
                list_at: r.list_at.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
                unlist_at: r.unlist_at.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
                create_time: r
                    .create_time
                    .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
            }
        })
        .collect();

    Ok((list, total, total_pages))
}

/// 从产品库选择产品加入展示清单（自动去重；新行默认上架，数量钳制为当前库存）
pub async fn add(db: &DatabaseConnection, req: &WebsiteProductAddRequest) -> Result<i64> {
    let website_id = default_site_id(db).await;
    let items = &req.items;
    if items.is_empty() {
        return Err(crate::core::errors::error::Error::from("请选择要上架的产品"));
    }
    let pids: Vec<i64> = items.iter().map(|it| it.product_id).collect();
    // 校验产品存在
    let products = product::Entity::find()
        .filter(product::Column::Id.is_in(pids.clone()))
        .filter(product::Column::Deleted.eq(0))
        .all(db)
        .await?;
    if products.len() != pids.len() {
        return Err(crate::core::errors::error::Error::from("部分产品不存在或已删除"));
    }
    let stock_map = stock_base_map(db, &pids).await?;

    // 已有行（含软删除）：在清单内的跳过；软删除的复活（否则唯一索引冲突）
    let existing_rows = website_product::Entity::find()
        .filter(website_product::Column::WebsiteId.eq(website_id))
        .filter(website_product::Column::ProductId.is_in(pids.clone()))
        .all(db)
        .await?;
    let existing_map: HashMap<i64, website_product::Model> = existing_rows
        .into_iter()
        .map(|r| (r.product_id, r))
        .collect();

    // 追加排序：接在当前清单末尾
    let max_sort = website_product::Entity::find()
        .filter(website_product::Column::WebsiteId.eq(website_id))
        .filter(website_product::Column::Deleted.eq(0))
        .all(db)
        .await?
        .iter()
        .map(|r| r.sort.unwrap_or(0))
        .max()
        .unwrap_or(0);

    let now = chrono::Local::now().naive_local();
    let mut added = 0i64;
    for (idx, item) in items.iter().enumerate() {
        let pid = item.product_id;
        // 防超卖：展示数量钳制为当前仓储库存
        let quantity = stock_map.get(&pid).copied()
            .map(|s| std::cmp::min(std::cmp::max(item.quantity, 0), s))
            .unwrap_or(0);
        if let Some(row) = existing_map.get(&pid) {
            if row.deleted.unwrap_or(0) == 0 {
                continue; // 已在清单内（含下架保留行）
            }
            // 复活软删除行
            website_product::Entity::update_many()
                .col_expr(website_product::Column::Deleted, Expr::value(0))
                .col_expr(website_product::Column::Status, Expr::value(1))
                .col_expr(
                    website_product::Column::Quantity,
                    Expr::value(quantity),
                )
                .col_expr(
                    website_product::Column::SkuIds,
                    Expr::value(format_sku_ids(&item.sku_ids)),
                )
                .col_expr(
                    website_product::Column::Sort,
                    Expr::value(max_sort + idx as i32 + 1),
                )
                .col_expr(website_product::Column::UpdateTime, Expr::value(now))
                .filter(website_product::Column::Id.eq(row.id))
                .exec(db)
                .await?;
            added += 1;
            continue;
        }
        let payload = website_product::ActiveModel {
            website_id: Set(website_id),
            product_id: Set(pid),
            status: Set(Some(1)),
            quantity: Set(Some(quantity as i32)),
            sku_ids: Set(Some(format_sku_ids(&item.sku_ids))),
            sort: Set(Some(max_sort + idx as i32 + 1)),
            remark: Set(None),
            deleted: Set(Some(0)),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            ..Default::default()
        };
        website_product::Entity::insert(payload).exec(db).await?;
        added += 1;
    }
    Ok(added)
}

/// 批量上架/下架
pub async fn shelf(db: &DatabaseConnection, req: &WebsiteProductShelfRequest) -> Result<()> {
    let ids: Vec<i64> = req.ids.clone();
    if ids.is_empty() {
        return Err(crate::core::errors::error::Error::from("请选择要操作的清单行"));
    }
    if req.status != 0 && req.status != 1 {
        return Err(crate::core::errors::error::Error::from("状态非法，仅支持 1=上架 0=下架"));
    }
    let now = chrono::Local::now().naive_local();
    website_product::Entity::update_many()
        .col_expr(website_product::Column::Status, Expr::value(req.status))
        .col_expr(website_product::Column::UpdateTime, Expr::value(now))
        .filter(website_product::Column::Id.is_in(ids))
        .filter(website_product::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(())
}

/// 调整排序
pub async fn update_sort(db: &DatabaseConnection, id: i64, sort: i64) -> Result<()> {
    let now = chrono::Local::now().naive_local();
    website_product::Entity::update_many()
        .col_expr(website_product::Column::Sort, Expr::value(sort as i32))
        .col_expr(website_product::Column::UpdateTime, Expr::value(now))
        .filter(website_product::Column::Id.eq(id))
        .filter(website_product::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(())
}

/// 调整展示数量（防超卖：钳制为不超过当前仓储库存）
pub async fn update_quantity(
    db: &DatabaseConnection,
    id: i64,
    quantity: i64,
) -> Result<i64> {
    let row = website_product::Entity::find()
        .filter(website_product::Column::Id.eq(id))
        .filter(website_product::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| crate::core::errors::error::Error::from("清单行不存在"))?;
    let stock_map = stock_base_map(db, &[row.product_id]).await?;
    let stock = stock_map.get(&row.product_id).copied().unwrap_or(0);
    if quantity < 0 {
        return Err(crate::core::errors::error::Error::from("展示数量不能为负数"));
    }
    let effective = std::cmp::min(quantity, stock);
    let now = chrono::Local::now().naive_local();
    website_product::Entity::update_many()
        .col_expr(website_product::Column::Quantity, Expr::value(effective as i32))
        .col_expr(website_product::Column::UpdateTime, Expr::value(now))
        .filter(website_product::Column::Id.eq(id))
        .exec(db)
        .await?;
    Ok(effective)
}

/// 归属栏目分类（栏目管理中内容类型=产品的栏目；0=清除归属）
pub async fn update_category(
    db: &DatabaseConnection,
    id: i64,
    category_id: i64,
) -> Result<()> {
    if category_id > 0 {
        let cat = article_category::Entity::find()
            .filter(article_category::Column::Id.eq(category_id))
            .filter(article_category::Column::Status.eq(1))
            .one(db)
            .await?
            .ok_or_else(|| crate::core::errors::error::Error::from("栏目分类不存在"))?;
        if cat.content_type != Some(ARTICLE_CONTENT_TYPE_PRODUCT) {
            return Err(crate::core::errors::error::Error::from(
                "只能归属内容类型为「产品」的栏目分类",
            ));
        }
    }
    let now = chrono::Local::now().naive_local();
    website_product::Entity::update_many()
        .col_expr(
            website_product::Column::CategoryId,
            Expr::value(if category_id > 0 { Some(category_id) } else { None }),
        )
        .col_expr(website_product::Column::UpdateTime, Expr::value(now))
        .filter(website_product::Column::Id.eq(id))
        .filter(website_product::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(())
}

/// 批量设置/取消推荐
pub async fn update_recommend(db: &DatabaseConnection, ids: &[i64], is_recommend: i32) -> Result<()> {
    if ids.is_empty() {
        return Err(crate::core::errors::error::Error::from("请选择要操作的清单行"));
    }
    if is_recommend != 0 && is_recommend != 1 {
        return Err(crate::core::errors::error::Error::from("推荐状态非法"));
    }
    let now = chrono::Local::now().naive_local();
    website_product::Entity::update_many()
        .col_expr(website_product::Column::IsRecommend, Expr::value(is_recommend))
        .col_expr(website_product::Column::UpdateTime, Expr::value(now))
        .filter(website_product::Column::Id.is_in(ids.to_vec()))
        .filter(website_product::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(())
}

/// 保存 SEO 设置（仅作用于前台详情页 head，不影响产品库）
pub async fn update_seo(
    db: &DatabaseConnection,
    id: i64,
    seo_title: Option<String>,
    seo_keywords: Option<String>,
    seo_description: Option<String>,
) -> Result<()> {
    let now = chrono::Local::now().naive_local();
    website_product::Entity::update_many()
        .col_expr(website_product::Column::SeoTitle, Expr::value(seo_title))
        .col_expr(website_product::Column::SeoKeywords, Expr::value(seo_keywords))
        .col_expr(website_product::Column::SeoDescription, Expr::value(seo_description))
        .col_expr(website_product::Column::UpdateTime, Expr::value(now))
        .filter(website_product::Column::Id.eq(id))
        .filter(website_product::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(())
}

/// 设置每人限购数量（0=不限）
pub async fn update_limit_buy(db: &DatabaseConnection, id: i64, limit_buy: i64) -> Result<()> {
    if limit_buy < 0 {
        return Err(crate::core::errors::error::Error::from("限购数量不能为负数"));
    }
    let now = chrono::Local::now().naive_local();
    website_product::Entity::update_many()
        .col_expr(website_product::Column::LimitBuy, Expr::value(limit_buy as i32))
        .col_expr(website_product::Column::UpdateTime, Expr::value(now))
        .filter(website_product::Column::Id.eq(id))
        .filter(website_product::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(())
}

/// 设置定时上架/下架时间（None=清除）
pub async fn update_schedule(
    db: &DatabaseConnection,
    id: i64,
    list_at: Option<chrono::NaiveDateTime>,
    unlist_at: Option<chrono::NaiveDateTime>,
) -> Result<()> {
    let now = chrono::Local::now().naive_local();
    website_product::Entity::update_many()
        .col_expr(website_product::Column::ListAt, Expr::value(list_at))
        .col_expr(website_product::Column::UnlistAt, Expr::value(unlist_at))
        .col_expr(website_product::Column::UpdateTime, Expr::value(now))
        .filter(website_product::Column::Id.eq(id))
        .filter(website_product::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(())
}

/// 保存相关产品/相关文章（逗号分隔存储；空 = 清除）
pub async fn update_related(
    db: &DatabaseConnection,
    id: i64,
    related_product_ids: &[i64],
    related_article_ids: &[i64],
) -> Result<()> {
    let row = website_product::Entity::find()
        .filter(website_product::Column::Id.eq(id))
        .filter(website_product::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| crate::core::errors::error::Error::from("清单行不存在"))?;
    let now = chrono::Local::now().naive_local();
    website_product::Entity::update_many()
        .col_expr(
            website_product::Column::RelatedProductIds,
            Expr::value(related_product_ids.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(",")),
        )
        .col_expr(
            website_product::Column::RelatedArticleIds,
            Expr::value(related_article_ids.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(",")),
        )
        .col_expr(website_product::Column::UpdateTime, Expr::value(now))
        .filter(website_product::Column::Id.eq(id))
        .exec(db)
        .await?;
    Ok(())
}

/// 相关文章列表（按清单行存的文章 ID，仅返回已发布文章）
pub async fn related_articles(
    db: &DatabaseConnection,
    id: i64,
) -> Result<Vec<crate::modules::articles::entity::article::Model>> {
    let row = website_product::Entity::find()
        .filter(website_product::Column::Id.eq(id))
        .filter(website_product::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| crate::core::errors::error::Error::from("清单行不存在"))?;
    let aids = parse_id_list(&row.related_article_ids);
    if aids.is_empty() {
        return Ok(Vec::new());
    }
    Ok(article_entity::Entity::find()
        .filter(article_entity::Column::Id.is_in(aids))
        .filter(article_entity::Column::Status.eq(1))
        .all(db)
        .await?)
}

/// 查询产品对应的清单行（供控制器读取 SEO/相关关系等扩展字段）
pub async fn shelf_row_for(
    db: &DatabaseConnection,
    website_id: i64,
    product_id: i64,
) -> Result<Option<website_product::Model>> {
    website_product::Entity::find()
        .filter(website_product::Column::WebsiteId.eq(website_id))
        .filter(website_product::Column::ProductId.eq(product_id))
        .filter(website_product::Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| crate::core::errors::error::Error::from(e.to_string()))
}

/// 相关产品（清单行存的相关产品 ID ∩ 已上架产品，按设置顺序）
/// 返回 None = 未设置相关产品（前台回退同类目自动推荐）
pub async fn related_shelf_products(
    db: &DatabaseConnection,
    website_id: i64,
    product_id: i64,
) -> Result<Option<Vec<ProductListVO>>> {
    let row = website_product::Entity::find()
        .filter(website_product::Column::WebsiteId.eq(website_id))
        .filter(website_product::Column::ProductId.eq(product_id))
        .filter(website_product::Column::Deleted.eq(0))
        .one(db)
        .await?;
    let Some(row) = row else { return Ok(None) };
    let rids = parse_id_list(&row.related_product_ids);
    if rids.is_empty() {
        return Ok(None);
    }
    let models = product::Entity::find()
        .filter(product::Column::Id.is_in(rids.clone()))
        .filter(product::Column::Deleted.eq(0))
        .filter(product::Column::IsActive.eq(true))
        .all(db)
        .await?;
    let rank: HashMap<i64, usize> = rids.iter().enumerate().map(|(i, pid)| (*pid, i)).collect();
    let mut models = models;
    models.sort_by_key(|m| rank.get(&m.id).copied().unwrap_or(usize::MAX));
    Ok(Some(
        models
            .into_iter()
            .map(|m| {
                let mut vo: ProductListVO = m.into();
                vo.total_stock = None;
                vo
            })
            .collect(),
    ))
}

/// 交易校验：加购/下单数量不得超过生效展示数量（未配置清单的产品放行）
/// `user_id` 用于检查购物车累加数量（防止多次加购绕过库存限制）
pub async fn check_purchase_quantity(
    db: &DatabaseConnection,
    product_id: i64,
    want: i64,
    user_id: Option<i64>,
) -> Result<i64> {
    let website_id = default_site_id(db).await;
    let listed = listed_product_ids(db, website_id).await?;
    if listed.is_none() {
        return Ok(i64::MAX); // 未配置清单：回退口径，不限
    }
    if want <= 0 {
        return Err(crate::core::errors::error::Error::from("商品数量必须大于0"));
    }
    let listed_ids = listed.unwrap();
    if !listed_ids.contains(&product_id) {
        return Err(crate::core::errors::error::Error::from("产品未上架或已下架"));
    }
    let row = website_product::Entity::find()
        .filter(website_product::Column::WebsiteId.eq(website_id))
        .filter(website_product::Column::ProductId.eq(product_id))
        .filter(website_product::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| crate::core::errors::error::Error::from("产品未上架或已下架"))?;
    let stock = stock_base_map(db, &[product_id]).await?
        .get(&product_id).copied().unwrap_or(0);
    let effective = std::cmp::min(row.quantity.unwrap_or(0) as i64, stock);
    if want > effective {
        return Err(crate::core::errors::error::Error::from(
            format!("库存不足，当前仅可购买 {} 件", effective),
        ));
    }
    // 每人限购（0=不限）
    let limit_buy = row.limit_buy.unwrap_or(0) as i64;
    if limit_buy > 0 && want > limit_buy {
        return Err(crate::core::errors::error::Error::from(
            format!("超过每人限购数量 {} 件", limit_buy),
        ));
    }
    // 购物车累加校验：已有数量 + 本次加购 ≤ 生效展示数量（防止多次加购绕过库存限制）
    if let Some(uid) = user_id {
        let cart_items = crate::modules::website::entity::website_cart::Entity::find()
            .filter(crate::modules::website::entity::website_cart::Column::UserId.eq(uid))
            .filter(crate::modules::website::entity::website_cart::Column::ProductId.eq(product_id))
            .all(db)
            .await?;
        let cart_qty: i64 = cart_items.iter().map(|c| c.quantity as i64).sum();
        if cart_qty + want > effective {
            return Err(crate::core::errors::error::Error::from(
                format!("购物车中已有 {} 件，加购 {} 件将超过库存上限 {} 件", cart_qty, want, effective),
            ));
        }
    }
    Ok(effective)
}

/// 定时上下架扫描（调度任务）：list_at 到点上架、unlist_at 到点下架
pub async fn run_scheduled_status(db: &DatabaseConnection) -> Result<(u64, u64)> {
    let now = chrono::Local::now().naive_local();
    let up = website_product::Entity::update_many()
        .col_expr(website_product::Column::Status, Expr::value(1))
        .col_expr(website_product::Column::ListAt, Expr::value(None::<chrono::NaiveDateTime>))
        .col_expr(website_product::Column::UpdateTime, Expr::value(now))
        .filter(website_product::Column::Deleted.eq(0))
        .filter(website_product::Column::Status.eq(0))
        .filter(website_product::Column::ListAt.is_not_null())
        .filter(website_product::Column::ListAt.lte(now))
        .exec(db)
        .await?
        .rows_affected;
    let down = website_product::Entity::update_many()
        .col_expr(website_product::Column::Status, Expr::value(0))
        .col_expr(website_product::Column::UnlistAt, Expr::value(None::<chrono::NaiveDateTime>))
        .col_expr(website_product::Column::UpdateTime, Expr::value(now))
        .filter(website_product::Column::Deleted.eq(0))
        .filter(website_product::Column::Status.eq(1))
        .filter(website_product::Column::UnlistAt.is_not_null())
        .filter(website_product::Column::UnlistAt.lte(now))
        .exec(db)
        .await?
        .rows_affected;
    Ok((up, down))
}

/// 库存预警扫描（调度任务）：生效展示数量 ≤ 阈值的清单行通知管理员
pub async fn run_stock_warn(db: &DatabaseConnection) -> Result<(usize, usize)> {
    let website_id = default_site_id(db).await;
    let rows = website_product::Entity::find()
        .filter(website_product::Column::WebsiteId.eq(website_id))
        .filter(website_product::Column::Status.eq(1))
        .filter(website_product::Column::Deleted.eq(0))
        .filter(website_product::Column::StockWarnThreshold.is_not_null())
        .all(db)
        .await?;
    if rows.is_empty() {
        return Ok((0, 0));
    }
    let pids: Vec<i64> = rows.iter().map(|r| r.product_id).collect();
    let stock_map = stock_base_map(db, &pids).await?;
    let names = product::Entity::find()
        .filter(product::Column::Id.is_in(pids.clone()))
        .all(db)
        .await?
        .into_iter()
        .map(|p| (p.id, p.name.unwrap_or_default()))
        .collect::<HashMap<i64, String>>();
    // 通知接收人：超管
    use crate::modules::system::entity::admin as admin_entity;
    let admins = admin_entity::Entity::find()
        .filter(admin_entity::Column::UserType.eq(1))
        .filter(admin_entity::Column::Deleted.eq(0))
        .all(db)
        .await?;
    let receiver_ids: Vec<i64> = admins.iter().map(|a| a.id).collect();
    let notification_service = crate::modules::message::service::notification_service::NotificationService;
    let mut warned = 0usize;
    let mut notified = 0usize;
    for r in &rows {
        let threshold = r.stock_warn_threshold.unwrap_or(0) as i64;
        let stock = stock_map.get(&r.product_id).copied().unwrap_or(0);
        let effective = std::cmp::min(r.quantity.unwrap_or(0) as i64, stock);
        if effective > threshold {
            continue;
        }
        warned += 1;
        let name = names.get(&r.product_id).cloned().unwrap_or_default();
        let content = format!(
            "产品「{}」前台生效展示数量 {} 已低于预警阈值 {}，请及时补充库存或调整上架数量",
            name, effective, threshold
        );
        for uid in &receiver_ids {
            if NotificationService::send_system_notification(
                    db,
                    *uid,
                    "网站产品库存预警".to_string(),
                    content.clone(),
                    9,
                    None,
                )
                .await
                .is_ok()
            {
                notified += 1;
            }
        }
    }
    Ok((warned, notified))
}

/// 栏目分类选项（栏目管理中内容类型=产品 且 显示中）
pub async fn shelf_category_options(
    db: &DatabaseConnection,
) -> Result<Vec<(i64, String)>> {
    let cats = article_category::Entity::find()
        .filter(article_category::Column::ContentType.eq(ARTICLE_CONTENT_TYPE_PRODUCT))
        .filter(article_category::Column::Status.eq(1))
        .filter(article_category::Column::IsShow.eq(1))
        .order_by_asc(article_category::Column::Sort)
        .all(db)
        .await?;
    Ok(cats
        .into_iter()
        .map(|c| (c.id, c.category_name.unwrap_or_default()))
        .collect())
}

/// 更新清单行展示的 SKU（空 = 全部 SKU）
pub async fn update_skus(
    db: &DatabaseConnection,
    id: i64,
    sku_ids: &[i64],
) -> Result<()> {
    let row = website_product::Entity::find()
        .filter(website_product::Column::Id.eq(id))
        .filter(website_product::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| crate::core::errors::error::Error::from("清单行不存在"))?;
    // 校验 SKU 归属该产品
    let all_skus = crate::modules::product::model::product::ProductModel::find_skus_by_product_id(db, row.product_id).await?;
    let valid: Vec<i64> = all_skus.iter().map(|s| s.id).collect();
    let keep: Vec<i64> = sku_ids.iter().filter(|s| valid.contains(s)).copied().collect();
    // 选择了全部 SKU 或未选择 → 存空（表示全部）
    let text = if keep.is_empty() || keep.len() == valid.len() {
        String::new()
    } else {
        keep.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(",")
    };
    let now = chrono::Local::now().naive_local();
    website_product::Entity::update_many()
        .col_expr(website_product::Column::SkuIds, Expr::value(text))
        .col_expr(website_product::Column::UpdateTime, Expr::value(now))
        .filter(website_product::Column::Id.eq(id))
        .exec(db)
        .await?;
    Ok(())
}

/// 产品 SKU 列表（选品抽屉展开用）
pub async fn product_skus(
    db: &DatabaseConnection,
    product_id: i64,
) -> Result<Vec<crate::modules::product::model::product::SkuVO>> {
    let skus =
        crate::modules::product::model::product::ProductModel::find_skus_by_product_id(db, product_id).await?;
    Ok(skus.into_iter().map(|s| s.into()).collect())
}

/// 保存 SKU 前台零售价（JSON 存 sku_prices；仅作用于前台在线销售，不写回产品库）
pub async fn update_sku_prices(
    db: &DatabaseConnection,
    id: i64,
    prices: &HashMap<String, f64>,
) -> Result<()> {
    let row = website_product::Entity::find()
        .filter(website_product::Column::Id.eq(id))
        .filter(website_product::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| crate::core::errors::error::Error::from("清单行不存在"))?;
    // 校验 SKU 归属该产品；负价拒绝
    let all_skus =
        crate::modules::product::model::product::ProductModel::find_skus_by_product_id(db, row.product_id).await?;
    let valid: Vec<i64> = all_skus.iter().map(|s| s.id).collect();
    let mut cleaned: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    for (sid, price) in prices {
        let Ok(sku_id) = sid.parse::<i64>() else {
            return Err(crate::core::errors::error::Error::from("SKU 格式非法"));
        };
        if !valid.contains(&sku_id) {
            return Err(crate::core::errors::error::Error::from("包含不属于该产品的 SKU"));
        }
        if *price < 0.0 {
            return Err(crate::core::errors::error::Error::from("零售价格不能为负数"));
        }
        cleaned.insert(sku_id.to_string(), serde_json::json!(price));
    }
    let text = if cleaned.is_empty() {
        String::new()
    } else {
        serde_json::Value::Object(cleaned).to_string()
    };
    let now = chrono::Local::now().naive_local();
    website_product::Entity::update_many()
        .col_expr(website_product::Column::SkuPrices, Expr::value(text))
        .col_expr(website_product::Column::UpdateTime, Expr::value(now))
        .filter(website_product::Column::Id.eq(id))
        .exec(db)
        .await?;
    Ok(())
}

/// 该产品的 SKU 前台零售价覆盖（None=未设置，沿用产品库价格）
pub async fn shelf_sku_prices(
    db: &DatabaseConnection,
    website_id: i64,
    product_id: i64,
) -> Result<Option<HashMap<i64, f64>>> {
    let row = website_product::Entity::find()
        .filter(website_product::Column::WebsiteId.eq(website_id))
        .filter(website_product::Column::ProductId.eq(product_id))
        .filter(website_product::Column::Deleted.eq(0))
        .one(db)
        .await?;
    Ok(match row {
        None => None,
        Some(r) => match r.sku_prices.as_deref().and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok()) {
            Some(v) if v.is_object() => {
                let mut map: HashMap<i64, f64> = HashMap::new();
                for (k, val) in v.as_object().unwrap() {
                    if let (Ok(sku_id), Some(price)) = (k.parse::<i64>(), val.as_f64()) {
                        map.insert(sku_id, price);
                    }
                }
                if map.is_empty() { None } else { Some(map) }
            }
            _ => None,
        },
    })
}

/// 前台该产品的 SKU 展示过滤（None=全部 SKU）
pub async fn shelf_sku_filter(
    db: &DatabaseConnection,
    website_id: i64,
    product_id: i64,
) -> Result<Option<Vec<i64>>> {
    let row = website_product::Entity::find()
        .filter(website_product::Column::WebsiteId.eq(website_id))
        .filter(website_product::Column::ProductId.eq(product_id))
        .filter(website_product::Column::Deleted.eq(0))
        .one(db)
        .await?;
    Ok(match row {
        None => None,
        Some(r) => {
            let ids = parse_sku_ids(&r.sku_ids);
            if ids.is_empty() { None } else { Some(ids) }
        }
    })
}

/// 批量移除出展示清单（软删除）
pub async fn delete(db: &DatabaseConnection, ids: &[i64]) -> Result<()> {
    if ids.is_empty() {
        return Err(crate::core::errors::error::Error::from("请选择要移除的产品"));
    }
    let ids: Vec<i64> = ids.iter().map(|v| *v as i64).collect();
    let now = chrono::Local::now().naive_local();
    website_product::Entity::update_many()
        .col_expr(website_product::Column::Deleted, Expr::value(1))
        .col_expr(website_product::Column::UpdateTime, Expr::value(now))
        .filter(website_product::Column::Id.is_in(ids))
        .exec(db)
        .await?;
    Ok(())
}

// ==================== 前台展示口径 ====================

/// 已上架产品 ID（按清单排序）。
/// 返回 None 表示站点从未配置展示清单——前台回退展示产品库全部产品（兼容历史行为）；
/// 返回 Some（可能为空）表示以后台上架清单为准（空清单即前台无产品）。
pub async fn listed_product_ids(db: &DatabaseConnection, website_id: i64) -> Result<Option<Vec<i64>>> {
    let any_row = website_product::Entity::find()
        .filter(website_product::Column::WebsiteId.eq(website_id))
        .filter(website_product::Column::Deleted.eq(0))
        .one(db)
        .await?
        .is_some();
    if !any_row {
        return Ok(None);
    }
    let rows = website_product::Entity::find()
        .filter(website_product::Column::WebsiteId.eq(website_id))
        .filter(website_product::Column::Status.eq(1))
        .filter(website_product::Column::Deleted.eq(0))
        .all(db)
        .await?;
    let mut rows: Vec<website_product::Model> = rows;
    rows.sort_by(|a, b| {
        a.sort.unwrap_or(0)
            .cmp(&b.sort.unwrap_or(0))
            .then(a.id.cmp(&b.id))
    });
    Ok(Some(rows.into_iter().map(|r| r.product_id).collect()))
}

/// 前台产品列表（上架口径）。
/// 无清单配置时回退 product_service::get_list 全量口径；有清单时按清单
/// 排序/过滤/分页。分类过滤：category_id 若为栏目管理中"产品"类型栏目
/// 则按清单行归属栏目过滤，否则按产品库分类过滤（兼容旧链接）。
/// 展示数量防超卖：模板可见库存 = min(上架数量, 当前仓储库存)。
pub async fn front_list(
    db: &DatabaseConnection,
    page_num: i64,
    page_size: i64,
    category_id: Option<i64>,
    keyword: Option<String>,
    order: Option<String>,
    min_price: Option<f64>,
    max_price: Option<f64>,
) -> Result<(Vec<ProductListVO>, i64, i64)> {
    let website_id = default_site_id(db).await;
    let listed = listed_product_ids(db, website_id).await?;
    let listed = match listed {
        None => {
            let query = ProductListQuery {
                page_num: Some(page_num),
                page_size: Some(page_size),
                keywords: keyword,
                category_id,
                warehouse_id: None,
                brand_id: None,
                is_active: Some(true),
                order,
            };
            return product_service::get_list(db, &query).await;
        }
        Some(ids) => ids,
    };
    if listed.is_empty() {
        return Ok((Vec::new(), 0, 1));
    }

    // 清单行（数量/栏目归属）
    let shelf_rows: Vec<website_product::Model> = website_product::Entity::find()
        .filter(website_product::Column::WebsiteId.eq(website_id))
        .filter(website_product::Column::Status.eq(1))
        .filter(website_product::Column::Deleted.eq(0))
        .all(db)
        .await?;
    let qty_map: HashMap<i64, i64> = shelf_rows
        .iter()
        .map(|r| (r.product_id, r.quantity.unwrap_or(0) as i64))
        .collect();

    // 栏目"产品"类型分类 ID 集合
    let shelf_cat_ids: Vec<i64> = article_category::Entity::find()
        .filter(article_category::Column::ContentType.eq(ARTICLE_CONTENT_TYPE_PRODUCT))
        .filter(article_category::Column::Status.eq(1))
        .all(db)
        .await?
        .into_iter()
        .map(|c| c.id)
        .collect();
    let by_shelf_category = category_id
        .map(|c| shelf_cat_ids.contains(&c))
        .unwrap_or(false);

    // 按关键词/分类过滤清单内产品
    let mut q = product::Entity::find()
        .filter(product::Column::Id.is_in(listed.clone()))
        .filter(product::Column::Deleted.eq(0))
        .filter(product::Column::IsActive.eq(true))
        .apply_if(keyword.as_deref().filter(|k| !k.trim().is_empty()), |q, k| {
            q.filter(product::Column::Name.contains(k.trim()))
        });
    if by_shelf_category {
        // 栏目分类过滤：仅保留归属该栏目的清单行产品
        let keep: Vec<i64> = shelf_rows
            .iter()
            .filter(|r| r.category_id == category_id)
            .map(|r| r.product_id)
            .collect();
        q = q.filter(product::Column::Id.is_in(keep));
    } else {
        // 产品库分类过滤（兼容旧链接）
        q = q.apply_if(category_id.filter(|c| *c > 0), |q, v| {
            q.filter(product::Column::CategoryId.eq(v))
        });
    }
    let models = q.all(db).await?;

    // 清单排序映射
    let sort_rank: HashMap<i64, usize> =
        listed.iter().enumerate().map(|(i, pid)| (*pid, i)).collect();
    let mut models = models;
    match order.as_deref() {
        Some("price_asc") => models.sort_by(|a, b| a.sale_price.partial_cmp(&b.sale_price).unwrap_or(std::cmp::Ordering::Equal)),
        Some("price_desc") => models.sort_by(|a, b| b.sale_price.partial_cmp(&a.sale_price).unwrap_or(std::cmp::Ordering::Equal)),
        _ => models.sort_by_key(|m| sort_rank.get(&m.id).copied().unwrap_or(usize::MAX)),
    }

    // 防超卖：模板可见数量 = min(上架数量, 当前仓储库存)（SKU 库存基准）
    let page_pids: Vec<i64> = models.iter().map(|m| m.id).collect();
    let stock_map = stock_base_map(db, &page_pids).await?;
    let now = chrono::Local::now().naive_local();
    // 组装 VO：生效数量 + 前台价格覆盖（促销价有效期内 > 前台零售价 > 产品库销售价）
    let mut priced: Vec<(ProductListVO, i64)> = models
        .into_iter()
        .map(|m| {
            let pid = m.id;
            let shelf_qty = qty_map.get(&pid).copied();
            let stock = stock_map.get(&pid).copied();
            let effective = match (shelf_qty, stock) {
                (Some(q), Some(s)) => std::cmp::min(q, s),
                (Some(q), None) => q,
                _ => 0,
            };
            let mut vo: ProductListVO = m.into();
            vo.total_stock = Some(effective);
            // 前台价格覆盖（双价格体系：清单行设置优先）
            if let Some(row) = shelf_rows.iter().find(|r| r.product_id == pid) {
                let promo_active = row.promo_price.is_some()
                    && row
                        .promo_start
                        .map(|s| s <= now)
                        .unwrap_or(true)
                    && row.promo_end.map(|e| now <= e).unwrap_or(true);
                if promo_active {
                    if let Some(pp) = row.promo_price {
                        vo.sale_price = Some(pp);
                    }
                } else if let Some(rp) = row.retail_price {
                    vo.sale_price = Some(rp);
                }
            }
            (vo, effective)
        })
        .collect();
    // 价格区间过滤（按前台生效售价）
    if min_price.is_some() || max_price.is_some() {
        use rust_decimal::prelude::ToPrimitive;
        priced.retain(|(vo, _)| {
            let Some(p) = vo.sale_price else { return true };
            let pf = p.to_f64().unwrap_or(0.0);
            if let Some(minp) = min_price.filter(|v| *v > 0.0) {
                if pf < minp { return false; }
            }
            if let Some(maxp) = max_price.filter(|v| *v > 0.0) {
                if pf > maxp { return false; }
            }
            true
        });
    }
    let total = priced.len() as i64;
    let total_pages = if total == 0 { 1 } else { (total + page_size - 1) / page_size };
    let list: Vec<ProductListVO> = priced
        .into_iter()
        .skip(((std::cmp::max(page_num, 1) - 1) * page_size) as usize)
        .take(page_size as usize)
        .map(|(vo, _)| vo)
        .collect();
    Ok((list, total, total_pages))
}

/// 前台产品详情可见性（上架口径）。无清单配置时回退全部可见。
pub async fn front_product_visible(db: &DatabaseConnection, website_id: i64, product_id: i64) -> Result<bool> {
    let listed = listed_product_ids(db, website_id).await?;
    Ok(match listed {
        None => true,
        Some(ids) => ids.contains(&product_id),
    })
}
