//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!


use std::collections::HashMap;
use minijinja::Value;
use sea_orm::*;
use serde_json::json;
use crate::core::errors::error::Result;
use crate::modules::website::entity::{
    template_var, website_banner, website_block, website_page, content_model,
    navigation, website_links, template_data, website_media,
};
use crate::modules::articles::entity::category;
use crate::modules::articles::model::article::{ArticleListVO, QueryPageRequest};
use crate::modules::articles::service::article_service;
use crate::modules::product::model::product::ProductListQuery;
use crate::modules::product::service::product_service;
use crate::modules::website::service::dynamic_table_service::DynamicTableService;

/// CMS 标签预取数据
///
/// 由于 minijinja 的 `add_function` 需要 `Fn` 闭包（同步），而数据库查询是异步的，
/// 因此采用预取数据模式：在渲染模板前先查询所有 CMS 数据放入该结构体，
/// 再通过闭包将数据暴露给模板标签函数使用。
#[derive(Default, Clone)]
pub struct CmsTagData {
    /// 模板变量（key -> value）
    pub template_vars: HashMap<String, String>,
    /// Banner 列表，按 position 分组
    pub banners: HashMap<String, Vec<Value>>,
    /// 区块内容，按 block_code 索引
    pub blocks: HashMap<String, Value>,
    /// 自定义页面，按 page_code 索引
    pub pages: HashMap<String, Value>,
    /// 内容模型列表
    pub content_models: Vec<Value>,
    /// 内容模型内容（T-P2.1）：模型编码 -> 内容行列表（每模型有界预取）
    pub model_contents: HashMap<String, Vec<Value>>,
    /// 导航列表，按 nav_type 分组（header/footer）
    pub navigations: HashMap<String, Vec<Value>>,
    /// 友情链接列表，按 link_type 分组
    pub links: HashMap<String, Vec<Value>>,
    /// 栏目列表（用于面包屑、栏目导航等），按 id 索引
    pub categories: HashMap<i64, category::Model>,
    /// 预取的最新文章列表（默认 100 条，供 get_articles 标签使用）
    pub articles: Vec<ArticleListVO>,
    /// 预取的产品列表（默认 100 条，供 get_products 标签使用）
    pub products: Vec<Value>,
    /// 当前站点模式：1=展示型 2=交易型 3=混合型
    pub site_mode: i32,
    /// 模板片段（type_id=4 的模板数据），按 name 索引
    /// 供 include_template 标签使用
    pub template_fragments: HashMap<String, String>,
    /// 媒体库（最近 200 条），按 id 索引，供 get_media / media_url 等标签使用
    pub media: HashMap<i64, Value>,
    /// 媒体列表（按 category_id 分组），供 get_media_list / get_media_gallery 标签使用
    pub media_by_category: HashMap<i64, Vec<Value>>,
    /// P-1.1（B4 修复）：产品分类树（mxx_product_category），供 get_product_categories 使用
    pub product_categories: Vec<Value>,
    /// 网站展示产品的栏目分类（栏目管理中内容类型=产品 的栏目），供 get_shelf_categories 使用
    pub shelf_categories: Vec<Value>,
    /// 推荐产品 ID（上架清单 is_recommend=1），供 get_recommend_products 使用
    pub recommend_ids: Vec<i64>,
    /// P-1.4（B6）：产品品牌（mxx_product_brand），供 get_product_brands 使用
    pub product_brands: Vec<Value>,
    /// P-0：公共页头模板原文（type_id=14 最新一条），供 page_head() 标签
    pub page_head_html: String,
    /// P-0：公共页脚模板原文（type_id=15 最新一条），供 page_foot() 标签
    pub page_foot_html: String,
}

