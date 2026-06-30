//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.!
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

extern crate bcrypt;

use crate::core::errors::error::{Error, Result};
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use actix_web_grants::protect;
use bcrypt::{hash, verify, DEFAULT_COST};
use std::time::Duration;

use crate::core::kit::config;
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::kit::CONTEXT;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::response::MetaResp;
use crate::modules::system::model::admin::{AdminSaveRequest, AdminUpdateRequest, UpdateAdminPasswordRequest, UpdateAdminRoleRequest, UpdateAdminStatusRequest, UpdateLoginRequest, UpdateResetPasswordRequest, UserLoginRequest, UserLoginVO, AdminModel};
use crate::modules::system::model::admin::{ListQuery, TokenVO};
use crate::modules::system::service::menu_service::find_user_role_keys;
use crate::modules::system::service::{admin_service, dept_service, post_service, role_service, system_log_service};

// 添加用户信息
#[post("/admin/add")]
pub async fn save_admin(state: web::Data<AppState>, item: web::Json<AdminSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.user_name.as_ref().map_or(true, |username| username.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "用户名称不能为空", "local")));
    }
    if admin_service::find_by_name_unique(&db, &item.user_name, &None).await.unwrap_or_default(){
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "用户名已存在", "local")));
    }
    if admin_service::find_by_mobile_unique(&db, &item.mobile, &None).await.unwrap_or_default(){
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "手机号已存在", "local")));
    }
    if item.email.is_some() {
        if admin_service::find_by_email_unique(&db, &item.email, &None).await.unwrap_or_default(){
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "邮箱已存在", "local")));
        }
    }
    if admin_service::find_by_nick_name_unique(&db, &item.nick_name, &None).await.unwrap_or_default(){
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "昵称已存在", "local")));
    }
    let result = admin_service::insert(&db, &item.0).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

/// 后台用户登录
#[post("/api/system/auth/login")]
pub async fn post_login(state: web::Data<AppState>,request: HttpRequest, item: web::Json<UserLoginRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.username.as_ref().map_or(true, |username| username.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "用户名称不能为空", "local")));
    }

    let hashed_password = hash(item.password.clone().unwrap_or_default(), DEFAULT_COST).map_err(|_| Error::from("密码哈希失败"))?;

    log::info!("hashed_password: {}", hashed_password);

    if item.password.as_ref().map_or(true, |password| password.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "密码名称不能为空", "local")));
    }
    // if let (Some(verify_code), Some(uuid)) = (item.captcha_code.clone(), item.captcha_key.clone()) {
    //     if verify_code.is_empty() || uuid.is_empty() {
    //         return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::error_msg("验证不能为空或者参数错误".to_string())));
    //     }

    //     // 查询缓存内的验证码
    //     let cache_captcha = CONTEXT.cache_service.get_string(&format!("captcha:cache_{}", uuid.as_str())).await.unwrap_or_default();
    //     if cache_captcha.is_empty() {
    //         return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::error_msg("验证码已过期或者不存在".to_string())));
    //     }

    //     // 比较验证码
    //     if cache_captcha != verify_code {
    //         return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::error_msg("验证码不正确".to_string())));
    //     }

    //     // 删除验证码缓存
    //     CONTEXT.cache_service.del(&format!("captcha:cache_{}", uuid.as_str())).await.unwrap_or_default();
    // } else {
    //     return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::error_msg("验证不能为空或者参数错误".to_string())));
    // } 
    
    let user_info = admin_service::find_by_name(&db, &item.username).await?.ok_or_else(|| { Error::from(format!("msg={},code={}", "未获取到用户信息".to_string(), 404))})?;
    let valid = verify(&item.password.clone().unwrap_or_default(), &user_info.password.clone().unwrap_or_default()).unwrap_or_default();
    if !valid {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "密码不正确", "local")));
    }
    //判断是否是管理员
    let is_admin = user_info.user_type == Option::from(1);

    //查询按用户关联的按钮权限
    let user_role_keys: Vec<String> = find_user_role_keys(&db, &is_admin, &Some(user_info.id)).await?;
    // let user_role_keys: Vec<String> = Vec::new();
    
    // 原来的token
    let old_token = CONTEXT.cache_service.get_string(&format!("user_{}", user_info.id.to_string().as_str())).await?;

    //过期时间
    let expire = Duration::from_secs(config::section::<i64>("server", "jwt_expire_admin", 1800) as u64);

    match JWTToken::new(Some(user_info.id), user_info.user_name.clone(), user_role_keys.clone(), None).create_token(&config::section::<String>("server", "jwt_secret_admin", "".to_string())) {
        Ok(token) => {
            // 缓存token
            CONTEXT.cache_service.set_string(&format!("user_{}", user_info.id.to_string().as_str()), &token.clone().as_str()).await?;
            
            // 记录登录日志
            let method = request.method().to_string();
            system_log_service::save_system_log(&db, &request, Some("用户登录".to_string()), Some(0),Some("system_admin_controller::login".to_string()), Some(method.to_string()),Some(1)).await.unwrap_or_default();

            let update_user = UpdateLoginRequest {
                id: Some(user_info.id),
                login_ip: Option::from(request.connection_info().realip_remote_addr().unwrap_or_default().to_string()),
                login_date: Option::from(chrono::Local::now().naive_local())
            };
            admin_service::update_login_info(&db, &update_user).await.unwrap_or_default();

            let user_token = TokenVO {
                access_token: Option::from(token.clone()),
                token_type: Option::from("Bearer".to_string()),
                refresh_token: Option::from(old_token),
                expires_in: Option::from(expire.as_secs() as i64),
                role: user_role_keys,
            };
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(user_token, "local")))
        }
        Err(err) => {
            Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &err.to_string(), "local")))
        }
    }
}

