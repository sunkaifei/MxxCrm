//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::{
    ColumnTrait, ConnectionTrait, DbConn, EntityTrait, PaginatorTrait, QueryFilter, TransactionTrait,
};

use crate::core::errors::error::{Error, Result};
use crate::modules::product::entity::product;
use crate::modules::product::model::product_unit::{
    find_by_id, find_by_name, find_options, insert, soft_delete, update_decimal_places,
    ProductUnitDeleteRequest, ProductUnitOptionVO, ProductUnitSaveRequest, ProductUnitUpdateRequest,
};

/// 单位选项列表（选择器用）
pub async fn get_options(db: &DbConn) -> Result<Vec<ProductUnitOptionVO>> {
    let list = find_options(db).await.map_err(|e| Error::from(e.to_string()))?;
    Ok(list.into_iter().map(|item| item.into()).collect())
}

/// 新增单位（登录即可，选择器内联新增）
pub async fn save(db: &DbConn, req: &ProductUnitSaveRequest, created_by: i64) -> Result<i64> {
    let name = req.name.as_deref().unwrap_or("").trim().to_string();
    if name.is_empty() {
        return Err(Error::from("单位名称不能为空".to_string()));
    }
    if find_by_name(db, &name)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .is_some()
    {
        return Err(Error::from("该单位已存在".to_string()));
    }
    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;
    let id = insert(&txn, &name, req.decimal_places, created_by)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            // 数据库层幂等兜底：并发添加同名单位时命中部分唯一索引 uq_product_unit_name，
            // 转译为业务友好提示（同 approval 模块先例）
            if msg.contains("duplicate key value violates unique constraint")
                && msg.contains("uq_product_unit_name")
            {
                Error::from("该单位已存在".to_string())
            } else {
                Error::from(msg)
            }
        })?;
    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}

/// 确保单位已收录（产品保存时对自由文本单位自动收录，默认0位小数）
pub async fn ensure_unit<C: ConnectionTrait>(db: &C, name: &str, user_id: i64) -> Result<()> {
    let name = name.trim();
    if name.is_empty() {
        return Ok(());
    }
    if find_by_name(db, name)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .is_some()
    {
        return Ok(());
    }
    // 极小概率竞态：并发保存产品时同名单位刚被他人收录，insert 命中 uq_product_unit_name。
    // 注意 PostgreSQL 事务内语句报错后即进入 aborted 状态，无法在同一事务内吞掉冲突继续，
    // 故仅转译为可重试的友好提示，由产品保存整体重试（重跑 find_by_name 即命中）。
    insert(db, name, Some(0), user_id)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("duplicate key value violates unique constraint") {
                Error::from("单位收录冲突，请重试保存".to_string())
            } else {
                Error::from(msg)
            }
        })?;
    Ok(())
}

/// 修改单位小数位精度（不支持改名——mxx_product.unit 按名称引用，改名会使存量产品失去关联）
pub async fn update_precision(
    db: &DbConn,
    req: &ProductUnitUpdateRequest,
    updated_by: i64,
) -> Result<i64> {
    let id = req.id.unwrap_or(0);
    if id <= 0 {
        return Err(Error::from("参数错误".to_string()));
    }
    find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("单位不存在".to_string()))?;
    let decimal_places = req.decimal_places.unwrap_or(0);
    if !(0..=6).contains(&decimal_places) {
        return Err(Error::from("小数位需在 0-6 位之间".to_string()));
    }
    update_decimal_places(db, id, decimal_places, updated_by)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}

/// 删除单位（软删除；默认单位与在用单位禁止删除）
pub async fn delete_unit(
    db: &DbConn,
    req: &ProductUnitDeleteRequest,
    updated_by: i64,
) -> Result<i64> {
    let id = req.id.unwrap_or(0);
    if id <= 0 {
        return Err(Error::from("参数错误".to_string()));
    }
    let item = find_by_id(db, id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("单位不存在".to_string()))?;
    if item.is_default == Some(true) {
        return Err(Error::from("默认单位不能删除".to_string()));
    }
    let name = item.name.clone().unwrap_or_default();
    let used = product::Entity::find()
        .filter(product::Column::Unit.eq(&name))
        .filter(product::Column::Deleted.eq(0))
        .count(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    if used > 0 {
        return Err(Error::from(format!("该单位已被 {used} 个产品使用，不能删除")));
    }
    soft_delete(db, id, updated_by)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}
