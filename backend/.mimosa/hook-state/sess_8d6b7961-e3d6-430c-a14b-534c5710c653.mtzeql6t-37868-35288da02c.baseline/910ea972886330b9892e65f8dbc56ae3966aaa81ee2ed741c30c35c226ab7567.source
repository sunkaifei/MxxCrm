//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::modules::system::entity::dept_default_role::{self, Entity as DeptDefaultRole};
use sea_orm::*;

pub struct DeptDefaultRoleModel;

impl DeptDefaultRoleModel {
    /// 查询部门默认角色映射（UNIQUE(dept_id)，最多一行）
    pub async fn find_by_dept(db: &DbConn, dept_id: i64) -> Result<Option<dept_default_role::Model>, DbErr> {
        DeptDefaultRole::find()
            .filter(dept_default_role::Column::DeptId.eq(dept_id))
            .one(db)
            .await
    }

    /// 保存部门默认角色映射：存在则更新 role_id，不存在则插入（13.3-5）
    pub async fn upsert(db: &DbConn, dept_id: i64, role_id: i64, operator: &str) -> Result<(), DbErr> {
        let existing = Self::find_by_dept(db, dept_id).await?;
        match existing {
            Some(row) => {
                if row.role_id == role_id {
                    return Ok(());
                }
                let mut payload: dept_default_role::ActiveModel = row.into();
                payload.role_id = Set(role_id);
                payload.update(db).await?;
            }
            None => {
                let payload = dept_default_role::ActiveModel {
                    dept_id: Set(dept_id),
                    role_id: Set(role_id),
                    create_by: Set(Some(operator.to_string())),
                    create_time: Set(Some(chrono::Local::now().naive_local())),
                    ..Default::default()
                };
                DeptDefaultRole::insert(payload).exec(db).await?;
            }
        }
        Ok(())
    }

    /// 清除部门默认角色映射（部门未配置时不影响审批通过流程，仅跳过分配）
    pub async fn delete_by_dept(db: &DbConn, dept_id: i64) -> Result<u64, DbErr> {
        let result = DeptDefaultRole::delete_many()
            .filter(dept_default_role::Column::DeptId.eq(dept_id))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }
}
