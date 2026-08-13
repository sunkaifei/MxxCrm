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
use actix_web::{web, HttpRequest, HttpResponse};
use bcrypt::{hash, verify, DEFAULT_COST};
use std::time::Duration;

use crate::core::kit::app::is_demo_mode;
use crate::core::kit::config;
use crate::core::kit::global::AppState;
use crate::core::kit::jwt_util::JWTToken;
use crate::core::kit::CONTEXT;
use crate::core::web::base_controller::get_user;
use crate::core::web::entity::common::{BathDeleteIdRequest, InfoId};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::system::model::admin::{AdminSaveRequest, AdminUpdateRequest, UpdateAdminPasswordRequest, UpdateAdminRoleRequest, UpdateAdminStatusRequest, UpdateLoginRequest, UpdateResetPasswordRequest, UserLoginRequest, UserRegisterRequest, CheckUsernameResult, UserLoginVO, AdminModel};
use crate::modules::system::model::admin::{ListQuery, TokenVO};
use crate::modules::system::service::menu_service::find_user_role_keys;
use crate::modules::system::service::{admin_service, dept_service, post_service, role_service, system_log_service, permission_cache_service, session_service};

// 添加用户信息
pub async fn save_admin(state: web::Data<AppState>, item: web::Json<AdminSaveRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.user_name.as_ref().map_or(true, |username| username.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "用户名称不能为空", "local")));
    }
    if admin_service::find_by_name_unique(&db, &item.user_name, &None).await.unwrap_or_default(){
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "用户名已存在", "local")));
    }
    if admin_service::find_by_mobile_unique(&db, &item.mobile, &None).await.unwrap_or_default(){
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "手机号已存在", "local")));
    }
    if item.email.is_some() {
        if admin_service::find_by_email_unique(&db, &item.email, &None).await.unwrap_or_default(){
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "邮箱已存在", "local")));
        }
    }
    let result = admin_service::insert(&db, &item.0).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

