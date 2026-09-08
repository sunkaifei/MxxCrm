use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, IntoActiveModel, PaginatorTrait,
    QueryFilter, QueryOrder, Set, TransactionTrait,
};

use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::sale::entity::logistics_tracking::{self, Entity as TrackingEntity, Column};
use crate::modules::sale::model::shipment::ShipmentModel;
use crate::modules::system::service::integration_config_service;

/// 第三方接口配置编码：快递100
const INTEGRATION_CODE_KUAIDI100: &str = "kuaidi100";

/// 签收状态：0=未签收, 1=已签收
const SIGNED_YES: i32 = 1;
const SIGNED_NO: i32 = 0;

/// 创建物流追踪记录
pub async fn create_tracking(
    db: &DbConn,
    shipment_id: i64,
    tracking_no: String,
    company_code: Option<String>,
    company_name: Option<String>,
) -> Result<i64> {
    let shipment = ShipmentModel::find_by_id(db, shipment_id)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from("发货单不存在"))?;

    let now = chrono::Local::now().naive_local();
    let active = logistics_tracking::ActiveModel {
        shipment_id: Set(Some(shipment_id)),
        order_id: Set(shipment.order_id),
        tracking_no: Set(Some(tracking_no)),
        logistics_company_code: Set(company_code),
        logistics_company_name: Set(company_name),
        traces: Set(Some(serde_json::json!([]))),
        is_signed: Set(Some(SIGNED_NO)),
        signed_time: Set(None),
        last_poll_time: Set(None),
        auto_track: Set(Some(1)),
        create_time: Set(Some(now)),
        update_time: Set(Some(now)),
        deleted: Set(Some(0)),
        ..Default::default()
    };

    let txn = db.begin().await?;
    let model = active.insert(&txn).await?;
    txn.commit().await?;

    Ok(model.id)
}

/// 查询物流轨迹（返回占位数据，实际对接快递100 API）
pub async fn query_tracking(db: &DbConn, tracking_id: i64) -> Result<serde_json::Value> {
    let tracking = TrackingEntity::find_by_id(tracking_id)
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("物流追踪记录不存在"))?;

    let traces = tracking.traces.clone().unwrap_or(serde_json::json!([]));

    // 检查快递100 是否已配置，未配置时在返回中附带提示
    let customer = integration_config_service::get_config_value(db, INTEGRATION_CODE_KUAIDI100, "customer").await;
    let key = integration_config_service::get_config_value(db, INTEGRATION_CODE_KUAIDI100, "key").await;
    let config_tip = if customer.as_deref().filter(|s| !s.is_empty()).is_none()
        || key.as_deref().filter(|s| !s.is_empty()).is_none()
    {
        Some("快递100未配置，请在接口配置中心设置 customer/key 后启用实时轨迹")
    } else {
        None
    };

    Ok(serde_json::json!({
        "id": tracking.id,
        "shipmentId": tracking.shipment_id,
        "orderId": tracking.order_id,
        "trackingNo": tracking.tracking_no,
        "logisticsCompanyCode": tracking.logistics_company_code,
        "logisticsCompanyName": tracking.logistics_company_name,
        "isSigned": tracking.is_signed,
        "signedTime": tracking.signed_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        "traces": traces,
        "configTip": config_tip,
    }))
}

/// 主动拉取最新轨迹
pub async fn poll_tracking(db: &DbConn, tracking_id: i64) -> Result<serde_json::Value> {
    let tracking = TrackingEntity::find_by_id(tracking_id)
        .filter(Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("物流追踪记录不存在"))?;

    // 从 integration_config 读取快递100 customer/key
    let customer = integration_config_service::get_config_value(db, INTEGRATION_CODE_KUAIDI100, "customer").await;
    let key = integration_config_service::get_config_value(db, INTEGRATION_CODE_KUAIDI100, "key").await;

    let traces = if customer.as_deref().filter(|s| !s.is_empty()).is_some()
        && key.as_deref().filter(|s| !s.is_empty()).is_some()
    {
        // 快递100 已配置：调用实时查询 API
        match query_kuaidi100_api(
            customer.as_deref().unwrap_or_default(),
            key.as_deref().unwrap_or_default(),
            tracking.tracking_no.as_deref().unwrap_or_default(),
            tracking.logistics_company_code.as_deref().unwrap_or_default(),
        ).await {
            Ok(data) => data,
            Err(e) => {
                log::warn!("[logistics] 快递100查询失败，使用占位数据：{}", e);
                serde_json::json!([
                    {
                        "time": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        "context": format!("快递100查询失败：{}", e),
                    }
                ])
            }
        }
    } else {
        // 快递100 未配置：返回占位数据 + 提示
        serde_json::json!([
            {
                "time": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                "context": "快递100未配置，请在接口配置中心设置 customer/key 后启用实时轨迹",
            }
        ])
    };

    let now = chrono::Local::now().naive_local();
    let mut active: logistics_tracking::ActiveModel = tracking.into_active_model();
    active.traces = Set(Some(traces.clone()));
    active.last_poll_time = Set(Some(now));
    active.update_time = Set(Some(now));
    active.update(db).await?;

    Ok(traces)
}

