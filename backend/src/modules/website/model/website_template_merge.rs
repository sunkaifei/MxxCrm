//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::modules::website::entity::{website_template_merge, website_template_merge::Entity as WebsiteTemplateMerge};
use sea_orm::*;


pub struct WebsiteTemplateMergeModel;

impl WebsiteTemplateMergeModel {
    pub async fn save(db: &DbConn, website_id: &Option<i64>, template_id: &Option<i64>) -> Result<i64, DbErr> {
        let payload = website_template_merge::ActiveModel {
            website_id:          Set(website_id.clone()),
            template_id:         Set(template_id.clone()),
            create_time:         Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        WebsiteTemplateMerge::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }
    
    pub async fn delete_by_website_id(db: &DbConn, website_id: &Option<i64>) -> Result<i64, DbErr> {
        WebsiteTemplateMerge::delete_many()
            .filter(website_template_merge::Column::WebsiteId.eq(website_id.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn delete_by_template_id(db: &DbConn, template_id: &Option<i64>) -> Result<i64, DbErr> {
        WebsiteTemplateMerge::delete_many()
            .filter(website_template_merge::Column::TemplateId.eq(template_id.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
}