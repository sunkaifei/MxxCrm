//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 下载链接防盗业务逻辑层
//!

use crate::core::errors::error::{Error, Result};
use crate::modules::sale::entity::order_delivery::{self, Entity, Column};
use crate::utils::encryption_utils;
use base64::{engine::general_purpose, Engine as _};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, QueryFilter, Set, TransactionTrait,
};
use serde::Serialize;

/// 签名密钥（项目无 hmac 库，用 md5(key + data) 替代）
const DOWNLOAD_SIGN_SECRET: &str = "mxxcrm_download_sign_v1";

#[derive(Debug, Serialize)]
pub struct SignedUrlVO {
    pub url: String,
    pub token: String,
    pub expire_at: i64,
}

#[derive(Debug, Serialize)]
pub struct DownloadAccessVO {
    pub delivery_id: i64,
    pub download_url: String,
    pub product_name: Option<String>,
}

/// 为下载链接类型的交付生成签名URL
pub async fn generate_signed_url(db: &DbConn, delivery_id: i64, expire_hours: i64) -> Result<SignedUrlVO> {
    let delivery = Entity::find_by_id(delivery_id)
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("交付记录不存在"))?;

    // 仅下载链接类型(2)支持生成签名URL
    let method = delivery.delivery_method.unwrap_or(0);
    if method != 2 {
        return Err(Error::from("仅下载链接类型(2)的交付支持生成签名URL"));
    }

    let now = chrono::Local::now().timestamp();
    let expire_at = now + expire_hours * 3600;

    // 签名 = md5(secret + delivery_id:expire_timestamp)
    let sign_data = format!("{}:{}", delivery_id, expire_at);
    let signature = encryption_utils::md5(&format!("{}{}", DOWNLOAD_SIGN_SECRET, sign_data));

    // token = base64(delivery_id:expire_timestamp:signature)
    let raw_token = format!("{}:{}:{}", delivery_id, expire_at, signature);
    let token = general_purpose::STANDARD.encode(&raw_token);

    let url = format!("/api/system/sale/download/access?token={}&id={}", token, delivery_id);

    Ok(SignedUrlVO {
        url,
        token,
        expire_at,
    })
}

/// 验证签名URL有效性，返回解密后的下载链接
pub async fn verify_and_serve(db: &DbConn, token: String, delivery_id: i64) -> Result<DownloadAccessVO> {
    // 解码 token
    let raw = general_purpose::STANDARD
        .decode(&token)
        .map_err(|_| Error::from("无效的token编码"))?;
    let raw_str = String::from_utf8(raw)
        .map_err(|_| Error::from("无效的token内容"))?;

    let parts: Vec<&str> = raw_str.splitn(3, ':').collect();
    if parts.len() != 3 {
        return Err(Error::from("token格式无效"));
    }

    let token_delivery_id: i64 = parts[0]
        .parse()
        .map_err(|_| Error::from("token中delivery_id无效"))?;
    let expire_at: i64 = parts[1]
        .parse()
        .map_err(|_| Error::from("token中过期时间无效"))?;
    let signature = parts[2];

    if token_delivery_id != delivery_id {
        return Err(Error::from("token与交付ID不匹配"));
    }

    // 验证签名
    let sign_data = format!("{}:{}", delivery_id, expire_at);
    let expected_signature = encryption_utils::md5(&format!("{}{}", DOWNLOAD_SIGN_SECRET, sign_data));
    if signature != expected_signature {
        return Err(Error::from("签名验证失败"));
    }

    // 检查是否过期
    let now = chrono::Local::now().timestamp();
    if now > expire_at {
        return Err(Error::from("下载链接已过期"));
    }

    // 查询交付记录
    let delivery = Entity::find_by_id(delivery_id)
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("交付记录不存在"))?;

    // 检查状态是否已撤销(4)
    let status = delivery.status.unwrap_or(0);
    if status == 4 {
        return Err(Error::from("该下载链接已被撤销"));
    }

    // 解密下载链接
    let download_url = delivery.download_url
        .as_ref()
        .map(|url| encryption_utils::decrypt_card(url))
        .unwrap_or_default();

    if download_url.is_empty() {
        return Err(Error::from("下载链接为空"));
    }

    Ok(DownloadAccessVO {
        delivery_id,
        download_url,
        product_name: delivery.product_name,
    })
}

/// 撤销访问
pub async fn revoke_access(db: &DbConn, delivery_id: i64) -> Result<i64> {
    let existing = Entity::find_by_id(delivery_id)
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("交付记录不存在"))?;

    let now = chrono::Local::now().naive_local();
    let txn = db.begin().await?;
    let mut active: order_delivery::ActiveModel = existing.into();
    active.status = Set(Some(4)); // 已撤销
    active.update_time = Set(Some(now));
    active.update(&txn).await?;
    txn.commit().await?;

    Ok(delivery_id)
}
