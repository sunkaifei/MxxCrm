//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::prelude::DateTime;
use sea_orm::*;

use crate::modules::system::entity::{
    onboarding_config, onboarding_config::Entity as OnboardingConfig, onboarding_step,
    onboarding_step::Entity as OnboardingStep,
};

/// 步骤保存 DTO（全量覆盖式保存；step_code 创建后不可修改，新增步骤固定 step_type=2）
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct OnboardingStepSaveDTO {
    #[serde(default, deserialize_with = "crate::utils::string_utils::deserialize_string_to_u64")]
    pub id: Option<i64>,
    pub step_code: Option<String>,
    pub step_name: Option<String>,
    pub step_desc: Option<String>,
    pub link_url: Option<String>,
    pub sort_order: Option<i32>,
    /// 状态（1=启用 0=停用）
    pub status: Option<i32>,
}

/// 配置保存 DTO（version 为乐观锁版本号，必须携带）
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct OnboardingConfigSaveDTO {
    pub announce_enabled: Option<i32>,
    pub todo_enabled: Option<i32>,
    pub quick_enabled: Option<i32>,
    pub quick_preset: Option<serde_json::Value>,
    pub version: Option<i32>,
}

/// POST /onboarding/save 请求体：{ steps: [...], config: {...} }
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, Default)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct OnboardingSaveRequest {
    pub steps: Option<Vec<OnboardingStepSaveDTO>>,
    pub config: Option<OnboardingConfigSaveDTO>,
}

/// 步骤视图对象（契约 9.4：自定义步骤 done 恒 null）
#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingStepVO {
    pub step_code: String,
    pub step_name: String,
    pub step_type: i16,
    pub link_url: Option<String>,
    pub done: Option<bool>,
    pub sort_order: i32,
}

/// 进度视图对象（仅统计 step_type=1）
#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingProgressVO {
    pub done: i64,
    pub total: i64,
}

/// 配置视图对象
#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingConfigVO {
    pub announce_enabled: i16,
    pub todo_enabled: i16,
    pub quick_enabled: i16,
    pub quick_preset: Option<serde_json::Value>,
}

/// 审批视图对象（state=in_approval/rejected 时非空）
#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingApprovalVO {
    pub current_node_name: Option<String>,
    pub submitted_at: Option<DateTime>,
    pub reject_reason: Option<String>,
}

/// 管理员态步骤视图（配置页用，含 id/状态/说明）
#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingAdminStepVO {
    pub id: i64,
    pub step_code: String,
    pub step_name: String,
    pub step_desc: Option<String>,
    pub step_type: i16,
    pub link_url: Option<String>,
    pub sort_order: i32,
    pub status: i16,
}

/// 管理员态额外字段（13.2-1：由 GET /onboarding/config 在管理员态返回，不另设接口）
#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingAdminVO {
    pub version: i32,
    pub steps: Vec<OnboardingAdminStepVO>,
}

/// GET /onboarding/config 响应（契约 9.4，字段名即实现字段名）
#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingConfigResponse {
    pub state: String,
    pub steps: Vec<OnboardingStepVO>,
    pub progress: OnboardingProgressVO,
    pub config: OnboardingConfigVO,
    pub approval: Option<OnboardingApprovalVO>,
    pub admin: Option<OnboardingAdminVO>,
}

pub struct OnboardingStepModel;

impl OnboardingStepModel {
    /// 查询全部未删除步骤（内置在前、自定义在后，均按 sort_order）
    pub async fn find_all<C: ConnectionTrait>(
        db: &C,
        only_enabled: bool,
    ) -> Result<Vec<onboarding_step::Model>, DbErr> {
        let mut select = OnboardingStep::find().filter(onboarding_step::Column::Deleted.eq(0));
        if only_enabled {
            select = select.filter(onboarding_step::Column::Status.eq(1));
        }
        select
            .order_by_asc(onboarding_step::Column::StepType)
            .order_by_asc(onboarding_step::Column::SortOrder)
            .order_by_asc(onboarding_step::Column::Id)
            .all(db)
            .await
    }