impl CmsTagData {
    /// 从数据库预取所有 CMS 数据
    ///
    /// 每个数据分区独立加载，单个分区失败不影响其他分区，
    /// 确保即使某张表缺失或字段不匹配，核心页面仍能渲染。
    pub async fn fetch(db: &DatabaseConnection) -> Result<Self> {
        let mut data = Self::default();

        // 1. 预取模板变量（失败不影响其他数据）
        if let Ok(vars) = template_var::Entity::find()
            .filter(template_var::Column::Deleted.eq(0))
            .filter(template_var::Column::Status.eq(1))
            .all(db).await
        {
            for v in vars {
                if let (Some(k), Some(val)) = (v.var_key, v.var_value) {
                    data.template_vars.insert(k, val);
                }
            }
        }

        // 2. 预取 Banners（按 position 分组，按 sort 升序）
        if let Ok(banners) = website_banner::Entity::find()
            .filter(website_banner::Column::Deleted.eq(0))
            .filter(website_banner::Column::Status.eq(1))
            .order_by_asc(website_banner::Column::Sort)
            .all(db).await
        {
            let mut banner_map: HashMap<String, Vec<Value>> = HashMap::new();
            for b in banners {
                let banner_value = json!({
                    "id": b.id,
                    "title": b.title,
                    "imageUrl": b.image_url,
                    "linkUrl": b.link_url,
                    "altText": b.alt_text,
                    "position": b.position,
                    "target": b.target,
                    "sort": b.sort,
                });
                let pos = b.position.unwrap_or_else(|| "home_top".to_string());
                banner_map.entry(pos).or_default().push(Value::from_serialize(&banner_value));
            }
            data.banners = banner_map;
        }

        // 3. 预取区块
        if let Ok(blocks) = website_block::Entity::find()
            .filter(website_block::Column::Deleted.eq(0))
            .filter(website_block::Column::Status.eq(1))
            .all(db).await
        {
            for blk in blocks {
                if let Some(code) = &blk.block_code {
                    let block_value = json!({
                        "id": blk.id,
                        "blockCode": blk.block_code,
                        "blockName": blk.block_name,
                        "blockType": blk.block_type,
                        "content": blk.content,
                        "imageUrl": blk.image_url,
                        "linkUrl": blk.link_url,
                    });
                    data.blocks.insert(code.clone(), Value::from_serialize(&block_value));
                }
            }
        }

        // 4. 预取页面
        if let Ok(pages) = website_page::Entity::find()
            .filter(website_page::Column::Deleted.eq(0))
            .filter(website_page::Column::Status.eq(1))
            .all(db).await
        {
            for p in pages {
                if let Some(code) = &p.page_code {
                    let page_value = json!({
                        "id": p.id,
                        "pageCode": p.page_code,
                        "pageName": p.page_name,
                        "pageTitle": p.page_title,
                        "pageContent": p.page_content,
                        "seoKeywords": p.seo_keywords,
                        "seoDescription": p.seo_description,
                    });
                    data.pages.insert(code.clone(), Value::from_serialize(&page_value));
                }
            }
        }

        // 5. 预取内容模型（按 sort 升序）
        let mut model_codes: Vec<String> = Vec::new();
        if let Ok(models) = content_model::Entity::find()
            .filter(content_model::Column::Deleted.eq(0))
            .filter(content_model::Column::Status.eq(1))
            .order_by_asc(content_model::Column::Sort)
            .all(db).await
        {
            for m in models {
                let model_value = json!({
                    "id": m.id,
                    "modelCode": m.model_code,
                    "modelName": m.model_name,
                    "modelIcon": m.model_icon,
                    "description": m.description,
                });
                data.content_models.push(Value::from_serialize(&model_value));
                if let Some(code) = m.model_code {
                    model_codes.push(code);
                }
            }
        }

        // 5.1 预取内容模型内容（T-P2.1）：每个模型有界取最新 200 条，供 get_model_list/get_model_detail
        for code in &model_codes {
            if let Ok((rows, _total)) =
                DynamicTableService::paginate(db, code, 1, 200, None, None, &[], &[]).await
            {
                let items: Vec<Value> = rows.iter().map(|r| Value::from_serialize(r)).collect();
                data.model_contents.insert(code.clone(), items);
            }
        }

        // 6. 导航预取在 fetch 末尾统一处理（需依赖 categories/pages/articles，见文件末尾「导航解析与建树」），
        //    此处不再处理，避免引用式解析/建树时上游数据尚未就绪。

        // 7. 预取友情链接（按 link_type 分组，按 sort 升序）
        if let Ok(links) = website_links::Entity::find()
            .filter(website_links::Column::Deleted.eq(0))
            .filter(website_links::Column::Status.eq(1))
            .order_by_asc(website_links::Column::Sort)
            .all(db).await
        {
            let mut link_map: HashMap<String, Vec<Value>> = HashMap::new();
            for l in links {
                let link_value = json!({
                    "id": l.id,
                    "websiteId": l.website_id,
                    "linkType": l.link_type,
                    "linkName": l.link_name,
                    "linkUrl": l.link_url,
                    "linkLogo": l.link_logo,
                    "status": l.status,
                    "sort": l.sort,
                });
                let link_type = format!("{}", l.link_type.unwrap_or(0));
                link_map.entry(link_type).or_default().push(Value::from_serialize(&link_value));
            }
            data.links = link_map;
        }

        // 8. 预取栏目（用于面包屑、栏目导航等），按 id 索引
        if let Ok(cats) = category::Entity::find().all(db).await {
            for c in cats {
                data.categories.insert(c.id, c);
            }
        }

        // 9. 预取最新文章（100 条，供 get_articles 标签使用）
        // 失败不阻塞渲染，仅影响文章列表标签
        let article_query = QueryPageRequest {
            title: None,
            page_num: Some(1),
            page_size: Some(100),
            category_id: None,
            website_id: None,
            status: Some(2), // 已发布
        };
        if let Ok(article_page) = article_service::get_by_page(db, article_query).await {
            data.articles = article_page.items;
        }

        // 10. 预取产品（100 条，供 get_products 标签使用）
        let product_query = ProductListQuery {
            keywords: None,
            category_id: None,
            warehouse_id: None,
            brand_id: None,
            is_active: Some(true),
            page_num: Some(1),
            page_size: Some(100),
        order: None,
        };
        if let Ok((products, _total, _pages)) = product_service::get_list(db, &product_query).await {
            let mut vos: Vec<Value> = products.into_iter()
                .map(|p| Value::from_serialize(&p))
                .collect();
            // 产品上架口径：站点配置了展示清单时，标签/相关推荐只保留已上架产品
            if let Some(site) = crate::modules::website::model::website::SiteModel::find_default(db).await.ok().flatten() {
                let site_id = site.id;
                if let Ok(Some(ids)) = crate::modules::website::service::website_product_service::listed_product_ids(db, site_id).await {
                    let set: std::collections::HashSet<i64> = ids.into_iter().collect();
                    vos.retain(|p| value_as_i64(p, "id").map(|id| set.contains(&id)).unwrap_or(false));
                }
            }
            data.products = vos;
        }

        // 11. 预取模板片段（type_id=4，供 include_template 标签使用）
        // 失败不阻塞渲染
        if let Ok(fragments) = template_data::Entity::find()
            .filter(template_data::Column::TypeId.eq(4))
            .filter(template_data::Column::Deleted.eq(0))
            .filter(template_data::Column::Status.eq(1))
            .all(db).await
        {
            for f in fragments {
                if let Some(name) = f.name.clone() {
                    data.template_fragments.insert(name, f.temptext.unwrap_or_default());
                }
            }
        }

        // 12. 预取媒体库（最近 200 条，供 get_media / media_url 等标签使用）
        // 失败不阻塞渲染
        if let Ok(medias) = website_media::Entity::find()
            .filter(website_media::Column::Deleted.eq(0))
            .filter(website_media::Column::Status.eq(1))
            .order_by_desc(website_media::Column::Id)
            .limit(200)
            .all(db).await
        {
            let mut by_cat: HashMap<i64, Vec<Value>> = HashMap::new();
            for m in medias {
                let mid = m.id;
                let media_value = json!({
                    "id": m.id,
                    "originalName": m.original_name,
                    "storageName": m.storage_name,
                    "filePath": m.file_path,
                    "fileUrl": m.file_url,
                    "fileExt": m.file_ext,
                    "fileSize": m.file_size,
                    "fileType": m.file_type,
                    "mimeType": m.mime_type,
                    "width": m.width,
                    "height": m.height,
                    "thumbSmall": m.thumb_small,
                    "thumbMedium": m.thumb_medium,
                    "thumbLarge": m.thumb_large,
                    "altText": m.alt_text,
                    "title": m.title,
                    "caption": m.caption,
                    "categoryId": m.category_id,
                });
                let cat_id = m.category_id.unwrap_or(0);
                let media_val = Value::from_serialize(&media_value);
                by_cat.entry(cat_id).or_default().push(media_val.clone());
                data.media.insert(mid, media_val);
            }
            data.media_by_category = by_cat;
        }


        // P-1.1（B4 修复）：预取产品分类（此前列表侧栏错用文章分类 get_categories）
        if let Ok(pcs) = crate::modules::product::entity::category::Entity::find()
            .filter(crate::modules::product::entity::category::Column::Deleted.eq(0))
            .order_by_asc(crate::modules::product::entity::category::Column::SortOrder)
            .all(db).await
        {
            data.product_categories = pcs.iter()
                .map(|c| Value::from_serialize(&json!({
                    "id": c.id,
                    "parentId": c.parent_id,
                    "name": c.name,
                    "image": c.image,
                    "sort": c.sort_order,
                })))
                .collect();
        }

        // 网站展示产品的栏目分类（栏目管理中内容类型=产品 的栏目）
        if let Ok(scs) = crate::modules::articles::entity::category::Entity::find()
            .filter(crate::modules::articles::entity::category::Column::ContentType.eq(2))
            .filter(crate::modules::articles::entity::category::Column::Status.eq(1))
            .filter(crate::modules::articles::entity::category::Column::IsShow.eq(1))
            .order_by_asc(crate::modules::articles::entity::category::Column::Sort)
            .all(db).await
        {
            data.shelf_categories = scs.iter()
                .map(|c| Value::from_serialize(&json!({
                    "id": c.id,
                    "parentId": c.parent_id,
                    "name": c.category_name,
                    "sort": c.sort,
                })))
                .collect();
        }

        // 网站展示产品的推荐 ID（上架清单 is_recommend=1）
        if let Ok(rcs) = crate::modules::website::entity::website_product::Entity::find()
            .filter(crate::modules::website::entity::website_product::Column::IsRecommend.eq(1))
            .filter(crate::modules::website::entity::website_product::Column::Status.eq(1))
            .filter(crate::modules::website::entity::website_product::Column::Deleted.eq(0))
            .order_by_asc(crate::modules::website::entity::website_product::Column::Sort)
            .all(db).await
        {
            data.recommend_ids = rcs.iter().filter_map(|r| Some(r.product_id)).collect();
        }

        // P-0：预取公共页头/页脚模板原文（type_id=14/15）
        if let Ok(h) = crate::modules::website::service::template_user_data_service::find_latest_by_template_and_type(db, &None, &Some(14)).await {
            data.page_head_html = h.temptext.unwrap_or_default();
        }
        if let Ok(f) = crate::modules::website::service::template_user_data_service::find_latest_by_template_and_type(db, &None, &Some(15)).await {
            data.page_foot_html = f.temptext.unwrap_or_default();
        }

        // P-1.4（B6）：预取产品品牌
        if let Ok(brands) = crate::modules::product::entity::brand::Entity::find()
            .filter(crate::modules::product::entity::brand::Column::Deleted.eq(0))
            .all(db).await
        {
            data.product_brands = brands.iter()
                .map(|b| Value::from_serialize(&json!({
                    "id": b.id,
                    "name": b.name,
                    "logo": b.logo,
                })))
                .collect();
        }

        // 13. 导航解析与建树（T-P1.1 引用解析 / T-P1.3 建树 / T-P2.2 icon / T-P2.5 rel）
        //     需在 categories/pages/articles 全部就绪后执行。
        if let Ok(navs) = navigation::Entity::find()
            .filter(navigation::Column::Deleted.eq(0))
            .order_by_asc(navigation::Column::Sort)
            .order_by_asc(navigation::Column::Id)
            .all(db).await
        {
            let mut by_type: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
            for n in navs {
                let data_type = n.data_type.clone().unwrap_or_else(|| "custom".to_string());
                // 引用式绑定：data_type + value → 真实 URL；custom 直接用 web_url
                let resolved = data.resolve_nav_url(&data_type, n.value, &n.web_url);
                // target 收敛（T-P2.6）：优先实体 target 列，其次由 is_new_window_open 派生
                let target = n.target.clone()
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or_else(|| {
                        if n.is_new_window_open.unwrap_or(0) == 1 { "_blank".to_string() } else { "_self".to_string() }
                    });
                let node = json!({
                    "id": n.id,
                    "websiteId": n.website_id,
                    "parentId": n.parent_id.unwrap_or(0),
                    "name": n.name,
                    "webUrl": resolved,
                    "value": n.value,
                    "dataType": data_type,
                    "navType": n.nav_type,
                    "sort": n.sort,
                    "isShow": n.is_show,
                    "isNewWindowOpen": n.is_new_window_open,
                    "target": target,
                    "icon": n.icon,
                    "rel": n.rel,
                    "visibleGuest": n.visible_guest,
                    "visibleDevices": n.visible_devices,
                    "children": Vec::<serde_json::Value>::new(),
                });
                let nav_type = n.nav_type.clone().unwrap_or_else(|| "header".to_string());
                by_type.entry(nav_type).or_default().push(node);
            }
            let mut nav_map: HashMap<String, Vec<Value>> = HashMap::new();
            for (nt, items) in by_type {
                let tree = build_nav_tree(items, 0);
                nav_map.insert(nt, tree.into_iter().map(|v| Value::from_serialize(&v)).collect());
            }
            data.navigations = nav_map;
        }

        Ok(data)
    }

