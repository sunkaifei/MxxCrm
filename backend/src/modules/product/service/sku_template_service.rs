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
use crate::modules::product::entity::{sku_template_spec, sku_template_spec_value};
use crate::modules::product::entity::sku_template_spec::Entity as TemplateSpec;
use crate::modules::product::entity::sku_template_spec_value::Entity as TemplateSpecValue;
use crate::modules::product::model::sku_template::*;
use sea_orm::*;
use sea_orm::prelude::*;

/// 获取模板的规格定义（含规格值）
pub async fn get_template_specs(db: &DbConn, template_id: i64) -> Result<Vec<SpecVO>> {
    let specs = TemplateSpec::find()
        .filter(sku_template_spec::Column::TemplateId.eq(template_id))
        .order_by_asc(sku_template_spec::Column::SortOrder)
        .order_by_asc(sku_template_spec::Column::Id)
        .all(db)
        .await?;

    let spec_ids: Vec<i64> = specs.iter().map(|s| s.id).collect();
    let values = if spec_ids.is_empty() {
        vec![]
    } else {
        TemplateSpecValue::find()
            .filter(sku_template_spec_value::Column::SpecId.is_in(spec_ids))
            .order_by_asc(sku_template_spec_value::Column::SortOrder)
            .order_by_asc(sku_template_spec_value::Column::Id)
            .all(db)
            .await?
    };

    let mut values_map: std::collections::BTreeMap<i64, Vec<sku_template_spec_value::Model>> =
        std::collections::BTreeMap::new();
    for v in &values {
        values_map.entry(v.spec_id).or_default().push(v.clone());
    }

    let spec_vos: Vec<SpecVO> = specs
        .iter()
        .map(|s| {
            let vals = values_map.remove(&s.id).unwrap_or_default();
            SpecVO {
                id: Some(s.id),
                name: Some(s.name.clone()),
                sort_order: s.sort_order,
                values: vals
                    .into_iter()
                    .map(|v| SpecValueVO {
                        id: Some(v.id),
                        value: Some(v.value),
                        sort_order: v.sort_order,
                    })
                    .collect(),
            }
        })
        .collect();

    Ok(spec_vos)
}

/// 获取模板详情（含规格）
pub async fn get_template_detail(db: &DbConn, template_id: i64) -> Result<SkuTemplateDetailVO> {
    let tmpl = SkuTemplateModel::find_by_id(db, template_id)
        .await?
        .ok_or_else(|| crate::core::errors::error::Error::NotFound("模板不存在".to_string()))?;

    let specs = get_template_specs(db, template_id).await?;

    Ok(SkuTemplateDetailVO {
        id: Some(tmpl.id),
        template_name: Some(tmpl.template_name),
        product_type: tmpl.product_type,
        description: tmpl.description,
        created_by: tmpl.created_by,
        create_time: tmpl.create_time,
        specs,
    })
}

/// 批量保存模板规格（先删后插）
pub async fn save_template_specs(
    db: &DbConn,
    form_data: &TemplateSpecBatchSaveRequest,
) -> Result<()> {
    let template_id = form_data.template_id;

    // 1. 删除旧规格值
    let old_specs = TemplateSpec::find()
        .filter(sku_template_spec::Column::TemplateId.eq(template_id))
        .all(db)
        .await?;
    let old_spec_ids: Vec<i64> = old_specs.iter().map(|s| s.id).collect();
    if !old_spec_ids.is_empty() {
        TemplateSpecValue::delete_many()
            .filter(sku_template_spec_value::Column::SpecId.is_in(old_spec_ids))
            .exec(db)
            .await?;
    }

    // 2. 删除旧规格
    TemplateSpec::delete_many()
        .filter(sku_template_spec::Column::TemplateId.eq(template_id))
        .exec(db)
        .await?;

    // 3. 插入新的规格和规格值
    let now = chrono::Local::now().naive_local().to_owned();
    for spec_item in &form_data.specs {
        let spec_am = sku_template_spec::ActiveModel {
            template_id: Set(template_id),
            name: Set(spec_item.name.clone()),
            sort_order: Set(spec_item.sort_order),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            ..Default::default()
        };

        let insert_result = TemplateSpec::insert(spec_am).exec(db).await?;
        let new_spec_id = insert_result.last_insert_id;

        for val_item in &spec_item.values {
            let val_am = sku_template_spec_value::ActiveModel {
                spec_id: Set(new_spec_id),
                value: Set(val_item.value.clone()),
                sort_order: Set(val_item.sort_order),
                create_time: Set(Some(now)),
                update_time: Set(Some(now)),
                ..Default::default()
            };
            TemplateSpecValue::insert(val_am).exec(db).await?;
        }
    }

    Ok(())
}

/// 分页查询SKU模板（含总数）
pub async fn list_templates(
    db: &DbConn,
    page: i64,
    page_size: i64,
    keywords: Option<String>,
    product_type: Option<String>,
) -> Result<(Vec<SkuTemplateListVO>, i64)> {
    let (items, _) = SkuTemplateModel::select_in_page(db, page, page_size, keywords, product_type).await?;
    let total = items.len() as i64;


    // 查询每个模板的规格数
    let template_ids: Vec<i64> = items.iter().map(|t| t.id).collect();
    let mut spec_counts: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
    if !template_ids.is_empty() {
        let all_specs = TemplateSpec::find()
            .filter(sku_template_spec::Column::TemplateId.is_in(template_ids))
            .all(db)
            .await?;
        for spec in &all_specs {
            *spec_counts.entry(spec.template_id).or_default() += 1;
        }
    }

    let vos: Vec<SkuTemplateListVO> = items
        .into_iter()
        .map(|item| {
            let count = spec_counts.get(&item.id).copied().unwrap_or(0);
            SkuTemplateListVO {
                id: Some(item.id),
                template_name: Some(item.template_name),
                product_type: item.product_type,
                description: item.description,
                spec_count: count,
                create_time: item.create_time,
            }
        })
        .collect();

    Ok((vos, total))
}

/// 获取模板规格数量
pub async fn count_template_specs(db: &DbConn, template_id: i64) -> Result<i64> {
    let count = TemplateSpec::find()
        .filter(sku_template_spec::Column::TemplateId.eq(template_id))
        .count(db)
        .await?;
    Ok(count as i64)
}

/// 创建SKU模板
pub async fn insert_template(db: &DbConn, req: &SkuTemplateSaveRequest) -> Result<i64> {
    Ok(SkuTemplateModel::insert(db, req).await?)
}

/// 更新SKU模板
pub async fn update_template(db: &DbConn, id: i64, req: &SkuTemplateUpdateRequest) -> Result<i64> {
    Ok(SkuTemplateModel::update_by_id(db, id, req).await?)
}

/// 删除SKU模板
pub async fn delete_template(db: &DbConn, id: i64) -> Result<i64> {
    Ok(SkuTemplateModel::delete_by_id(db, id).await?)
}
