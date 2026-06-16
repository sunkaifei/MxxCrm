//! Shop Model
//! 店铺数据模型

use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::shop::entity::shop::{self, Entity as ShopEntity};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};
use sea_orm::prelude::DateTime;
use sea_orm::*;
use rust_decimal::prelude::ToPrimitive;

/// Shop Save Request
/// 店铺保存请求结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShopSaveRequest {
    /// 用户ID
    pub user_id: Option<String>,
    /// 店铺LOGO
    pub store_logo: Option<String>,
    /// 店铺名称
    pub store_name: Option<String>,
    /// 是否自营: 0=否, 1=是
    pub self_operated: Option<i16>,
    /// 店铺详细地址
    pub store_address_detail: Option<String>,
    /// 店铺地址ID路径
    pub store_address_id_path: Option<String>,
    /// 店铺地址路径
    pub store_address_path: Option<String>,
    /// 店铺营业结束时间
    pub store_end_time: Option<String>,
    /// 店铺状态: 0=正常, 1=关闭
    pub store_disable: Option<i16>,
    /// 店铺中心位置
    pub store_center: Option<String>,
    /// 店铺简介
    pub store_desc: Option<String>,
    /// 配送评分
    pub delivery_score: Option<f64>,
    /// 描述评分
    pub description_score: Option<f64>,
    /// 服务评分
    pub service_score: Option<f64>,
    /// 商品数量
    pub goods_num: Option<i32>,
    /// 收藏数量
    pub collection_num: Option<i32>,
    /// 银豹小程序签名
    pub yzf_mp_sign: Option<String>,
    /// 银豹签名
    pub yzf_sign: Option<String>,
    /// 商家EUID
    pub merchant_euid: Option<String>,
    /// 是否页面展示: 0=否, 1=是
    pub page_show: Option<i16>,
    /// 是否支持自提: 0=否, 1=是
    pub self_pick_flag: Option<i16>,
}

impl From<ShopSaveRequest> for ShopSaveDTO {
    fn from(req: ShopSaveRequest) -> Self {
        ShopSaveDTO {
            id: None,
            store_logo: req.store_logo,
            store_name: req.store_name,
            self_operated: req.self_operated.unwrap_or(0),
            store_address_detail: req.store_address_detail,
            store_address_id_path: req.store_address_id_path,
            store_address_path: req.store_address_path,
            store_end_time: req.store_end_time.and_then(|s| chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").ok()),
            store_disable: req.store_disable,
            store_center: req.store_center,
            store_desc: req.store_desc,
            delivery_score: req.delivery_score,
            description_score: req.description_score,
            service_score: req.service_score,
            goods_num: req.goods_num,
            collection_num: req.collection_num,
            yzf_mp_sign: req.yzf_mp_sign,
            yzf_sign: req.yzf_sign,
            merchant_euid: req.merchant_euid,
            page_show: req.page_show,
            self_pick_flag: req.self_pick_flag,
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
            delete_flag: Some(0),
        }
    }
}

/// Shop Update Request
/// 店铺更新请求结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShopUpdateRequest {
    /// 主键ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 用户ID
    pub user_id: Option<String>,
    /// 店铺LOGO
    pub store_logo: Option<String>,
    /// 店铺名称
    pub store_name: Option<String>,
    /// 是否自营: 0=否, 1=是
    pub self_operated: Option<i16>,
    /// 店铺详细地址
    pub store_address_detail: Option<String>,
    /// 店铺地址ID路径
    pub store_address_id_path: Option<String>,
    /// 店铺地址路径
    pub store_address_path: Option<String>,
    /// 店铺营业结束时间
    pub store_end_time: Option<String>,
    /// 店铺状态: 0=正常, 1=关闭
    pub store_disable: Option<i16>,
    /// 店铺中心位置
    pub store_center: Option<String>,
    /// 店铺简介
    pub store_desc: Option<String>,
    /// 配送评分
    pub delivery_score: Option<f64>,
    /// 描述评分
    pub description_score: Option<f64>,
    /// 服务评分
    pub service_score: Option<f64>,
    /// 商品数量
    pub goods_num: Option<i32>,
    /// 收藏数量
    pub collection_num: Option<i32>,
    /// 银豹小程序签名
    pub yzf_mp_sign: Option<String>,
    /// 银豹签名
    pub yzf_sign: Option<String>,
    /// 商家EUID
    pub merchant_euid: Option<String>,
    /// 是否页面展示: 0=否, 1=是
    pub page_show: Option<i16>,
    /// 是否支持自提: 0=否, 1=是
    pub self_pick_flag: Option<i16>,
}