/// 后台用户登录
pub async fn post_login(state: web::Data<AppState>,request: HttpRequest, item: web::Json<UserLoginRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.username.as_ref().map_or(true, |username| username.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "用户名不能为空", "local")));
    }

    if item.password.as_ref().map_or(true, |password| password.trim().is_empty()) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "密码不能为空", "local")));
    }
    // if let (Some(verify_code), Some(uuid)) = (item.captcha_code.clone(), item.captcha_key.clone()) {
    //     if verify_code.is_empty() || uuid.is_empty() {
    //         return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::error_msg("验证不能为空或者参数错误".to_string())));
    //     }

    //     // 查询缓存内的验证码
    //     let cache_captcha = CONTEXT.cache_service.get_string(&format!("captcha:cache_{}", uuid.as_str())).await.unwrap_or_default();
    //     if cache_captcha.is_empty() {
    //         return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::error_msg("验证码已过期或者不存在".to_string())));
    //     }

    //     // 比较验证码
    //     if cache_captcha != verify_code {
    //         return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::error_msg("验证码不正确".to_string())));
    //     }

    //     // 删除验证码缓存
    //     CONTEXT.cache_service.del(&format!("captcha:cache_{}", uuid.as_str())).await.unwrap_or_default();
    // } else {
    //     return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::error_msg("验证不能为空或者参数错误".to_string())));
    // }

    // 提取登录请求中输入的用户名，用于审计日志记录（即便后续校验失败，也要知道是谁在尝试登录）
    let input_username = item.username.clone().unwrap_or_default();
    let oper_param_json = format!("{{\"username\":\"{}\"}}", input_username.replace('"', "\\\""));
    let request_method = request.method().to_string();
    let oper_ip = request.connection_info().realip_remote_addr().map(|s| s.to_string());

    // 用户不存在
    let user_info = match admin_service::find_by_name(&db, &item.username).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            record_login_log(
                &db,
                &request,
                Some(input_username.clone()),
                Some(oper_param_json.clone()),
                Some(1),
                Some("用户不存在".to_string()),
                oper_ip.clone(),
            ).await;
            return Err(Error::from(format!("msg={},code={}", "未获取到用户信息".to_string(), 404)));
        }
        Err(e) => return Err(e),
    };

    // 密码校验
    let valid = verify(&item.password.clone().unwrap_or_default(), &user_info.password.clone().unwrap_or_default()).unwrap_or_default();
    if !valid {
        record_login_log(
            &db,
            &request,
            Some(input_username.clone()),
            Some(oper_param_json.clone()),
            Some(1),
            Some("密码不正确".to_string()),
            oper_ip.clone(),
        ).await;
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "密码不正确", "local")));
    }

    // 用户被禁用
    if user_info.status != Some(1) {
        record_login_log(
            &db,
            &request,
            Some(input_username.clone()),
            Some(oper_param_json.clone()),
            Some(1),
            Some("用户已被禁用，无法登录".to_string()),
            oper_ip.clone(),
        ).await;
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "用户已被禁用，无法登录", "local")));
    }
    //判断是否是管理员
    let is_admin = user_info.user_type == Option::from(1);

    //查询按用户关联的按钮权限
    let user_role_keys: Vec<String> = find_user_role_keys(&db, &is_admin, &Some(user_info.id)).await?;
    // let user_role_keys: Vec<String> = Vec::new();

    // v2.0: 将权限码写入缓存，实现权限实时生效
    if let Err(e) = permission_cache_service::set_permissions(user_info.id, &user_role_keys).await {
        log::warn!("[登录] 权限缓存写入失败 user_id={}, err={}", user_info.id, e);
    }

    // 原来的token
    let old_token = CONTEXT.cache_service.get_string(&format!("user_{}", user_info.id.to_string().as_str())).await?;

    // v1.1: 会话超时优先从安全设置（缓存/配置表）动态读取，回退配置文件默认值
    let expire_secs = permission_cache_service::get_session_timeout_secs().await;
    let expire = Duration::from_secs(expire_secs);

    match JWTToken::new_with_expire(Some(user_info.id), user_info.user_name.clone(), user_role_keys.clone(), None, expire_secs).create_token(&config::section::<String>("server", "jwt_secret_admin", "".to_string())) {
        Ok(token) => {
            let multi = permission_cache_service::is_multi_device_mode().await;

            // v1.1: 根据登录模式写入 token 缓存
            if multi {
                // 多设备模式：将 token 追加到用户 token 集合，并按最大设备数限制踢出最旧设备
                let key = format!("user_tokens_{}", user_info.id);
                let mut tokens: Vec<String> = CONTEXT.cache_service.get_json(&key).await.unwrap_or_default();
                let max_devices = permission_cache_service::get_max_devices().await;
                if max_devices > 0 && tokens.len() >= max_devices {
                    // tokens 按登录先后有序追加，移除最旧的
                    let drop_count = tokens.len() - max_devices + 1;
                    tokens.drain(0..drop_count);
                }
                tokens.push(token.clone());
                CONTEXT.cache_service.set_json(&key, &tokens).await?;
            } else {
                // 单设备模式：覆盖旧 token（现有逻辑不变）
                CONTEXT.cache_service.set_string(&format!("user_{}", user_info.id.to_string().as_str()), &token.clone().as_str()).await?;
            }

            // v1.2: 同步写入 DB session 表（mem 模式重启后降级验证用）
            // 单设备模式：先清掉该用户所有旧 DB session，确保旧 token 不能通过 DB 降级复活
            // 多设备模式：直接追加（create_session 内部只清过期的）
            if !multi {
                if let Err(e) = session_service::get_session_store()
                    .remove_session(&db, user_info.id)
                    .await
                {
                    log::warn!("[登录] 清理旧 DB session 失败 user_id={}: {}", user_info.id, e);
                }
            }
            if let Err(e) = session_service::get_session_store()
                .create_session(&db, user_info.id, &token, &oper_ip.clone().unwrap_or_default(), expire_secs as i64)
                .await
            {
                log::warn!("[登录] 写入 DB session 失败 user_id={}: {}", user_info.id, e);
            }

            // 记录登录成功日志（同步落库，登录响应延迟几毫秒可接受）
            // json_result 字段名与 TokenVO 实际响应字段保持一致，敏感字段已脱敏
            let json_result = format!(
                "{{\"accessToken\":\"***\",\"tokenType\":\"Bearer\",\"refreshToken\":\"***\",\"expiresIn\":{},\"roleCount\":{}}}",
                expire.as_secs(),
                user_role_keys.len()
            );
            let ctx = system_log_service::SaveLogContext {
                request: &request,
                title: Some("用户登录".to_string()),
                business_type: Some(0),
                method: Some("system_admin_controller::post_login".to_string()),
                request_method: Some(request_method.clone()),
                operator_type: Some(1),
                oper_name: user_info.user_name.clone(),
                dept_name: None,
                oper_param: Some(oper_param_json.clone()),
                json_result: Some(json_result),
                status: Some(0),
                error_msg: None,
                status_code: Some(200),
                elapsed: None,
            };
            if let Err(e) = system_log_service::save_log_with_ip(&db, ctx, oper_ip.clone()).await {
                log::warn!("[登录日志] 写入失败: {}", e);
            }

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
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(user_token, "local")))
        }
        Err(err) => {
            // 生成 token 失败也记录日志
            record_login_log(
                &db,
                &request,
                Some(input_username.clone()),
                Some(oper_param_json.clone()),
                Some(1),
                Some(format!("生成Token失败: {}", err)),
                oper_ip.clone(),
            ).await;
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &err.to_string(), "local")))
        }
    }
}