    /// 转换为 minijinja context Value
    #[allow(dead_code)]
    pub fn to_value(&self) -> Value {
        let mut ctx: HashMap<&str, Value> = HashMap::new();
        ctx.insert("tpl_vars", Value::from_serialize(&self.template_vars));
        ctx.insert("banners", Value::from_serialize(&self.banners));
        ctx.insert("blocks", Value::from_serialize(&self.blocks));
        ctx.insert("pages", Value::from_serialize(&self.pages));
        ctx.insert("content_models", Value::from_serialize(&self.content_models));
        ctx.insert("navigations", Value::from_serialize(&self.navigations));
        ctx.insert("links", Value::from_serialize(&self.links));
        Value::from_serialize(&ctx)
    }

    /// 获取模板变量值
    #[allow(dead_code)]
    pub fn get_var(&self, key: &str) -> String {
        self.template_vars.get(key).cloned().unwrap_or_default()
    }

    /// 获取指定位置的 Banners
    #[allow(dead_code)]
    pub fn get_banners(&self, position: &str) -> Vec<Value> {
        self.banners.get(position).cloned().unwrap_or_default()
    }

    /// 获取指定 code 的区块
    #[allow(dead_code)]
    pub fn get_block(&self, code: &str) -> Option<&Value> {
        self.blocks.get(code)
    }

