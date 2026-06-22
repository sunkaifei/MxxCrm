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
use crate::modules::product::entity::{sku, spec, spec_value};
use crate::modules::product::entity::spec::Entity as Spec;
use crate::modules::product::entity::spec_value::Entity as SpecValue;
use crate::modules::product::entity::sku::Entity as ProductSku;
use crate::modules::product::model::spec::*;
use sea_orm::*;
use sea_orm::prelude::*;

/// 获取产品的规格定义（含规格值和SKU列表）
pub async fn get_specs(db: &DbConn, product_id: i64) -> Result<SpecGroupVO> {
    // 1. 查询规格定义
    let specs = Spec::find()
        .filter(spec::Column::ProductId.eq(product_id))
        .order_by_asc(spec::Column::SortOrder)
        .order_by_asc(spec::Column::Id)
        .all(db)
        .await?;

    // 2. 查询各规格的值
    let spec_ids: Vec<i64> = specs.iter().map(|s| s.id).collect();
    let values = if spec_ids.is_empty() {
        vec![]
    } else {
        SpecValue::find()
            .filter(spec_value::Column::SpecId.is_in(spec_ids))
            .order_by_asc(spec_value::Column::SortOrder)
            .order_by_asc(spec_value::Column::Id)
            .all(db)
            .await?
    };

    // 3. 按 spec_id 分组
    let mut values_map: std::collections::BTreeMap<i64, Vec<spec_value::Model>> =
        std::collections::BTreeMap::new();
    for v in &values {
        values_map.entry(v.spec_id).or_default().push(v.clone());
    }

    // 4. 构造 SpecVO 列表
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

    // 5. 查询已保存的SKU
    let skus = ProductSku::find()
        .filter(sku::Column::ProductId.eq(product_id))
        .filter(sku::Column::IsActive.eq(true))
        .all(db)
        .await?;

    let sku_vos = skus.into_iter().map(|s| s.into()).collect();

    Ok(SpecGroupVO {
        specs: spec_vos,
        skus: sku_vos,
    })
}

/// 批量保存规格（先删后插）
pub async fn save_specs(
    db: &DbConn,
    form_data: &SpecBatchSaveRequest,
) -> Result<()> {
    let product_id = form_data.product_id;

    // 1. 删除旧的规格（级联删除规格值）
    Spec::delete_many()
        .filter(spec::Column::ProductId.eq(product_id))
        .exec(db)
        .await?;

    // 2. 插入新的规格和规格值
    let now = chrono::Local::now().naive_local().to_owned();
    for spec_item in &form_data.specs {
        let spec_am = spec::ActiveModel {
            product_id: Set(product_id),
            name: Set(spec_item.name.clone()),
            sort_order: Set(spec_item.sort_order),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            ..Default::default()
        };

        let insert_result = Spec::insert(spec_am).exec(db).await?;
        let new_spec_id = insert_result.last_insert_id;

        for val_item in &spec_item.values {
            let val_am = spec_value::ActiveModel {
                spec_id: Set(new_spec_id),
                value: Set(val_item.value.clone()),
                sort_order: Set(val_item.sort_order),
                create_time: Set(Some(now)),
                update_time: Set(Some(now)),
                ..Default::default()
            };
            SpecValue::insert(val_am).exec(db).await?;
        }
    }

    Ok(())
}

/// 根据规格组合自动生成SKU（笛卡尔积），返回生成的SKU列表（不保存到数据库）
pub async fn generate_skus(db: &DbConn, product_id: i64) -> Result<Vec<GeneratedSkuVO>> {
    // 1. 查询所有规格及其值
    let specs = Spec::find()
        .filter(spec::Column::ProductId.eq(product_id))
        .order_by_asc(spec::Column::SortOrder)
        .order_by_asc(spec::Column::Id)
        .all(db)
        .await?;

    if specs.is_empty() {
        return Ok(vec![]);
    }

    let spec_ids: Vec<i64> = specs.iter().map(|s| s.id).collect();
    let values = SpecValue::find()
        .filter(spec_value::Column::SpecId.is_in(spec_ids))
        .order_by_asc(spec_value::Column::SortOrder)
        .order_by_asc(spec_value::Column::Id)
        .all(db)
        .await?;

    // 2. 按 spec_id 分组
    let mut values_by_spec: std::collections::BTreeMap<i64, Vec<spec_value::Model>> =
        std::collections::BTreeMap::new();
    for v in &values {
        values_by_spec.entry(v.spec_id).or_default().push(v.clone());
    }

    // 3. 构建每个规格及其值列表（保持有序）
    let mut spec_value_groups: Vec<(String, Vec<spec_value::Model>)> = Vec::new();
    for s in &specs {
        if let Some(vals) = values_by_spec.remove(&s.id) {
            if !vals.is_empty() {
                spec_value_groups.push((s.name.clone(), vals));
            }
        }
    }

    if spec_value_groups.is_empty() {
        return Ok(vec![]);
    }

    // 4. 笛卡尔积：生成所有组合
    // 使用迭代方式计算笛卡尔积
    let combinations = cartesian_product(&spec_value_groups);

    // 5. 构造返回VO
    let result: Vec<GeneratedSkuVO> = combinations
        .into_iter()
        .map(|(label, specs_json)| GeneratedSkuVO {
            label,
            specs: specs_json,
            sku_code: None,
            price: 0.0,
            stock: 0,
        })
        .collect();

    Ok(result)
}

/// 计算规格值的笛卡尔积
/// 返回 (标签, 规格JSON) 列表
fn cartesian_product(
    groups: &[(String, Vec<spec_value::Model>)],
) -> Vec<(String, serde_json::Value)> {
    if groups.is_empty() {
        return vec![];
    }

    // 每个元素是 (labels, spec_map) 的列表
    // labels: 各维度值的组合标签，如 "红色/S"
    // spec_map: 规格名到值的映射，如 {"颜色": "红色", "尺寸": "S"}
    let mut results: Vec<(Vec<String>, std::collections::BTreeMap<String, String>)> = vec![(
        vec![],
        std::collections::BTreeMap::new(),
    )];

    for (spec_name, vals) in groups {
        let mut new_results = Vec::new();
        for (labels, mut map) in results {
            for v in vals {
                let mut new_labels = labels.clone();
                new_labels.push(v.value.clone());

                let mut new_map = map.clone();
                new_map.insert(spec_name.clone(), v.value.clone());

                new_results.push((new_labels, new_map));
            }
        }
        results = new_results;
    }

    results
        .into_iter()
        .map(|(labels, map)| {
            let label = labels.join("/");
            let json = serde_json::to_value(map).unwrap_or_default();
            (label, json)
        })
        .collect()
}
