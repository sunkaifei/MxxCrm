//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::modules::website::entity::{admin_template_merge, admin_template_merge::Entity as AdminTemplateMerge};
use sea_orm::*;





pub struct AdminTemplateMergeModel;

impl AdminTemplateMergeModel {
    
    pub async fn save(db: &DbConn, admin_id: &Option<i64>, template_id: &Option<i64>) -> Result<i64, DbErr> {
        let payload = admin_template_merge::ActiveModel {
            admin_id:          Set(admin_id.clone()),
            template_id:       Set(template_id.clone()),
            create_time:       Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        AdminTemplateMerge::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id as i64)
    }
    
    pub async fn batch_delete_by_admin_ids(db: &DbConn, admin_id: &Option<i64>) -> Result<i64, DbErr> {
        AdminTemplateMerge::delete_many()
            .filter(admin_template_merge::Column::AdminId.eq(admin_id.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
    
    pub async fn batch_delete_by_template_ids(db: &DbConn, template_id: &Option<i64>) -> Result<i64, DbErr> {
        AdminTemplateMerge::delete_many()
            .filter(admin_template_merge::Column::TemplateId.eq(template_id.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
    
    
    pub async fn find_by_admin_id_and_template_id(db: &DbConn, admin_id: &Option<i64>, template_id: &Option<i64>) -> Result<i64, DbErr> {
        AdminTemplateMerge::find()
            .filter(admin_template_merge::Column::AdminId.eq(admin_id.clone()))
            .filter(admin_template_merge::Column::TemplateId.eq(template_id.clone()))
            .count(db)
            .await
            .map(|c| c as i64)
    }
}
