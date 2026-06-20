//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::kit::global::Serialize;
use crate::modules::user::entity::{user_platform, user_platform::Entity as UserPlatform};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string, u64_to_string};
use sea_orm::prelude::DateTime;
use sea_orm::*;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Validate, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserPlatformSaveRequest {
    pub user_id: Option<i64>,
    pub system_type: Option<String>,
    pub platform: Option<String>,
    pub token: Option<String>,
    pub alipay_openid: Option<String>,
    pub weixin_openid: Option<String>,
    pub weixin_unionid: Option<String>,
    pub weixin_web_openid: Option<String>,
    pub baidu_openid: Option<String>,
    pub toutiao_openid: Option<String>,
    pub toutiao_unionid: Option<String>,
    pub qq_openid: Option<String>,
    pub qq_unionid: Option<String>,
    pub kuaishou_openid: Option<String>,
}

impl From<UserPlatformSaveRequest> for UserPlatformSaveDTO {
    fn from(req: UserPlatformSaveRequest) -> Self {
        UserPlatformSaveDTO {
            id: None,
            user_id: req.user_id,
            system_type: req.system_type,
            platform: req.platform,
            token: req.token,
            alipay_openid: req.alipay_openid,
            weixin_openid: req.weixin_openid,
            weixin_unionid: req.weixin_unionid,
            weixin_web_openid: req.weixin_web_openid,
            baidu_openid: req.baidu_openid,
            toutiao_openid: req.toutiao_openid,
            toutiao_unionid: req.toutiao_unionid,
            qq_openid: req.qq_openid,
            qq_unionid: req.qq_unionid,
            kuaishou_openid: req.kuaishou_openid,
            create_time: None,
            update_time: None,
        }
    }
}

#[derive(Debug, Validate, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserPlatformUpdateRequest {
    pub id: Option<i64>,
    pub user_id: Option<i64>,
    pub system_type: Option<String>,
    pub platform: Option<String>,
    pub token: Option<String>,
    pub alipay_openid: Option<String>,
    pub weixin_openid: Option<String>,
    pub weixin_unionid: Option<String>,
    pub weixin_web_openid: Option<String>,
    pub baidu_openid: Option<String>,
    pub toutiao_openid: Option<String>,
    pub toutiao_unionid: Option<String>,
    pub qq_openid: Option<String>,
    pub qq_unionid: Option<String>,
    pub kuaishou_openid: Option<String>,
}