// 删除用户信息
#[delete("/admin/batch_delete")]
pub async fn admin_batch_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        for id_opt in ids_vec.iter() {
            if let Some(id) = id_opt {
                if id == "1" {
                    return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "含有不能删除的超级管理员账户", "local")));
                }
            }
        }

        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }

        let result = admin_service::batch_delete_by_ids(&db, &ids_vec).await;
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
    } else {
        Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

/// ### 软删除用户
#[delete("/admin/delete/{id}")]
#[protect("system:admin:delete")]
pub async fn admin_soft_delete(state: web::Data<AppState>, path: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let id = path.into_inner();
    if id == 1 {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "不能删除超级管理员账户", "local")));
    }
    let result = admin_service::soft_delete_by_id(&db, id).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

#[put("/update_user_role")]
pub async fn update_user_role(state: web::Data<AppState>, item: web::Json<UpdateAdminRoleRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let user_role = item.0;
    let result = role_service::batch_update_role(&db, &Some(user_role.role_ids), &user_role.admin_id).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}


// 更新用户信息
#[put("/admin/update")]
pub async fn admin_update(state: web::Data<AppState>, item: web::Json<AdminUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;
    if let Some(id) = item.id {
        if id == 1 && item.status == Option::from(0) {
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "超级管理员不能禁用", "local")));
        }
    } else {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "更新的用户id不能为空", "local")));
    }
    if admin_service::find_by_name_unique(&db, &item.user_name, &item.id).await.unwrap_or_default(){
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "用户名已存在", "local")));
    }
    if admin_service::find_by_mobile_unique(&db, &item.mobile, &item.id).await.unwrap_or_default(){
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "手机号已存在", "local")));
    }
    if item.email.is_some() {
        if admin_service::find_by_email_unique(&db, &item.email, &item.id).await.unwrap_or_default(){
            return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "邮箱已存在", "local")));
        }
    }
    if admin_service::find_by_nick_name_unique(&db, &item.nick_name, &item.id).await.unwrap_or_default(){
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "昵称已存在", "local")));
    }

    let result = admin_service::get_by_detail(&db, &item.id).await?;
    if result.id.unwrap_or_default() == 0 {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "用户信息不存在", "local")));
    }
    let result = admin_service::update_admin(&db, &item).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

