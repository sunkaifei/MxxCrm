//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//!

use sea_orm::{DbConn, DbErr, TransactionTrait};
use bcrypt::{hash, verify, DEFAULT_COST};
use crate::core::errors::error::{Error, Result};
use crate::core::kit::config;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::web::response::ResultPage;
use crate::modules::website::entity::website_user;
use crate::modules::website::model::website_user::{
    WebsiteUserLoginRequest, WebsiteUserLoginVO, WebsiteUserModel, WebsiteUserRegisterRequest,
    WebsiteUserSaveDTO, WebsiteUserUpdateRequest, WebsiteUserVO, WebsiteUserListQuery,
};

const USER_TOKEN_ISSUER: &str = "mxx_B2B_user";

/// 校验用户名格式
fn validate_username(username: &str) -> Result<()> {
    if username.len() < 3 || username.len() > 32 {
        return Err(Error::from("用户名长度必须在3-32个字符之间"));
    }
    if !username.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(Error::from("用户名只能包含字母、数字和下划线"));
    }
    Ok(())
}

/// 校验密码强度
fn validate_password(password: &str) -> Result<()> {
    if password.len() < 6 || password.len() > 64 {
        return Err(Error::from("密码长度必须在6-64个字符之间"));
    }
    Ok(())
}

/// 前台用户注册
pub async fn register(db: &DbConn, req: WebsiteUserRegisterRequest, register_ip: Option<String>) -> Result<i64> {
    validate_username(&req.username)?;
    validate_password(&req.password)?;

    // 校验唯一性
    if WebsiteUserModel::find_by_username(db, &req.username).await?.is_some() {
        return Err(Error::from("用户名已存在"));
    }
    if let Some(phone) = &req.phone {
        if !phone.is_empty() && WebsiteUserModel::find_by_phone(db, phone).await?.is_some() {
            return Err(Error::from("手机号已被注册"));
        }
    }
    if let Some(email) = &req.email {
        if !email.is_empty() && WebsiteUserModel::find_by_email(db, email).await?.is_some() {
            return Err(Error::from("邮箱已被注册"));
        }
    }

    let hashed = hash(&req.password, DEFAULT_COST).map_err(|e| Error::from(format!("密码加密失败: {}", e)))?;
    let username_clone = req.username.clone();
    let phone_clone = req.phone.clone();
    let email_clone = req.email.clone();
    let real_name_clone = req.real_name.clone();
    let register_ip_clone = register_ip.clone();
    let hashed_clone = hashed.clone();

    let user_id = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move {
                WebsiteUserModel::register(
                    txn,
                    username_clone,
                    hashed_clone,
                    phone_clone,
                    email_clone,
                    real_name_clone,
                    register_ip_clone,
                )
                .await
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(user_id)
}

/// 前台用户登录（用户名/手机号/邮箱 均可）
pub async fn login(db: &DbConn, req: WebsiteUserLoginRequest, login_ip: Option<String>) -> Result<WebsiteUserLoginVO> {
    let user = if req.account.contains('@') {
        WebsiteUserModel::find_by_email(db, &req.account).await?
    } else if req.account.chars().all(|c| c.is_ascii_digit()) {
        WebsiteUserModel::find_by_phone(db, &req.account).await?
            .or_else(|| {
                // 手机号查不到，回退到用户名（数字用户名）
                None
            })
    } else {
        None
    };

    let user = match user {
        Some(u) => u,
        None => WebsiteUserModel::find_by_username(db, &req.account).await?
            .ok_or_else(|| Error::from("账号不存在"))?,
    };

    if user.status.unwrap_or(0) == 1 {
        return Err(Error::from("账号已被停用，请联系管理员"));
    }

    let valid = verify(&req.password, &user.password).map_err(|e| Error::from(format!("密码校验失败: {}", e)))?;
    if !valid {
        return Err(Error::from("账号或密码错误"));
    }

    // 更新登录信息（非事务必要，失败不影响登录）
    let user_id = user.id;
    let _ = WebsiteUserModel::update_login_info(db, user_id, login_ip).await;

    // 生成 Token
    let jwt = JWTToken::new(Some(user_id), Some(user.username.clone()), vec![], Some(USER_TOKEN_ISSUER));
    let jwt_secret = config::section::<String>("server", "jwt_secret_user", "mxx_secret_key".to_string());
    let token = jwt.create_token(&jwt_secret)?;

    Ok(WebsiteUserLoginVO {
        token,
        user: user.into(),
    })
}

/// 获取当前登录用户信息
pub async fn get_profile(db: &DbConn, user_id: i64) -> Result<WebsiteUserVO> {
    let user = WebsiteUserModel::find_by_id(db, user_id)
        .await?
        .ok_or_else(|| Error::from("用户不存在"))?;
    Ok(user.into())
}

/// 用户更新自身资料
pub async fn update_profile(db: &DbConn, user_id: i64, req: WebsiteUserUpdateRequest) -> Result<i64> {
    // 手机号/邮箱唯一性校验
    if let Some(phone) = &req.phone {
        if !phone.is_empty() {
            if let Some(existing) = WebsiteUserModel::find_by_phone(db, phone).await? {
                if existing.id != user_id {
                    return Err(Error::from("手机号已被其他账号占用"));
                }
            }
        }
    }
    if let Some(email) = &req.email {
        if !email.is_empty() {
            if let Some(existing) = WebsiteUserModel::find_by_email(db, email).await? {
                if existing.id != user_id {
                    return Err(Error::from("邮箱已被其他账号占用"));
                }
            }
        }
    }

    db.transaction::<_, i64, DbErr>(|txn| {
        let req_clone = req.clone();
        Box::pin(async move {
            WebsiteUserModel::update_profile(txn, user_id, &req_clone).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    Ok(user_id)
}

/// 修改密码
pub async fn change_password(db: &DbConn, user_id: i64, old_password: String, new_password: String) -> Result<i64> {
    validate_password(&new_password)?;

    let user = WebsiteUserModel::find_by_id(db, user_id)
        .await?
        .ok_or_else(|| Error::from("用户不存在"))?;

    let valid = verify(&old_password, &user.password).map_err(|e| Error::from(format!("密码校验失败: {}", e)))?;
    if !valid {
        return Err(Error::from("原密码错误"));
    }

    let hashed = hash(&new_password, DEFAULT_COST).map_err(|e| Error::from(format!("密码加密失败: {}", e)))?;
    let hashed_clone = hashed.clone();
    db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move {
            WebsiteUserModel::update_password(txn, user_id, hashed_clone).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(user_id)
}

// ==================== 后台管理 ====================

/// 后台新增用户
pub async fn admin_create(db: &DbConn, mut req: WebsiteUserSaveDTO) -> Result<i64> {
    let username = req.username.clone().unwrap_or_default();
    if username.is_empty() {
        return Err(Error::from("用户名不能为空"));
    }
    validate_username(&username)?;
    if WebsiteUserModel::find_by_username(db, &username).await?.is_some() {
        return Err(Error::from("用户名已存在"));
    }

    let password = req.password.take().unwrap_or_default();
    if password.is_empty() {
        return Err(Error::from("密码不能为空"));
    }
    validate_password(&password)?;
    let hashed = hash(&password, DEFAULT_COST).map_err(|e| Error::from(format!("密码加密失败: {}", e)))?;

    // 临时把加密密码塞回 dto 中（insert 会使用 dto.password）
    let mut dto = req.clone();
    dto.password = Some(hashed);

    db.transaction::<_, i64, DbErr>(|txn| {
        let dto_clone = dto.clone();
        Box::pin(async move {
            WebsiteUserModel::insert(txn, &dto_clone).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    Ok(0)
}

/// 后台编辑用户
pub async fn admin_update(db: &DbConn, id: i64, req: WebsiteUserSaveDTO) -> Result<i64> {
    // 若传了新密码，单独处理
    if let Some(new_pwd) = &req.password {
        if !new_pwd.is_empty() {
            validate_password(new_pwd)?;
            let hashed = hash(new_pwd, DEFAULT_COST).map_err(|e| Error::from(format!("密码加密失败: {}", e)))?;
            let hashed_clone = hashed.clone();
            db.transaction::<_, i64, DbErr>(|txn| {
                Box::pin(async move {
                    WebsiteUserModel::update_password(txn, id, hashed_clone).await
                })
            })
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        }
    }

    let req_clone = req.clone();
    db.transaction::<_, i64, DbErr>(|txn| {
        let req_clone2 = req_clone.clone();
        Box::pin(async move {
            WebsiteUserModel::update_by_admin(txn, id, &req_clone2).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}

/// 后台重置密码
pub async fn admin_reset_password(db: &DbConn, id: i64, new_password: String) -> Result<i64> {
    validate_password(&new_password)?;
    let hashed = hash(&new_password, DEFAULT_COST).map_err(|e| Error::from(format!("密码加密失败: {}", e)))?;
    let hashed_clone = hashed.clone();
    db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move {
            WebsiteUserModel::update_password(txn, id, hashed_clone).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;
    Ok(id)
}

/// 后台分页查询
pub async fn admin_get_by_page(db: &DbConn, query: WebsiteUserListQuery) -> Result<ResultPage<Vec<WebsiteUserVO>>> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).max(1).min(100);
    let (list, total) = WebsiteUserModel::select_in_page(db, page, page_size, query.username, query.phone, query.status).await?;
    let list_vo: Vec<WebsiteUserVO> = list.into_iter().map(|m| m.into()).collect();
    Ok(ResultPage::new(list_vo, total, page, page_size))
}

/// 后台用户详情
pub async fn admin_get_by_id(db: &DbConn, id: i64) -> Result<WebsiteUserVO> {
    let user = WebsiteUserModel::find_by_id(db, id)
        .await?
        .ok_or_else(|| Error::from("用户不存在"))?;
    Ok(user.into())
}

/// 后台批量软删除
pub async fn admin_batch_delete(db: &DbConn, ids: Vec<i64>) -> Result<i64> {
    db.transaction::<_, i64, DbErr>(|txn| {
        let ids_clone = ids.clone();
        Box::pin(async move {
            WebsiteUserModel::batch_soft_delete(txn, ids_clone).await
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))
}

/// 后台更新状态（启用/停用）
pub async fn admin_update_status(db: &DbConn, id: i64, status: i32) -> Result<i64> {
    use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};
    use crate::modules::website::entity::website_user::{Entity as WebsiteUser, Column};
    let now = chrono::Local::now().naive_local().to_owned();
    let result = WebsiteUser::update_many()
        .col_expr(Column::Status, sea_orm::sea_query::Expr::value(status))
        .col_expr(Column::UpdateTime, sea_orm::sea_query::Expr::value(now))
        .filter(Column::Id.eq(id))
        .filter(Column::Deleted.eq(0))
        .exec(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(result.rows_affected as i64)
}

#[allow(dead_code)]
pub async fn find_by_id(db: &DbConn, id: i64) -> Result<website_user::Model> {
    WebsiteUserModel::find_by_id(db, id)
        .await?
        .ok_or_else(|| Error::from("用户不存在"))
}