impl From<UserPlatformUpdateRequest> for UserPlatformSaveDTO {
    fn from(req: UserPlatformUpdateRequest) -> Self {
        UserPlatformSaveDTO {
            id: req.id,
            user_id: req.user_id,
            system_type: req.system_type,
            platform: req.platform,
            token: req.token,
            alipay_openid: req.alipay_openid,
            weixin_openid: req.weixin_openid,
            weixin_unionid: req.weixin_unionid,
            weixin_web_openid: req.weixin_web_openid,
            baidu_openid: req.baidu_openid,
            toutiao_openid: req.toutiao_openid,
            toutiao_unionid: req.toutiao_unionid,
            qq_openid: req.qq_openid,
            qq_unionid: req.qq_unionid,
            kuaishou_openid: req.kuaishou_openid,
            create_time: None,
            update_time: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserPlatformSaveDTO {
    pub id: Option<i64>,
    pub user_id: Option<i64>,
    pub system_type: Option<String>,
    pub platform: Option<String>,
    pub token: Option<String>,
    pub alipay_openid: Option<String>,
    pub weixin_openid: Option<String>,
    pub weixin_unionid: Option<String>,
    pub weixin_web_openid: Option<String>,
    pub baidu_openid: Option<String>,
    pub toutiao_openid: Option<String>,
    pub toutiao_unionid: Option<String>,
    pub qq_openid: Option<String>,
    pub qq_unionid: Option<String>,
    pub kuaishou_openid: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_time: Option<DateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub id: i64,
    pub nick_name: String,
    pub avatar_url: String,
    pub gender: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WechatLoginRequest {
    pub login_code: String,
    pub phone_code: Option<String>,
    pub r#type: Option<String>,
}

/// 微信手机号授权登录请求
/// 支持单独手机号登录或同时绑定微信平台信息
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WechatPhoneLoginRequest {
    /// 手机号授权获取的code（必填）
    pub phone_code: String,
    /// 微信登录code（可选，用于同时绑定微信平台信息）
    pub login_code: Option<String>,
    /// 登录类型（可选，用于区分不同登录场景）
    pub r#type: Option<String>,
}

/// 微信平台code登录请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WechatCodeLoginRequest {
    /// 微信登录code
    pub code: String,
    /// 可选的unionid
    pub r#type: Option<String>,
}

impl WechatLoginRequest {
    /// 转换为 UserSaveDTO（用于创建新用户）
    pub fn to_user_save_dto(&self, _openid: &str, phone: Option<String>) -> crate::modules::user::model::user::UserSaveDTO {
        crate::modules::user::model::user::UserSaveDTO {
            id: None,
            username: None,
            nickname: None,
            avatar: None,
            email: None,
            mobile: phone,
            loginfailure: Some(0),
            lastlogintime: None,
            lastloginip: None,
            password: None,
            salt: None,
            motto: None,
            status: Some("1".to_string()),
        }
    }
    
    /// 转换为 ShopSaveDTO（用于创建店铺）
    pub fn to_shop_save_dto(&self, _user_id: i64) -> crate::modules::shop::model::shop::ShopSaveDTO {
        crate::modules::shop::model::shop::ShopSaveDTO {
            id: None,
            store_logo: None,
            store_name: None,
            self_operated: 0,
            store_address_detail: None,
            store_address_id_path: None,
            store_address_path: None,
            store_end_time: None,
            store_disable: Some(0),
            store_center: None,
            store_desc: None,
            delivery_score: None,
            description_score: None,
            service_score: None,
            goods_num: Some(0),
            collection_num: Some(0),
            yzf_mp_sign: None,
            yzf_sign: None,
            merchant_euid: None,
            page_show: Some(0),
            self_pick_flag: Some(0),
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
            delete_flag: Some(0),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WechatAuthResponse {
    pub openid: Option<String>,
    pub unionid: Option<String>,
    pub session_key: Option<String>,
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
}

impl From<WechatAuthResponse> for UserPlatformSaveDTO {
    fn from(resp: WechatAuthResponse) -> Self {
        UserPlatformSaveDTO {
            id: None,
            user_id: None,
            system_type: Some("wechat".to_string()),
            platform: Some("mini_program".to_string()),
            token: None,
            alipay_openid: None,
            weixin_openid: resp.openid,
            weixin_unionid: resp.unionid,
            weixin_web_openid: None,
            baidu_openid: None,
            toutiao_openid: None,
            toutiao_unionid: None,
            qq_openid: None,
            qq_unionid: None,
            kuaishou_openid: None,
            create_time: Some(chrono::Local::now().naive_local()),
            update_time: Some(chrono::Local::now().naive_local()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserPlatformDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub user_id: Option<i64>,
    pub system_type: Option<String>,
    pub platform: Option<String>,
    pub token: Option<String>,
    pub alipay_openid: Option<String>,
    pub weixin_openid: Option<String>,
    pub weixin_unionid: Option<String>,
    pub weixin_web_openid: Option<String>,
    pub baidu_openid: Option<String>,
    pub toutiao_openid: Option<String>,
    pub toutiao_unionid: Option<String>,
    pub qq_openid: Option<String>,
    pub qq_unionid: Option<String>,
    pub kuaishou_openid: Option<String>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}

impl From<user_platform::Model> for UserPlatformDetailVO {
    fn from(model: user_platform::Model) -> Self {
        Self {
            id: Option::from(model.id),
            user_id: Option::from(model.user_id),
            system_type: model.system_type,
            platform: model.platform,
            token: model.token,
            alipay_openid: model.alipay_openid,
            weixin_openid: model.weixin_openid,
            weixin_unionid: model.weixin_unionid,
            weixin_web_openid: model.weixin_web_openid,
            baidu_openid: model.baidu_openid,
            toutiao_openid: model.toutiao_openid,
            toutiao_unionid: model.toutiao_unionid,
            qq_openid: model.qq_openid,
            qq_unionid: model.qq_unionid,
            kuaishou_openid: model.kuaishou_openid,
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserPlatformListVO {
    #[serde(serialize_with = "u64_to_string")]
    pub id: i64,
    #[serde(serialize_with = "u64_to_string")]
    pub user_id: i64,
    pub system_type: Option<String>,
    pub platform: Option<String>,
    pub token: Option<String>,
    pub create_time: Option<String>,
}

impl From<user_platform::Model> for UserPlatformListVO {
    fn from(model: user_platform::Model) -> Self {
        Self {
            id: model.id,
            user_id: model.user_id,
            system_type: model.system_type,
            platform: model.platform,
            token: model.token,
            create_time: model.create_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub user_id: Option<i64>,
    pub system_type: Option<String>,
    pub platform: Option<String>,
}

#[derive(Clone)]
pub struct PageWhere {
    pub user_id: Option<i64>,
    pub system_type: Option<String>,
    pub platform: Option<String>,
}

impl PageWhere {
    pub fn format(&self) -> Self {
        let mut user_id = None;
        if self.user_id != Some(0) {
            user_id = self.user_id;
        }

        let mut system_type = None;
        if self.system_type != Some("".to_string()) {
            system_type = self.system_type.clone();
        }

        let mut platform = None;
        if self.platform != Some("".to_string()) {
            platform = self.platform.clone();
        }

        Self {
            user_id,
            system_type,
            platform,
        }
    }
}

pub struct UserPlatformModel;

impl UserPlatformModel {
    pub async fn insert<C: ConnectionTrait>(db: &C, form_data: &UserPlatformSaveDTO) -> Result<i64, DbErr> {
        let payload = user_platform::ActiveModel {
            user_id: Set(form_data.user_id.unwrap_or_default()),
            system_type: Set(form_data.system_type.clone()),
            platform: Set(form_data.platform.clone()),
            token: Set(form_data.token.clone()),
            alipay_openid: Set(form_data.alipay_openid.clone()),
            weixin_openid: Set(form_data.weixin_openid.clone()),
            weixin_unionid: Set(form_data.weixin_unionid.clone()),
            weixin_web_openid: Set(form_data.weixin_web_openid.clone()),
            baidu_openid: Set(form_data.baidu_openid.clone()),
            toutiao_openid: Set(form_data.toutiao_openid.clone()),
            toutiao_unionid: Set(form_data.toutiao_unionid.clone()),
            qq_openid: Set(form_data.qq_openid.clone()),
            qq_unionid: Set(form_data.qq_unionid.clone()),
            kuaishou_openid: Set(form_data.kuaishou_openid.clone()),
            create_time: Set(Option::from(chrono::Local::now().naive_local())),
            update_time: Set(Option::from(chrono::Local::now().naive_local())),
            ..Default::default()
        };

        UserPlatform::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        Ok(UserPlatform::delete_many()
            .filter(user_platform::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await?.rows_affected as i64)
    }

    pub async fn update_by_id(
        db: &DbConn,
        id: i64,
        form_data: &UserPlatformSaveDTO,
    ) -> Result<i64, DbErr> {
        let payload = user_platform::ActiveModel {
            id: Set(id),
            user_id: Set(form_data.user_id.unwrap_or_default()),
            system_type: Set(form_data.system_type.clone()),
            platform: Set(form_data.platform.clone()),
            token: Set(form_data.token.clone()),
            alipay_openid: Set(form_data.alipay_openid.clone()),
            weixin_openid: Set(form_data.weixin_openid.clone()),
            weixin_unionid: Set(form_data.weixin_unionid.clone()),
            weixin_web_openid: Set(form_data.weixin_web_openid.clone()),
            baidu_openid: Set(form_data.baidu_openid.clone()),
            toutiao_openid: Set(form_data.toutiao_openid.clone()),
            toutiao_unionid: Set(form_data.toutiao_unionid.clone()),
            qq_openid: Set(form_data.qq_openid.clone()),
            qq_unionid: Set(form_data.qq_unionid.clone()),
            kuaishou_openid: Set(form_data.kuaishou_openid.clone()),
            update_time: Set(Option::from(chrono::Local::now().naive_local())),
            ..Default::default()
        };

        UserPlatform::update_many()
            .set(payload)
            .filter(user_platform::Column::Id.eq(id))
            .exec(db)
            .await
            .map(|result| result.rows_affected as i64)
    }

    pub async fn find_by_id(db: &DbConn, id: &i64) -> Result<Option<user_platform::Model>, DbErr> {
        UserPlatform::find_by_id(id.clone())
            .one(db)
            .await
    }

    pub async fn find_by_user_id(db: &DbConn, user_id: &i64) -> Result<Vec<user_platform::Model>, DbErr> {
        UserPlatform::find()
            .filter(user_platform::Column::UserId.eq(user_id.clone()))
            .all(db)
            .await
    }

    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        UserPlatform::find()
            .apply_if(wheres.user_id, |query, v| {
                query.filter(user_platform::Column::UserId.eq(v))
            })
            .apply_if(wheres.system_type, |query, v| {
                query.filter(user_platform::Column::SystemType.contains(format!("%{}%", v)))
            })
            .apply_if(wheres.platform, |query, v| {
                query.filter(user_platform::Column::Platform.contains(format!("%{}%", v)))
            })
            .count(db)
            .await
            .map(|c| c as i64)
    }

    pub async fn find_by_weixin_openid(db: &DbConn, openid: &String) -> Result<Option<user_platform::Model>, DbErr> {
        UserPlatform::find()
            .filter(user_platform::Column::WeixinOpenid.eq(openid))
            .one(db)
            .await
    }

    pub async fn find_by_weixin_unionid(db: &DbConn, unionid: &String) -> Result<Option<user_platform::Model>, DbErr> {
        UserPlatform::find()
            .filter(user_platform::Column::WeixinUnionid.eq(unionid))
            .one(db)
            .await
    }

    pub async fn find_openid_by_user_id(db: &DbConn, user_id: i64) -> Result<Option<String>, DbErr> {
        UserPlatform::find()
            .filter(user_platform::Column::UserId.eq(user_id))
            .one(db)
            .await
            .map(|opt| opt.and_then(|m| m.weixin_openid))
    }

    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        wheres: PageWhere,
    ) -> Result<(Vec<user_platform::Model>, i64), DbErr> {
        let paginator = UserPlatform::find()
            .apply_if(wheres.user_id, |query, v| {
                query.filter(user_platform::Column::UserId.eq(v))
            })
            .apply_if(wheres.system_type, |query, v| {
                query.filter(user_platform::Column::SystemType.contains(format!("%{}%", v)))
            })
            .apply_if(wheres.platform, |query, v| {
                query.filter(user_platform::Column::Platform.contains(format!("%{}%", v)))
            })
            .order_by_desc(user_platform::Column::Id)
            .paginate(db, per_page as u64);

        let total = paginator.num_items().await? as i64;
        let items = paginator.fetch_page(page.saturating_sub(1) as u64).await?;

        Ok((items, total))
    }
}

/// Token刷新请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenRefreshRequest {
    pub token: String,
}

/// 临时Token请求（微信小程序授权）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TempTokenRequest {
    pub login_code: String,
}

/// 微信获取手机号响应
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WechatPhoneResponse {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    pub phone_info: Option<PhoneInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PhoneInfo {
    pub phone_number: Option<String>,
    pub pure_phone_number: Option<String>,
    pub country_code: Option<String>,
    pub watermark: Option<Watermark>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Watermark {
    pub timestamp: Option<i64>,
    pub appid: Option<String>,
}

/// 用户基本信息VO（微信小程序使用）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserBasicInfoVO {
    /// 用户ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 昵称
    pub nickname: Option<String>,
    /// 头像
    pub avatar: Option<String>,
    /// 获赞数
    pub like_count: Option<i64>,
    /// 粉丝数
    pub follower_count: Option<i64>,
    /// 关注数
    pub follow_count: Option<i64>,
    /// 会员等级
    pub member_level: Option<String>,
}