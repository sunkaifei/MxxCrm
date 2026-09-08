//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 销售流程模式配置服务
//! 模式取值：A=仅标准流程（必须经过报价单），B=仅简易流程（跳过报价单），both=两种都允许
//!

use crate::core::errors::error::{Error, Result};
use crate::modules::system::entity::config::{self as config_entity, Entity as ConfigEntity};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, QueryFilter, Set};

/// 销售流程模式配置键
pub const SALES_FLOW_CONFIG_KEY: &str = "sales_flow_mode";

/// 模式：仅标准流程（客户→商机→报价单→订单→合同）
pub const MODE_STANDARD: &str = "A";
/// 模式：仅简易流程（客户→商机→订单→合同，跳过报价单）
pub const MODE_SIMPLE: &str = "B";
/// 模式：两种流程都允许
pub const MODE_BOTH: &str = "both";

/// 默认模式（两种都允许）
pub const DEFAULT_MODE: &str = MODE_BOTH;

/// 合法模式列表
pub const VALID_MODES: &[&str] = &[MODE_STANDARD, MODE_SIMPLE, MODE_BOTH];

/// 获取当前销售流程模式
pub async fn get_mode(db: &DbConn) -> String {
    match ConfigEntity::find()
        .filter(config_entity::Column::ConfigKey.eq(SALES_FLOW_CONFIG_KEY))
        .one(db)
        .await
    {
        Ok(Some(model)) => model.config_value.unwrap_or_else(|| DEFAULT_MODE.to_string()),
        _ => DEFAULT_MODE.to_string(),
    }
}

/// 设置销售流程模式（不存在则新增，存在则更新）
pub async fn set_mode(db: &DbConn, mode: &str) -> Result<()> {
    if !VALID_MODES.contains(&mode) {
        return Err(Error::from(format!(
            "无效的销售流程模式：{}，合法值为 A/B/both",
            mode
        )));
    }

    let now = chrono::Local::now().naive_local();

    let existing = ConfigEntity::find()
        .filter(config_entity::Column::ConfigKey.eq(SALES_FLOW_CONFIG_KEY))
        .one(db)
        .await?;

    match existing {
        Some(model) => {
            let mut active: config_entity::ActiveModel = model.into();
            active.config_value = Set(Some(mode.to_string()));
            active.update_time = Set(Some(now));
            active.update(db).await?;
        }
        None => {
            let new_config = config_entity::ActiveModel {
                config_name: Set(Some("销售流程模式".to_string())),
                config_key: Set(Some(SALES_FLOW_CONFIG_KEY.to_string())),
                config_value: Set(Some(mode.to_string())),
                config_type: Set(Some("Y".to_string())),
                remark: Set(Some(
                    "销售流程模式：A=仅标准流程,B=仅简易流程,both=两种都允许".to_string(),
                )),
                sort: Set(Some(100)),
                create_time: Set(Some(now)),
                ..Default::default()
            };
            new_config.insert(db).await?;
        }
    }

    Ok(())
}

/// 判断模式是否要求订单必须关联报价单
pub fn is_quotation_required(mode: &str) -> bool {
    mode == MODE_STANDARD
}

/// 判断模式是否允许跳过报价单
pub fn can_skip_quotation(mode: &str) -> bool {
    mode == MODE_SIMPLE || mode == MODE_BOTH
}

/// 判断模式是否允许走标准流程（经过报价单）
pub fn allows_standard_flow(mode: &str) -> bool {
    mode == MODE_STANDARD || mode == MODE_BOTH
}