/// 调用快递100 实时查询 API（subscribe 智能订阅接口）
///
/// 文档：https://api.kuaidi100.com/applyquery
async fn query_kuaidi100_api(
    customer: &str,
    key: &str,
    tracking_no: &str,
    company_code: &str,
) -> Result<serde_json::Value> {
    if tracking_no.is_empty() {
        return Err(Error::from("物流单号为空"));
    }
    if company_code.is_empty() {
        return Err(Error::from("物流公司编码为空"));
    }

    let param = serde_json::json!({
        "com": company_code,
        "num": tracking_no,
        "phone": "",
        "from": "",
        "to": "",
        "resultv2": "4",
        "show": "0",
        "order": "desc",
    });
    let param_str = serde_json::to_string(&param)
        .map_err(|e| Error::from(format!("序列化快递100参数失败: {}", e)))?;

    // 签名 = MD5(param + key + customer)
    let sign_str = format!("{}{}{}", param_str, key, customer);
    let sign = format!("{:x}", md5::compute(sign_str.as_bytes()));
    let sign = url_encode(&sign);

    let url = "https://poll.kuaidi100.com/poll/query.do";
    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .form(&[
            ("customer", customer),
            ("sign", sign.as_str()),
            ("param", param_str.as_str()),
        ])
        .send()
        .await
        .map_err(|e| Error::from(format!("快递100请求失败: {}", e)))?;

    let text = resp
        .text()
        .await
        .map_err(|e| Error::from(format!("读取快递100响应失败: {}", e)))?;

    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| Error::from(format!("解析快递100响应失败: {} - 原文: {}", e, text)))?;

    if value.get("status").and_then(|v| v.as_str()) == Some("200") {
        if let Some(data) = value.get("data") {
            return Ok(data.clone());
        }
    }

    // 失败时把原始返回作为一条 trace 返回，便于排查
    let msg = value
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("快递100查询失败");
    Ok(serde_json::json!([
        {
            "time": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            "context": format!("快递100: {}", msg),
        }
    ]))
}

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

/// 批量拉取所有未签收的物流轨迹（定时任务调用）
pub async fn batch_poll(db: &DbConn) -> Result<i64> {
    let trackings = TrackingEntity::find()
        .filter(Column::IsSigned.eq(SIGNED_NO))
        .filter(Column::Deleted.eq(0))
        .all(db)
        .await?;

    let mut count = 0i64;
    for t in trackings {
        let id = t.id;
        if let Ok(_) = poll_tracking(db, id).await {
            count += 1;
        }
    }

    Ok(count)
}

/// 按发货单查询轨迹
pub async fn get_tracking_by_shipment(
    db: &DbConn,
    shipment_id: i64,
) -> Result<ResultPage<Vec<logistics_tracking::Model>>> {
    let paginator = TrackingEntity::find()
        .filter(Column::ShipmentId.eq(shipment_id))
        .filter(Column::Deleted.eq(0))
        .order_by_desc(Column::Id)
        .paginate(db, 50);

    let total = paginator.num_items().await.map_err(|e| Error::from(e.to_string()))? as i64;
    let items = paginator
        .fetch_page(0)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(ResultPage::new(items, total, 1, 50))
}

/// 检查已签收并自动更新发货单/订单状态
pub async fn auto_sign_check(db: &DbConn) -> Result<i64> {
    let signed_trackings = TrackingEntity::find()
        .filter(Column::IsSigned.eq(SIGNED_YES))
        .filter(Column::Deleted.eq(0))
        .all(db)
        .await?;

    let mut count = 0i64;
    for t in signed_trackings {
        let shipment_id = match t.shipment_id {
            Some(sid) => sid,
            None => continue,
        };
        // 更新发货单状态为已签收（status=3）
        if let Err(_) = ShipmentModel::update_status(db, shipment_id, 3).await {
            continue;
        }
        // 更新订单状态为已签收（status=9）
        if let Some(order_id) = t.order_id {
            let _ = crate::modules::sale::model::order::OrderModel::update_status(
                db, order_id, 9, None, None,
            ).await;
        }
        count += 1;
    }

    Ok(count)
}