impl From<ShopUpdateRequest> for ShopSaveDTO {
    fn from(req: ShopUpdateRequest) -> Self {
        ShopSaveDTO {
            id: req.id,
            store_logo: req.store_logo,
            store_name: req.store_name,
            self_operated: req.self_operated.unwrap_or(0),
            store_address_detail: req.store_address_detail,
            store_address_id_path: req.store_address_id_path,
            store_address_path: req.store_address_path,
            store_end_time: req.store_end_time.and_then(|s| chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").ok()),
            store_disable: req.store_disable,
            store_center: req.store_center,
            store_desc: req.store_desc,
            delivery_score: req.delivery_score,
            description_score: req.description_score,
            service_score: req.service_score,
            goods_num: req.goods_num,
            collection_num: req.collection_num,
            yzf_mp_sign: req.yzf_mp_sign,
            yzf_sign: req.yzf_sign,
            merchant_euid: req.merchant_euid,
            page_show: req.page_show,
            self_pick_flag: req.self_pick_flag,
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
            delete_flag: None,
        }
    }
}

/// Shop Save DTO
/// 店铺保存数据传输对象
pub struct ShopSaveDTO {
    /// 主键ID
    pub id: Option<i64>,
    /// 店铺LOGO
    pub store_logo: Option<String>,
    /// 店铺名称
    pub store_name: Option<String>,
    /// 是否自营: 0=否, 1=是
    pub self_operated: i16,
    /// 店铺详细地址
    pub store_address_detail: Option<String>,
    /// 店铺地址ID路径
    pub store_address_id_path: Option<String>,
    /// 店铺地址路径
    pub store_address_path: Option<String>,
    /// 店铺营业结束时间
    pub store_end_time: Option<DateTime>,
    /// 店铺状态: 0=正常, 1=关闭
    pub store_disable: Option<i16>,
    /// 店铺中心位置
    pub store_center: Option<String>,
    /// 店铺简介
    pub store_desc: Option<String>,
    /// 配送评分
    pub delivery_score: Option<f64>,
    /// 描述评分
    pub description_score: Option<f64>,
    /// 服务评分
    pub service_score: Option<f64>,
    /// 商品数量
    pub goods_num: Option<i32>,
    /// 收藏数量
    pub collection_num: Option<i32>,
    /// 银豹小程序签名
    pub yzf_mp_sign: Option<String>,
    /// 银豹签名
    pub yzf_sign: Option<String>,
    /// 商家EUID
    pub merchant_euid: Option<String>,
    /// 是否页面展示: 0=否, 1=是
    pub page_show: Option<i16>,
    /// 是否支持自提: 0=否, 1=是
    pub self_pick_flag: Option<i16>,
    /// 创建人
    pub create_by: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新人
    pub update_by: Option<String>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 删除标志: 0=未删除, 1=已删除
    pub delete_flag: Option<i16>,
}

/// Shop Detail VO
/// 店铺详情视图对象
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShopDetailVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 店铺LOGO
    pub store_logo: Option<String>,
    /// 店铺名称
    pub store_name: Option<String>,
    /// 是否自营: 0=否, 1=是
    pub self_operated: Option<i16>,
    /// 店铺详细地址
    pub store_address_detail: Option<String>,
    /// 店铺地址ID路径
    pub store_address_id_path: Option<String>,
    /// 店铺地址路径
    pub store_address_path: Option<String>,
    /// 店铺营业结束时间
    pub store_end_time: Option<String>,
    /// 店铺状态: 0=正常, 1=关闭
    pub store_disable: Option<i16>,
    /// 店铺中心位置
    pub store_center: Option<String>,
    /// 店铺简介
    pub store_desc: Option<String>,
    /// 配送评分
    pub delivery_score: Option<f64>,
    /// 描述评分
    pub description_score: Option<f64>,
    /// 服务评分
    pub service_score: Option<f64>,
    /// 商品数量
    pub goods_num: Option<i32>,
    /// 收藏数量
    pub collection_num: Option<i32>,
    /// 银豹小程序签名
    pub yzf_mp_sign: Option<String>,
    /// 银豹签名
    pub yzf_sign: Option<String>,
    /// 商家EUID
    pub merchant_euid: Option<String>,
    /// 是否页面展示: 0=否, 1=是
    pub page_show: Option<i16>,
    /// 是否支持自提: 0=否, 1=是
    pub self_pick_flag: Option<i16>,
    /// 创建人
    pub create_by: Option<String>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 更新人
    pub update_by: Option<String>,
    /// 更新时间
    pub update_time: Option<String>,
}