    /// 按编码查询
    pub async fn find_by_code<C: ConnectionTrait>(
        db: &C,
        code: &str,
    ) -> Result<Option<onboarding_step::Model>, DbErr> {
        OnboardingStep::find()
            .filter(onboarding_step::Column::StepCode.eq(code))
            .filter(onboarding_step::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 新增步骤（管理员新增固定 step_type=2 自定义）
    pub async fn insert<C: ConnectionTrait>(
        db: &C,
        req: &OnboardingStepSaveDTO,
        create_by: &str,
    ) -> Result<i64, DbErr> {
        let payload = onboarding_step::ActiveModel {
            id: Default::default(),
            step_code: Set(req.step_code.clone().unwrap_or_default().trim().to_string()),
            step_name: Set(req.step_name.clone().unwrap_or_default().trim().to_string()),
            step_desc: Set(req.step_desc.clone()),
            step_type: Set(2),
            link_url: Set(req.link_url.clone()),
            sort_order: Set(req.sort_order.unwrap_or(100)),
            status: Set(req.status.map(|v| v as i16).unwrap_or(1)),
            deleted: Set(0),
            create_by: Set(Some(create_by.to_string())),
            create_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
        };
        OnboardingStep::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 按主键更新（step_code/step_type 不允许修改）
    pub async fn update_by_id<C: ConnectionTrait>(
        db: &C,
        id: i64,
        req: &OnboardingStepSaveDTO,
    ) -> Result<(), DbErr> {
        let mut payload = onboarding_step::ActiveModel {
            id: Set(id),
            ..Default::default()
        };
        if let Some(v) = req.step_name.as_deref() {
            let v = v.trim();
            if !v.is_empty() {
                payload.step_name = Set(v.to_string());
            }
        }
        if let Some(v) = req.step_desc.clone() {
            payload.step_desc = Set(Some(v));
        }
        if let Some(v) = req.link_url.clone() {
            payload.link_url = Set(Some(v));
        }
        if let Some(v) = req.sort_order {
            payload.sort_order = Set(v);
        }
        if let Some(v) = req.status {
            payload.status = Set(v as i16);
        }
        payload.update(db).await.map(|_| ())
    }

    /// 软删除
    pub async fn soft_delete_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<(), DbErr> {
        let payload = onboarding_step::ActiveModel {
            id: Set(id),
            deleted: Set(1),
            ..Default::default()
        };
        payload.update(db).await.map(|_| ())
    }
}

pub struct OnboardingConfigModel;

impl OnboardingConfigModel {
    /// 查询单行配置
    pub async fn find_first<C: ConnectionTrait>(
        db: &C,
    ) -> Result<Option<onboarding_config::Model>, DbErr> {
        OnboardingConfig::find().one(db).await
    }

    /// 插入默认配置行（announce/todo/quick=1，version=0）
    pub async fn insert_default<C: ConnectionTrait>(db: &C) -> Result<i64, DbErr> {
        let payload = onboarding_config::ActiveModel {
            id: Default::default(),
            announce_enabled: Set(1),
            todo_enabled: Set(1),
            quick_enabled: Set(1),
            quick_preset: Set(None),
            version: Set(0),
            update_by: Set(None),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
        };
        OnboardingConfig::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 更新配置并递增版本号（乐观锁校验在 service 层完成）
    pub async fn update_config<C: ConnectionTrait>(
        db: &C,
        row_id: i64,
        req: &OnboardingConfigSaveDTO,
        update_by: &str,
        new_version: i32,
    ) -> Result<(), DbErr> {
        let payload = onboarding_config::ActiveModel {
            id: Set(row_id),
            announce_enabled: Set(req.announce_enabled.unwrap_or(1) as i16),
            todo_enabled: Set(req.todo_enabled.unwrap_or(1) as i16),
            quick_enabled: Set(req.quick_enabled.unwrap_or(1) as i16),
            quick_preset: Set(req.quick_preset.clone()),
            version: Set(new_version),
            update_by: Set(Some(update_by.to_string())),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
        };
        payload.update(db).await.map(|_| ())
    }
}