/// 更新用户密码
#[put("/admin/update_password")]
pub async fn update_password(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<UpdateResetPasswordRequest>
) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;

    // 检查密码是否为空
    if item.password.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "密码不能为空", "local")));
    }

    // 获取当前用户id
    let admin_token: JWTToken = get_user(&req).unwrap_or_default();

    // 防止修改当前用户密码
    if admin_token.id == item.user_id {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "不可通过列表页面修改当前用户密码", "local")));
    }

    // 检查用户是否存在
    let sys_admin_result = admin_service::get_by_detail(&db, &item.user_id).await?;
    if sys_admin_result.id.is_none() || sys_admin_result.id == Some(0) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "用户不存在", "local")));
    }

    // 哈希密码
    let hashed_password = hash(item.password.clone().unwrap_or_default(), DEFAULT_COST).map_err(|_| Error::from("密码哈希失败"))?;

    // 更新密码
    let result = admin_service::update_user_password(&db, &item.user_id, &Some(hashed_password)).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}

// 登录用户更新自己的登录密码
#[put("/admin/update_my_password")]
pub async fn update_my_password(state: web::Data<AppState>, req: HttpRequest, item: web::Json<UpdateAdminPasswordRequest>) -> Result<HttpResponse>  {
    let db = &state.db;
    let user_pwd = item.into_inner();
    
    // 1. 获取当前用户id
    let admin_token: JWTToken = get_user(&req).unwrap_or_default();
    let user_id = match admin_token.id {
        Some(id) if id > 0 => id,
        _ => return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "用户未登录或token无效", "local"))),
    };
    
    // 2. 查询管理员信息（使用Model直接查询，包含密码）
    let admin = match AdminModel::find_by_id(db, &Some(user_id)).await {
        Ok(Some(admin)) => admin,
        _ => return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "用户不存在", "local"))),
    };
    
    // 3. 获取旧密码和新密码
    let old_password = match user_pwd.old_password {
        Some(pwd) if !pwd.is_empty() => pwd,
        _ => return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "旧密码不能为空", "local"))),
    };
    
    let new_password = match user_pwd.new_password {
        Some(pwd) if !pwd.is_empty() => pwd,
        _ => return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "新密码不能为空", "local"))),
    };
    
    let confirm_password = match user_pwd.confirm_password {
        Some(pwd) if !pwd.is_empty() => pwd,
        _ => return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "确认密码不能为空", "local"))),
    };
    
    // 4. 验证旧密码是否正确
    let stored_password = match admin.password {
        Some(pwd) => pwd,
        None => return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "用户密码不存在", "local"))),
    };
    
    if !verify(old_password, &stored_password).unwrap_or(false) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "旧密码不正确", "local")));
    }
    
    // 5. 确认新密码和确认密码一致
    if new_password != confirm_password {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "新密码和确认密码不一致", "local")));
    }
    
    // 6. 更新密码
    let hashed = hash(new_password, DEFAULT_COST).unwrap_or_default();
    let result = admin_service::update_user_password(db, &Some(admin.id), &Some(hashed)).await;
    
    match result {
        Ok(_) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(200, "密码更新成功", "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &format!("密码更新失败: {}", e), "local"))),
    }
}

#[put("/admin/update-status")]
pub async fn update_admin_status(state: web::Data<AppState>, item: web::Json<UpdateAdminStatusRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let admin_status = item.0;
    if admin_status.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "用户id不能为空", "local")))
    }
    if admin_status.id == Option::from(1) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "超级管理员不能禁用", "local")))
    }
    let result = admin_service::update_user_status(&db, &admin_status).await;
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<i64>::handle_result(result)))
}



#[get("/admin/detail/{id}")]
pub async fn get_user_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "角色id不能为空", "local")));
    }
    let user_result = admin_service::get_by_detail(&db, &item.id).await?;
    if user_result.id.is_none() || user_result.id == Some(0) {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "用户不存在", "local")))
    }
    let mut admin_detail = user_result;
    // 查询用户关联的角色
    let result_roles = role_service::select_by_admin_id(&db, &admin_detail.id).await.unwrap_or_default();
    let role_data: Vec<Option<String>> = result_roles
        .into_iter()
        .map(|role| role.id.map(|id| id.to_string()))
        .collect();
    admin_detail.role_ids = Some(role_data);
    
    // 查询用户关联的部门
    let result_depts = dept_service::select_by_admin_id(&db, &admin_detail.id).await.unwrap_or_default();
    let dept_data: Vec<Option<String>> = result_depts
        .into_iter()
        .map(|dept| dept.id.map(|id| id.to_string()))
        .collect();
    admin_detail.dept_ids = Some(dept_data);
    
    // 查询用户关联的岗位
    let result_posts = post_service::select_by_admin_id(&db, &admin_detail.id).await.unwrap_or_default();
    let post_data: Vec<Option<String>> = result_posts
        .into_iter()
        .map(|post| post.id.map(|id| id.to_string()))
        .collect();
    admin_detail.post_ids = Some(post_data);
    
    
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(admin_detail, "local")))
}

