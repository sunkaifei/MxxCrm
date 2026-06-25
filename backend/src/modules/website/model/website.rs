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
use crate::modules::website::entity::{website, website::Entity as Site};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};
use sea_orm::prelude::DateTime;
use sea_orm::*;


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SiteSaveRequest {
    /// 站点名字
    pub site_name: Option<String>,
    /// 用户id
    pub user_id: Option<i64>,
    /// 是否在首页显示Banner  1显示 0不显示
    pub show_banner: Option<i32>,
    /// 模版名称
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub template_id: Option<i64>,
    /// 二级域名
    pub domain: Option<String>,
    /// PC端的LOGO
    pub logo: Option<String>,
    /// 客户端类型，1:PC，  2:WAP，3:CMS
    pub client: Option<i32>,
    /// 搜索的关键词
    pub keywords: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 绑定的域名
    pub bind_domain: Option<String>,
    /// 站点状态，1正常；2冻结
    pub status: Option<i32>,
    /// 是否是默认站点，1是默认，0不是默认,一个用户只能有一个默认的网站
    pub is_default: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 站点备注，代理商给网站的备注，方便代理商记录这个网站干嘛的
    pub remark: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}

impl From<SiteSaveRequest> for SiteSaveDTO {
    fn from(request: SiteSaveRequest) -> Self {
        SiteSaveDTO {
            id: None,
            site_name: request.site_name,
            user_id: request.user_id,
            show_banner: request.show_banner,
            template_id: request.template_id,
            domain: request.domain,
            logo: request.logo,
            client: request.client,
            keywords: request.keywords,
            description: request.description,
            bind_domain: request.bind_domain,
            status: request.status,
            is_default: request.is_default,
            sort: request.sort,
            remark: request.remark,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SiteUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 站点名字
    pub site_name: Option<String>,
    /// 用户id，0为游客
    pub user_id: Option<i64>,
    /// 是否在首页显示Banner  1显示 0不显示
    pub show_banner: Option<i32>,
    /// 模版编号
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub template_id: Option<i64>,
    /// 二级域名
    pub domain: Option<String>,
    /// PC端的LOGO
    pub logo: Option<String>,
    /// 客户端类型，1:PC，  2:WAP，3:CMS
    pub client: Option<i32>,
    /// 搜索的关键词
    pub keywords: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 绑定的域名
    pub bind_domain: Option<String>,
    /// 站点状态，1正常；2冻结
    pub status: Option<i32>,
    /// 是否是默认站点，1是默认，0不是默认,一个用户只能有一个默认的网站
    pub is_default: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 站点备注，代理商给网站的备注，方便代理商记录这个网站干嘛的
    pub remark: Option<String>,
}

impl From<SiteUpdateRequest> for SiteSaveDTO {
    fn from(request: SiteUpdateRequest) -> Self {
        SiteSaveDTO {
            id: request.id,
            site_name: request.site_name,
            user_id: request.user_id,
            show_banner: request.show_banner,
            template_id: request.template_id,
            domain: request.domain,
            logo: request.logo,
            client: request.client,
            keywords: request.keywords,
            description: request.description,
            bind_domain: request.bind_domain,
            status: request.status,
            is_default: request.is_default,
            sort: request.sort,
            remark: request.remark,
        }
    }
}

pub struct SiteSaveDTO {
    /// 主键
    pub id: Option<i64>,
    /// 站点名字
    pub site_name: Option<String>,
    /// 用户id
    pub user_id: Option<i64>,
    /// 是否在首页显示Banner  1显示 0不显示
    pub show_banner: Option<i32>,
    /// 模版名称
    pub template_id: Option<i64>,
    /// 二级域名
    pub domain: Option<String>,
    /// PC端的LOGO
    pub logo: Option<String>,
    /// 客户端类型，1:PC，  2:WAP，3:CMS
    pub client: Option<i32>,
    /// 搜索的关键词
    pub keywords: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 绑定的域名
    pub bind_domain: Option<String>,
    /// 站点状态，1正常；2冻结
    pub status: Option<i32>,
    /// 是否是默认站点，1是默认，0不是默认,一个用户只能有一个默认的网站
    pub is_default: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 站点备注，代理商给网站的备注，方便代理商记录这个网站干嘛的
    pub remark: Option<String>,
}


/// 管理员查看站点信息实体
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SiteAdminListVO {
    /// 主键
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id:  Option<i64>,
    /// 用户id
    pub user_id: Option<i64>,
    /// 站点名字
    pub site_name: Option<String>,
    /// 用户名
    pub user_name: Option<String>,
    /// 二级域名
    pub domain: Option<String>,
    /// 模版编号
    pub template_id: Option<i64>,
    /// PC端的LOGO
    pub logo: Option<String>,
    /// 客户端类型，1:PC，  2:WAP，3:CMS
    pub client: Option<i32>,
    /// 站点状态，1正常；2冻结
    pub status: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 是否是默认站点，1是默认，0不是默认,一个用户只能有一个默认的网站
    pub is_default: Option<i32>,
    /// 站点备注，代理商给网站的备注，方便代理商记录这个网站干嘛的
    pub remark: Option<String>,
    /// 创建时间
    pub create_time: Option<String>,
}

impl From<website::Model> for SiteAdminListVO {
    fn from(model: website::Model) -> Self {
        SiteAdminListVO {
            id: Option::from(model.id),
            user_id: model.user_id,
            site_name: model.site_name,
            user_name: None,
            domain: model.domain,
            template_id: model.template_id,
            logo: model.logo,
            client: model.client,
            status: model.status,
            sort: model.sort,
            is_default: model.is_default,
            remark: model.remark,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// 管理员查看站点信息实体
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SiteDetailVO {
    /// 主键
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 站点名字
    pub site_name: Option<String>,
    /// 用户id，0为游客
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub user_id: Option<i64>,
    /// 是否在首页显示Banner  1显示 0不显示
    pub show_banner: Option<i32>,
    /// 模版名称
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub template_id: Option<i64>,
    /// 二级域名
    pub domain: Option<String>,
    /// PC端的LOGO
    pub logo: Option<String>,
    /// 客户端类型，1:PC，  2:WAP，3:CMS
    pub client: Option<i32>,
    /// 搜索的关键词
    pub keywords: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 绑定的域名
    pub bind_domain: Option<String>,
    /// 站点状态，1正常；2冻结
    pub status: Option<i32>,
    /// 是否是默认站点，1是默认，0不是默认,一个用户只能有一个默认的网站
    pub is_default: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 站点备注，代理商给网站的备注，方便代理商记录这个网站干嘛的
    pub remark: Option<String>,
    /// 创建时间
    pub create_time: Option<String>,
}

impl From<website::Model> for SiteDetailVO {
    fn from(arg: website::Model) -> Self {
        Self {
            id: Option::from(arg.id),
            site_name: arg.site_name,
            user_id: arg.user_id,
            show_banner: arg.show_banner,
            template_id: arg.template_id,
            domain: arg.domain,
            logo: arg.logo,
            client: arg.client,
            keywords: arg.keywords,
            description: arg.description,
            bind_domain: arg.bind_domain,
            status: arg.status,
            is_default: arg.is_default,
            sort: arg.sort,
            remark: arg.remark,
            create_time: arg.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }

}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WebSiteVO {
    /// 主键
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 站点名字
    pub site_name: Option<String>,
}

impl From<SiteDetailVO> for WebSiteVO {
    fn from(arg: SiteDetailVO) -> Self {
        Self {
            id: Option::from(arg.id),
            site_name: arg.site_name,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatusRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    pub user_id: Option<i64>,
    pub status: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateStatusDTO {
    pub id: Option<i64>,
    pub status: Option<i32>,
}

impl From<UpdateStatusRequest> for UpdateStatusDTO {
    fn from(arg: UpdateStatusRequest) -> Self {
        Self {
            id: arg.id,
            status: arg.status,
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDefaultRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    pub is_default: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateDefaultDTO {
    pub id: Option<i64>,
    pub user_id: Option<i64>,
    pub is_default: Option<i32>,
}

impl From<UpdateDefaultRequest> for UpdateDefaultDTO {
    fn from(arg: UpdateDefaultRequest) -> Self {
        Self {
            id: arg.id,
            user_id: None,
            is_default: arg.is_default,
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub keywords: Option<String>,
    pub user_id: Option<i64>,
    pub bind_domain: Option<String>,
    pub domain: Option<String>,
    pub status: Option<i32>,
    
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
}

/// 条件
#[derive(Clone)]
pub struct PageWhere {
    pub site_name: Option<String>,
    pub user_id: Option<i64>,
    pub bind_domain: Option<String>,
    pub domain: Option<String>,
    pub status: Option<i32>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut site_name = None;
        if self.site_name != Some("".to_string()) {
            site_name = self.site_name.clone();
        }

        let mut user_id = None;
        if self.user_id > Some(0) {
            user_id = self.user_id.clone();
        }

        let mut bind_domain = None;
        if self.bind_domain != Some("".to_string()) {
            bind_domain = self.bind_domain.clone();
        }

        let mut domain = None;
        if self.domain != Some("".to_string()) {
            domain = self.domain.clone();
        }

        let mut status = None;
        if self.status == Some(0) {
            status = self.status;
        }

        Self {
            site_name,
            user_id,
            bind_domain,
            domain,
            status,
        }
    }
}

pub struct SiteModel;

impl SiteModel {

    /// 添加网站数据
    /// * `db` 数据库链接
    pub async fn insert(db: &DbConn, form_data: &SiteSaveDTO) -> Result<i64, DbErr> {
        let payload = website::ActiveModel {
            user_id:         Set(form_data.user_id.to_owned()),
            template_id:     Set(form_data.template_id.to_owned()),
            site_name:       Set(form_data.site_name.to_owned()),
            logo:            Set(form_data.logo.to_owned()),
            client:          Set(form_data.client.to_owned()),
            bind_domain:     Set(form_data.bind_domain.to_owned()),
            domain:          Set(form_data.domain.to_owned()),
            keywords:        Set(form_data.keywords.to_owned()),
            description:     Set(form_data.description.to_owned()),
            show_banner:     Set(form_data.show_banner.to_owned()),
            is_default:      Set(form_data.is_default.to_owned()),
            sort:            Set(form_data.sort.to_owned()),
            status:          Set(form_data.status.to_owned()),
            remark:          Set(form_data.remark.to_owned()),
            create_time:     Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            update_time:     Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        
        Site::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id as i64)
    }
    
    /// # 按id删除网站数据
    pub async fn delete_by_id(db: &DbConn, id: i64)-> Result<i64, DbErr> {
        Site::delete_by_id(id)
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }


    /// 按id批量删除
    /// * `db` 数据库链接
    /// * `ids` id数组
    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        Site::delete_many()
            .filter(website::Column::Id.is_in(ids))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
    
    /// # 更新网站数据
    /// * `db` 数据库链接
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, form_data: &SiteSaveDTO) -> Result<i64, DbErr> {
        let payload = website::ActiveModel {
            user_id:         Set(form_data.user_id.to_owned()),
            template_id:     Set(form_data.template_id.to_owned()),
            site_name:       Set(form_data.site_name.to_owned()),
            logo:            Set(form_data.logo.to_owned()),
            client:          Set(form_data.client.to_owned()),
            bind_domain:     Set(form_data.bind_domain.to_owned()),
            domain:          Set(form_data.domain.to_owned()),
            keywords:        Set(form_data.keywords.to_owned()),
            description:     Set(form_data.description.to_owned()),
            show_banner:     Set(form_data.show_banner.to_owned()),
            sort:            Set(form_data.sort.to_owned()),
            status:          Set(form_data.status.to_owned()),
            remark:          Set(form_data.remark.to_owned()),
            update_time:     Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        let update_result: UpdateResult = Site::update_many()
            .set(payload)
            .filter(website::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }

    /// # 更新网站为假删除状态
    /// * `db` 数据库链接
    pub async fn batch_update_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        let payload = website::ActiveModel {
            deleted:         Set(Some(1).to_owned()),
            ..Default::default()
        };

        let update_result: UpdateResult = Site::update_many()
            .set(payload)
            .filter(website::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }

    
    /// 更新状态
    /// * `db` 数据库链接
    pub async fn update_status_by_id(db: &DbConn, form_data: UpdateStatusDTO) -> Result<i64, DbErr> {
        let payload = website::ActiveModel {
            status:  Set(form_data.status.to_owned()),
            ..Default::default()
        };
        let update_result: UpdateResult = Site::update_many()
            .set(payload)
            .filter(website::Column::Id.eq(form_data.id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// 更新默认站点
    /// * `db` 数据库链接
    /// * `user_id` 需要重置默认站点的用户id
    pub async fn update_by_reset_default<C: ConnectionTrait>(db: &C, user_id: &Option<i64>) -> Result<i64, DbErr> {
        let payload = website::ActiveModel {
            is_default:  Set(Some(0).to_owned()),
            ..Default::default()
        };
        
        let update_result: UpdateResult = Site::update_many()
            .set(payload)
            .filter(website::Column::UserId.eq(user_id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        
        Ok(update_result.rows_affected as i64)
     }

    /// 更新默认站点
    /// * `db` 数据库链接
    /// * `id` 需要更新默认站点的站点id
    pub async fn update_by_default_id<C: ConnectionTrait>(db: &C, id: &Option<i64>) -> Result<i64, DbErr> {
        let payload = website::ActiveModel {
            is_default:  Set(Some(1).to_owned()),
            ..Default::default()
        };

        let update_result: UpdateResult = Site::update_many()
            .set(payload)
            .filter(website::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }
    

    /// 查询站点名称是否已存在
    /// * `db` 数据库链接
    /// * `site_name` 站点名称
    /// * `user_id` 用户id
    /// * `id` 是否排除当前id
    pub async fn find_by_name_count(
        db: &DbConn, 
        site_name: &Option<String>, 
        user_id: &Option<i64>, 
        id: &Option<i64>
    ) -> Result<i64, DbErr> {
        Site::find()
            .filter(website::Column::SiteName.eq(site_name.clone().unwrap_or_default()))
            .filter(website::Column::UserId.eq(user_id.clone().unwrap_or_default()))
            .apply_if(id.clone(), |query, v| {
                query.filter(website::Column::Id.ne(v))
            })
            .count(db)
            .await
            .map(|c| c as i64)
     }

    /// 查询绑定二级域名是否已存在
    /// * `db` 数据库链接
    pub async fn find_by_domain_count(db: &DbConn, domain: &Option<String>, id: &Option<i64>) -> Result<i64, DbErr> {
        Site::find()
            .filter(website::Column::Domain.eq(domain.clone().unwrap_or_default()))
            .filter(website::Column::Id.ne(id.clone().unwrap_or_default()))
            .count(db)
            .await
            .map(|c| c as i64)
    }

    /// 查询绑定的域名是否已存在
    /// * `db` 数据库链接
    /// * `bind_domain` 绑定域名
    /// * `id` id
    pub async fn find_by_bind_domain_count(db: &DbConn, bind_domain: &Option<String>, id: &Option<i64>) -> Result<i64, DbErr> {
        Site::find()
            .filter(website::Column::BindDomain.eq(bind_domain.clone().unwrap_or_default()))
            .filter(website::Column::Id.ne(id.clone().unwrap_or_default()))
            .count(db)
            .await
            .map(|c| c as i64)
    }


    /// 查询默认站点是否已存在
    /// * `db` 数据库链接
    /// * `user_id` 当前用户id
    /// * `is_default` 默认站点设置
    pub async fn find_by_default_count(db: &DbConn, user_id: &Option<i64>, is_default: &Option<i32>) -> Result<i64, DbErr> {
        Site::find()
            .filter(website::Column::UserId.eq(user_id.clone().unwrap_or_default()))
            .filter(website::Column::IsDefault.eq(is_default.clone().unwrap_or_default()))
            .count(db)
            .await
            .map(|c| c as i64)
    }
    
    
    /// 根据id查询站点信息
    /// * `db` 数据库链接
    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<website::Model>, DbErr> {
        Site::find_by_id(id.clone().unwrap_or_default())
            .one(db)
            .await
    }
    
    /// 根据用户id查询站点信息
    /// * `db` 数据库链接
    pub async fn find_by_user_id(db: &DbConn, user_id: &Option<i64>) -> Result<Option<website::Model>, DbErr> {
        Site::find()
            .filter(website::Column::UserId.eq(user_id.clone().unwrap_or_default()))
            .filter(website::Column::IsDefault.eq(1))
            .order_by_desc(website::Column::Id)
            .one(db)
            .await
    }
    
    ///用户所有默认站点列表
    /// * `db` 数据库链接
    pub async fn find_by_user_default_list(db: &DbConn, user_id: &Option<i64>) -> Result<Vec<website::Model>, DbErr> {
        Site::find()
            .filter(website::Column::UserId.eq(user_id.clone().unwrap_or_default()))
            .filter(website::Column::IsDefault.eq(1))
            .order_by_desc(website::Column::Id)
            .all(db)
            .await
    }

    
    /// 根据二级域名查询站点信息
    /// * `db` 数据库链接
    pub async fn find_by_domain(db: &DbConn, domain: &Option<String>) -> Result<Option<website::Model>, DbErr> {
        Site::find()
            .filter(website::Column::Domain.eq(domain.clone().unwrap_or_default()))
            .one(db)
            .await
    }

    
    /// 查询站点列表统计
    /// * `db` 数据库链接
    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        Site::find()
            .apply_if(wheres.site_name, |query, v| {
                query.filter(website::Column::SiteName.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.user_id, |query, v| {
                query.filter(website::Column::UserId.eq(v))
            })
            .apply_if(wheres.bind_domain, |query, v| {
                query.filter(website::Column::BindDomain.eq(v))
            })
            .apply_if(wheres.domain, |query, v| {
                query.filter(website::Column::Domain.eq(v))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(website::Column::Status.eq(v))
            })
            .count(db)
            .await
            .map(|c| c as i64)
    }


    /// 分页查询站点列表
    /// * `db` 数据库链接
    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        wheres: PageWhere,
    ) -> Result<(Vec<website::Model>, i64), DbErr> {
        let paginator = Site::find()
            .apply_if(wheres.site_name, |query, v| {
                query.filter(website::Column::SiteName.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.user_id, |query, v| {
                query.filter(website::Column::UserId.eq(v))
            })
            .apply_if(wheres.bind_domain, |query, v| {
                query.filter(website::Column::BindDomain.eq(v))
            })
            .apply_if(wheres.domain, |query, v| {
                query.filter(website::Column::Domain.eq(v))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(website::Column::Status.eq(v))
            })
            .order_by_desc(website::Column::Id)
            .paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;
        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}
