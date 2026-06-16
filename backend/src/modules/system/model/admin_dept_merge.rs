//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::*;
use sea_orm::prelude::DateTime;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::system::entity::{admin_dept_merge, admin_dept_merge::Entity as AdminDeptMerge};


/// 角色和部门关联表
#[derive(PartialEq, Default, Clone, Debug, Serialize, Deserialize)]
pub struct AdminDeptMergeSaveDTO {
    ///角色ID
    pub id: Option<i64>,
    ///角色名称
    pub admin_id: Option<i64>,
    ///角色权限字符串
    pub dept_id: Option<i64>,
    ///创建时间
    pub create_time: Option<DateTime>,
}



pub struct AdminDeptMergeModel;

impl AdminDeptMergeModel {
    
    pub async fn save(db: &DbConn,admin_id: i64,dept_id: i64) -> Result<i64, DbErr> {
        let result = AdminDeptMerge::insert(admin_dept_merge::ActiveModel {
            admin_id: Set(Option::from(admin_id)),
            dept_id: Set(Option::from(dept_id)),
            ..Default::default()
        })
        .exec(db)
        .await?;
        Ok(result.last_insert_id)
    }
    
    pub async fn insert_batch<C: ConnectionTrait>(db: &C, list: &Vec<AdminDeptMergeSaveDTO>) -> Result<i64, DbErr> {

        let result: Vec<admin_dept_merge::ActiveModel> = list.iter().map(|item| admin_dept_merge::ActiveModel {
            admin_id:       Set(item.admin_id),
            dept_id:        Set(item.dept_id),
            create_time:    Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        }).collect();

        let insert_result = AdminDeptMerge::insert_many(result)
            .exec(db)
            .await?;
        Ok(insert_result.last_insert_id.unwrap_or_default())
    }
    
    /// 按用户id删除关联id
    pub async fn delete_by_admin_id<C: ConnectionTrait>(db: &C,admin_id: &Option<i64>) -> Result<i64, DbErr> {
        let result = AdminDeptMerge::delete_many()
            .filter(admin_dept_merge::Column::AdminId.eq(admin_id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }
    
    pub async fn delete_by_dept_id(db: &DbConn,dept_id: i64) -> Result<i64, DbErr> {
        let result = AdminDeptMerge::delete_many()
            .filter(admin_dept_merge::Column::DeptId.eq(dept_id))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn find_by_admin_id(db: &DbConn, admin_id: i64) -> Result<Vec<admin_dept_merge::Model>, DbErr> {
        let result =  AdminDeptMerge::find()
            .filter(admin_dept_merge::Column::AdminId.eq(admin_id))
            .all(db)
            .await?;
        Ok(result)
    }
    pub async fn find_by_admin_ids(db: &DbConn,admin_id: Vec<i64>) -> Result<Vec<admin_dept_merge::Model>, DbErr> {
        let result =  AdminDeptMerge::find()
            .filter(admin_dept_merge::Column::AdminId.is_in(admin_id))
            .all(db)
            .await?;
        Ok(result)
    }
    
    pub async fn find_by_dept_id(db: &DbConn,dept_id: Vec<i64>) -> Result<Vec<admin_dept_merge::Model>, DbErr> {
        let result =  AdminDeptMerge::find()
            .filter(admin_dept_merge::Column::DeptId.is_in(dept_id))
            .all(db)
            .await?;
        Ok(result)
    }
}
