//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::{web, HttpRequest, HttpResponse};

use crate::core::kit::global::AppState;
use crate::core::web::base_controller::{get_current_user, get_current_user_id};
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::approval::service::approval_service::ApprovalService;
use crate::modules::system::model::admin::AdminModel;
use crate::modules::system::model::profile::{
    BankRequest, BasicUpdateRequest, EmailOtpSendRequest, EmailUpdateRequest,
    EmergencyContactItem, IdCardRequest, MobileUpdateRequest, ProfileLogQuery, ResumeItem,
};
use crate::modules::system::model::resign::ResignApplyRequest;
use crate::modules::system::service::{hr_archive_service, otp_service, profile_service, resign_service};

fn unauthorized() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::<String>::fail(401, "未登录", "local"))
}

/// GET /profile/my - 本人聚合档案（脱敏）
pub async fn get_my(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let admin_id = get_current_user_id(&req);
    if admin_id <= 0 {
        return unauthorized();
    }
    match profile_service::get_my_profile(&state.db, admin_id).await {
        Ok(data) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// POST /profile/resign/apply - 个人中心离职申请（本人发起）
/// 被离职员工强制取 JWT 本人，忽略 body.admin_id（防越权）
pub async fn my_resign_apply(
    state: web::Data<AppState>,
    req: HttpRequest,
    item: web::Json<ResignApplyRequest>,
) -> HttpResponse {
    let db = &state.db;
    let (operator_id, operator_name) = get_current_user(&req);
    if operator_id <= 0 {
        return unauthorized();
    }
    match resign_service::submit_resign(
        db,
        operator_id,
        operator_id,
        &operator_name,
        item.resign_type,
        item.resign_date.clone(),
        item.reason.clone(),
        item.transfer_to_admin_id,
    )
    .await
    {
        Ok(id) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// GET /profile/audit/my - 我的入职审核（B9：查询本人审核状态与历次审批实例，身份只信 JWT）
pub async fn my_audit(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let admin_id = get_current_user_id(&req);
    if admin_id <= 0 {
        return unauthorized();
    }
    let db = &state.db;
    let admin = match AdminModel::find_by_id(db, &Some(admin_id)).await {
        Ok(Some(a)) => a,
        _ => {
            return HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(400, "查询账号信息失败", "local"))
        }
    };
    let instances = match ApprovalService::find_instance_history(db, "user", admin_id).await {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(500, &e.to_string(), "local"))
        }
    };
    let latest = instances.last();
    let data = serde_json::json!({
        "auditStatus": admin.audit_status.unwrap_or(0),
        "latestInstanceId": latest.map(|i| i.id),
        "approvalStatus": latest.map(|i| i.status),
        "instances": instances,
    });
    HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::success(data, "local"))
}

/// GET /profile/resign/my - 我的离职申请（B9：本人交接单列表 + 历次离职审批实例，身份只信 JWT）
pub async fn my_resign(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let admin_id = get_current_user_id(&req);
    if admin_id <= 0 {
        return unauthorized();
    }
    let db = &state.db;
    let records = match resign_service::get_my_list(db, admin_id).await {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(500, &e.to_string(), "local"))
        }
    };
    let instances = match ApprovalService::find_instance_history(db, "resign", admin_id).await {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(500, &e.to_string(), "local"))
        }
    };
    let data = serde_json::json!({
        "records": records,
        "instances": instances,
    });
    HttpResponse::Ok()
        .content_type(MPACK)
        .body(MetaResp::success(data, "local"))
}