    /// 获取指定 code 的页面
    #[allow(dead_code)]
    pub fn get_page(&self, code: &str) -> Option<&Value> {
        self.pages.get(code)
    }

    /// 获取指定类型的导航
    #[allow(dead_code)]
    pub fn get_navigations(&self, nav_type: &str) -> Vec<Value> {
        self.navigations.get(nav_type).cloned().unwrap_or_default()
    }

    /// 获取某内容模型的内容列表（T-P2.1，从预取数据分页）
    #[allow(dead_code)]
    pub fn get_model_list(&self, model_code: &str, limit: Option<usize>, page: Option<usize>) -> Vec<Value> {
        let limit = limit.unwrap_or(10).min(200);
        let page = page.unwrap_or(1).max(1);
        let offset = (page - 1) * limit;
        match self.model_contents.get(model_code) {
            Some(list) => list.iter().skip(offset).take(limit).cloned().collect(),
            None => Vec::new(),
        }
    }

    /// 获取某内容模型的单条内容（T-P2.1）
    #[allow(dead_code)]
    pub fn get_model_detail(&self, model_code: &str, id: i64) -> Option<Value> {
        self.model_contents
            .get(model_code)?
            .iter()
            .find(|v| v.get_attr("id").ok().and_then(|x| x.as_i64()) == Some(id))
            .cloned()
    }

