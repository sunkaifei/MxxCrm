//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::modules::website::entity::{website_links, website_links::Entity as WebsiteLinks};
use sea_orm::prelude::{DateTime};
use sea_orm::*;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};



#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LinkSaveRequest {

    /// 链接类型：0文字链接，1logo链接
    #[serde(default)]
    pub link_type: Option<i32>,
    /// 网站名称
    #[serde(default)]
    pub link_name: Option<String>,
    /// 网站地址
    #[serde(default)]
    pub link_url: Option<String>,
    /// 网站logo地址
    #[serde(default)]
    pub link_logo: Option<String>,
    /// 显示状态：0不显示，1显示
    #[serde(default)]
    pub status: Option<i32>,
    /// 排序序号
    #[serde(default)]
    pub sort: Option<i32>,
    /// 创建时间
    pub create_time: Option<DateTime>,
}

impl From<LinkSaveRequest> for LinkSaveDTO {
    fn from(req: LinkSaveRequest) -> Self {
        LinkSaveDTO {
            id: None,
            website_id: None,
            link_type: req.link_type,
            link_name: req.link_name,
            link_url: req.link_url,
            link_logo: req.link_logo,
            status: req.status,
            sort: req.sort,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LinkUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 链接类型：0文字链接，1logo链接
    #[serde(default)]
    pub link_type: Option<i32>,
    /// 网站名称
    #[serde(default)]
    pub link_name: Option<String>,
    /// 网站地址
    #[serde(default)]
    pub link_url: Option<String>,
    /// 网站logo地址
    #[serde(default)]
    pub link_logo: Option<String>,
    /// 显示状态：0不显示，1显示
    #[serde(default)]
    pub status: Option<i32>,
    /// 排序序号
    #[serde(default)]
    pub sort: Option<i32>,
}

impl From<LinkUpdateRequest> for LinkSaveDTO {
    fn from(req: LinkUpdateRequest) -> Self {
        LinkSaveDTO {
            id: req.id,
            website_id: None,
            link_type: req.link_type,
            link_name: req.link_name,
            link_url: req.link_url,
            link_logo: req.link_logo,
            status: req.status,
            sort: req.sort,
        }
   }
}


pub struct LinkSaveDTO {
    pub id: Option<i64>,
    /// 网站id
    pub website_id: Option<i64>,
    /// 链接类型：0文字链接，1logo链接
    pub link_type: Option<i32>,
    /// 网站名称
    pub link_name: Option<String>,
    /// 网站地址
    pub link_url: Option<String>,
    /// 网站logo地址
    pub link_logo: Option<String>,
    /// 显示状态：0不显示，1显示
    pub status: Option<i32>,
    /// 排序序号
    pub sort: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LinkListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 网站id
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub website_id: Option<i64>,
    /// 链接类型：0文字链接，1logo链接
    #[serde(default)]
    pub link_type: Option<i32>,
    /// 网站名称
    #[serde(default)]
    pub link_name: Option<String>,
    /// 网站地址
    #[serde(default)]
    pub link_url: Option<String>,
    /// 网站logo地址
    #[serde(default)]
    pub link_logo: Option<String>,
    /// 显示状态：0不显示，1显示
    #[serde(default)]
    pub status: Option<i32>,
    /// 排序序号
    #[serde(default)]
    pub sort: Option<i32>,
    /// 创建时间
    pub create_time: Option<String>,
}

impl From<website_links::Model> for LinkListVO {
    fn from(model: website_links::Model) -> Self {
        Self {
            id: Option::from(model.id),
            website_id: model.website_id,
            link_type: model.link_type,
            link_name: model.link_name,
            link_url: model.link_url,
            link_logo: model.link_logo,
            status: model.status,
            sort: model.sort,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LinkDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 网站id
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub website_id: Option<i64>,
    /// 链接类型：0文字链接，1logo链接
    #[serde(default)]
    pub link_type: Option<i32>,
    /// 网站名称
    #[serde(default)]
    pub link_name: Option<String>,
    /// 网站地址
    #[serde(default)]
    pub link_url: Option<String>,
    /// 网站logo地址
    #[serde(default)]
    pub link_logo: Option<String>,
    /// 显示状态：0不显示，1显示
    #[serde(default)]
    pub status: Option<i32>,
    /// 排序序号
    #[serde(default)]
    pub sort: Option<i32>,
    /// 创建时间
    pub create_time: Option<String>,
}

impl From<website_links::Model> for LinkDetailVO {
    fn from(model: website_links::Model) -> Self {
        Self {
            id: Option::from(model.id),
            website_id: model.website_id,
            link_type: model.link_type,
            link_name: model.link_name,
            link_url: model.link_url,
            link_logo: model.link_logo,
            status: model.status,
            sort: model.sort,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// 查询条件
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery{
    pub website_id: Option<i64>,
    pub link_name: Option<String>,
    pub link_type: Option<i32>,
    pub link_url: Option<String>,
    pub status: Option<i32>,
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Clone)]
pub struct PageWhere {
    pub website_id: Option<i64>,
    pub link_name: Option<String>,
    pub link_type: Option<i32>,
    pub link_url: Option<String>,
    pub status: Option<i32>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut website_id = None;
        if self.website_id >= Some(0) {
            website_id = self.website_id.clone();
        }

        let mut link_name = None;
        if self.link_name != Some("".to_string()) {
            link_name = self.link_name.clone();
        }

        let mut link_type = None;
        if self.link_type == Some(1) || self.link_type == Some(0) {
            link_type = self.link_type.clone();
        }

        let mut link_url = None;
        if self.link_url != Some("".to_string()) {
            link_url = self.link_url.clone();
        }

        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        Self {
            website_id,
            link_name,
            link_type,
            link_url,
            status,
        }
    }
}

pub struct WebsiteLinksModel;

impl WebsiteLinksModel {

    pub async fn insert(db: &DbConn, form_data: &LinkSaveDTO) -> Result<i64, DbErr> {
        let payload = website_links::ActiveModel {
            id:              Set(form_data.id.unwrap_or_default().to_owned()),
            website_id:       Set(form_data.website_id.to_owned()),
            link_name:        Set(form_data.link_name.to_owned()),
            link_type:        Set(form_data.link_type.to_owned()),
            link_url:         Set(form_data.link_url.to_owned()),
            link_logo:        Set(form_data.link_logo.to_owned()),
            status:           Set(form_data.status.to_owned()),
            sort:             Set(form_data.sort.to_owned()),
            create_time:      Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        log::info!("insert website_links: {:?}", payload);

        WebsiteLinks::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        WebsiteLinks::delete_many()
            .filter(website_links::Column::Id.is_in(ids))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, form_data: &LinkSaveDTO) -> Result<i64, DbErr> {
        let payload = website_links::ActiveModel {
            link_name:        Set(form_data.link_name.to_owned()),
            link_type:        Set(form_data.link_type.to_owned()),
            link_url:         Set(form_data.link_url.to_owned()),
            link_logo:        Set(form_data.link_logo.to_owned()),
            status:           Set(form_data.status.to_owned()),
            sort:             Set(form_data.sort.to_owned()),
            ..Default::default()
        };

        let update_result: UpdateResult = WebsiteLinks::update_many()
            .set(payload)
            .filter(website_links::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }


    /// 批量更新假删除状态
    pub async fn batch_update_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        let update_result: UpdateResult = WebsiteLinks::update_many()
            .set(website_links::ActiveModel {
                deleted: Set(Some(1).to_owned()),
                ..Default::default()
            })
            .filter(website_links::Column::Id.is_in(ids))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }


    pub async fn find_by_link_url_unique(db: &DbConn, link_url: &Option<String>, id: &Option<i64>) -> Result<i64, DbErr> {
        WebsiteLinks::find()
            .filter(website_links::Column::LinkUrl.eq(link_url.clone().unwrap_or_default()))
            .apply_if(id.clone(), |query, v| {
                query.filter(website_links::Column::Id.ne(v))
            })
            .count(db)
            .await
            .map(|c| c as i64)
    }

    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<website_links::Model>, DbErr> {
        let website_links = WebsiteLinks::find_by_id(id.clone().unwrap_or_default())
            .one(db)
            .await?;
        Ok(website_links)
    }


    pub async fn find_by_website_id(db: &DbConn, website_id: &Option<i64>) -> Result<Vec<website_links::Model>, DbErr> {
        let website_links = WebsiteLinks::find()
            .filter(website_links::Column::WebsiteId.eq(website_id.clone().unwrap_or_default()))
            .all(db)
            .await?;
        Ok(website_links)
    }

    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        WebsiteLinks::find()
            .apply_if(wheres.website_id, |query, v| {
                query.filter(website_links::Column::WebsiteId.eq(v))
            })
            .apply_if(wheres.link_name, |query, v| {
                query.filter(website_links::Column::LinkName.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.link_type, |query, v| {
                query.filter(website_links::Column::LinkType.eq(v))
            })
            .apply_if(wheres.link_url, |query, v| {
                query.filter(website_links::Column::LinkUrl.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(website_links::Column::Status.eq(v))
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
    ) -> Result<(Vec<website_links::Model>, i64), DbErr> {
        let query = WebsiteLinks::find()
            .apply_if(wheres.website_id, |query, v| {
                query.filter(website_links::Column::WebsiteId.eq(v))
            })
            .apply_if(wheres.link_name, |query, v| {
                query.filter(website_links::Column::LinkName.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.link_type, |query, v| {
                query.filter(website_links::Column::LinkType.eq(v))
            })
            .apply_if(wheres.link_url, |query, v| {
                query.filter(website_links::Column::LinkUrl.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(website_links::Column::Status.eq(v))
            })
            .order_by_desc(website_links::Column::Id);

        let sql = query.build(DbBackend::Postgres);
        log::info!("Generated SQL: {}", sql.to_string());

        let paginator = query.paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}