/// 记录登录失败日志的辅助函数（同步落库，但失败时仅打印日志不影响主流程）
async fn record_login_log(
    db: &sea_orm::DbConn,
    request: &HttpRequest,
    oper_name: Option<String>,
    oper_param: Option<String>,
    status: Option<i32>,
    error_msg: Option<String>,
    oper_ip: Option<String>,
) {
    let ctx = system_log_service::SaveLogContext {
        request,
        title: Some("用户登录".to_string()),
        business_type: Some(0),
        method: Some("system_admin_controller::post_login".to_string()),
        request_method: Some(request.method().to_string()),
        operator_type: Some(1),
        oper_name,
        dept_name: None,
        oper_param,
        json_result: None,
        status,
        error_msg,
        status_code: Some(400),
        elapsed: None,
    };
    if let Err(e) = system_log_service::save_log_with_ip(db, ctx, oper_ip).await {
        log::warn!("[登录日志] 写入失败: {}", e);
    }
}

/// 检查用户名是否已存在
pub async fn check_username(state: web::Data<AppState>, query: web::Query<UserRegisterRequest>) -> HttpResponse {
    let db = &state.db;
    let username = query.username.clone().unwrap_or_default();
    if username.trim().is_empty() {
        return HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(CheckUsernameResult { exists: false, message: "".to_string() }, "local"));
    }
    let exists = admin_service::find_by_name_unique(&db, &Some(username.clone()), &None).await.unwrap_or_default();
    if exists {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(CheckUsernameResult { exists: true, message: "用户名已存在".to_string() }, "local"))
    } else {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(CheckUsernameResult { exists: false, message: "用户名可用".to_string() }, "local"))
    }
}

/// 查询注册开关状态（免鉴权接口，供登录页判断是否显示注册入口）
pub async fn register_status() -> Result<HttpResponse> {
    let enabled = permission_cache_service::is_register_enabled().await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(
        serde_json::json!({ "registerEnabled": enabled }),
        "local",
    )))
}

/// 用户注册
pub async fn user_register(state: web::Data<AppState>, item: web::Json<UserRegisterRequest>) -> Result<HttpResponse> {
    let db = &state.db;

    // 注册开关：关闭时拒绝注册（云服务器场景防止外部人员随意注册）
    if !permission_cache_service::is_register_enabled().await {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(403, "系统未开放注册", "local")));
    }

    let username = item.username.clone().unwrap_or_default();
    let password = item.password.clone().unwrap_or_default();
    
    if username.trim().is_empty() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "用户名不能为空", "local")));
    }
    if password.trim().is_empty() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "密码不能为空", "local")));
    }
    if username.len() < 3 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "用户名至少需要3个字符", "local")));
    }
    if password.len() < 6 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "密码至少需要6个字符", "local")));
    }
    
    if admin_service::find_by_name_unique(&db, &Some(username.clone()), &None).await.unwrap_or_default() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "用户名已存在", "local")));
    }
    
    if item.mobile.is_some() {
        if admin_service::find_by_mobile_unique(&db, &item.mobile, &None).await.unwrap_or_default() {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "手机号已存在", "local")));
        }
    }
    
    if item.email.is_some() {
        if admin_service::find_by_email_unique(&db, &item.email, &None).await.unwrap_or_default() {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "邮箱已存在", "local")));
        }
    }
    
    let save_request: AdminSaveRequest = item.into_inner().into();
    
    let result = admin_service::register(&db, &save_request).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