    /// 获取指定类型的友情链接
    #[allow(dead_code)]
    pub fn get_links(&self, link_type: &str) -> Vec<Value> {
        self.links.get(link_type).cloned().unwrap_or_default()
    }

    /// 获取文章列表（从预取数据中过滤）
    ///
    /// 参数：
    /// - `category_id`: 栏目 ID，None 或 0 表示不限栏目
    /// - `limit`: 返回条数，默认 10
    /// - `page`: 页码，默认 1（基于 limit 分页）
    pub fn get_articles(&self, category_id: Option<i64>, limit: Option<usize>, page: Option<usize>) -> Vec<Value> {
        let limit = limit.unwrap_or(10).min(50);
        let page = page.unwrap_or(1).max(1);
        let offset = (page - 1) * limit;

        let filtered: Vec<&ArticleListVO> = self.articles.iter()
            .filter(|a| {
                match category_id {
                    Some(cid) if cid > 0 => a.category_id == Some(cid),
                    _ => true,
                }
            })
            .collect();

        filtered.into_iter()
            .skip(offset)
            .take(limit)
            .map(|a| Value::from_serialize(a))
            .collect()
    }

    /// 获取推荐文章（isrecommend=1）
    pub fn get_recommend_articles(&self, limit: Option<usize>) -> Vec<Value> {
        let limit = limit.unwrap_or(5).min(20);
        self.articles.iter()
            .filter(|a| a.isrecommend.unwrap_or(0) == 1)
            .take(limit)
            .map(|a| Value::from_serialize(a))
            .collect()
    }

