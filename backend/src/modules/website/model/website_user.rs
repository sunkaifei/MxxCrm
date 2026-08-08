//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//!

use sea_orm::*;
use sea_orm::prelude::{DateTime, Decimal};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::website::entity::{website_user, website_user::Entity as WebsiteUser};

// ==================== DTO ====================

/// 前台用户注册请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WebsiteUserRegisterRequest {
    pub username: String,
    pub password: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub real_name: Option<String>,
}

/// 前台用户登录请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WebsiteUserLoginRequest {
    /// 用户名/手机号/邮箱 均可
    pub account: String,
    pub password: String,
}

/// 前台用户更新自己信息
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WebsiteUserUpdateRequest {
    pub real_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub avatar: Option<String>,
    pub gender: Option<i16>,
}

/// 后台管理：用户新增/编辑
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WebsiteUserSaveDTO {
    pub id: Option<i64>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub real_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub avatar: Option<String>,
    pub gender: Option<i16>,
    pub status: Option<i32>,
    pub member_level: Option<i32>,
    pub remark: Option<String>,
}

/// 后台管理：列表查询
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WebsiteUserListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub username: Option<String>,
    pub phone: Option<String>,
    pub status: Option<i32>,
}

// ==================== VO ====================

/// 前台用户详情VO（不包含密码）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct WebsiteUserVO {
    pub id: Option<i64>,
    pub username: Option<String>,
    pub real_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub avatar: Option<String>,
    pub gender: Option<i16>,
    pub status: Option<i32>,
    pub member_level: Option<i32>,
    pub total_points: Option<i32>,
    pub total_spent: Option<Decimal>,
    pub order_count: Option<i32>,
    pub last_login_time: Option<DateTime>,
    pub register_source: Option<String>,
    pub remark: Option<String>,
    pub create_time: Option<DateTime>,
}

impl From<website_user::Model> for WebsiteUserVO {
    fn from(item: website_user::Model) -> Self {
        WebsiteUserVO {
            id: Option::from(item.id),
            username: Some(item.username),
            real_name: item.real_name,
            phone: item.phone,
            email: item.email,
            avatar: item.avatar,
            gender: item.gender,
            status: item.status,
            member_level: item.member_level,
            total_points: item.total_points,
            total_spent: item.total_spent,
            order_count: item.order_count,
            last_login_time: item.last_login_time,
            register_source: item.register_source,
            remark: item.remark,
            create_time: item.create_time,
        }
    }
}

/// 登录成功响应
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct WebsiteUserLoginVO {
    pub token: String,
    pub user: WebsiteUserVO,
}

// ==================== Model ====================

/// 前台用户数据模型操作类
pub struct WebsiteUserModel;

