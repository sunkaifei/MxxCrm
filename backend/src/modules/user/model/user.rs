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
use crate::utils::string_utils::{serialize_option_u64_to_string, deserialize_string_to_u64};
use crate::modules::user::entity::{user, user::Entity as UserEntity};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserSaveRequest {
    /// 用户名
    pub username: Option<String>,
    /// 昵称
    pub nickname: Option<String>,
    /// 头像
    pub avatar: Option<String>,
    /// 邮箱
    pub email: Option<String>,
    /// 手机
    pub mobile: Option<String>,
    /// 密码
    pub password: Option<String>,
    /// 密码盐
    pub salt: Option<String>,
    /// 签名
    pub motto: Option<String>,
    /// 状态:0=禁用,1=启用
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserUpdateRequest {
    /// ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 用户名
    pub username: Option<String>,
    /// 昵称
    pub nickname: Option<String>,
    /// 头像
    pub avatar: Option<String>,
    /// 邮箱
    pub email: Option<String>,
    /// 手机
    pub mobile: Option<String>,
    /// 密码
    pub password: Option<String>,
    /// 密码盐
    pub salt: Option<String>,
    /// 签名
    pub motto: Option<String>,
    /// 状态:0=禁用,1=启用
    pub status: Option<String>,
}


pub struct UserSaveDTO {
    /// ID
    pub id: Option<i64>,
    /// 用户名
    pub username: Option<String>,
    /// 昵称
    pub nickname: Option<String>,
    /// 头像
    pub avatar: Option<String>,
    /// 邮箱
    pub email: Option<String>,
    /// 手机
    pub mobile: Option<String>,
    /// 登录失败次数
    pub loginfailure: Option<i32>,
    /// 上次登录时间
    pub lastlogintime: Option<String>,
    /// 上次登录IP
    pub lastloginip: Option<String>,
    /// 密码
    pub password: Option<String>,
    /// 密码盐
    pub salt: Option<String>,
    /// 签名
    pub motto: Option<String>,
    /// 状态:0=禁用,1=启用
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct UserListVO {
    /// ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 用户名
    pub username: Option<String>,
    /// 昵称
    pub nickname: Option<String>,
    /// 头像
    pub avatar: Option<String>,
    /// 邮箱
    pub email: Option<String>,
    /// 手机
    pub mobile: Option<String>,
    /// 登录失败次数
    pub loginfailure: Option<i32>,
    /// 上次登录时间
    pub lastlogintime: Option<String>,
    /// 上次登录IP
    pub lastloginip: Option<String>,
    /// 密码
    pub password: Option<String>,
    /// 密码盐
    pub salt: Option<String>,
    /// 签名
    pub motto: Option<String>,
    /// 状态:0=禁用,1=启用
    pub status: Option<String>,
}

impl From<user::Model> for UserListVO {
    fn from(model: user::Model) -> Self {
        UserListVO {
            id: Some(model.id),
            username: model.username,
            nickname: model.nickname,
            avatar: model.avatar,
            email: model.email,
            mobile: model.mobile,
            loginfailure: model.loginfailure,
            lastlogintime: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            lastloginip: model.lastloginip,
            password: None,
            salt: model.salt,
            motto: model.motto,
            status: model.status,
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct UserDetailVO {
    /// ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 用户名
    pub username: Option<String>,
    /// 昵称
    pub nickname: Option<String>,
    /// 头像
    pub avatar: Option<String>,
    /// 邮箱
    pub email: Option<String>,
    /// 手机
    pub mobile: Option<String>,
    /// 登录失败次数
    pub loginfailure: Option<i32>,
    /// 上次登录时间
    pub lastlogintime: Option<String>,
    /// 上次登录IP
    pub lastloginip: Option<String>,
    /// 密码
    pub password: Option<String>,
    /// 密码盐
    pub salt: Option<String>,
    /// 签名
    pub motto: Option<String>,
    /// 状态:0=禁用,1=启用
    pub status: Option<String>,
}

impl From<user::Model> for UserDetailVO {
    fn from(model: user::Model) -> Self {
        UserDetailVO {
            id: Some(model.id),
            username: model.username,
            nickname: model.nickname,
            avatar: model.avatar,
            email: model.email,
            mobile: model.mobile,
            loginfailure: model.loginfailure,
            lastlogintime: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            lastloginip: model.lastloginip,
            password: None,
            salt: model.salt,
            motto: model.motto,
            status: model.status,
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNicknameRequest {
    pub nickname: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAvatarRequest {
    pub avatar: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery{
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 用户名
    pub username: Option<String>,
    /// 昵称
    pub nickname: Option<String>,
    /// 邮箱
    pub email: Option<String>,
    /// 手机
    pub mobile: Option<String>,
    pub status: Option<i32>,
    
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
}


/// 条件
#[derive(Clone)]
pub struct PageWhere {
    pub id: Option<i64>,
    /// 用户名
    pub username: Option<String>,
    /// 昵称
    pub nickname: Option<String>,
    /// 邮箱
    pub email: Option<String>,
    /// 手机
    pub mobile: Option<String>,
    pub status: Option<i32>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut id = None;
        if self.id > Some(0) {
            id = self.id.clone();
        }

        let mut username = None;
        if self.username != Some("".to_string()) {
            username = self.username.clone();
        }

        let mut nickname = None;
        if self.nickname != Some("".to_string()) {
            nickname = self.nickname.clone();
        }

        let mut email = None;
        if self.email != Some("".to_string()) {
            email = self.email.clone();
        }
        let mut mobile = None;
        if self.mobile != Some("".to_string()) {
            mobile = self.mobile.clone();
        }

        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        Self {
            id,
            username,
            nickname,
            email,
            status,
            mobile,
        }
    }
}

pub struct UserModel;

impl UserModel{
    pub async fn insert<C: ConnectionTrait>(db: &C, form_data: &UserSaveDTO) -> Result<i64, DbErr> {
        let payload = user::ActiveModel {
            username:         Set(form_data.username.to_owned()),
            nickname:         Set(form_data.nickname.to_owned()),
            avatar:           Set(form_data.avatar.to_owned()),
            email:            Set(form_data.email.to_owned()),
            mobile:           Set(form_data.mobile.to_owned()),
            loginfailure:     Set(form_data.loginfailure.to_owned()),
            lastlogintime:    Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            lastloginip:      Set(form_data.lastloginip.to_owned()),
            password:         Set(form_data.password.to_owned()),
            salt:             Set(form_data.salt.to_owned()),
            motto:            Set(form_data.motto.to_owned()),
            status:           Set(form_data.status.to_owned()),
            create_time:      Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            update_time:      Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        UserEntity::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }


    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        UserEntity::delete_many()
            .filter(user::Column::Id.is_in(ids))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }
    
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, form_data: &UserSaveDTO) -> Result<i64, DbErr> {
        let payload = user::ActiveModel {
            username:         Set(form_data.username.to_owned()),
            nickname:         Set(form_data.nickname.to_owned()),
            avatar:           Set(form_data.avatar.to_owned()),
            email:            Set(form_data.email.to_owned()),
            mobile:           Set(form_data.mobile.to_owned()),
            loginfailure:     Set(form_data.loginfailure.to_owned()),
            lastlogintime:    Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            lastloginip:      Set(form_data.lastloginip.to_owned()),
            password:         Set(form_data.password.to_owned()),
            salt:             Set(form_data.salt.to_owned()),
            motto:            Set(form_data.motto.to_owned()),
            status:           Set(form_data.status.to_owned()),
            update_time:      Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let update_result: UpdateResult = UserEntity::update_many()
            .set(payload)
            .filter(user::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }
    
    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<user::Model>, DbErr> {
        UserEntity::find_by_id(id.clone().unwrap_or_default())
            .one(db)
            .await
    }
    
    pub async fn find_by_name(db: &DbConn, username: &Option<String>) -> Result<Option<user::Model>, DbErr> {
        UserEntity::find()
            .filter(user::Column::Username.eq(username.clone().unwrap_or_default()))
            .one(db)
            .await
    }

    pub async fn find_by_mobile(db: &DbConn, mobile: &str) -> Result<Option<user::Model>, DbErr> {
        UserEntity::find()
            .filter(user::Column::Mobile.eq(mobile))
            .one(db)
            .await
    }

    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        UserEntity::find()
            .apply_if(wheres.id, |query, v| {
                query.filter(user::Column::Id.eq(v))
            })
            .apply_if(wheres.username, |query, v| {
                query.filter(user::Column::Username.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.nickname, |query, v| {
                query.filter(user::Column::Nickname.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.email, |query, v| {
                query.filter(user::Column::Email.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.mobile, |query, v| {
                query.filter(user::Column::Mobile.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(user::Column::Status.eq(v))
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
    ) -> Result<(Vec<user::Model>, i64), DbErr> {
        let paginator = UserEntity::find()
            .apply_if(wheres.id, |query, v| {
                query.filter(user::Column::Id.eq(v))
            })
            .apply_if(wheres.username, |query, v| {
                query.filter(user::Column::Username.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.nickname, |query, v| {
                query.filter(user::Column::Nickname.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.email, |query, v| {
                query.filter(user::Column::Email.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.mobile, |query, v| {
                query.filter(user::Column::Mobile.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(user::Column::Status.eq(v))
            })
            .order_by_desc(user::Column::Id)
            .paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
    
    pub async fn update_avatar(db: &DbConn, user_id: i64, avatar: Option<String>) -> Result<i64, DbErr> {
        let payload = user::ActiveModel {
            id: Set(user_id),
            avatar: Set(avatar),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        
        let update_result: UpdateResult = UserEntity::update_many()
            .set(payload)
            .filter(user::Column::Id.eq(user_id))
            .exec(db)
            .await?;
        
        Ok(update_result.rows_affected as i64)
    }
    
}