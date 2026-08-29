//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 低库存自动采购建议服务
//!

use rust_decimal::Decimal;
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter};
use std::collections::HashMap;

use crate::core::errors::error::{Error, Result};
use crate::modules::inventory::entity::{alert_rule, stock, warehouse};
use crate::modules::message::model::notification::SendNotificationRequest;
use crate::modules::message::service::notification_service::NotificationService;
use crate::modules::product::entity::product as product_entity;
use crate::modules::system::entity::admin;
use crate::modules::purchase::model::purchase_requisition::{
    RequisitionItemDTO, RequisitionSaveRequest,
};

/// 低库存建议明细项
#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SuggestionItem {
    pub product_id: i64,
    pub product_name: Option<String>,
    pub product_sku: Option<String>,
    pub unit: Option<String>,
    pub warehouse_id: Option<i64>,
    pub warehouse_name: Option<String>,
    pub available_quantity: Decimal,
    pub alert_min_quantity: Decimal,
    /// 建议采购数量（警戒线 - 可用 + 警戒线，即补到警戒线的两倍）
    pub suggest_quantity: Decimal,
}

/// 低库存建议结果
#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SuggestionResult {
    pub items: Vec<SuggestionItem>,
    pub total: u64,
    /// 生成的采购申请单ID（若执行了生成）
    pub requisition_id: Option<i64>,
}

