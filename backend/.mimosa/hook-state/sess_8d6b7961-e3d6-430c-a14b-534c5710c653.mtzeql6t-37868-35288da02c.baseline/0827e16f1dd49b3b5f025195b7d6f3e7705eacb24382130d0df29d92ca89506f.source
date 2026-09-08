//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 电子签约 Service
//!
//! 用于管理合同的电子签约流程（对接 e签宝等第三方平台）。
//! 当前 sign_url 为占位实现，实际对接第三方平台时替换为真实调用。
//!

use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set, TransactionTrait};

use crate::core::errors::error::{Error, Result};
use crate::modules::crm::entity::electronic_signature::{self, Entity, Column};
use crate::modules::system::service::integration_config_service;

/// 第三方接口配置编码：e签宝
const INTEGRATION_CODE_ESIGN_CN: &str = "esign_cn";

/// 签约状态：1=待签约, 2=已签约, 3=已撤销, 4=已过期
const SIGN_STATUS_PENDING: i32 = 1;
const SIGN_STATUS_SIGNED: i32 = 2;
const SIGN_STATUS_CANCELLED: i32 = 3;

/// 简易 URL 编码（仅编码特殊字符，避免引入新依赖）
fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

/// 创建签约请求
///
/// 流程：
/// 1. 生成签约编号 ESIGN + yyyyMMdd + 4 位流水
/// 2. 生成 sign_url（占位，实际对接 e签宝时替换）
/// 3. 设置过期时间（7 天后）
/// 4. 写入 mxx_crm_electronic_signature 表
/// 5. 返回签约 ID
pub async fn create_signature(
    db: &DbConn,
    contract_id: i64,
    platform: i32,
    signer_name: String,
    signer_phone: String,
    signer_email: String,
    user_id: i64,
) -> Result<i64> {
    if contract_id <= 0 {
        return Err(Error::from("合同ID不能为空"));
    }

    // 1. 生成签约编号
    let date_prefix = format!("ESIGN{}", chrono::Local::now().format("%Y%m%d"));
    let today_count = Entity::find()
        .filter(Column::SignNo.starts_with(&date_prefix))
        .count(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    let seq = today_count + 1;
    let sign_no = format!("{}{:04}", date_prefix, seq);

    // 2. 生成 sign_url
    //    从 integration_config 读取 e签宝 配置（app_id/app_secret/base_url）
    //    配置完整时调用真实接口；否则保持占位 + 提示用户去接口配置中心设置
    let app_id = integration_config_service::get_config_value(db, INTEGRATION_CODE_ESIGN_CN, "app_id").await;
    let app_secret = integration_config_service::get_config_value(db, INTEGRATION_CODE_ESIGN_CN, "app_secret").await;
    let base_url = match integration_config_service::get_by_code(db, INTEGRATION_CODE_ESIGN_CN).await {
        Ok(Some(cfg)) => cfg.api_base_url.filter(|s| !s.is_empty()),
        _ => None,
    };

    let sign_url = if app_id.as_deref().filter(|s| !s.is_empty()).is_some()
        && app_secret.as_deref().filter(|s| !s.is_empty()).is_some()
    {
        // e签宝 已配置：TODO 对接 e签宝 真实创建签署接口
        // 此处保留占位 URL，实际对接时替换为通过 base_url + app_id/app_secret 调用获取的真实链接
        let base = base_url.unwrap_or_default();
        if base.is_empty() {
            format!("/esign/pending/{}?contractId={}&tip={}", sign_no, contract_id, url_encode("e签宝 base_url 未配置"))
        } else {
            format!("{}/api/v2/files/sign/{}", base, sign_no)
        }
    } else {
        // e签宝 未配置：保持占位 + 提示
        format!(
            "/esign/pending/{}?contractId={}&tip={}",
            sign_no, contract_id,
            url_encode("e签宝未配置，请先在接口配置中心设置 app_id/app_secret/base_url")
        )
    };

    // 3. 设置过期时间（7 天后）
    let expire_time = chrono::Local::now()
        .checked_add_signed(chrono::Duration::days(7))
        .ok_or_else(|| Error::from("计算过期时间失败"))?
        .naive_local();

    // 4. 写入签约记录（事务包裹，符合新增模块事务强制规则）
    let active = electronic_signature::ActiveModel {
        contract_id: Set(Some(contract_id)),
        sign_no: Set(Some(sign_no.clone())),
        platform: Set(Some(platform)),
        sign_url: Set(Some(sign_url)),
        status: Set(Some(SIGN_STATUS_PENDING)),
        signer_name: Set(Some(signer_name)),
        signer_phone: Set(Some(signer_phone)),
        signer_email: Set(Some(signer_email)),
        expire_time: Set(Some(expire_time)),
        create_by: Set(Some(user_id)),
        create_time: Set(Some(chrono::Local::now().naive_local())),
        ..Default::default()
    };

    let result = db.transaction::<_, electronic_signature::Model, sea_orm::DbErr>(|txn| {
        Box::pin(async move {
            active.insert(txn).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(result.id)
}

/// 查询签约详情
pub async fn get_signature_info(db: &DbConn, id: i64) -> Result<electronic_signature::Model> {
    Entity::find()
        .filter(Column::Id.eq(id))
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("签约记录不存在"))
}

/// 按合同查询签约记录列表
pub async fn get_by_contract(db: &DbConn, contract_id: i64) -> Result<Vec<electronic_signature::Model>> {
    Entity::find()
        .filter(Column::ContractId.eq(contract_id))
        .filter(Column::Deleted.eq(0))
        .order_by_desc(Column::Id)
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))
}

/// 签约回调处理
///
/// 由第三方平台（e签宝等）在签约完成后回调本系统。
/// - `sign_no`：签约编号
/// - `status`：回调状态（2=已签约，3=已撤销，4=已过期）
/// - `signed_pdf_url`：签署完成后的 PDF 文件 URL
pub async fn handle_sign_callback(
    db: &DbConn,
    sign_no: String,
    status: i32,
    signed_pdf_url: Option<String>,
) -> Result<i64> {
    let record = Entity::find()
        .filter(Column::SignNo.eq(&sign_no))
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from(format!("签约编号 [{}] 不存在", sign_no)))?;

    let id = record.id;
    let now = chrono::Local::now().naive_local();

    let mut active: electronic_signature::ActiveModel = electronic_signature::ActiveModel {
        id: Set(id),
        status: Set(Some(status)),
        ..Default::default()
    };

    // 签署成功时记录签署完成时间与签署后 PDF
    if status == SIGN_STATUS_SIGNED {
        active.signed_time = Set(Some(now));
        if let Some(url) = signed_pdf_url {
            active.signed_pdf_url = Set(Some(url));
        }
    }

    let result = db.transaction::<_, electronic_signature::Model, sea_orm::DbErr>(|txn| {
        let active = active.clone();
        Box::pin(async move {
            active.update(txn).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(result.id)
}

/// 撤销签约
///
/// 仅"待签约"状态可撤销，已签约/已撤销等终态不允许撤销。
pub async fn cancel_signature(db: &DbConn, id: i64) -> Result<i64> {
    let record = Entity::find()
        .filter(Column::Id.eq(id))
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("签约记录不存在"))?;

    // 终态校验：已签约、已撤销不允许撤销
    let current = record.status.unwrap_or(SIGN_STATUS_PENDING);
    if matches!(current, SIGN_STATUS_SIGNED | SIGN_STATUS_CANCELLED) {
        return Err(Error::from(format!("当前签约状态({})不允许撤销", current)));
    }

    let active: electronic_signature::ActiveModel = electronic_signature::ActiveModel {
        id: Set(id),
        status: Set(Some(SIGN_STATUS_CANCELLED)),
        update_time: Set(Some(chrono::Local::now().naive_local())),
        ..Default::default()
    };

    let result = db.transaction::<_, electronic_signature::Model, sea_orm::DbErr>(|txn| {
        let active = active.clone();
        Box::pin(async move {
            active.update(txn).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(result.id)
}