impl From<shop::Model> for ShopDetailVO {
    fn from(model: shop::Model) -> Self {
        Self {
            id: Some(model.id as i64),
            store_logo: model.store_logo,
            store_name: model.store_name,
            self_operated: Some(model.self_operated),
            store_address_detail: model.store_address_detail,
            store_address_id_path: model.store_address_id_path,
            store_address_path: model.store_address_path,
            store_end_time: model.store_end_time.map(|dt| dt.to_string()),
            store_disable: Some(model.store_disable),
            store_center: model.store_center,
            store_desc: model.store_desc,
            delivery_score: model.delivery_score.and_then(|d| d.to_f64()),
            description_score: model.description_score.and_then(|d| d.to_f64()),
            service_score: model.service_score.and_then(|d| d.to_f64()),
            goods_num: model.goods_num,
            collection_num: model.collection_num,
            yzf_mp_sign: model.yzf_mp_sign,
            yzf_sign: model.yzf_sign,
            merchant_euid: model.merchant_euid,
            page_show: model.page_show,
            self_pick_flag: model.self_pick_flag,
            create_by: model.create_by,
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_by: model.update_by,
            update_time: model.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// Shop List VO
/// 店铺列表视图对象
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShopListVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 店铺LOGO
    pub shop_logo: Option<String>,
    /// 店铺名称
    pub shop_name: Option<String>,
    /// 店铺简介
    pub shop_desc: Option<String>,
    /// 联系人姓名
    pub contact_name: Option<String>,
    /// 联系电话
    pub contact_phone: Option<String>,
    /// 店铺状态: 0=待审核, 1=已开通, 2=已驳回, 3=已冻结
    pub status: i16,
    /// 佣金比例
    pub commission_rate: f64,
    /// 结算周期: 1=月结
    pub settlement_cycle: i16,
    /// 可结算余额(分)
    pub balance: i64,
    /// 创建时间
    pub create_time: Option<String>,
}

impl From<shop::Model> for ShopListVO {
    fn from(model: shop::Model) -> Self {
        Self {
            id: Some(model.id as i64),
            shop_logo: model.store_logo,
            shop_name: model.store_name,
            shop_desc: model.store_desc,
            contact_name: None,
            contact_phone: model.store_phone,
            status: model.status,
            commission_rate: model.commission_rate.to_f64().unwrap_or(0.0),
            settlement_cycle: model.settlement_period,
            balance: model.balance.to_i64().unwrap_or(0),
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// List Query
/// 店铺列表查询参数
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    /// 页码
    pub page: Option<i64>,
    /// 每页数量
    pub page_size: Option<i64>,
    /// 关键词(店铺名称模糊搜索)
    pub keyword: Option<String>,
    /// 店铺状态: 0=待审核, 1=已开通, 2=已驳回, 3=已冻结
    pub status: Option<i16>,
    /// 是否自营: 0=否, 1=是
    pub self_operated: Option<i16>,
}

/// PageWhere
/// 分页查询条件
#[derive(Clone)]
pub struct PageWhere {
    /// 店铺名称(模糊搜索)
    pub store_name: Option<String>,
    /// 店铺状态: 0=待审核, 1=已开通, 2=已驳回, 3=已冻结
    pub status: Option<i16>,
    /// 是否自营: 0=否, 1=是
    pub self_operated: Option<i16>,
}

impl PageWhere {
    /// 格式化查询条件(去除空值)
    pub fn format(&self) -> Self {
        let mut store_name = None;
        if let Some(ref name) = self.store_name {
            if !name.trim().is_empty() {
                store_name = Some(name.clone());
            }
        }

        Self {
            store_name,
            status: self.status,
            self_operated: self.self_operated,
        }
    }
}

/// ShopModel
/// 店铺数据操作模型
pub struct ShopModel;

impl ShopModel {
    /// 插入店铺记录
    pub async fn insert<C: ConnectionTrait>(db: &C, form_data: &ShopSaveDTO) -> Result<i64, DbErr> {
        let payload = shop::ActiveModel {
            store_logo: Set(form_data.store_logo.clone()),
            store_name: Set(form_data.store_name.clone()),
            self_operated: Set(form_data.self_operated),
            store_address_detail: Set(form_data.store_address_detail.clone()),
            store_address_id_path: Set(form_data.store_address_id_path.clone()),
            store_address_path: Set(form_data.store_address_path.clone()),
            store_end_time: Set(form_data.store_end_time),
            store_disable: Set(form_data.store_disable.unwrap_or(0)),
            store_center: Set(form_data.store_center.clone()),
            store_desc: Set(form_data.store_desc.clone()),
            delivery_score: Set(form_data.delivery_score.map(|d| rust_decimal::Decimal::from_f64_retain(d).unwrap_or_default())),
            description_score: Set(form_data.description_score.map(|d| rust_decimal::Decimal::from_f64_retain(d).unwrap_or_default())),
            service_score: Set(form_data.service_score.map(|d| rust_decimal::Decimal::from_f64_retain(d).unwrap_or_default())),
            goods_num: Set(form_data.goods_num),
            collection_num: Set(form_data.collection_num),
            yzf_mp_sign: Set(form_data.yzf_mp_sign.clone()),
            yzf_sign: Set(form_data.yzf_sign.clone()),
            merchant_euid: Set(form_data.merchant_euid.clone()),
            page_show: Set(form_data.page_show),
            self_pick_flag: Set(form_data.self_pick_flag),
            create_by: Set(form_data.create_by.clone()),
            create_time: Set(Some(chrono::Local::now().naive_local())),
            update_by: Set(form_data.update_by.clone()),
            update_time: Set(Some(chrono::Local::now().naive_local())),
            delete_flag: Set(form_data.delete_flag),
            ..Default::default()
        };

        ShopEntity::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id as i64)
    }

    /// 批量删除店铺
    pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        Ok(ShopEntity::delete_many()
            .filter(shop::Column::Id.is_in(ids.iter().map(|&id| id as i64).collect::<Vec<i64>>()))
            .exec(db)
            .await?.rows_affected as i64)
    }

    /// 根据ID更新店铺
    pub async fn update_by_id(db: &DbConn, form_data: &ShopSaveDTO) -> Result<i64, DbErr> {
        if let Some(id) = form_data.id {
            let payload = shop::ActiveModel {
                id: Set(id),
                store_logo: Set(form_data.store_logo.clone()),
                store_name: Set(form_data.store_name.clone()),
                self_operated: Set(form_data.self_operated),
                store_address_detail: Set(form_data.store_address_detail.clone()),
                store_address_id_path: Set(form_data.store_address_id_path.clone()),
                store_address_path: Set(form_data.store_address_path.clone()),
                store_end_time: Set(form_data.store_end_time),
                store_disable: Set(form_data.store_disable.unwrap_or(0)),
                store_center: Set(form_data.store_center.clone()),
                store_desc: Set(form_data.store_desc.clone()),
                delivery_score: Set(form_data.delivery_score.map(|d| rust_decimal::Decimal::from_f64_retain(d).unwrap_or_default())),
                description_score: Set(form_data.description_score.map(|d| rust_decimal::Decimal::from_f64_retain(d).unwrap_or_default())),
                service_score: Set(form_data.service_score.map(|d| rust_decimal::Decimal::from_f64_retain(d).unwrap_or_default())),
                goods_num: Set(form_data.goods_num),
                collection_num: Set(form_data.collection_num),
                yzf_mp_sign: Set(form_data.yzf_mp_sign.clone()),
                yzf_sign: Set(form_data.yzf_sign.clone()),
                merchant_euid: Set(form_data.merchant_euid.clone()),
                page_show: Set(form_data.page_show),
                self_pick_flag: Set(form_data.self_pick_flag),
                update_by: Set(form_data.update_by.clone()),
                update_time: Set(Some(chrono::Local::now().naive_local())),
                ..Default::default()
            };

            ShopEntity::update_many()
                .set(payload)
                .filter(shop::Column::Id.eq(id as i64))
                .exec(db)
                .await
                .map(|result| result.rows_affected as i64)
        } else {
            Err(DbErr::Custom("ID不能为空".to_string()))
        }
    }

    /// 根据ID查询店铺
    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<shop::Model>, DbErr> {
        if let Some(id_val) = id {
            ShopEntity::find_by_id(*id_val)
                .one(db)
                .await
        } else {
            Ok(None)
        }
    }

    /// 根据店铺名称检查是否存在
    pub async fn exists_by_store_name<C: ConnectionTrait>(db: &C, store_name: &str) -> Result<bool, DbErr> {
        ShopEntity::find()
            .filter(shop::Column::StoreName.eq(store_name))
            .one(db)
            .await
            .map(|r| r.is_some())
    }

    /// 根据用户ID查询店铺
    pub async fn find_by_user_id<C: ConnectionTrait>(db: &C, user_id: i64) -> Result<Option<shop::Model>, DbErr> {
        use crate::modules::shop::model::shop_user_merge;
        
        let shop_ids = match shop_user_merge::find_shop_ids_by_user_id(db, user_id).await {
            Ok(ids) => ids,
            Err(e) => return Err(DbErr::Custom(format!("查询用户店铺关联失败: {:?}", e))),
        };
        
        if let Some(&shop_id) = shop_ids.first() {
            ShopEntity::find_by_id(shop_id)
                .one(db)
                .await
        } else {
            Ok(None)
        }
    }

    /// 统计店铺数量
    pub async fn select_count(db: &DbConn, wheres: PageWhere) -> Result<i64, DbErr> {
        ShopEntity::find()
            .apply_if(wheres.store_name, |query, v| {
                query.filter(shop::Column::StoreName.contains(format!("%{}%", v)))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(shop::Column::Status.eq(v))
            })
            .apply_if(wheres.self_operated, |query, v| {
                query.filter(shop::Column::SelfOperated.eq(v))
            })
            .count(db)
            .await
            .map(|c| c as i64)
    }

    /// 分页查询店铺列表
    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        wheres: PageWhere,
    ) -> Result<(Vec<shop::Model>, i64), DbErr> {
        let paginator = ShopEntity::find()
            .apply_if(wheres.store_name, |query, v| {
                query.filter(shop::Column::StoreName.contains(format!("%{}%", v)))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(shop::Column::Status.eq(v))
            })
            .apply_if(wheres.self_operated, |query, v| {
                query.filter(shop::Column::SelfOperated.eq(v))
            })
            .order_by_desc(shop::Column::Id)
            .paginate(db, per_page as u64);

        let total = paginator.num_items().await? as i64;
        let items = paginator.fetch_page(page.saturating_sub(1) as u64).await?;

        Ok((items, total))
    }
}