/// 扫描低库存产品（可用库存 < 最低警戒线）
pub async fn scan_low_stock(db: &DbConn) -> Result<Vec<SuggestionItem>> {
    // 查询所有未删除且设置了最低警戒线的库存记录
    let stocks = stock::Entity::find()
        .filter(stock::Column::Deleted.eq(0))
        .filter(stock::Column::AlertMinQuantity.gt(Decimal::ZERO))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    // 筛选可用库存低于警戒线的记录
    let low_stock_list: Vec<&stock::Model> = stocks
        .iter()
        .filter(|s| {
            let available = s.available_quantity.unwrap_or_default();
            let alert_min = s.alert_min_quantity.unwrap_or_default();
            available < alert_min
        })
        .collect();

    if low_stock_list.is_empty() {
        return Ok(Vec::new());
    }

    // 批量查询产品信息
    let product_ids: Vec<i64> = low_stock_list
        .iter()
        .filter_map(|s| s.product_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let products = if product_ids.is_empty() {
        Vec::new()
    } else {
        product_entity::Entity::find()
            .filter(product_entity::Column::Id.is_in(product_ids))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?
    };
    let product_map: HashMap<i64, product_entity::Model> =
        products.into_iter().map(|p| (p.id, p)).collect();

    // 批量查询仓库名称
    let warehouse_ids: Vec<i64> = low_stock_list
        .iter()
        .filter_map(|s| s.warehouse_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let warehouses = if warehouse_ids.is_empty() {
        Vec::new()
    } else {
        warehouse::Entity::find()
            .filter(warehouse::Column::Id.is_in(warehouse_ids))
            .filter(warehouse::Column::Deleted.eq(0))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?
    };
    let warehouse_map: HashMap<i64, warehouse::Model> =
        warehouses.into_iter().map(|w| (w.id, w)).collect();

    // 构建建议列表
    let mut items: Vec<SuggestionItem> = Vec::new();
    for s in low_stock_list {
        let product_id = s.product_id.unwrap_or_default();
        let product_info = product_map.get(&product_id);
        let available = s.available_quantity.unwrap_or_default();
        let alert_min = s.alert_min_quantity.unwrap_or_default();
        // 建议采购数量 = 警戒线 - 可用 + 警戒线（补到警戒线的两倍，留出缓冲）
        let suggest = if available < alert_min {
            (alert_min - available) + alert_min
        } else {
            Decimal::ZERO
        };

        items.push(SuggestionItem {
            product_id,
            product_name: product_info.and_then(|p| p.name.clone()),
            product_sku: product_info.and_then(|p| p.sku.clone()),
            unit: product_info.and_then(|p| p.unit.clone()),
            warehouse_id: s.warehouse_id,
            warehouse_name: s
                .warehouse_id
                .and_then(|wid| warehouse_map.get(&wid).and_then(|w| w.name.clone())),
            available_quantity: available,
            alert_min_quantity: alert_min,
            suggest_quantity: suggest,
        });
    }

    Ok(items)
}

/// 生成低库存采购建议（仅返回建议清单，不创建采购申请单）
pub async fn get_suggestions(db: &DbConn) -> Result<SuggestionResult> {
    let items = scan_low_stock(db).await?;
    let total = items.len() as u64;
    Ok(SuggestionResult {
        items,
        total,
        requisition_id: None,
    })
}

/// 自动生成采购申请单（基于低库存建议）
/// 扫描低库存产品，生成一张采购申请单
pub async fn generate_requisition(
    db: &DbConn,
    operator_id: i64,
) -> Result<SuggestionResult> {
    let items = scan_low_stock(db).await?;
    if items.is_empty() {
        return Ok(SuggestionResult {
            items: Vec::new(),
            total: 0,
            requisition_id: None,
        });
    }

    // 构建采购申请单请求
    let pr_items: Vec<RequisitionItemDTO> = items
        .iter()
        .map(|it| RequisitionItemDTO {
            product_id: Some(it.product_id),
            product_name: it.product_name.clone(),
            product_sku: it.product_sku.clone(),
            spec: None,
            unit: it.unit.clone(),
            quantity: Some(it.suggest_quantity),
            estimated_price: None,
            estimated_amount: None,
            remark: Some(format!(
                "低库存自动建议：可用 {} 低于警戒线 {}",
                it.available_quantity, it.alert_min_quantity
            )),
        })
        .collect();

    let today = chrono::Local::now().naive_local().date();
    let req = RequisitionSaveRequest {
        id: None,
        pr_type: Some("auto_low_stock".to_string()),
        title: Some(format!(
            "低库存自动采购建议-{}",
            chrono::Local::now().format("%Y%m%d")
        )),
        department_id: None,
        requester_id: Some(operator_id),
        expected_date: Some(today),
        urgency: Some("normal".to_string()),
        total_amount: None,
        currency: None,
        reason: Some("系统自动检测到部分产品库存低于警戒线，自动生成采购建议".to_string()),
        remark: Some("由库存系统自动生成".to_string()),
        items: pr_items,
    };

    let requisition_id =
        crate::modules::purchase::service::purchase_requisition_service::insert(db, &req, operator_id)
            .await?;

    let total = items.len() as u64;
    Ok(SuggestionResult {
        items,
        total,
        requisition_id: Some(requisition_id),
    })
}

/// 低库存扫描通知结果
#[derive(Debug, Clone)]
pub struct ScanNotifyResult {
    /// 本次扫描到的低库存项数
    pub low_stock_count: usize,
    /// 实际成功通知的用户数
    pub notified_users: usize,
    /// 是否兜底通知了超管（预警规则均未配置订阅人）
    pub fallback_to_admin: bool,
}

/// 扫描低库存并按预警规则订阅人发送站内通知
///
/// 订阅人来源：启用低库存预警规则的 notify_users（逗号分隔用户ID）并集；
/// 未配置订阅人的规则默认通知其覆盖仓库的主管（manager_id，不限仓库则取全部启用仓库主管）；
/// 全部无有效订阅人时，兜底通知全部启用状态的超级管理员（user_type=1）。
pub async fn scan_and_notify(db: &DbConn) -> Result<ScanNotifyResult> {
    // 1. 扫描低库存明细
    let items = scan_low_stock(db).await?;
    let low_stock_count = items.len();
    if items.is_empty() {
        return Ok(ScanNotifyResult {
            low_stock_count: 0,
            notified_users: 0,
            fallback_to_admin: false,
        });
    }

    // 2. 收集订阅人：启用低库存预警规则的 notify_users 并集（逗号分隔，容错非数字/空串）
    let rules = alert_rule::Entity::find()
        .filter(alert_rule::Column::Deleted.eq(0))
        .filter(alert_rule::Column::EnableLowAlert.eq(true))
        .all(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let mut subscriber_set: std::collections::HashSet<i64> = std::collections::HashSet::new();
    // 未配置订阅人的规则：默认通知仓库主管（按规则限定仓库取 manager_id，不限仓库则取全部启用仓库主管）
    let mut default_warehouse_ids: std::collections::HashSet<i64> = std::collections::HashSet::new();
    let mut default_all_warehouses = false;
    for rule in &rules {
        let mut parsed: Vec<i64> = Vec::new();
        if let Some(raw) = &rule.notify_users {
            // 兼容两种存储格式：JSON 数组 "[1,2]" 与逗号分隔 "1,2"（含中文逗号/空格）
            let normalized: String = raw
                .chars()
                .map(|c| if matches!(c, '[' | ']' | '"' | ' ' | '，' | '；') { ',' } else { c })
                .collect();
            for part in normalized.split(',') {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                if let Ok(uid) = part.parse::<i64>() {
                    if uid > 0 {
                        parsed.push(uid);
                    }
                }
            }
        }
        if parsed.is_empty() {
            match rule.warehouse_id {
                Some(wid) => {
                    default_warehouse_ids.insert(wid);
                }
                None => {
                    default_all_warehouses = true;
                }
            }
        } else {
            for uid in parsed {
                subscriber_set.insert(uid);
            }
        }
    }
    // 批量查询仓库主管（防 N+1）：作为未配置订阅人规则的默认订阅人
    if default_all_warehouses || !default_warehouse_ids.is_empty() {
        let mut wq = warehouse::Entity::find()
            .filter(warehouse::Column::Deleted.eq(0))
            .filter(warehouse::Column::IsActive.eq(true))
            .filter(warehouse::Column::ManagerId.is_not_null());
        if !default_all_warehouses {
            let wids: Vec<i64> = default_warehouse_ids.into_iter().collect();
            wq = wq.filter(warehouse::Column::Id.is_in(wids));
        }
        let warehouses = wq
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        for w in warehouses {
            if let Some(mid) = w.manager_id {
                if mid > 0 {
                    subscriber_set.insert(mid);
                }
            }
        }
    }
    let mut subscriber_ids: Vec<i64> = subscriber_set.into_iter().collect();

    // 过滤有效订阅人（存在且启用未删除），避免向已删除/停用账号发送通知
    if !subscriber_ids.is_empty() {
        let valid_users = admin::Entity::find()
            .filter(admin::Column::Id.is_in(subscriber_ids.clone()))
            .filter(admin::Column::Status.eq(1))
            .filter(admin::Column::Deleted.eq(0))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        let valid_ids: std::collections::HashSet<i64> =
            valid_users.into_iter().map(|a| a.id).collect();
        subscriber_ids.retain(|uid| valid_ids.contains(uid));
    }

    // 3. 无有效订阅者时兜底通知全部启用状态的超级管理员
    let mut fallback_to_admin = false;
    if subscriber_ids.is_empty() {
        fallback_to_admin = true;
        let admins = admin::Entity::find()
            .filter(admin::Column::UserType.eq(1))
            .filter(admin::Column::Status.eq(1))
            .filter(admin::Column::Deleted.eq(0))
            .all(db)
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        subscriber_ids = admins.into_iter().map(|a| a.id).collect();
    }

    if subscriber_ids.is_empty() {
        log::warn!(
            "[low_stock_suggestion] 检测到 {} 项低库存，但无订阅人且无启用超管，跳过通知",
            low_stock_count
        );
        return Ok(ScanNotifyResult {
            low_stock_count,
            notified_users: 0,
            fallback_to_admin,
        });
    }

    // 4. 组装通知内容（明细最多展示 5 条，避免内容超长）
    let mut content = format!("当前共有 {} 项产品可用库存低于警戒线：\n", low_stock_count);
    for item in items.iter().take(5) {
        content.push_str(&format!(
            "· {}（{}）可用 {} / 警戒线 {}，建议采购 {}\n",
            item.product_name.as_deref().unwrap_or("未知产品"),
            item.warehouse_name.as_deref().unwrap_or("未知仓库"),
            item.available_quantity,
            item.alert_min_quantity,
            item.suggest_quantity
        ));
    }
    if low_stock_count > 5 {
        content.push_str(&format!("……等共 {} 项。", low_stock_count));
    }
    content.push_str("请前往库存预警页面查看处理。");

    // 5. 批量发送站内通知（type=3 业务通知；发送失败仅记录日志，不影响扫描结果）
    let req = SendNotificationRequest {
        title: "低库存预警提醒".to_string(),
        content: Some(content),
        r#type: 3,
        biz_type: Some("low_stock_alert".to_string()),
        biz_id: None,
        receiver_id: None,
        receiver_ids: Some(subscriber_ids.clone()),
        link_url: Some("/inventory-alert".to_string()),
    };
    let notified_users = subscriber_ids.len();
    if let Err(e) = NotificationService::send_notification(db, None, subscriber_ids, req).await {
        log::warn!("[low_stock_suggestion] 站内通知发送失败：{:?}", e);
        return Ok(ScanNotifyResult {
            low_stock_count,
            notified_users: 0,
            fallback_to_admin,
        });
    }

    Ok(ScanNotifyResult {
        low_stock_count,
        notified_users,
        fallback_to_admin,
    })
}
