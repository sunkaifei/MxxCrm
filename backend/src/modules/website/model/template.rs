//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::website::entity::{admin_template_merge, template, template::Entity as Template};
use sea_orm::prelude::{DateTime, Decimal};
use sea_orm::*;
use crate::utils::string_utils::{deserialize_string_to_u64,serialize_option_u64_to_string};



#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TemplateSaveRequest {
    /// 模版所属分类id
    #[serde(deserialize_with = "deserialize_string_to_u64")]
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
    /// 审核状态：0不显示，1显示
    pub status: Option<i32>,
}

impl From<TemplateSaveRequest> for TemplateSaveDTO {
    fn from(req: TemplateSaveRequest) -> Self {
        Self {
            id: None,
            category_id: req.category_id,
            name: req.name,
            template_folder: req.template_folder,
            user_id: req.user_id,
            remark: req.remark,
            preview_url: req.preview_url,
            price: req.price,
            promotion_price: req.promotion_price,
            companyname: req.companyname,
            username: req.username,
            siteurl: req.siteurl,
            terminal_mobile: req.terminal_mobile,
            terminal_pc: req.terminal_pc,
            terminal_ipad: req.terminal_ipad,
            terminal_display: req.terminal_display,
            iscommon: req.iscommon,
            sort: req.sort,
            wscso_down_url: req.wscso_down_url,
            zip_down_url: req.zip_down_url,
            preview_pic: req.preview_pic,
            resource_import: req.resource_import,
            status: req.status,
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TemplateUpdateRequest {
    /// 模版id
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 模版所属分类id
    #[serde(deserialize_with = "deserialize_string_to_u64")]
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
    /// 审核状态：0不显示，1显示
    pub status: Option<i32>,
}

impl From<TemplateUpdateRequest> for TemplateSaveDTO {
    fn from(req: TemplateUpdateRequest) -> Self {
        Self {
            id: req.id,
            category_id: req.category_id,
            name: req.name,
            template_folder: req.template_folder,
            user_id: req.user_id,
            remark: req.remark,
            preview_url: req.preview_url,
            price: req.price,
            promotion_price: req.promotion_price,
            companyname: req.companyname,
            username: req.username,
            siteurl: req.siteurl,
            terminal_mobile: req.terminal_mobile,
            terminal_pc: req.terminal_pc,
            terminal_ipad: req.terminal_ipad,
            terminal_display: req.terminal_display,
            iscommon: req.iscommon,
            sort: req.sort,
            wscso_down_url: req.wscso_down_url,
            zip_down_url: req.zip_down_url,
            preview_pic: req.preview_pic,
            resource_import: req.resource_import,
            status: req.status,
        }
    }
}


pub struct TemplateSaveDTO {
    pub id: Option<i64>,
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
    /// 审核状态：0不显示，1显示
    pub status: Option<i32>,
}




#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TemplateListVO {
    /// 主键
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 模版所属分类id
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub category_id: Option<i64>,
    /// 模版的名字，编码，唯一，限制50个字符以内
    pub name: Option<String>,
    /// 模板存储的文件夹名称，设置网站模板时直接使用这个目录
    pub template_folder: Option<String>,
    /// 模版预览网址，示例网站网址，绝对路径
    pub preview_url: Option<String>,
    /// 商品价格
    pub price: Option<Decimal>,
    /// 促销价
    pub promotion_price: Option<Decimal>,
    /// 网站模版是否支持手机端, 1支持，0不支持
    pub terminal_mobile: Option<i32>,
    /// 网站模版是否支持PC端, 1支持，0不支持
    pub terminal_pc: Option<i32>,
    /// 网站模版是否支持平板电脑, 1支持，0不支持
    pub terminal_ipad: Option<i32>,
    /// 网站模版是否支持展示机, 1支持，0不支持
    pub terminal_display: Option<i32>,
    /// 模版预览图的网址，preview.jpg 图片的网址
    pub preview_pic: Option<String>,
    /// 模版添加时间
    pub create_time: Option<String>,
    /// 模版的排序，数字越小越靠前
    pub sort: Option<i32>,
    /// 审核状态：0不显示，1显示
    pub status: Option<i32>,
}

impl From<template::Model> for TemplateListVO {
    fn from(model: template::Model) -> Self {
        Self {
            id: Some(model.id),
            category_id: model.category_id,
            name: model.name,
            template_folder: model.template_folder,
            preview_url: model.preview_url,
            price: model.price,
            promotion_price: model.promotion_price,
            terminal_mobile: model.terminal_mobile,
            terminal_pc: model.terminal_pc,
            terminal_ipad: model.terminal_ipad,
            terminal_display: model.terminal_display,
            preview_pic: model.preview_pic,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            sort: model.sort,
            status: model.status,
        }
    }
}


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TemplateDetailVO {
    /// 主键
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 模版所属分类id
    #[serde(serialize_with = "serialize_option_u64_to_string")]
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
    /// 模版的排序，数字越小越靠前
    pub sort: Option<i32>,
    /// 审核状态：0不显示，1显示
    pub status: Option<i32>,
}

impl From<template::Model> for TemplateDetailVO {
    fn from(model: template::Model) -> Self {
        Self {
            id: Some(model.id),
            category_id: model.category_id,
            name: model.name,
            template_folder: model.template_folder,
            user_id: model.user_id,
            remark: model.remark,
            preview_url: model.preview_url,
            companyname: model.companyname,
            username: model.username,
            siteurl: model.siteurl,
            terminal_mobile: model.terminal_mobile,
            terminal_pc: model.terminal_pc,
            terminal_ipad: model.terminal_ipad,
            terminal_display: model.terminal_display,
            iscommon: model.iscommon,
            sort: model.sort,
            wscso_down_url: model.wscso_down_url,
            zip_down_url: model.zip_down_url,
            preview_pic: model.preview_pic,
            resource_import: model.resource_import,
            create_time: model.create_time,
            update_time: model.update_time,
            status: model.status,
        }
    }
}



/// 树节点结构
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FileTreeNode {
    // 文件夹或文件名
    pub name: String,
    // 是否是文件夹
    pub is_dir: bool,
    // 相对路径地址
    pub relative_path: String,
    // 子节点
    pub children: Vec<FileTreeNode>,   
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery{
    pub keywords: Option<String>,
    pub status: Option<i32>,
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MyListQuery{
    pub admin_id: Option<i64>,
    pub keywords: Option<String>,
    pub status: Option<i32>,
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
}
#[derive(Clone)]
pub struct PageWhere {
    pub name: Option<String>,
    pub status: Option<i32>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut name = None;
        if self.name != Some("".to_string()) {
            name = self.name.clone();
        }

        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        Self {
            name,
            status,
        }
    }
}


#[derive(Clone)]
pub struct MyPageWhere {
    pub admin_id: Option<i64>,
    pub name: Option<String>,
    pub status: Option<i32>,
}

impl MyPageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut admin_id = None;
        if self.admin_id != Some(0) {
            admin_id = self.admin_id.clone();
        }
        
        let mut name = None;
        if self.name != Some("".to_string()) {
            name = self.name.clone();
        }

        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        Self {
            admin_id,
            name,
            status,
        }
    }
}
pub struct TemplateModel;

impl TemplateModel {
    /// 添加网站数据
    /// * `db` 数据库链接
    pub async fn insert(db: &DbConn, form_data: &TemplateSaveDTO) -> Result<i64, DbErr> {
        let payload = template::ActiveModel {
            category_id:           Set(form_data.category_id),
            user_id:               Set(form_data.user_id),
            name:                  Set(form_data.name.to_owned()),
            template_folder:       Set(form_data.template_folder.to_owned()),
            remark:                Set(form_data.remark.to_owned()),
            price:                 Set(form_data.price.to_owned()),
            promotion_price:       Set(form_data.promotion_price.to_owned()),
            companyname:           Set(form_data.companyname.to_owned()),
            username:              Set(form_data.username.to_owned()),
            siteurl:               Set(form_data.siteurl.to_owned()),
            terminal_mobile:       Set(form_data.terminal_mobile.to_owned()),
            terminal_pc:           Set(form_data.terminal_pc.to_owned()),
            terminal_ipad:         Set(form_data.terminal_ipad.to_owned()),
            terminal_display:      Set(form_data.terminal_display.to_owned()),
            iscommon:              Set(form_data.iscommon.to_owned()),
            wscso_down_url:        Set(form_data.wscso_down_url.to_owned()),
            zip_down_url:          Set(form_data.zip_down_url.to_owned()),
            preview_pic:           Set(form_data.preview_pic.to_owned()),
            resource_import:       Set(form_data.resource_import.to_owned()),
            sort:                  Set(form_data.sort.to_owned()),
            status:                Set(form_data.status.to_owned()),
            create_time:           Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            update_time:           Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        Template::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id as i64)
    }

    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        Template::delete_many()
            .filter(template::Column::Id.is_in(ids))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
    
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, form_data: &TemplateSaveDTO) -> Result<i64, DbErr> {
        let payload = template::ActiveModel {
            category_id:         Set(form_data.category_id),
            name:                Set(form_data.name.to_owned()),
            template_folder:     Set(form_data.template_folder.to_owned()),
            remark:              Set(form_data.remark.to_owned()),
            price:               Set(form_data.price.to_owned()),
            promotion_price:     Set(form_data.promotion_price.to_owned()),
            companyname:         Set(form_data.companyname.to_owned()),
            username:            Set(form_data.username.to_owned()),
            siteurl:             Set(form_data.siteurl.to_owned()),
            terminal_mobile:     Set(form_data.terminal_mobile.to_owned()),
            terminal_pc:         Set(form_data.terminal_pc.to_owned()),
            terminal_ipad:       Set(form_data.terminal_ipad.to_owned()),
            terminal_display:    Set(form_data.terminal_display.to_owned()),
            iscommon:            Set(form_data.iscommon.to_owned()),
            wscso_down_url:      Set(form_data.wscso_down_url.to_owned()),
            zip_down_url:        Set(form_data.zip_down_url.to_owned()),
            preview_pic:         Set(form_data.preview_pic.to_owned()),
            resource_import:     Set(form_data.resource_import.to_owned()),
            sort:                Set(form_data.sort.to_owned()),
            status:              Set(form_data.status.to_owned()),
            update_time:         Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        let update_result: UpdateResult = Template::update_many()
            .set(payload)
            .filter(template::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }

    /// 检测名称是否唯一
    pub async fn find_by_name_unique(db: &DbConn, name: &Option<String>, id: &Option<i64>) -> Result<i64, DbErr> {
        Template::find()
            .filter(template::Column::Name.eq(name.clone().unwrap_or_default()))
            .apply_if(id.clone(), |query, v| {
                query.filter(template::Column::Id.ne(v))
            })
            .count(db)
            .await
            .map(|c| c as i64)
    }

    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<template::Model>, DbErr> {
        let template = Template::find_by_id(id.unwrap_or(0)).one(db).await?;
        Ok(template)
    }

    /// 查询公共模板
    pub async fn select_by_iscommon(db: &DbConn, iscommon: &Option<i32>) -> Result<Vec<template::Model>, DbErr> {
        let template = Template::find()
            .filter(template::Column::Iscommon.eq(iscommon.clone().unwrap_or_default()))
            .filter(template::Column::Status.eq(1))
            .filter(template::Column::Deleted.eq(0))
            .order_by_desc(template::Column::Id)
            .all(db)
            .await?;
        Ok(template)
    }
    
    
    pub async fn select_my_list_count(
        db: &DbConn,
        wheres: MyPageWhere,
    ) -> Result<i64, DbErr> {
        template::Entity::find()
            .join_rev(
                JoinType::LeftJoin,
                admin_template_merge::Relation::Template.def(),
            )
            .filter(admin_template_merge::Column::AdminId.eq(wheres.admin_id))
            .apply_if(wheres.name, |query, v| {
                query.filter(template::Column::Name.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(template::Column::Status.eq(v))
            })
            .count(db)
            .await
            .map(|c| c as i64)
    }

    pub async fn select_my_list_by_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        wheres: MyPageWhere,
    ) -> Result<(Vec<template::Model>, i64), DbErr> {
        let query = template::Entity::find()
            .join_rev(
                JoinType::LeftJoin,
                admin_template_merge::Relation::Template.def(),
            )
            .filter(admin_template_merge::Column::AdminId.eq(wheres.admin_id))
            .apply_if(wheres.name, |query, v| {
                query.filter(template::Column::Name.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(template::Column::Status.eq(v))
            })
            .order_by_desc(template::Column::Id);
        let sql = query.build(DbBackend::Postgres);
        log::info!("my_list_template SQL: {}", sql.to_string());

        let paginator = query.paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;
        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }


    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        template::Entity::find()
            .apply_if(wheres.name, |query, v| {
                query.filter(template::Column::Name.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(template::Column::Status.eq(v))
            })
            .count(db)
            .await
            .map(|c| c as i64)
    }

    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        wheres: PageWhere,
    ) -> Result<(Vec<template::Model>, i64), DbErr> {
        let paginator = template::Entity::find()
            .apply_if(wheres.name, |query, v| {
                query.filter(template::Column::Name.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(template::Column::Status.eq(v))
            })
            .order_by_desc(template::Column::Id)
            .paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}