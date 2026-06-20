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
use sea_orm::{QueryOrder, QueryTrait, Set, UpdateResult};
use serde::{Deserialize, Serialize};
use crate::modules::system::entity::{admin, ip_address, ip_address::Entity as IpAddress};
use crate::utils::string_utils::{u64_to_string, serialize_option_u64_to_string, deserialize_string_to_u64};


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IpAddressSaveRequest {
    /// IP段起始
    pub start_ip: Option<u32>,
    /// IP段结束
    pub end_ip: Option<u32>,
    /// 国家
    pub country: Option<String>,
    /// 省份
    pub province: Option<String>,
    /// 市级地区
    pub city: Option<String>,
    /// 县
    pub county: Option<String>,
    /// 详细地址
    pub address: Option<String>,
    /// 地址描述
    pub local: Option<String>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IpAddressUpdateRequest {
    /// IP段ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// IP段起始
    pub start_ip: Option<u32>,
    /// IP段结束
    pub end_ip: Option<u32>,
    /// 国家
    pub country: Option<String>,
    /// 省份
    pub province: Option<String>,
    /// 市级地区
    pub city: Option<String>,
    /// 县
    pub county: Option<String>,
    /// 详细地址
    pub address: Option<String>,
    /// 地址描述
    pub local: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id:  Option<i64>,
    /// IP段起始
    pub start_ip: Option<String>,
    /// IP段结束
    pub end_ip: Option<String>,
    /// 国家
    pub country: Option<String>,
    /// 省份
    pub province: Option<String>,
    /// 市级地区
    pub city: Option<String>,
    /// 县
    pub county: Option<String>,
    /// 详细地址
    pub address: Option<String>,
    /// 地址描述
    pub local: Option<String>,
}

impl From<ip_address::Model> for IpListVO {
    fn from(model: ip_address::Model) -> Self {
        Self {
            id: Some(model.id),
            start_ip: None,
            end_ip: None,
            country: model.country,
            province: model.province,
            city: model.city,
            county: model.county,
            address: model.address,
            local: model.local,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    /// IP段起始
    pub start_ip: Option<String>,
    /// IP段结束
    pub end_ip: Option<String>,
    /// 国家
    pub country: Option<String>,
    /// 省份
    pub province: Option<String>,
    /// 市级地区
    pub city: Option<String>,
    /// 县
    pub county: Option<String>,
    // 当前页码数
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    // 每页条数
    pub page_size: Option<i64>,
}

/// 条件
#[derive(Clone)]
pub struct PageWhere {
    /// IP段起始
    pub start_ip: Option<u32>,
    /// IP段结束
    pub end_ip: Option<u32>,
    /// 国家
    pub country: Option<String>,
    /// 省份
    pub province: Option<String>,
    /// 市级地区
    pub city: Option<String>,
    /// 县
    pub county: Option<String>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut start_ip = None;
        if self.start_ip > Some(0) {
            start_ip = self.start_ip.clone();
        }

        let mut end_ip = None;
        if self.end_ip > Some(0) {
            end_ip = self.end_ip.clone();
        }

        let mut country = None;
        if self.country != Some("".to_string()) {
            country = self.country.clone();
        }

        let mut province = None;
        if self.country != Some("".to_string()) {
            province = self.province.clone();
        }

        let mut city = None;
        if self.city != Some("".to_string()) {
            city = self.city.clone();
        }

        let mut county = None;
        if self.county != Some("".to_string()) {
            county = self.county.clone();
        }


        Self {
            start_ip,
            end_ip,
            country,
            province,
            city,
            county,
        }
    }
}

pub struct IpAddressModel;

impl IpAddressModel {
    pub async fn insert(db: &DbConn, form_data: IpAddressSaveRequest) -> Result<i64, DbErr> {
        let entity = ip_address::ActiveModel {
            start_ip:   Set(form_data.start_ip.to_owned()),
            end_ip:     Set(form_data.end_ip.to_owned()),
            country:    Set(form_data.country.to_owned()),
            province:   Set(form_data.province.to_owned()),
            city:       Set(form_data.city.to_owned()),
            county:     Set(form_data.county.to_owned()),
            address:    Set(form_data.address.to_owned()),
            ..Default::default()
        };
        
        IpAddress::insert(entity)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }
    
    /// 批量删除
    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        IpAddress::delete_many()
            .filter(ip_address::Column::Id.is_in(ids))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn update(db: &DbConn, form_data: IpAddressUpdateRequest) -> Result<i64, DbErr> {
        let payload = ip_address::ActiveModel {
            start_ip:   Set(form_data.start_ip.to_owned()),
            end_ip:     Set(form_data.end_ip.to_owned()),
            country:    Set(form_data.country.to_owned()),
            province:   Set(form_data.province.to_owned()),
            city:       Set(form_data.city.to_owned()),
            county:     Set(form_data.county.to_owned()),
            address:    Set(form_data.address.to_owned()),
            local:      Set(form_data.local.to_owned()),
            ..Default::default()
        };
        let update_result: UpdateResult = IpAddress::update_many()
            .set(payload)
            .filter(admin::Column::Id.eq(form_data.id))
            .exec(db).await?;
        Ok(update_result.rows_affected as i64)
    }
    
    /// 根据ID查询
    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<ip_address::Model>, DbErr> {
        IpAddress::find_by_id(id).one(db).await
    }

    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        IpAddress::find()
            .apply_if(wheres.start_ip, |query, v| {
                query.filter(ip_address::Column::StartIp.eq(v))
            })
            .apply_if(wheres.end_ip, |query, v| {
                query.filter(ip_address::Column::EndIp.eq(v))
            })
            .apply_if(wheres.country, |query, v| {
                query.filter(ip_address::Column::Country.eq(v))
            })
            .apply_if(wheres.province, |query, v| {
                query.filter(ip_address::Column::Province.eq(v))
            })
            .apply_if(wheres.city, |query, v| {
                query.filter(ip_address::Column::City.eq(v))
            })
            .apply_if(wheres.county, |query, v| {
                query.filter(ip_address::Column::County.eq(v))
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
    ) -> Result<(Vec<ip_address::Model>, i64), DbErr> {
        let paginator = IpAddress::find()
            .apply_if(wheres.start_ip, |query, v| {
                query.filter(ip_address::Column::StartIp.eq(v))
            })
            .apply_if(wheres.end_ip, |query, v| {
                query.filter(ip_address::Column::EndIp.eq(v))
            })
            .apply_if(wheres.country, |query, v| {
                query.filter(ip_address::Column::Country.eq(v))
            })
            .apply_if(wheres.province, |query, v| {
                query.filter(ip_address::Column::Province.eq(v))
            })
            .apply_if(wheres.city, |query, v| {
                query.filter(ip_address::Column::City.eq(v))
            })
            .apply_if(wheres.county, |query, v| {
                query.filter(ip_address::Column::County.eq(v))
            })
            .order_by_asc(ip_address::Column::Id)
            .paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}