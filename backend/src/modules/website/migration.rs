//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 网站模块结构迁移（版本化：已应用则跳过）
//!

use crate::core::db_migration;
use sea_orm::*;

/// 网站展示产品 SKU 销售库存列迁移批次名（后续调整结构时递增）
const MIGRATION_WEBSITE_PRODUCT_SKU_QUANTITIES: &str = "website_product_sku_quantities_v1";

/// 初始化 mxx_website_product.sku_quantities 列（每 SKU 前台销售库存，JSON 存 {"<skuId>": 数量}）
pub async fn init_website_product_sku_quantities(db: &DbConn) -> Result<(), DbErr> {
    if db_migration::migration_applied(db, MIGRATION_WEBSITE_PRODUCT_SKU_QUANTITIES).await? {
        return Ok(());
    }
    // 老库兼容：表不存在（模块尚未初始化），直接标记已迁移，避免重复执行
    if !db_migration::table_exists(db, "mxx_website_product").await? {
        db_migration::mark_migration_applied(db, MIGRATION_WEBSITE_PRODUCT_SKU_QUANTITIES).await?;
        return Ok(());
    }

    let sql = r#"
        ALTER TABLE mxx_website_product
        ADD COLUMN IF NOT EXISTS sku_quantities VARCHAR(2048)
    "#;
    db.execute_unprepared(sql).await?;

    db_migration::mark_migration_applied(db, MIGRATION_WEBSITE_PRODUCT_SKU_QUANTITIES).await?;
    Ok(())
}
