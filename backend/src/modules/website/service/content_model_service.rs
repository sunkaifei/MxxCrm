//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::{ConnectionTrait, DbConn, EntityTrait, QueryFilter, ColumnTrait, Set};
use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::website::model::content_model::{ListQuery, PageWhere, ContentModelDetailVO, ContentModelListVO, ContentModelModel, ContentModelSaveDTO};
use crate::modules::website::entity::content_model_field;
use crate::modules::website::service::dynamic_table_service::{is_valid_identifier, DynamicTableService};
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;

/// 按钮（增删改查）权限定义：每个模型菜单下自动生成
const MODEL_MENU_BUTTONS: &[(&str, &str)] = &[
    ("查询", "content:data:list"),
    ("详情", "content:data:view"),
    ("新增", "content:data:add"),
    ("编辑", "content:data:update"),
    ("删除", "content:data:delete"),
];

/// 添加模型后自动创建菜单+按钮权限+授权（对齐帝国CMS/WordPress：模型即菜单）
///
/// 生成结构：MENU 行（网站 345 下）+ BUTTON×5 子节点（查询/详情/新增/编辑/删除），
/// 全部自动授权给已拥有 content:model:list 的角色。
async fn auto_create_model_menu(db: &DbConn, model_id: i64, model_code: &str, model_name: &str) -> Result<()> {
    use crate::modules::system::entity::menu;
    use sea_orm::QueryOrder;

    // 找排序号（挂「网站」根菜单 345 下，与栏目/文章等平级；不挂「模型管理」740 下——
    // 子菜单路径含 ?code= 会破坏父菜单 /website/content-model 的路由匹配）
    let max_sort = menu::Entity::find()
        .filter(menu::Column::ParentId.eq(345))
        .all(db)
        .await?
        .iter()
        .map(|m| m.sort.unwrap_or(0))
        .max()
        .unwrap_or(0);

    let now = chrono::Local::now().naive_local();
    let payload = menu::ActiveModel {
        parent_id: Set(345),
        name: Set(Some(model_name.to_string())),
        perm: Set(Some("content:data:list".to_string())),
        r#type: Set(Some("MENU".to_string())),
        // 纯路径参数形式（vue-router 路由 path 不能带 ?query）；页面读取 params.code 兜底 query.code
        path: Set(Some(format!("/website/content-data/{}", model_code))),
        route_name: Set(Some(format!("ContentData_{}", model_code))),
        component: Set(Some("views/website/content-data/index.vue".to_string())),
        sort: Set(Some(max_sort + 1)),
        icon: Set(Some("lucide:file-text".to_string())),
        status: Set(1),
        deleted: Set(Some(0)),
        keep_alive: Set(0),
        hide_in_menu: Set(0),
        create_time: Set(Some(now)),
        update_time: Set(Some(now)),
        ..Default::default()
    };
    let inserted = menu::Entity::insert(payload).exec(db).await?;
    let menu_id = inserted.last_insert_id;

    // 增删改查等按钮子节点：权限管理树里按模型粒度勾选
    let mut grant_menu_ids: Vec<i64> = vec![menu_id];
    for (idx, (btn_name, perm)) in MODEL_MENU_BUTTONS.iter().enumerate() {
        let btn = menu::ActiveModel {
            parent_id: Set(menu_id),
            name: Set(Some(format!("{}{}", model_name, btn_name))),
            perm: Set(Some(perm.to_string())),
            r#type: Set(Some("BUTTON".to_string())),
            sort: Set(Some(100 + idx as i32)),
            status: Set(1),
            deleted: Set(Some(0)),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            ..Default::default()
        };
        let res = menu::Entity::insert(btn).exec(db).await?;
        grant_menu_ids.push(res.last_insert_id);
    }

    // 自动授权：模型菜单行 + 全部按钮行，授权给已拥有 content:model:list 的角色
    //（父链 345 已通，按钮权限码生效）
    for target_menu_id in grant_menu_ids {
        let raw_sql = format!(
            "INSERT INTO mxx_system_role_menu_merge (role_id, menu_id, status, create_time, update_time) \
             SELECT rm.role_id, {}, 0, now(), now() \
             FROM mxx_system_role_menu_merge rm \
             JOIN mxx_system_menu m ON m.id = rm.menu_id \
             WHERE m.perm = 'content:model:list' \
             AND NOT EXISTS (SELECT 1 FROM mxx_system_role_menu_merge r2 WHERE r2.role_id = rm.role_id AND r2.menu_id = {})",
            target_menu_id, target_menu_id
        );
        db.execute_unprepared(&raw_sql).await?;
    }
    Ok(())
}