// 删除用户信息
pub async fn admin_batch_delete(state: web::Data<AppState>, item: web::Json<BathDeleteIdRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    if let Some(ids_vec) = item.ids.clone() {
        for id_opt in ids_vec.iter() {
            if let Some(id) = id_opt {
                if id == "1" {
                    return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "含有不能删除的超级管理员账户", "local")));
                }
            }
        }

        if ids_vec.is_empty() {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")));
        }

        let result = admin_service::batch_delete_by_ids(&db, &ids_vec).await;
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
    } else {
        Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "删除的ID不能为空", "local")))
    }
}

/// ### 软删除用户
pub async fn admin_soft_delete(state: web::Data<AppState>, path: web::Path<i64>) -> Result<HttpResponse> {
    let db = &state.db;
    let id = path.into_inner();
    if id == 1 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "不能删除超级管理员账户", "local")));
    }
    let result = admin_service::soft_delete_by_id(&db, id).await;
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

pub async fn update_user_role(state: web::Data<AppState>, item: web::Json<UpdateAdminRoleRequest>) -> Result<HttpResponse> {
    if is_demo_mode() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "演示站模式下禁止修改用户角色", "local")));
    }
    let db = &state.db;
    let user_role = item.0;
    let admin_id = user_role.admin_id;
    let result = role_service::batch_update_role(&db, &Some(user_role.role_ids), &admin_id).await;
    // v2.0: 用户角色变更后，清除该用户的权限缓存
    if result.is_ok() {
        if let Some(uid) = admin_id {
            permission_cache_service::invalidate_by_user_id(uid).await;
        }
    }
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}


// 更新用户信息
pub async fn admin_update(state: web::Data<AppState>, item: web::Json<AdminUpdateRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;
    if let Some(id) = item.id {
        if id == 1 && item.status == Option::from(0) {
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "超级管理员不能禁用", "local")));
        }
    } else {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "更新的用户id不能为空", "local")));
    }
    if admin_service::find_by_name_unique(&db, &item.user_name, &item.id).await.unwrap_or_default(){
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "用户名已存在", "local")));
    }
    if admin_service::find_by_mobile_unique(&db, &item.mobile, &item.id).await.unwrap_or_default(){
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "手机号已存在", "local")));
    }
    if item.email.is_some() {
        if admin_service::find_by_email_unique(&db, &item.email, &item.id).await.unwrap_or_default(){
            return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "邮箱已存在", "local")));
        }
    }

    let result = admin_service::get_by_detail(&db, &item.id).await?;
    if result.id.unwrap_or_default() == 0 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "用户信息不存在", "local")));
    }
    let result = admin_service::update_admin(&db, &item).await;
    // v2.0: 用户信息变更后清除缓存
    if result.is_ok() {
        if let Some(uid) = item.id {
            if item.status == Some(0) {
                // 用户被禁用：清除Token + 权限缓存，立即踢下线
                permission_cache_service::invalidate_user_session(uid).await;
            } else {
                // 其他变更：仅清除权限缓存
                permission_cache_service::invalidate_by_user_id(uid).await;
            }
        }
    }
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

/// 更新用户密码
pub async fn update_password(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<UpdateResetPasswordRequest>
) -> Result<HttpResponse> {
    let db = &state.db;
    let item = item.0;

    // 检查密码是否为空
    if item.password.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "密码不能为空", "local")));
    }

    // 获取当前用户id
    let admin_token: JWTToken = get_user(&req).unwrap_or_default();

    // 防止修改当前用户密码
    if admin_token.id == item.user_id {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "不可通过列表页面修改当前用户密码", "local")));
    }

    // 检查用户是否存在
    let sys_admin_result = admin_service::get_by_detail(&db, &item.user_id).await?;
    if sys_admin_result.id.is_none() || sys_admin_result.id == Some(0) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "用户不存在", "local")));
    }

    // 哈希密码
    let hashed_password = hash(item.password.clone().unwrap_or_default(), DEFAULT_COST).map_err(|_| Error::from("密码哈希失败"))?;

    // 更新密码
    let result = admin_service::update_user_password(&db, &item.user_id, &Some(hashed_password)).await;
    // v1.1: 改密成功后强制目标用户所有设备重新登录（防止密码泄露后旧 token 仍可用）
    if let Ok(_) = result {
        permission_cache_service::invalidate_user_session(item.user_id.unwrap_or_default()).await;
    }
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}

