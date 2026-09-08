//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 卡密池 Service
//!

use sea_orm::{ColumnTrait, Condition, DbConn, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, TransactionTrait};

use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::sale::entity::card_pool::{Column as CardPoolColumn, Entity as CardPoolEntity};
use crate::modules::sale::model::card_pool::{
    card_status_name, CardPoolImportRequest, CardPoolListQuery, CardPoolListVO,
    CardPoolModel, CardPoolSaveRequest,
};

/// 卡密脱敏
fn mask_card(key: &str) -> String {
    let len = key.chars().count();
    if len <= 8 { return "****".to_string(); }
    let chars: Vec<char> = key.chars().collect();
    let prefix: String = chars[..4].iter().collect();
    let suffix: String = chars[len-4..].iter().collect();
    format!("{}****{}", prefix, suffix)
}

/// 列表
pub async fn get_list(
    db: &DbConn, query: &CardPoolListQuery
) -> Result<ResultPage<Vec<CardPoolListVO>>> {
    let page = query.page_num.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).max(1);

    let mut cond = Condition::all();
    cond = cond.add(CardPoolColumn::Deleted.eq(0));
    if let Some(pid) = query.product_id { cond = cond.add(CardPoolColumn::ProductId.eq(pid)); }
    if let Some(s) = query.status { cond = cond.add(CardPoolColumn::Status.eq(s)); }
    if let Some(ref bn) = query.batch_no { cond = cond.add(CardPoolColumn::BatchNo.eq(bn)); }

    let paginator = CardPoolEntity::find()
        .filter(cond)
        .order_by_desc(CardPoolColumn::Id)
        .paginate(db, page_size as u64);

    let total = paginator.num_items().await.map_err(|e| Error::from(e.to_string()))? as i64;
    let rows = paginator.fetch_page((page - 1) as u64).await
        .map_err(|e| Error::from(e.to_string()))?;

    let items: Vec<CardPoolListVO> = rows.into_iter().map(|m| CardPoolListVO {
        id: m.id,
        product_id: m.product_id,
        batch_no: m.batch_no,
        card_key_masked: m.card_key.as_deref().map(mask_card),
        status: m.status,
        status_name: m.status.map(|v| card_status_name(v).to_string()),
        lock_order_id: m.lock_order_id,
        sold_order_id: m.sold_order_id,
        sold_time: m.sold_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        import_batch: m.import_batch,
        expire_time: m.expire_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        remark: m.remark,
        create_time: m.create_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
    }).collect();

    Ok(ResultPage::new(items, total, page, page_size))
}

/// 新增单张卡密
pub async fn create(db: &DbConn, req: CardPoolSaveRequest) -> Result<i64> {
    if req.product_id.is_none() { return Err(Error::from("商品ID不能为空")); }
    if req.card_key.as_deref().unwrap_or("").is_empty() {
        return Err(Error::from("卡密内容不能为空"));
    }
    // 加密卡密内容
    let mut req = req;
    if let Some(ref k) = req.card_key {
        req.card_key = Some(crate::utils::encryption_utils::encrypt_card(k));
    }
    CardPoolModel::insert(db, &req).await.map_err(|e| Error::from(e.to_string()))
}

/// 批量导入
pub async fn import(db: &DbConn, req: CardPoolImportRequest) -> Result<i64> {
    let product_id = req.product_id.ok_or_else(|| Error::from("商品ID不能为空"))?;
    if req.card_keys.is_empty() { return Err(Error::from("卡密列表不能为空")); }

    // 加密每张卡密
    let encrypted_keys: Vec<String> = req.card_keys.iter()
        .map(|k| crate::utils::encryption_utils::encrypt_card(k))
        .collect();

    let import_batch = req.import_batch.clone()
        .unwrap_or_else(|| format!("batch-{}", chrono::Local::now().format("%Y%m%d%H%M%S")));
    let expire_naive = req.expire_time.map(|t| t.naive_local());

    let txn = db.begin().await.map_err(|e| Error::from(e.to_string()))?;
    let count = CardPoolModel::insert_batch(
        &txn, product_id, req.batch_no.as_deref(), &encrypted_keys,
        &import_batch, expire_naive, req.remark.as_deref()
    ).await.map_err(|e| Error::from(e.to_string()))?;
    txn.commit().await.map_err(|e| Error::from(e.to_string()))?;

    Ok(count)
}

/// 删除
pub async fn delete(db: &DbConn, id: i64) -> Result<i64> {
    CardPoolModel::soft_delete(db, id).await.map_err(|e| Error::from(e.to_string()))
}

/// 可用卡密数量
pub async fn count_unsold(db: &DbConn, product_id: i64) -> Result<i64> {
    CardPoolModel::count_unsold(db, product_id).await.map_err(|e| Error::from(e.to_string()))
}
