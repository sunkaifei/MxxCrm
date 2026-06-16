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
use crate::modules::system::entity::{dept_menu_merge, dept_menu_merge::Entity as DeptMenuMerge};

pub struct DeptMenuMergeModel;

impl DeptMenuMergeModel {
    
    pub async fn save(db: &DbConn, dept_id: i64, menu_id: i64) -> Result<i64, DbErr> {   
        let result = DeptMenuMerge::insert(dept_menu_merge::ActiveModel {
            dept_id: Set(Option::from(dept_id)),
            menu_id: Set(Option::from(menu_id)),
            ..Default::default()
        })
        .exec(db)
        .await?;
        Ok(result.last_insert_id)
    }
    
    pub async fn delete_by_admin_id(db: &DbConn, dept_id: i64) -> Result<i64, DbErr> {   
        let result = DeptMenuMerge::delete_many()
            .filter(dept_menu_merge::Column::DeptId.eq(dept_id))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }
    
    pub async fn delete_by_dept_id(db: &DbConn, dept_id: i64) -> Result<i64, DbErr> {   
        let result = DeptMenuMerge::delete_many()
            .filter(dept_menu_merge::Column::DeptId.eq(dept_id))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }
    pub async fn find_by_dept_id(db: &DbConn, dept_id: i64) -> Result<Vec<dept_menu_merge::Model>, DbErr> {
        let result =  DeptMenuMerge::find()
            .filter(dept_menu_merge::Column::DeptId.eq(dept_id))
            .all(db)
            .await?;
        Ok(result)
    }
}