#[get("/admin/userinfo")]
pub async fn get_user_info(state: web::Data<AppState>,req: HttpRequest, ) -> Result<HttpResponse> {
    let db = &state.db;
    //获取当前用户id
    let admin_token:JWTToken = get_user(&req).unwrap_or_default();
    
    let user_info = admin_service::find_by_id(&db,&admin_token.id).await?.ok_or_else(|| { Error::from(format!("msg={},code={}", "未获取到用户信息".to_string(), 404))})?;
    //判断是否是管理员
    let is_admin = user_info.user_type == Option::from(1);
    //查询用户所在权限字符符串
    let permissions: Vec<String> = find_user_role_keys(&db, &is_admin, &Some(user_info.id)).await?;
    //查询用户所在权限组
    let roles: Vec<String> = role_service::user_by_role_group(&db, &Some(user_info.id)).await?;

    let user_info = UserLoginVO {
        id: Option::from(user_info.id),
        username: user_info.user_name,
        nickname: user_info.nick_name,
        avatar: user_info.avatar,
        roles,
        permissions,
    };
    
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(user_info, "local")))
}

/// 更新当前登录用户头像请求体
#[derive(serde::Deserialize)]
pub struct UpdateAvatarRequest {
    /// 头像访问地址（由附件上传接口返回，含缓存破坏版本号）
    pub avatar: String,
}

/// # 更新当前登录用户头像
///
/// 供“个人中心-更换头像”使用：头像文件上传成功后，将返回的访问地址持久化到
/// `mxx_system_admin.avatar`，使刷新后仍能读到最新头像。
///
/// - 无需权限注解（仅操作本人数据）
/// - 用户id从 JWT 提取
#[put("/admin/avatar")]
pub async fn update_avatar(state: web::Data<AppState>, req: HttpRequest, item: web::Json<UpdateAvatarRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let admin_token: JWTToken = get_user(&req).unwrap_or_default();
    let user_id = match admin_token.id {
        Some(id) if id > 0 => id,
        _ => return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "用户未登录或token无效", "local"))),
    };

    let avatar = item.into_inner().avatar;
    if avatar.trim().is_empty() {
        return Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, "头像地址不能为空", "local")));
    }

    match AdminModel::update_avatar(db, user_id, &avatar).await {
        Ok(_) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(avatar, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::fail(400, &format!("更新头像失败: {}", e), "local"))),
    }
}

// 查询用户列表
#[get("/admin/list")]
#[protect("system:admin:list")]
pub async fn admin_list(state: web::Data<AppState>, query: web::Query<ListQuery>,) -> Result<HttpResponse> {
    let db = &state.db;
    admin_service::get_by_page(db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(page_data, "local"))
    })
}

#[get("/admin/options")]
pub async fn admin_options(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    admin_service::get_admin_options(db).await.map(|list_data| {
        HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(list_data, "local"))
    })
}


// 获取权限码列表
#[get("/auth/codes")]
pub async fn get_auth_codes(state: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse> {
    let db = &state.db;
    
    // 获取当前用户token
    let admin_token: JWTToken = get_user(&req).unwrap_or_default();
    
    // 查询用户信息
    let user_info = admin_service::find_by_id(&db, &admin_token.id).await?.ok_or_else(|| { 
        Error::from(format!("msg={},code={}", "未获取到用户信息".to_string(), 404))
    })?;
    
    // 判断是否是管理员
    let is_admin = user_info.user_type == Option::from(1);
    
    // 查询用户关联的按钮权限
    let user_role_keys: Vec<String> = find_user_role_keys(&db, &is_admin, &Some(user_info.id)).await?;
    
    Ok(HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::success(user_role_keys, "local")))
}

// 退出登录
#[delete("/api/auth/logout")]
pub async fn logout() -> HttpResponse {
    HttpResponse::Ok().content_type("application/msgpack").body(MetaResp::<String>::success(String::new(), "local"))
}