// 登录用户更新自己的登录密码
pub async fn update_my_password(state: web::Data<AppState>, req: HttpRequest, item: web::Json<UpdateAdminPasswordRequest>) -> Result<HttpResponse>  {
    let db = &state.db;
    let user_pwd = item.into_inner();
    
    // 1. 获取当前用户id
    let admin_token: JWTToken = get_user(&req).unwrap_or_default();
    let user_id = match admin_token.id {
        Some(id) if id > 0 => id,
        _ => return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "用户未登录或token无效", "local"))),
    };
    
    // 2. 查询管理员信息（使用Model直接查询，包含密码）
    let admin = match AdminModel::find_by_id(db, &Some(user_id)).await {
        Ok(Some(admin)) => admin,
        _ => return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "用户不存在", "local"))),
    };
    
    // 3. 获取旧密码和新密码
    let old_password = match user_pwd.old_password {
        Some(pwd) if !pwd.is_empty() => pwd,
        _ => return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "旧密码不能为空", "local"))),
    };
    
    let new_password = match user_pwd.new_password {
        Some(pwd) if !pwd.is_empty() => pwd,
        _ => return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "新密码不能为空", "local"))),
    };
    
    let confirm_password = match user_pwd.confirm_password {
        Some(pwd) if !pwd.is_empty() => pwd,
        _ => return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "确认密码不能为空", "local"))),
    };
    
    // 4. 验证旧密码是否正确
    let stored_password = match admin.password {
        Some(pwd) => pwd,
        None => return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "用户密码不存在", "local"))),
    };
    
    if !verify(old_password, &stored_password).unwrap_or(false) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "旧密码不正确", "local")));
    }
    
    // 5. 确认新密码和确认密码一致
    if new_password != confirm_password {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "新密码和确认密码不一致", "local")));
    }
    
    // 6. 更新密码
    let hashed = hash(new_password, DEFAULT_COST).unwrap_or_default();
    let result = admin_service::update_user_password(db, &Some(admin.id), &Some(hashed)).await;

    // v1.1: 改密成功后强制自己所有设备重新登录（本请求已放行，下次请求即 401）
    if let Ok(_) = result {
        permission_cache_service::invalidate_user_session(admin.id).await;
    }

    match result {
        Ok(_) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(200, "密码更新成功", "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &format!("密码更新失败: {}", e), "local"))),
    }
}

/// 踢用户下线（清除该用户所有设备会话 + 断开 WebSocket）
///
/// 权限：system:admin:kick；不可踢超级管理员（id=1）
pub async fn kick_offline(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let _db = &state.db;
    let user_id = path.into_inner();

    // 不能踢超级管理员
    if user_id == 1 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "不能踢超级管理员下线", "local")));
    }

    // 清除该用户所有缓存（token + 权限 + 多设备集合）并断开 WebSocket
    permission_cache_service::invalidate_user_session(user_id).await;

    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success("已强制下线".to_string(), "local")))
}

/// 审核注册用户
///
/// 请求体: { "auditStatus": 1 }  1=通过 0=拒绝(保持待审核)
/// 审核通过时自动将用户 status 设为 1（正常启用）
pub async fn audit_user(state: web::Data<AppState>, path: web::Path<i64>, item: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = path.into_inner();
    let audit_status = item.get("auditStatus").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    if user_id == 1 {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "超级管理员无需审核", "local")));
    }

    let result = admin_service::update_audit_status(db, user_id, audit_status).await;
    match result {
        Ok(_) => {
            let msg = if audit_status == 1 { "审核已通过，用户已启用" } else { "已拒绝" };
            Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success(msg.to_string(), "local")))
        }
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &format!("审核失败: {}", e), "local"))),
    }
}

pub async fn update_admin_status(state: web::Data<AppState>, item: web::Json<UpdateAdminStatusRequest>) -> Result<HttpResponse> {
    let db = &state.db;
    let admin_status = item.0;
    if admin_status.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "用户id不能为空", "local")))
    }
    if admin_status.id == Option::from(1) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "超级管理员不能禁用", "local")))
    }
    let result = admin_service::update_user_status(&db, &admin_status).await;
    // v1.1: 禁用用户（status=0）时即时清理会话 + 断开 WebSocket，启用则无需处理
    if let Ok(_) = result {
        if admin_status.status.unwrap_or(1) == 0 {
            permission_cache_service::invalidate_user_session(admin_status.id.unwrap_or_default()).await;
        }
    }
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<i64>::handle_result(result)))
}