impl WebsiteUserModel {
    /// 新增用户
    pub async fn insert<C: ConnectionTrait>(db: &C, req: &WebsiteUserSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = website_user::ActiveModel {
            username: Set(req.username.clone().unwrap_or_default()),
            password: Set(req.password.clone().unwrap_or_default()),
            real_name: Set(req.real_name.clone()),
            phone: Set(req.phone.clone()),
            email: Set(req.email.clone()),
            avatar: Set(req.avatar.clone()),
            gender: Set(req.gender),
            status: Set(Some(req.status.unwrap_or(0))),
            member_level: Set(req.member_level),
            total_points: Set(Some(0)),
            total_spent: Set(Some(Decimal::from(0))),
            order_count: Set(Some(0)),
            register_source: Set(Some("admin".to_string())),
            remark: Set(req.remark.clone()),
            create_time: Set(Some(now.clone())),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        WebsiteUser::insert(payload).exec(db).await.map(|r| r.last_insert_id)
    }

    /// 前台注册：通过 username + 已加密的密码
    pub async fn register<C: ConnectionTrait>(
        db: &C,
        username: String,
        hashed_password: String,
        phone: Option<String>,
        email: Option<String>,
        real_name: Option<String>,
        register_ip: Option<String>,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = website_user::ActiveModel {
            username: Set(username),
            password: Set(hashed_password),
            real_name: Set(real_name),
            phone: Set(phone),
            email: Set(email),
            status: Set(Some(0)),
            member_level: Set(Some(0)),
            total_points: Set(Some(0)),
            total_spent: Set(Some(Decimal::from(0))),
            order_count: Set(Some(0)),
            register_source: Set(Some("website".to_string())),
            register_ip: Set(register_ip),
            create_time: Set(Some(now.clone())),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        WebsiteUser::insert(payload).exec(db).await.map(|r| r.last_insert_id)
    }

    /// 根据ID查询
    pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<Option<website_user::Model>, DbErr> {
        WebsiteUser::find_by_id(id)
            .filter(website_user::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 根据用户名查询（不区分大小写）
    pub async fn find_by_username<C: ConnectionTrait>(db: &C, username: &str) -> Result<Option<website_user::Model>, DbErr> {
        WebsiteUser::find()
            .filter(website_user::Column::Username.eq(username.to_lowercase()))
            .filter(website_user::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 根据手机号查询
    pub async fn find_by_phone<C: ConnectionTrait>(db: &C, phone: &str) -> Result<Option<website_user::Model>, DbErr> {
        WebsiteUser::find()
            .filter(website_user::Column::Phone.eq(phone))
            .filter(website_user::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 根据邮箱查询
    pub async fn find_by_email<C: ConnectionTrait>(db: &C, email: &str) -> Result<Option<website_user::Model>, DbErr> {
        WebsiteUser::find()
            .filter(website_user::Column::Email.eq(email))
            .filter(website_user::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 更新最后登录信息
    pub async fn update_login_info<C: ConnectionTrait>(
        db: &C,
        id: i64,
        ip: Option<String>,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let result: UpdateResult = WebsiteUser::update_many()
            .col_expr(website_user::Column::LastLoginTime, sea_orm::sea_query::Expr::value(now))
            .col_expr(website_user::Column::LastLoginIp, sea_orm::sea_query::Expr::value(ip))
            .col_expr(website_user::Column::UpdateTime, sea_orm::sea_query::Expr::value(chrono::Local::now().naive_local().to_owned()))
            .filter(website_user::Column::Id.eq(id))
            .filter(website_user::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 前台用户更新自身资料
    pub async fn update_profile<C: ConnectionTrait>(
        db: &C,
        id: i64,
        req: &WebsiteUserUpdateRequest,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let mut payload = website_user::ActiveModel {
            update_time: Set(Some(now)),
            ..Default::default()
        };
        if let Some(v) = &req.real_name { payload.real_name = Set(Some(v.clone())); }
        if let Some(v) = &req.phone { payload.phone = Set(Some(v.clone())); }
        if let Some(v) = &req.email { payload.email = Set(Some(v.clone())); }
        if let Some(v) = &req.avatar { payload.avatar = Set(Some(v.clone())); }
        if let Some(v) = req.gender { payload.gender = Set(Some(v)); }

        let result: UpdateResult = WebsiteUser::update_many()
            .set(payload)
            .filter(website_user::Column::Id.eq(id))
            .filter(website_user::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 修改密码
    pub async fn update_password<C: ConnectionTrait>(
        db: &C,
        id: i64,
        hashed_password: String,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let result: UpdateResult = WebsiteUser::update_many()
            .col_expr(website_user::Column::Password, sea_orm::sea_query::Expr::value(hashed_password))
            .col_expr(website_user::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
            .filter(website_user::Column::Id.eq(id))
            .filter(website_user::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 后台更新用户（不含密码）
    pub async fn update_by_admin<C: ConnectionTrait>(
        db: &C,
        id: i64,
        req: &WebsiteUserSaveDTO,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = website_user::ActiveModel {
            real_name: Set(req.real_name.clone()),
            phone: Set(req.phone.clone()),
            email: Set(req.email.clone()),
            avatar: Set(req.avatar.clone()),
            gender: Set(req.gender),
            status: Set(req.status),
            member_level: Set(req.member_level),
            remark: Set(req.remark.clone()),
            update_time: Set(Some(now)),
            ..Default::default()
        };
        let result: UpdateResult = WebsiteUser::update_many()
            .set(payload)
            .filter(website_user::Column::Id.eq(id))
            .filter(website_user::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 累计消费金额与订单数（订单完成时调用）
    pub async fn add_spent<C: ConnectionTrait>(
        db: &C,
        user_id: i64,
        amount: Decimal,
    ) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let result: UpdateResult = WebsiteUser::update_many()
            .col_expr(website_user::Column::TotalSpent, sea_orm::sea_query::Expr::col(website_user::Column::TotalSpent).add(sea_orm::sea_query::Expr::value(amount)))
            .col_expr(website_user::Column::OrderCount, sea_orm::sea_query::Expr::col(website_user::Column::OrderCount).add(sea_orm::sea_query::Expr::value(1)))
            .col_expr(website_user::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
            .filter(website_user::Column::Id.eq(user_id))
            .filter(website_user::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    /// 分页查询
    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        username: Option<String>,
        phone: Option<String>,
        status: Option<i32>,
    ) -> Result<(Vec<website_user::Model>, i64), DbErr> {
        let mut query = WebsiteUser::find()
            .filter(website_user::Column::Deleted.eq(0));
        if let Some(u) = username { query = query.filter(website_user::Column::Username.like(format!("%{}%", u))); }
        if let Some(p) = phone { query = query.filter(website_user::Column::Phone.like(format!("%{}%", p))); }
        if let Some(s) = status { query = query.filter(website_user::Column::Status.eq(s)); }

        let paginator = query
            .order_by_desc(website_user::Column::CreateTime)
            .paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;
        let rows = paginator.fetch_page((page - 1) as u64).await?;
        Ok((rows, total))
    }

    /// 软删除（批量）
    pub async fn batch_soft_delete<C: ConnectionTrait>(db: &C, ids: Vec<i64>) -> Result<i64, DbErr> {
        if ids.is_empty() { return Ok(0); }
        let now = chrono::Local::now().naive_local().to_owned();
        let result: UpdateResult = WebsiteUser::update_many()
            .col_expr(website_user::Column::Deleted, sea_orm::sea_query::Expr::value(1))
            .col_expr(website_user::Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
            .filter(website_user::Column::Id.is_in(ids))
            .filter(website_user::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }
}
