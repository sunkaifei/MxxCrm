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
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::website::entity::{navigation, navigation::Entity as Navigation};

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct NavigationSaveDTO {
    /// 主键
    pub id: i64,
    /// 网站id
    pub website_id: Option<i64>,
    /// 父id
    pub parent_id: Option<i64>,
    /// 导航名称
    pub name: Option<String>,
    /// 自定义url地址
    pub web_url: Option<String>,
    /// 数据 id
    pub value: Option<u32>,
    /// 数据类型（custom:自定义导航, article_class:文章分类, customview:自定义页面）
    pub data_type: Option<String>,
    /// 导航类型（header:顶部导航, footer:底部导航）
    pub nav_type: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 是否显示（0否，1是）
    pub is_show: Option<i32>,
    /// 是否新窗口打开（0否，1是）
    pub is_new_window_open: Option<i32>,
}

pub struct NavigationModel;

impl NavigationModel {
    
    /// # 新增导航设置
    /// * `db` 数据库链接
    /// * `website_id` 网站id，主要是为了防止忘了填写网站id才给单独提出来
    /// * `form_data` 需要提交的数据
    #[allow(dead_code)]
    pub async fn insert(db: &DbConn, website_id: i64, form_data: &NavigationSaveDTO) -> Result<i64, DbErr> {
        let payload = navigation::ActiveModel {
            id:                  Set(form_data.id.to_owned()),
            website_id:          Set(Some(website_id).to_owned()),
            parent_id:           Set(form_data.parent_id.to_owned()),
            name:                Set(form_data.name.to_owned()),
            web_url:             Set(form_data.web_url.to_owned()),
            value:               Set(form_data.value.to_owned()),
            data_type:           Set(form_data.data_type.to_owned()),
            nav_type:            Set(form_data.nav_type.to_owned()),
            sort:                Set(form_data.sort.to_owned()),
            is_show:             Set(form_data.is_show.to_owned()),
            is_new_window_open:  Set(form_data.is_new_window_open.to_owned()),
            create_time:         Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            update_time:         Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        
        let res = Navigation::insert(payload).exec(db).await;
        Ok(res.map(|r| r.last_insert_id)?)
    }
    
    /// # 批量删除
    /// * `db` 数据库链接
    /// * `website_id` 网站id，主要是为了防止忘了填写网站id才给单独提出来
    /// * `ids` 批量删除的id
    #[allow(dead_code)]
    pub async fn batch_delete_by_ids(db: &DbConn, website_id: i64, ids: Vec<i64>) -> Result<i64, DbErr> {
        Navigation::delete_many()
            .filter(navigation::Column::Id.is_in(ids))
            .filter(navigation::Column::WebsiteId.eq(website_id))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
    
    /// # 更新导航设置
    /// * `db` 数据库链接
    /// * `website_id` 网站id，主要是为了防止忘了填写网站id才给单独提出来
    /// * `form_data` 需要提交的数据
    #[allow(dead_code)]
    pub async fn update(db: &DbConn, website_id: i64, form_data: &NavigationSaveDTO) -> Result<i64, DbErr> {
        let update_result = Navigation::update_many().set(
            navigation::ActiveModel {
                website_id:          Set(Some(website_id).to_owned()),
                parent_id:           Set(form_data.parent_id.to_owned()),
                name:                Set(form_data.name.to_owned()),
                web_url:             Set(form_data.web_url.to_owned()),
                value:               Set(form_data.value.to_owned()),
                data_type:           Set(form_data.data_type.to_owned()),
                nav_type:            Set(form_data.nav_type.to_owned()),
                sort:                Set(form_data.sort.to_owned()),
                is_show:             Set(form_data.is_show.to_owned()),
                is_new_window_open:  Set(form_data.is_new_window_open.to_owned()),
                update_time:         Set(Option::from(chrono::Local::now().naive_local().to_owned())),
                ..Default::default()
            }
        ).filter(navigation::Column::Id.eq(form_data.id))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }

    /// # 根据名称查询
    /// * `db` 数据库链接
    /// * `website_id` 网站id
    /// * `data_type` 数据类型
    /// * `name` 导航名称
    #[allow(dead_code)]
    pub async fn find_by_name_unique(db: &DbConn, website_id: &Option<i64>, data_type: &Option<String>, name: &Option<String>) -> Result<i64, DbErr> {
        let res = Navigation::find()
            .filter(navigation::Column::WebsiteId.eq(website_id.to_owned()))
            .filter(navigation::Column::DataType.eq(data_type.to_owned()))
            .filter(navigation::Column::Name.eq(name.to_owned()))
            .one(db)
            .await?;
        Ok(res.map(|r| r.id).unwrap_or(0))
    }
    
    /// # 根据id查询
    /// * `db` 数据库链接
    /// 
    #[allow(dead_code)]
    pub async fn find_by_id(db: &DbConn, _website_id: i64, id: i64) -> Result<Option<navigation::Model>, DbErr> {
        let res = Navigation::find_by_id(id).one(db).await?;
        Ok(res)
    }
    
    /// # 查询所有
    /// * `db` 数据库链接
    /// *`website_id` 网站id
    #[allow(dead_code)]
    pub async fn find_all(db: &DbConn, website_id: i64) -> Result<Vec<navigation::Model>, DbErr> {
        let res = Navigation::find()
            .filter(navigation::Column::WebsiteId.eq(website_id))
            .order_by_asc(navigation::Column::Sort)
            .all(db)
            .await?;
        Ok(res)
    }
}