pub async fn get_user_detail(state: web::Data<AppState>, item: web::Path<InfoId>) -> Result<HttpResponse> {
    let db = &state.db;
    if item.id.is_none() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "角色id不能为空", "local")));
    }
    let user_result = admin_service::get_by_detail(&db, &item.id).await?;
    if user_result.id.is_none() || user_result.id == Some(0) {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "用户不存在", "local")))
    }
    let mut admin_detail = user_result;
    // 查询用户关联的角色
    let result_roles = role_service::select_by_admin_id(&db, &admin_detail.id).await.unwrap_or_default();
    let role_data: Vec<Option<String>> = result_roles
        .iter()
        .map(|role| role.id.map(|id| id.to_string()))
        .collect();
    let role_name_data: Vec<Option<String>> = result_roles
        .iter()
        .map(|role| role.role_name.clone())
        .collect();
    admin_detail.role_ids = Some(role_data);
    admin_detail.role_names = Some(role_name_data);

    // 查询用户关联的部门
    let result_depts = dept_service::select_by_admin_id(&db, &admin_detail.id).await.unwrap_or_default();
    let dept_data: Vec<Option<String>> = result_depts
        .iter()
        .map(|dept| dept.id.map(|id| id.to_string()))
        .collect();
    let dept_name_data: Vec<Option<String>> = result_depts
        .iter()
        .map(|dept| dept.dept_name.clone())
        .collect();
    admin_detail.dept_ids = Some(dept_data);
    admin_detail.dept_names = Some(dept_name_data);

    // 查询用户关联的岗位
    let result_posts = post_service::select_by_admin_id(&db, &admin_detail.id).await.unwrap_or_default();
    let post_data: Vec<Option<String>> = result_posts
        .iter()
        .map(|post| post.id.map(|id| id.to_string()))
        .collect();
    let post_name_data: Vec<Option<String>> = result_posts
        .iter()
        .map(|post| post.post_name.clone())
        .collect();
    admin_detail.post_ids = Some(post_data);
    admin_detail.post_names = Some(post_name_data);


    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(admin_detail, "local")))
}

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
    //查询用户数据权限范围（取最小值，数值越小权限越大）
    let role_details = role_service::select_by_admin_id(&db, &Some(user_info.id)).await?;
    let data_scope = role_details.iter()
        .filter_map(|r| r.data_scope)
        .min();

    let user_info = UserLoginVO {
        id: Option::from(user_info.id),
        username: user_info.user_name,
        nickname: user_info.nick_name,
        email: user_info.email,
        avatar: user_info.avatar,
        roles,
        permissions,
        data_scope,
        post_names: post_service::select_by_admin_id(&db, &Some(user_info.id))
            .await
            .unwrap_or_default()
            .iter()
            .filter_map(|p| p.post_name.clone())
            .collect(),
        dept_names: dept_service::select_by_admin_id(&db, &Some(user_info.id))
            .await
            .unwrap_or_default()
            .iter()
            .filter_map(|d| d.dept_name.clone())
            .collect(),
    };
    
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(user_info, "local")))
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
pub async fn update_avatar(state: web::Data<AppState>, req: HttpRequest, item: web::Json<UpdateAvatarRequest>) -> Result<HttpResponse> {
    if is_demo_mode() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "演示站模式下禁止修改头像", "local")));
    }
    let db = &state.db;
    let admin_token: JWTToken = get_user(&req).unwrap_or_default();
    let user_id = match admin_token.id {
        Some(id) if id > 0 => id,
        _ => return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "用户未登录或token无效", "local"))),
    };

    let avatar = item.into_inner().avatar;
    if avatar.trim().is_empty() {
        return Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, "头像地址不能为空", "local")));
    }

    match AdminModel::update_avatar(db, user_id, &avatar).await {
        Ok(_) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(avatar, "local"))),
        Err(e) => Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::fail(400, &format!("更新头像失败: {}", e), "local"))),
    }
}

// 查询用户列表
pub async fn admin_list(state: web::Data<AppState>, query: web::Query<ListQuery>,) -> Result<HttpResponse> {
    let db = &state.db;
    admin_service::get_by_page(db, query.into_inner()).await.map(|page_data| {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(page_data, "local"))
    })
}

