//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!


use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_template")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 模版所属分类id
    pub category_id: Option<i64>,
    /// 模版的名字，编码，唯一，限制50个字符以内
    pub name: Option<String>,
    /// 模板存储的文件夹名称，设置网站模板时直接使用这个目录
    pub template_folder: Option<String>,
    /// 此模版所属的用户，user.id。如果此模版是用户的私有模版，也就是 iscommon=0 时，这里存储导入此模版的用户的id
    pub user_id: Option<i64>,
    /// 模版的简介，备注说明，限制300字以内
    pub remark: Option<String>,
    /// 模版预览网址，示例网站网址，绝对路径
    pub preview_url: Option<String>,
    /// 商品价格
    pub price: Option<Decimal>,
    /// 促销价
    pub promotion_price: Option<Decimal>,
    /// 模版开发者公司名字。如果没有公司，则填写个人姓名
    pub companyname: Option<String>,
    /// 模版开发人员的名字，姓名
    pub username: Option<String>,
    /// 模版开发者官方网站、企业官网。如果是企业，这里是企业官网的网址，格式如： http://www.leimingyun.com  ，如果是个人，则填写个人网站即可
    pub siteurl: Option<String>,
    /// 网站模版是否支持手机端, 1支持，0不支持
    pub terminal_mobile: Option<i32>,
    /// 网站模版是否支持PC端, 1支持，0不支持
    pub terminal_pc: Option<i32>,
    /// 网站模版是否支持平板电脑, 1支持，0不支持
    pub terminal_ipad: Option<i32>,
    /// 网站模版是否支持展示机, 1支持，0不支持
    pub terminal_display: Option<i32>,
    /// 是否是公共的模版 1是公共的模版， 0不是公共的，私有的，是用户自己开通网站导入的
    pub iscommon: Option<i32>,
    /// 模版的排序，数字越小越靠前
    pub sort: Option<i32>,
    /// wscso模版文件下载的url地址
    pub wscso_down_url: Option<String>,
    /// zip模版素材包文件下载的url地址
    pub zip_down_url: Option<String>,
    /// 模版预览图的网址，preview.jpg 图片的网址
    pub preview_pic: Option<String>,
    /// js、css等资源引用方式。 cloud：使用云端模版库； private:使用私有模版库，也就是本地的
    pub resource_import: Option<String>,
    /// 模版添加时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 审核状态：0不显示，1显示
    pub status: Option<i32>,
    /// 是否删除：0否，1是
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::admin_template_merge::Entity",
        from = "Column::Id",
        to = "super::admin_template_merge::Column::TemplateId"
    )]
    AdminTemplateMerge
}

impl Related<super::admin_template_merge::Entity> for Entity {
    fn to() -> RelationDef {
        super::admin_template_merge::Relation::Template.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::admin_template_merge::Relation::Template.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}