/// GET /profile/resign/transfer/my - 我的交接任务（assignee 视角，身份只信 JWT，无需权限码）
pub async fn my_transfer(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let admin_id = get_current_user_id(&req);
    if admin_id <= 0 {
        return unauthorized();
    }
    match resign_service::get_my_transfer_items(&state.db, admin_id).await {
        Ok(data) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// PUT /profile/basic - 白名单字段更新
pub async fn update_basic(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<BasicUpdateRequest>,
) -> HttpResponse {
    let admin_id = get_current_user_id(&req);
    if admin_id <= 0 {
        return unauthorized();
    }
    match profile_service::update_basic(&state.db, admin_id, payload.0).await {
        Ok(_) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<i32>::success(1, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// POST /profile/otp/send - 发送邮箱修改验证码
/// action=email_old 发到当前绑定邮箱；action=email_new 发到请求中的新邮箱
pub async fn otp_send(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<EmailOtpSendRequest>,
) -> HttpResponse {
    let admin_id = get_current_user_id(&req);
    if admin_id <= 0 {
        return unauthorized();
    }
    let action = payload.0.action.trim().to_string();
    let target = match action.as_str() {
        "email_old" => match AdminModel::find_by_id(&state.db, &Some(admin_id)).await {
            Ok(Some(a)) => a.email.unwrap_or_default(),
            _ => {
                return HttpResponse::Ok()
                    .content_type(MPACK)
                    .body(MetaResp::<String>::fail(400, "查询当前账号邮箱失败", "local"))
            }
        },
        "email_new" => payload.0.email.unwrap_or_default(),
        _ => {
            return HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::<String>::fail(400, "不支持的验证码用途", "local"))
        }
    };
    match otp_service::send_to_email(&state.db, admin_id, &action, &target).await {
        Ok(masked) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(masked, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e, "local")),
    }
}

/// PUT /profile/email - 修改本人邮箱（登录密码 + 按需验证码）
pub async fn update_email(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<EmailUpdateRequest>,
) -> HttpResponse {
    let admin_id = get_current_user_id(&req);
    if admin_id <= 0 {
        return unauthorized();
    }
    match profile_service::update_email(&state.db, admin_id, payload.0).await {
        Ok(_) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<i32>::success(1, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// PUT /profile/mobile - 修改本人手机号（登录密码验证）
pub async fn update_mobile(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<MobileUpdateRequest>,
) -> HttpResponse {
    let admin_id = get_current_user_id(&req);
    if admin_id <= 0 {
        return unauthorized();
    }
    match profile_service::update_mobile(&state.db, admin_id, payload.0).await {
        Ok(_) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<i32>::success(1, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// POST /profile/id-card - 身份证首填
pub async fn submit_id_card(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<IdCardRequest>,
) -> HttpResponse {
    let admin_id = get_current_user_id(&req);
    if admin_id <= 0 {
        return unauthorized();
    }
    match profile_service::submit_id_card(&state.db, admin_id, payload.0).await {
        Ok(_) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<i32>::success(1, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// POST /profile/bank - 工资卡首填
pub async fn submit_bank(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<BankRequest>,
) -> HttpResponse {
    let admin_id = get_current_user_id(&req);
    if admin_id <= 0 {
        return unauthorized();
    }
    match profile_service::submit_bank(&state.db, admin_id, payload.0).await {
        Ok(_) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<i32>::success(1, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// GET /profile/resume - 简历列表
pub async fn resume_list(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let admin_id = get_current_user_id(&req);
    if admin_id <= 0 {
        return unauthorized();
    }
    match profile_service::resume_list(&state.db, admin_id).await {
        Ok(data) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local")),
    }
}

/// POST /profile/resume - 新增简历条目
pub async fn resume_save(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<ResumeItem>,
) -> HttpResponse {
    let admin_id = get_current_user_id(&req);
    if admin_id <= 0 {
        return unauthorized();
    }
    let op = get_current_user(&req).1;
    match profile_service::resume_save(&state.db, admin_id, payload.0, &op).await {
        Ok(id) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// PUT /profile/resume - 修改简历条目
pub async fn resume_update(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<ResumeItem>,
) -> HttpResponse {
    let admin_id = get_current_user_id(&req);
    if admin_id <= 0 {
        return unauthorized();
    }
    let op = get_current_user(&req).1;
    match profile_service::resume_update(&state.db, admin_id, payload.0, &op).await {
        Ok(_) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<i32>::success(1, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// DELETE /profile/resume/{id} - 删除简历条目
pub async fn resume_delete(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<i64>,
) -> HttpResponse {
    let admin_id = get_current_user_id(&req);
    if admin_id <= 0 {
        return unauthorized();
    }
    match profile_service::resume_delete(&state.db, admin_id, path.into_inner()).await {
        Ok(_) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<i32>::success(1, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// GET /profile/emergency-contact - 紧急联系人列表
pub async fn contact_list(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let admin_id = get_current_user_id(&req);
    if admin_id <= 0 {
        return unauthorized();
    }
    match profile_service::contact_list(&state.db, admin_id).await {
        Ok(data) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local")),
    }
}

/// POST /profile/emergency-contact - 新增紧急联系人
pub async fn contact_save(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<EmergencyContactItem>,
) -> HttpResponse {
    let admin_id = get_current_user_id(&req);
    if admin_id <= 0 {
        return unauthorized();
    }
    let op = get_current_user(&req).1;
    match profile_service::contact_save(&state.db, admin_id, payload.0, &op).await {
        Ok(id) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(id, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// PUT /profile/emergency-contact - 修改紧急联系人
pub async fn contact_update(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<EmergencyContactItem>,
) -> HttpResponse {
    let admin_id = get_current_user_id(&req);
    if admin_id <= 0 {
        return unauthorized();
    }
    let op = get_current_user(&req).1;
    match profile_service::contact_update(&state.db, admin_id, payload.0, &op).await {
        Ok(_) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<i32>::success(1, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// DELETE /profile/emergency-contact/{id} - 删除紧急联系人
pub async fn contact_delete(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<i64>,
) -> HttpResponse {
    let admin_id = get_current_user_id(&req);
    if admin_id <= 0 {
        return unauthorized();
    }
    match profile_service::contact_delete(&state.db, admin_id, path.into_inner()).await {
        Ok(_) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<i32>::success(1, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// GET /profile/card/{adminId} - 同事名片（登录即可）
pub async fn get_card(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<i64>,
) -> HttpResponse {
    if get_current_user_id(&req) <= 0 {
        return unauthorized();
    }
    match profile_service::get_card(&state.db, path.into_inner()).await {
        Ok(data) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local")),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local")),
    }
}

/// GET /profile/log - 本人变更日志
pub async fn my_log(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<ProfileLogQuery>,
) -> HttpResponse {
    let admin_id = get_current_user_id(&req);
    if admin_id <= 0 {
        return unauthorized();
    }
    let mut q = query.0;
    q.admin_id = Some(admin_id);
    match hr_archive_service::get_log_page(&state.db, q).await {
        Ok(page) => HttpResponse::Ok().content_type(MPACK).body(MetaResp::success_with_page(
            page.items, "local", page.current_page as u32, page.total as u32,
        )),
        Err(e) => HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local")),
    }
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/profile")
            .route("/my", web::get().to(get_my))
            .route("/basic", web::put().to(update_basic))
            // 个人中心离职申请（本人发起，身份只信 JWT）
            .route("/resign/apply", web::post().to(my_resign_apply))
            // B9：我的入职审核 / 我的离职申请（身份只信 JWT）
            .route("/audit/my", web::get().to(my_audit))
            .route("/resign/my", web::get().to(my_resign))
            // 我的交接任务（assignee 视角，身份只信 JWT，无需权限码）
            .route("/resign/transfer/my", web::get().to(my_transfer))
            .route("/otp/send", web::post().to(otp_send))
            .route("/email", web::put().to(update_email))
            .route("/mobile", web::put().to(update_mobile))
            .route("/id-card", web::post().to(submit_id_card))
            .route("/bank", web::post().to(submit_bank))
            .route("/resume", web::get().to(resume_list))
            .route("/resume", web::post().to(resume_save))
            .route("/resume", web::put().to(resume_update))
            .route("/resume/{id}", web::delete().to(resume_delete))
            .route("/emergency-contact", web::get().to(contact_list))
            .route("/emergency-contact", web::post().to(contact_save))
            .route("/emergency-contact", web::put().to(contact_update))
            .route("/emergency-contact/{id}", web::delete().to(contact_delete))
            .route("/card/{adminId}", web::get().to(get_card))
            .route("/log", web::get().to(my_log)),
    );
}