    /// 获取产品列表（从预取数据中过滤）
    ///
    /// 参数：
    /// - `category_id`: 栏目 ID，None 或 0 表示不限栏目
    /// - `limit`: 返回条数，默认 10
    /// - `page`: 页码，默认 1
    /// - `order`: 排序方式 "new"=最新（默认）
    pub fn get_products(&self, category_id: Option<i64>, limit: Option<usize>, page: Option<usize>, _order: Option<String>) -> Vec<Value> {
        let limit = limit.unwrap_or(10).min(50);
        let page = page.unwrap_or(1).max(1);
        let offset = (page - 1) * limit;

        let filtered: Vec<&Value> = self.products.iter()
            .filter(|p| {
                match category_id {
                    Some(cid) if cid > 0 => {
                        p.get_attr("categoryId").ok()
                            .and_then(|v| v.as_i64())
                            .map(|id| id == cid)
                            .unwrap_or(false)
                    },
                    _ => true,
                }
            })
            .collect();

        let mut filtered: Vec<Value> = filtered.into_iter().cloned().collect();
        // P-1.7（B11）：order 排序——new=预取序（最新），price_asc/price_desc=价格升降
        match _order.as_deref() {
            Some("price_asc") => filtered.sort_by(|a, b| {
                let pa = value_as_f64(a, "salePrice").unwrap_or(f64::MAX);
                let pb = value_as_f64(b, "salePrice").unwrap_or(f64::MAX);
                pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal)
            }),
            Some("price_desc") => filtered.sort_by(|a, b| {
                let pa = value_as_f64(a, "salePrice").unwrap_or(f64::MIN);
                let pb = value_as_f64(b, "salePrice").unwrap_or(f64::MIN);
                pb.partial_cmp(&pa).unwrap_or(std::cmp::Ordering::Equal)
            }),
            _ => {}
        }
        filtered.into_iter()
            .skip(offset)
            .take(limit)
            .collect()
    }

    /// P-1.1（B4 修复）：产品分类（parent_id=0 取顶级；配合 category_url 标签使用）
    pub fn get_product_categories(&self, parent_id: Option<i64>) -> Vec<Value> {
        let pid = parent_id.unwrap_or(0);
        self.product_categories.iter()
            .filter(|c| {
                let cp = c.get_attr("parentId").ok()
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                if pid == 0 { cp == 0 } else { cp == pid }
            })
            .cloned()
            .collect()
    }

    /// 网站展示产品的栏目分类（栏目管理中内容类型=产品 的栏目）
    pub fn get_shelf_categories(&self, parent_id: Option<i64>) -> Vec<Value> {
        let pid = parent_id.unwrap_or(0);
        self.shelf_categories.iter()
            .filter(|c| {
                let cp = c.get_attr("parentId").ok()
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                if pid == 0 { cp == 0 } else { cp == pid }
            })
            .cloned()
            .collect()
    }

    /// 推荐产品（上架清单 is_recommend=1，按清单排序）
    pub fn get_recommend_products(&self, limit: Option<usize>) -> Vec<Value> {
        let limit = limit.unwrap_or(8).min(50);
        self.products.iter()
            .filter(|p| {
                value_as_i64(p, "id")
                    .map(|id| self.recommend_ids.contains(&id))
                    .unwrap_or(false)
            })
            .take(limit)
            .cloned()
            .collect()
    }

    /// P-1.4（B6）：产品品牌
    pub fn get_product_brands(&self) -> Vec<Value> {
        self.product_brands.clone()
    }

    /// P-1.4（B6）：相关推荐（同分类、排除自身，取 limit 条）
    pub fn get_related_products(&self, product_id: i64, limit: Option<usize>) -> Vec<Value> {
        let limit = limit.unwrap_or(4).min(20);
        let current_category = self.products.iter()
            .find(|p| value_as_i64(p, "id") == Some(product_id))
            .and_then(|p| value_as_i64(p, "categoryId"));
        self.products.iter()
            .filter(|p| {
                let pid = value_as_i64(p, "id").unwrap_or(-1);
                if pid == product_id { return false; }
                match current_category {
                    Some(cid) => value_as_i64(p, "categoryId") == Some(cid),
                    None => true,
                }
            })
            .take(limit)
            .cloned()
            .collect()
    }

    /// 获取栏目树（按 parent_id 过滤，默认返回顶级栏目）
    pub fn get_categories(&self, parent_id: Option<i64>) -> Vec<Value> {
        let pid = parent_id.unwrap_or(0);
        self.categories.values()
            .filter(|c| {
                if pid == 0 {
                    c.parent_id.unwrap_or(0) == 0
                } else {
                    c.parent_id == Some(pid)
                }
            })
            .map(|c| {
                Value::from_serialize(&json!({
                    "id": c.id,
                    "categoryName": c.category_name,
                    "shortUrl": c.short_url,
                    "parentId": c.parent_id,
                    "description": c.description,
                    "isShow": c.is_show,
                    "sort": c.sort,
                }))
            })
            .collect()
    }

    /// 根据栏目 id 构建面包屑路径（从根到当前栏目）
    /// 返回 Vec<(id, name, short_url)>，已按从根到当前的顺序排列
    /// 防止循环引用：最多向上回溯 20 层
    pub fn get_breadcrumbs(&self, category_id: i64) -> Vec<(i64, String, String)> {
        let mut chain: Vec<(i64, String, String)> = Vec::new();
        let mut current_id = Some(category_id);
        let mut depth = 0;
        let mut visited = std::collections::HashSet::new();

        while let Some(cid) = current_id {
            if depth >= 20 || visited.contains(&cid) {
                break;
            }
            visited.insert(cid);
            match self.categories.get(&cid) {
                Some(c) => {
                    let name = c.category_name.clone().unwrap_or_default();
                    let short_url = c.short_url.clone().unwrap_or_default();
                    chain.push((cid, name, short_url));
                    current_id = c.parent_id;
                    depth += 1;
                }
                None => break,
            }
        }
        chain.reverse();
        chain
    }

    /// 根据栏目 id 渲染面包屑 HTML（DEDECMS 风格）
    /// 输出形如：<a href="/">首页</a> &gt; <a href="/category/news">新闻</a> &gt; 行业动态
    pub fn render_breadcrumbs_html(&self, category_id: i64) -> String {
        let chain = self.get_breadcrumbs(category_id);
        let mut parts: Vec<String> = Vec::new();
        parts.push(r#"<a href="/" class="breadcrumb-home">首页</a>"#.to_string());
        let total = chain.len();
        for (idx, (_id, name, short_url)) in chain.iter().enumerate() {
            if name.is_empty() {
                continue;
            }
            let is_last = idx == total.saturating_sub(1);
            // HTML 转义简单处理
            let esc_name = escape_html(name);
            if is_last || short_url.is_empty() {
                parts.push(format!(r#"<span class="breadcrumb-current">{}</span>"#, esc_name));
            } else {
                let esc_url = escape_html(short_url);
                parts.push(format!(
                    r#"<a href="/category/{}" class="breadcrumb-link">{}</a>"#,
                    esc_url, esc_name
                ));
            }
        }
        parts.join(" &gt; ")
    }
}

impl CmsTagData {
    /// 引用式绑定解析器（T-P1.1 / D2 / D10）：
    /// `data_type` + `value` → 真实 URL。custom 直接用 `web_url`。
    ///
    /// | data_type | 解析目标 |
    /// |---|---|
    /// | custom | 直接用 web_url |
    /// | article_class | 文章分类 → `/category/{short_url}` |
    /// | product_class | 产品分类 → `/product?category_id={id}` |
    /// | customview | 自定义页面 → `/page/{page_code}` |
    /// | article | 文章详情 → `/article/{short_url}` |
    /// | product | 产品详情 → `/product/{id}` |
    /// | link_group | 纯分组标题（无链接，返回空串） |
    fn resolve_nav_url(&self, data_type: &str, value: Option<i64>, web_url: &Option<String>) -> String {
        let fallback = web_url.clone().unwrap_or_default();
        match data_type {
            "custom" => {
                if fallback.is_empty() { "#".to_string() } else { fallback }
            }
            "link_group" => String::new(),
            "article_class" => value
                .and_then(|id| self.categories.get(&id))
                .and_then(|c| c.short_url.clone())
                .filter(|s| !s.is_empty())
                .map(|slug| format!("/category/{}", slug))
                .unwrap_or_else(|| "#".to_string()),
            "product_class" => match value {
                Some(id) if id > 0 => format!("/product?category_id={}", id),
                _ => "#".to_string(),
            },
            "customview" => self.pages.values()
                .find(|p| p.get_attr("id").ok().and_then(|v| v.as_i64()) == value)
                .and_then(|p| p.get_attr("pageCode").ok().and_then(|v| v.as_str().map(|s| s.to_string())))
                .filter(|s| !s.is_empty())
                .map(|code| format!("/page/{}", code))
                .unwrap_or_else(|| "#".to_string()),
            "article" => self.articles.iter()
                .find(|a| {
                    a.id.as_deref().and_then(|s| s.parse::<i64>().ok()) == value
                })
                .and_then(|a| a.short_url.clone())
                .filter(|s| !s.is_empty())
                .map(|slug| format!("/article/{}", slug))
                .or_else(|| value.map(|id| format!("/article/{}", id)))
                .unwrap_or_else(|| "#".to_string()),
            "product" => value
                .filter(|id| *id > 0)
                .map(|id| format!("/product/{}", id))
                .unwrap_or_else(|| "#".to_string()),
            _ => {
                if fallback.is_empty() { "#".to_string() } else { fallback }
            }
        }
    }
}

/// 按 `parentId` 将扁平导航列表构建为树（T-P1.3）。
/// 同 `nav_type` 内组装 children；找不到父级或自身为父级的视为根；
/// 最多递归 10 层，防止脏数据造成的环引用导致无限递归。
fn build_nav_tree(items: Vec<serde_json::Value>, depth: usize) -> Vec<serde_json::Value> {
    if depth >= 10 || items.is_empty() {
        return items;
    }
    let mut id_to_idx: HashMap<i64, usize> = HashMap::new();
    for (i, it) in items.iter().enumerate() {
        if let Some(id) = it.get("id").and_then(|v| v.as_i64()) {
            id_to_idx.insert(id, i);
        }
    }
    let mut children_map: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut roots: Vec<usize> = Vec::new();
    for (i, it) in items.iter().enumerate() {
        let pid = it.get("parentId").and_then(|v| v.as_i64()).unwrap_or(0);
        match id_to_idx.get(&pid) {
            Some(&pi) if pi != i => children_map.entry(pi).or_default().push(i),
            _ => roots.push(i),
        }
    }
    let mut result: Vec<serde_json::Value> = Vec::with_capacity(roots.len());
    for r in roots {
        let mut node = items[r].clone();
        let kids: Vec<serde_json::Value> = children_map
            .get(&r)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|k| items[k].clone())
            .collect();
        if let Some(obj) = node.as_object_mut() {
            obj.insert("children".to_string(), serde_json::Value::Array(build_nav_tree(kids, depth + 1)));
        }
        result.push(node);
    }
    result
}

/// 从 minijinja Value 中取数字（兼容字符串/数字两种序列化——
/// ProductListVO 的 id/Decimal 经 serde 输出为字符串）
fn value_as_f64(v: &Value, key: &str) -> Option<f64> {
    v.get_attr(key).ok().and_then(|attr| {
        attr.as_i64().map(|i| i as f64)
            .or_else(|| attr.as_str().and_then(|s| s.parse::<f64>().ok()))
    })
}

fn value_as_i64(v: &Value, key: &str) -> Option<i64> {
    v.get_attr(key).ok().and_then(|attr| {
        attr.as_i64().or_else(|| attr.as_str().and_then(|s| s.parse::<i64>().ok()))
    })
}

/// 简单 HTML 转义（用于面包屑输出，避免 XSS）
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

// ============================================================
// 表单标签（cart_button / lead_form）
// ============================================================
//
// 这两个标签是纯 HTML 生成器，不依赖数据库查询，
// 可直接作为 minijinja 的 `Fn` 闭包注册。
// 渲染逻辑根据 `site_mode` 分流：
//   - 展示型 (1)：渲染"立即咨询"按钮 + lead_form 弹窗
//   - 交易型 (2)：渲染"加入购物车"/"立即购买"按钮
//   - 混合型 (3)：两者都渲染

/// 购物车/咨询按钮：根据站点模式渲染不同按钮
///
/// 模板调用：{{ cart_button(product_id, site_mode) }} 或 {{ cart_button(p.id, site_mode, p.sale_price, p.name) }}
///
/// P-0（B1/B9 修复，方案乙）：
/// - site_mode=1（展示型）：渲染"立即咨询"按钮 + **自动内联留言 modal**（修一次标签全站生效）
/// - site_mode=2（交易型）：渲染"加入购物车"+"立即购买"（行为由 /static/default/js/cms.js 提供）
/// - site_mode=3（混合型）：咨询 + 加购 + 立即购买
/// - 按钮统一 `cms-btn` 系列 class（样式见 /static/default/css/cms.css）
/// - 输出 data-price/data-name 属性，供 cms.js 组装购物车请求
pub fn cart_button_html(product_id: i64, site_mode: Option<i32>, price: Option<String>, name: Option<String>) -> String {
    let mode = site_mode.unwrap_or(1);
    let pid = product_id.to_string();
    // 属性转义：价格/名称进入 HTML 属性，防注入
    let price_attr = escape_html(price.as_deref().unwrap_or(""));
    let name_attr = escape_html(name.as_deref().unwrap_or(""));
    let mut html = String::new();
    html.push_str(&format!(
        r#"<div class="cms-cart-button" data-product-id="{}" data-price="{}" data-name="{}">"#,
        pid, price_attr, name_attr
    ));

    match mode {
        1 => {
            html.push_str(&format!(
                r#"<button type="button" class="cms-btn cms-btn-lead" onclick="cmsOpenLead({})">立即咨询</button>"#,
                pid
            ));
        }
        2 => {
            html.push_str(&format!(
                r#"<button type="button" class="cms-btn cms-btn-cart" onclick="cmsAddCart({})">加入购物车</button>"#,
                pid
            ));
            html.push_str(&format!(
                r#"<button type="button" class="cms-btn cms-btn-buy" onclick="cmsBuyNow({})">立即购买</button>"#,
                pid
            ));
        }
        3 => {
            html.push_str(&format!(
                r#"<button type="button" class="cms-btn cms-btn-lead" onclick="cmsOpenLead({})">立即咨询</button>"#,
                pid
            ));
            html.push_str(&format!(
                r#"<button type="button" class="cms-btn cms-btn-cart" onclick="cmsAddCart({})">加入购物车</button>"#,
                pid
            ));
            html.push_str(&format!(
                r#"<button type="button" class="cms-btn cms-btn-buy" onclick="cmsBuyNow({})">立即购买</button>"#,
                pid
            ));
        }
        _ => {
            html.push_str(&format!(
                r#"<button type="button" class="cms-btn cms-btn-lead" onclick="cmsOpenLead({})">立即咨询</button>"#,
                pid
            ));
        }
    }

    html.push_str("</div>");

    // 方案乙：展示型/混合型自动内联留言 modal（修一次标签，全站产品模板无需成对调用 lead_form）
    if mode == 1 || mode == 3 {
        html.push_str(&lead_form_html(Some(product_id)));
    }

    html
}

/// 线索/咨询表单：渲染遮罩 modal（P-0.2/P-0.3 修复）
///
/// 模板调用：{{ lead_form(product_id=p.id) }} 或 {{ lead_form() }}
/// 提交由 /static/default/js/cms.js 接管（fetch POST /api/open/leave_msg/submit，
/// 字段 contact_name/contact_phone/contact_email/content/product_id → 自动转线索）
pub fn lead_form_html(product_id: Option<i64>) -> String {
    let pid = product_id.unwrap_or(0);
    let mut html = String::new();

    // 遮罩 modal（默认隐藏；cms.js 的 cmsOpenLead/cmsCloseLead 控制显隐）
    html.push_str(&format!(
        r#"<div id="lead-form-{}" class="cms-lead-modal" style="display:none" role="dialog" aria-modal="true">"#,
        pid
    ));
    html.push_str(r#"<div class="cms-lead-card">"#);
    html.push_str(&format!(
        r#"<button type="button" class="cms-lead-close" aria-label="关闭" onclick="cmsCloseLead({})">&times;</button>"#,
        pid
    ));
    html.push_str(r#"<h3 class="cms-lead-title">在线咨询</h3>"#);
    html.push_str(&format!(
        r#"<form class="cms-lead-form" onsubmit="return cmsSubmitLead(event, {})">"#,
        pid
    ));

    // 隐藏字段：产品 ID + 来源标识
    if pid > 0 {
        html.push_str(&format!(r#"<input type="hidden" name="product_id" value="{}">"#, pid));
    }
    html.push_str(r#"<input type="hidden" name="source" value="website">"#);

    // 姓名
    html.push_str(r#"<div class="cms-form-row"><label>姓名 <span class="req">*</span></label>"#);
    html.push_str(r#"<input type="text" name="contactName" required placeholder="请输入您的姓名"></div>"#);

    // 电话
    html.push_str(r#"<div class="cms-form-row"><label>电话 <span class="req">*</span></label>"#);
    html.push_str(r#"<input type="tel" name="contactPhone" required placeholder="请输入联系电话"></div>"#);

    // 邮箱
    html.push_str(r#"<div class="cms-form-row"><label>邮箱</label>"#);
    html.push_str(r#"<input type="email" name="contactEmail" placeholder="请输入邮箱（选填）"></div>"#);

    // 留言内容
    html.push_str(r#"<div class="cms-form-row"><label>留言内容 <span class="req">*</span></label>"#);
    html.push_str(r#"<textarea name="content" rows="3" required placeholder="请输入您的需求"></textarea></div>"#);

    // 提交按钮（cms.js 防重复提交：提交中禁用按钮）
    html.push_str(r#"<div class="cms-form-row"><button type="submit" class="cms-btn cms-btn-submit">提交咨询</button>"#);
    html.push_str(&format!(
        r#"<button type="button" class="cms-btn cms-btn-cancel" onclick="cmsCloseLead({})">取消</button></div>"#,
        pid
    ));
    html.push_str(r#"<p class="cms-lead-feedback" style="display:none"></p>"#);

    html.push_str(r#"</form></div></div>"#);
    html
}