pub async fn admin_options(state: web::Data<AppState>) -> Result<HttpResponse> {
    let db = &state.db;
    admin_service::get_admin_options(db).await.map(|list_data| {
        HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(list_data, "local"))
    })
}


// 获取权限码列表
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
    
    Ok(HttpResponse::Ok().content_type(MPACK).body(MetaResp::success(user_role_keys, "local")))
}

// 退出登录
pub async fn logout() -> HttpResponse {
    HttpResponse::Ok().content_type(MPACK).body(MetaResp::<String>::success(String::new(), "local"))
}

// ==================== 路由注册（方案 C：单点维护）====================

/// 注册后台用户管理模块所有路由
///
/// 本函数集中管理本模块所有路由的路径、HTTP 方法、权限码。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(system_admin_controller::register)` 注册。
///
/// ## 路由分组
/// - `/admin/*` — 后台用户管理（增删改查、改密码、改头像、改状态、改角色）
/// - `/auth/*`  — 认证相关（登录、注册、检查用户名、获取权限码）
/// - `/api/auth/logout` — 注销（绝对路径，不在 `/api/system` scope 下，用 web::resource 注册）
///
/// ## 重要：Route::to() 与 wrap() 的调用顺序
/// `Route::to()` 会覆盖之前 `wrap()` 设置的中间件，所以**必须先调用 `to()`
/// 再调用 `wrap()`**，否则权限中间件不会生效。
pub fn register(cfg: &mut web::ServiceConfig) {
    // ============ /admin 路由组（后台用户管理）============
    cfg.service(
        web::scope("/admin")
            // POST /admin/add - 添加用户
            .route("/add", web::post().to(save_admin))
            // DELETE /admin/batch_delete - 批量删除用户
            .route("/batch_delete", web::delete().to(admin_batch_delete))
            // PUT /admin/update - 更新用户信息
            .route("/update", web::put().to(admin_update))
            // PUT /admin/update_password - 管理员重置用户密码
            .route("/update_password", web::put().to(update_password))
            // PUT /admin/update_my_password - 用户修改自己的登录密码
            .route("/update_my_password", web::put().to(update_my_password))
            // PUT /admin/update-status - 更新用户状态
            .route("/update-status", web::put().to(update_admin_status))
            // PUT /admin/update_user_role - 更新用户角色
            .route("/update_user_role", web::put().to(update_user_role))
            // GET /admin/detail/{id} - 用户详情
            .route("/detail/{id}", web::get().to(get_user_detail))
            // GET /admin/userinfo - 获取当前登录用户信息
            .route("/userinfo", web::get().to(get_user_info))
            // PUT /admin/avatar - 修改头像
            .route("/avatar", web::put().to(update_avatar))
            // GET /admin/options - 用户下拉选项
            .route("/options", web::get().to(admin_options))
            // GET /admin/list - 用户列表（带权限校验）
            .route(
                "/list",
                web::get()
                    .to(admin_list)
                    .wrap(require_permission("system:admin:list")),
            )
            // DELETE /admin/delete/{id} - 软删除用户（带权限校验）
            .route(
                "/delete/{id}",
                web::delete()
                    .to(admin_soft_delete)
                    .wrap(require_permission("system:admin:delete")),
            )
            // POST /admin/kick-offline/{id} - 踢用户下线（v1.1 登录安全）
            .route(
                "/kick-offline/{id}",
                web::post()
                    .to(kick_offline)
                    .wrap(require_permission("system:admin:kick")),
            )
            // PUT /admin/audit/{id} - 审核注册用户（通过/拒绝）
            .route(
                "/audit/{id}",
                web::put()
                    .to(audit_user)
                    .wrap(require_permission("system:admin:audit")),
            ),
    );

    // ============ /auth 路由组（认证相关）============
    cfg.service(
        web::scope("/auth")
            // POST /auth/login - 后台用户登录
            .route("/login", web::post().to(post_login))
            // POST /auth/register - 用户注册
            .route("/register", web::post().to(user_register))
            // GET /auth/register-status - 查询注册开关（免鉴权，供登录页判断是否显示注册入口）
            .route("/register-status", web::get().to(register_status))
            // GET /auth/check-username - 检查用户名是否已存在
            .route("/check-username", web::get().to(check_username))
            // GET /auth/codes - 获取当前用户权限码列表
            .route("/codes", web::get().to(get_auth_codes)),
    );
}