/// 删除模型时联动清理菜单
async fn auto_delete_model_menu(db: &DbConn, model_code: &str) -> Result<()> {
    use crate::modules::system::entity::menu;
    let now = chrono::Local::now().naive_local();
    menu::Entity::update_many()
        .col_expr(menu::Column::Deleted, sea_orm::sea_query::Expr::value(1))
        .col_expr(menu::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
        .filter(menu::Column::Path.ends_with(&format!("code={}", model_code)))
        .filter(menu::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(())
}

/// 模型状态切换时联动菜单显隐
async fn auto_sync_model_menu_status(db: &DbConn, model_code: &str, status: i32) -> Result<()> {
    use crate::modules::system::entity::menu;
    let now = chrono::Local::now().naive_local();
    menu::Entity::update_many()
        .col_expr(menu::Column::Status, sea_orm::sea_query::Expr::value(status))
        .col_expr(menu::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
        .filter(menu::Column::Path.ends_with(&format!("code={}", model_code)))
        .filter(menu::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn insert(db: &DbConn, form_data: &ContentModelSaveDTO) -> Result<i64> {
    // 编码前置校验：建物理表依赖合法编码（表名 mxx_model_{code}），非法直接报错
    let model_code = form_data.model_code.as_ref().ok_or_else(|| Error::from("模型编码不能为空"))?;
    if !is_valid_identifier(model_code) {
        return Err(Error::from(
            "模型编码不合法：仅允许英文字母、数字、下划线，且以字母开头（建议全小写）",
        ));
    }
    // 先建物理表再写模型记录：建表失败（如表已存在冲突）直接报错返回，
    // 不产生「模型有了、表没建」的半成品（此前失败仅记日志被吞，用户误以为表已建）
    DynamicTableService::create_table(&db, model_code, &[]).await?;

    let result = ContentModelModel::insert(&db, form_data).await?;
    if result > 0 {
        // 新建模型通常无字段定义；后续「加字段」由 content_model_field_service 负责 ALTER 加列
        let field_rows = content_model_field::Entity::find()
            .filter(content_model_field::Column::ModelId.eq(result))
            .filter(content_model_field::Column::Deleted.eq(0))
            .filter(
                sea_orm::Condition::any()
                    .add(content_model_field::Column::Status.eq(1))
                    .add(content_model_field::Column::Status.is_null()),
            )
            .all(db)
            .await
            .unwrap_or_default();
        let fields: Vec<(String, i32, bool)> = field_rows
            .iter()
            .filter_map(|f| {
                let name = f.field_name.clone()?;
                Some((name, f.field_type.unwrap_or(1), f.is_required.unwrap_or(0) == 1))
            })
            .collect();

        // 复制/导入场景可能带字段定义：逐列补齐（幂等）
        for (name, ftype, _req) in &fields {
            if let Err(e) =
                DynamicTableService::add_column_if_not_exists(&db, model_code, name, *ftype).await
            {
                log::warn!("[content_model] 补列失败: {:?}", e);
            }
        }

        // 自动创建菜单+按钮权限+授权（模型即菜单，对齐帝国CMS/WordPress）
        let model_name = form_data.model_name.as_deref().unwrap_or(model_code);
        if let Err(e) = auto_create_model_menu(&db, result, model_code, model_name).await {
            log::warn!("自动创建模型菜单失败: {:?}", e);
        }
    }

    Ok(result)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<Option<String>>) -> Result<i64> {
    if ids_vec.is_empty() { return Ok(0); }
    let ids = convert_vec_option_string_to_vec_u64(ids_vec.clone());

    // 先查询模型编码，用于删除动态表
    for id in &ids {
        if let Ok(Some(model)) = ContentModelModel::find_by_id(&db, &Some(*id)).await {
            if let Some(code) = &model.model_code {
                // 系统内置模型不允许删除动态表
                if model.is_system.unwrap_or(0) == 0 {
                    let _ = DynamicTableService::drop_table(&db, code).await;
                }
                // 联动清理菜单
                if let Err(e) = auto_delete_model_menu(&db, code).await {
                    log::warn!("联动清理模型菜单失败: {:?}", e);
                }
            }
        }
    }

    let result = ContentModelModel::batch_delete_by_ids(db, ids).await?;
    Ok(result)
}

pub async fn update_by_id(db: &DbConn, form_data: &ContentModelSaveDTO) -> Result<i64> {
    // 读取旧状态
    let old_model = ContentModelModel::find_by_id(db, &form_data.id).await?;
    let old_status = old_model.as_ref().and_then(|m| m.status).unwrap_or(1);
    let model_code = old_model.as_ref().and_then(|m| m.model_code.clone());

    let result = ContentModelModel::update_by_id(&db, &form_data.id, form_data).await?;

    // 状态变化时联动菜单显隐
    let new_status = form_data.status.unwrap_or(old_status);
    if new_status != old_status {
        if let Some(code) = &model_code {
            if let Err(e) = auto_sync_model_menu_status(db, code, new_status).await {
                log::warn!("联动模型菜单状态失败: {:?}", e);
            }
        }
    }
    Ok(result)
}

/// 复制模型（12-E）：复制模型定义 + 全部字段定义；新编码 = 原编码_copy（冲突自增后缀）；
/// 不复制动态表内容数据（新表按字段定义重建）
pub async fn copy_model(db: &DbConn, id: i64) -> Result<i64> {
    use crate::modules::website::entity::content_model;
    use sea_orm::QueryOrder;

    let source = ContentModelModel::find_by_id(db, &Some(id)).await?
        .ok_or_else(|| Error::from(format!("内容模型不存在，id={}", id)))?;
    let old_code = source.model_code.clone().unwrap_or_default();
    if old_code.is_empty() {
        return Err(Error::from("源模型编码为空，无法复制"));
    }
    // 生成唯一编码：{code}_copy，冲突则 _copy2/_copy3...
    let mut new_code = format!("{}_copy", old_code);
    let mut seq = 2u32;
    while ContentModelModel::find_by_code(db, &Some(new_code.clone())).await?.is_some() {
        new_code = format!("{}_copy{}", old_code, seq);
        seq += 1;
    }
    let new_name = format!("{}副本", source.model_name.clone().unwrap_or_else(|| old_code.clone()));

    // 复制模型定义（副本一定是自定义模型）
    let now = chrono::Local::now().naive_local();
    let payload = content_model::ActiveModel {
        model_code: Set(Some(new_code.clone())),
        model_name: Set(Some(new_name.clone())),
        model_icon: Set(source.model_icon.clone()),
        description: Set(source.description.clone()),
        has_title: Set(source.has_title),
        has_content: Set(source.has_content),
        has_cover: Set(source.has_cover),
        has_author: Set(source.has_author),
        has_summary: Set(source.has_summary),
        has_seo: Set(source.has_seo),
        has_images: Set(source.has_images),
        has_attachment: Set(source.has_attachment),
        list_template_id: Set(source.list_template_id),
        detail_template_id: Set(source.detail_template_id),
        sort: Set(source.sort),
        status: Set(Some(1)),
        is_system: Set(Some(0)),
        deleted: Set(Some(0)),
        create_time: Set(Some(now)),
        update_time: Set(Some(now)),
        ..Default::default()
    };
    let new_id = content_model::Entity::insert(payload).exec(db).await?.last_insert_id;

    // 复制字段定义 + 重建动态表
    let field_rows = content_model_field::Entity::find()
        .filter(content_model_field::Column::ModelId.eq(id))
        .filter(content_model_field::Column::Deleted.eq(0))
        .order_by_asc(content_model_field::Column::Id)
        .all(db)
        .await
        .unwrap_or_default();

    let mut fields: Vec<(String, i32, bool)> = Vec::new();
    for f in &field_rows {
        let Some(name) = f.field_name.clone() else { continue };
        let ftype = f.field_type.unwrap_or(1);
        fields.push((name.clone(), ftype, f.is_required.unwrap_or(0) == 1));
        let new_field = content_model_field::ActiveModel {
            model_id: Set(Some(new_id)),
            field_name: Set(Some(name)),
            field_label: Set(f.field_label.clone()),
            field_type: Set(f.field_type),
            field_options: Set(f.field_options.clone()),
            default_value: Set(f.default_value.clone()),
            placeholder: Set(f.placeholder.clone()),
            is_required: Set(f.is_required),
            is_searchable: Set(f.is_searchable),
            is_list_show: Set(f.is_list_show),
            is_detail_show: Set(f.is_detail_show),
            sort: Set(f.sort),
            status: Set(Some(1)),
            deleted: Set(Some(0)),
            create_time: Set(Some(now)),
            ..Default::default()
        };
        if let Err(e) = content_model_field::Entity::insert(new_field).exec(db).await {
            log::warn!("复制字段失败（不影响模型复制）: {:?}", e);
        }
    }
    if let Err(e) = DynamicTableService::create_table(db, &new_code, &fields).await {
        log::warn!("复制模型建表失败（不影响模型复制）: {:?}", e);
    }

    // 模型即菜单：副本也自动建菜单+授权
    auto_create_model_menu(db, new_id, &new_code, &new_name).await?;
    Ok(new_id)
}

pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<ContentModelDetailVO> {
    let result = ContentModelModel::find_by_id(&db, id).await?.ok_or_else(|| {
        Error::from(format!("{}={}", "内容模型不存在，id".to_string(), &id.unwrap_or_default()))
    })?;
    let result = ContentModelDetailVO::from(result);
    Ok(result)
}

pub async fn get_by_page(db: &DbConn, query: ListQuery) -> Result<ResultPage<Vec<ContentModelListVO>>> {
    let select_where = PageWhere {
        model_name: query.keywords,
        status: query.status,
    };
    let select_where = select_where.format();
    // SeaORM paginate 的 fetch_page 从 0 开始，前端传的 page 从 1 开始
    let page_num = std::cmp::max(query.page_num.unwrap_or(1), 1);
    let (list, _num_pages) = ContentModelModel::select_in_page(&db, page_num - 1, query.page_size.unwrap_or(10), select_where.clone()).await?;
    let list_data: Vec<ContentModelListVO> = list.into_iter().map(|item| ContentModelListVO::from(item)).collect();
    let count = ContentModelModel::select_count(db, select_where.clone()).await.unwrap_or(0);
    let page_data = ResultPage::new_simple(list_data, count);
    Ok(page_data)
}
