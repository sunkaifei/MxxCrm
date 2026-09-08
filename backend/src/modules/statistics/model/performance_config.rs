//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 业绩统计配置中心（WP3）
//!
//! 存储：复用 `mxx_system_config` KV 表，单行 `config_key='statistics.performance.config'`，
//! `config_value` 存 JSON（不新建表）。
//!
//! 配置模型（裁剪原死页的伪字段，只保留后端真正会消费的 5 项）：
//! ```jsonc
//! {
//!   "statScope": "all_users",     // plan_sales=仅当年有已通过计划者 | all_users=当年有业务数据者(默认)
//!   "showZeroTargetRows": false,  // 榜单是否显示 0 目标行
//!   "hideOtherActual": false,     // 排名隐藏他人实际金额(打码 **)
//!   "hideOtherTarget": false,     // 排名隐藏他人目标金额
//!   "rankingTopN": 10             // 排行默认条数
//! }
//! ```

use sea_orm::{ActiveModelTrait, DbConn, Set};
use serde::{Deserialize, Serialize};

use crate::core::errors::error::Result;
use crate::modules::system::entity::config;
use crate::modules::system::model::config::ConfigModel;

/// 配置中心在 mxx_system_config 表中的键名
pub const PERFORMANCE_CONFIG_KEY: &str = "statistics.performance.config";

/// 统计对象口径位
pub const STAT_SCOPE_PLAN_SALES: &str = "plan_sales";
pub const STAT_SCOPE_ALL_USERS: &str = "all_users";

/// 业绩统计配置 DTO（与 DB JSON / 前端表单同构，camelCase）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceConfigDTO {
    /// 统计对象：plan_sales=仅当年有已通过计划者 | all_users=当年有业务数据者(默认，保持现状)
    pub stat_scope: String,
    /// 榜单是否显示 0 目标行（stat_scope=all_users 时有意义）
    pub show_zero_target_rows: bool,
    /// 排名隐藏他人实际金额（打码为 **，销售易模式）
    pub hide_other_actual: bool,
    /// 排名隐藏他人目标金额
    pub hide_other_target: bool,
    /// 排行默认条数
    pub ranking_top_n: i64,
}

impl Default for PerformanceConfigDTO {
    fn default() -> Self {
        Self {
            stat_scope: STAT_SCOPE_ALL_USERS.to_string(),
            show_zero_target_rows: false,
            hide_other_actual: false,
            hide_other_target: false,
            ranking_top_n: 10,
        }
    }
}

impl PerformanceConfigDTO {
    /// 解析 DB 存储的 JSON（缺省/非法时回落默认值，不阻断统计主流程）
    pub fn from_json_str(raw: Option<String>) -> Self {
        let mut cfg = match raw {
            Some(s) => serde_json::from_str::<PerformanceConfigDTO>(&s).unwrap_or_default(),
            None => PerformanceConfigDTO::default(),
        };
        // 兜底：数据库里即使没存该字段也要有默认值
        if cfg.stat_scope.is_empty() {
            cfg.stat_scope = STAT_SCOPE_ALL_USERS.to_string();
        }
        if cfg.ranking_top_n <= 0 {
            cfg.ranking_top_n = 10;
        }
        cfg
    }

    /// 序列化为 DB JSON 存储文本
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// 从 DB 加载配置（行不存在时回落默认值，不落库）
    pub async fn load(db: &DbConn) -> Result<Self> {
        let row = ConfigModel::find_by_key(db, PERFORMANCE_CONFIG_KEY).await?;
        Ok(Self::from_json_str(row.and_then(|r| r.config_value)))
    }

    /// 保存配置：行存在则更新 value，不存在则插入新行（幂等）
    pub async fn save(db: &DbConn, cfg: &PerformanceConfigDTO) -> Result<()> {
        let value = cfg.to_json_string();
        let exists = ConfigModel::find_by_key(db, PERFORMANCE_CONFIG_KEY)
            .await?
            .is_some();
        if exists {
            ConfigModel::update_value_by_key(db, PERFORMANCE_CONFIG_KEY, &value).await?;
        } else {
            let now = chrono::Local::now().naive_local();
            let active = config::ActiveModel {
                config_name: Set(Some("业绩统计配置".to_string())),
                config_key: Set(Some(PERFORMANCE_CONFIG_KEY.to_string())),
                config_value: Set(Some(value)),
                config_type: Set(Some("Y".to_string())),
                remark: Set(Some("业绩概览统计口径与隐私配置（WP3 配置中心）".to_string())),
                sort: Set(Some(0)),
                create_by: Set(Some("system".to_string())),
                create_time: Set(Some(now)),
                update_by: Set(Some("system".to_string())),
                update_time: Set(Some(now)),
                ..Default::default()
            };
            active.insert(db).await?;
        }
        Ok(())
    }

    /// 是否为「仅有计划销售」口径
    pub fn is_plan_sales_scope(&self) -> bool {
        self.stat_scope == STAT_SCOPE_PLAN_SALES
    }
}
