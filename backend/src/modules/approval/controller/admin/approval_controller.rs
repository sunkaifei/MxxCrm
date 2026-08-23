//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
use crate::core::errors::error::{Error, Result};
use crate::core::kit::global::AppState;
use crate::core::web::base_controller::{get_current_user, get_current_user_id};
use crate::core::web::permission_guard::require_permission;
use crate::core::web::response::{MetaResp, MPACK};
use crate::modules::approval::model::approval::{
    ApprovalProcessRequest, ApprovalSubmitRequest, FlowListQuery, FlowSaveRequest,
    ApprovalCancelRequest, ApprovalRejectToRequest, ApprovalTransferRequest,
    ApprovalDelegateRequest, ApprovalAddSignRequest, ApprovalCcRequest,
};
use crate::modules::approval::service::approval_service::ApprovalService;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use actix_web_grants::authorities::AuthDetails;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PageQuery {
    #[serde(rename = "page")]
    pub page_num: u64,
    #[serde(rename = "pageSize")]
    pub page_size: u64,
    #[serde(rename = "flowName")]
    pub flow_name: Option<String>,
    #[serde(rename = "businessType")]
    pub business_type: Option<String>,
}

#[derive(Deserialize)]
pub struct CcListQuery {
    #[serde(rename = "page")]
    pub page: u64,
    #[serde(rename = "pageSize")]
    pub page_size: u64,
    /// 已读状态过滤：0=未读,1=已读,不传=全部
    pub is_read: Option<i32>,
}

pub async fn save_flow(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<FlowSaveRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let (_, operator) = get_current_user(&req);
    match ApprovalService::save_flow(db, &payload.0, &operator).await {
        Ok(id) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(id, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local"))),
    }
}

pub async fn flow_detail(
    state: web::Data<AppState>,
    id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match ApprovalService::find_flow_by_id(db, id.into_inner()).await {
        Ok(data) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local"))),
    }
}

pub async fn flow_list(
    state: web::Data<AppState>,
    query: web::Query<PageQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let q = FlowListQuery {
        page_num: query.page_num,
        page_size: query.page_size,
        flow_name: query.flow_name.clone(),
        business_type: query.business_type.clone(),
    };
    match ApprovalService::find_flow_list(db, &q).await {
        Ok(data) => {
            let page = data.current_page as u32;
            let total = data.total as u32;
            Ok(HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::success_with_page(data, "local", page, total)))
        }
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local"))),
    }
}

/// 按流程编码预览流程（B2：提交审核抽屉 / 个人中心提交前展示，仅登录鉴权，无 system:approval:list 要求）
pub async fn flow_preview(
    state: web::Data<AppState>,
    code: web::Path<String>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match ApprovalService::find_flow_vo_by_code(db, &code.into_inner()).await {
        Ok(Some(data)) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(data, "local"))),
        Ok(None) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(404, "审批流程不存在或未启用", "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local"))),
    }
}

pub async fn toggle_flow(
    state: web::Data<AppState>,
    id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match ApprovalService::toggle_flow(db, id.into_inner()).await {
        Ok(_) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(true, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local"))),
    }
}

pub async fn delete_flow(
    state: web::Data<AppState>,
    id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    match ApprovalService::delete_flow(db, id.into_inner()).await {
        Ok(_) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(true, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local"))),
    }
}

pub async fn submit_approval(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<ApprovalSubmitRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    // 安全修复：submitter_id 从 JWT 取，防止客户端伪造发起人身份
    let (current_user_id, username) = get_current_user(&req);
    if current_user_id == 0 {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(401, "未获取到登录用户信息", "local")));
    }
    let mut req_data = payload.0;
    req_data.submitter_id = current_user_id;
    req_data.submitter_name = Some(username);
    match ApprovalService::submit(db, &req_data).await {
        Ok(id) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(id, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local"))),
    }
}

pub async fn process_approval(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<ApprovalProcessRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    // 安全修复：approver_id 从 JWT 取，防止客户端伪造审批人身份
    let current_user_id = get_current_user_id(&req);
    if current_user_id == 0 {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(401, "未获取到登录用户信息", "local")));
    }
    let mut req_data = payload.0;
    // 强制覆盖为当前登录用户，防止代审
    req_data.approver_id = current_user_id;
    match ApprovalService::process(db, &req_data).await {
        Ok(_) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(true, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local"))),
    }
}

pub async fn approval_detail(
    state: web::Data<AppState>,
    req: HttpRequest,
    id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let instance_id = id.into_inner();
    match ApprovalService::find_instance_by_id(db, instance_id).await {
        Ok(Some(data)) => {
            let current_user_id = get_current_user_id(&req);
            // 放行条件（B3）：发起人本人，或持有 system:approval:todo（审批人/管理员入口）
            let is_submitter = data.submitter_id == current_user_id;
            let has_todo = req
                .extensions()
                .get::<AuthDetails<String>>()
                .map(|d| d.authorities.contains("system:approval:todo"))
                .unwrap_or(false);
            if !is_submitter && !has_todo {
                return Ok(HttpResponse::Ok()
                    .content_type(MPACK)
                    .body(MetaResp::<String>::fail(403, "无权查看该审批实例", "local")));
            }
            Ok(HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::success(data, "local")))
        }
        Ok(None) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(404, "审批实例不存在", "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local"))),
    }
}

pub async fn approval_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<PageQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let approver_id = get_current_user_id(&req);
    match ApprovalService::find_instance_list(db, approver_id, query.page_num, query.page_size).await {
        Ok(data) => {
            let page = data.current_page as u32;
            let total = data.total as u32;
            Ok(HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::success_with_page(data, "local", page, total)))
        }
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(500, &e.to_string(), "local"))),
    }
}

// ==================== 审批增强功能：取消/退回/转办/委派/加签/抄送 ====================

/// 发起人取消（撤回）审批
pub async fn cancel_approval(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<ApprovalCancelRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let (current_user_id, username) = get_current_user(&req);
    if current_user_id == 0 {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(401, "未获取到登录用户信息", "local")));
    }
    match ApprovalService::cancel_instance(db, &payload.0, current_user_id, &username).await {
        Ok(_) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(true, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 退回（退回到发起人或指定节点）
pub async fn reject_to_approval(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<ApprovalRejectToRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let (current_user_id, username) = get_current_user(&req);
    if current_user_id == 0 {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(401, "未获取到登录用户信息", "local")));
    }
    match ApprovalService::reject_to(db, &payload.0, current_user_id, &username).await {
        Ok(_) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(true, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 转办（当前审批人转给他人，责任转移）
pub async fn transfer_approval(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<ApprovalTransferRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let (current_user_id, username) = get_current_user(&req);
    if current_user_id == 0 {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(401, "未获取到登录用户信息", "local")));
    }
    match ApprovalService::transfer(db, &payload.0, current_user_id, &username).await {
        Ok(_) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(true, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 委派（委托他人处理，责任仍归原审批人）
pub async fn delegate_approval(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<ApprovalDelegateRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let (current_user_id, username) = get_current_user(&req);
    if current_user_id == 0 {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(401, "未获取到登录用户信息", "local")));
    }
    match ApprovalService::delegate(db, &payload.0, current_user_id, &username).await {
        Ok(_) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(true, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 加签（前加签/后加签/并加签）
pub async fn add_sign_approval(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<ApprovalAddSignRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let (current_user_id, username) = get_current_user(&req);
    if current_user_id == 0 {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(401, "未获取到登录用户信息", "local")));
    }
    match ApprovalService::add_sign(db, &payload.0, current_user_id, &username).await {
        Ok(_) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(true, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 添加抄送
pub async fn add_cc_approval(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<ApprovalCcRequest>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let (current_user_id, username) = get_current_user(&req);
    if current_user_id == 0 {
        return Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(401, "未获取到登录用户信息", "local")));
    }
    match ApprovalService::add_cc(db, &payload.0, current_user_id, &username).await {
        Ok(_) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(true, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 抄送列表（我的抄送）
pub async fn cc_list(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<CcListQuery>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    let is_read = query.is_read;
    match ApprovalService::find_cc_list(db, user_id, is_read, query.page, query.page_size).await {
        Ok(data) => {
            let page = data.current_page as u32;
            let total = data.total as u32;
            Ok(HttpResponse::Ok()
                .content_type(MPACK)
                .body(MetaResp::success_with_page(data, "local", page, total)))
        }
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

/// 标记抄送为已读
pub async fn cc_mark_read(
    state: web::Data<AppState>,
    req: HttpRequest,
    id: web::Path<i64>,
) -> Result<HttpResponse> {
    let db = &state.db;
    let user_id = get_current_user_id(&req);
    match ApprovalService::mark_cc_read(db, id.into_inner(), user_id).await {
        Ok(_) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::success(true, "local"))),
        Err(e) => Ok(HttpResponse::Ok()
            .content_type(MPACK)
            .body(MetaResp::<String>::fail(400, &e.to_string(), "local"))),
    }
}

// ==================== 路由注册（单点维护）====================

/// 注册审批模块所有路由
///
/// 修改路径、HTTP 方法只需修改本函数。
/// 调用方在 `admin_routes.rs` 中通过 `cfg.configure(approval_controller::register)` 注册。
/// 权限设计（与 mxx_system_menu 表权限码对应）：
/// - 流程管理（save/toggle/delete）：system:approval:add / system:approval:toggle / system:approval:delete
/// - 流程查询（detail/list）：system:approval:list / system:approval:todo
/// - 审批操作（submit/process）：登录鉴权 + 候选池校验（业务权限由各业务模块控制器校验）
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/approval")
            // POST /approval/flow/save - 保存审批流（需要 system:approval:add）
            .route(
                "/flow/save",
                web::post().to(save_flow).wrap(require_permission("system:approval:add")),
            )
            // GET /approval/flow/detail/{id} - 审批流详情（需要 system:approval:list）
            .route(
                "/flow/detail/{id}",
                web::get().to(flow_detail).wrap(require_permission("system:approval:list")),
            )
            // GET /approval/flow/list - 审批流列表（需要 system:approval:list）
            .route(
                "/flow/list",
                web::get().to(flow_list).wrap(require_permission("system:approval:list")),
            )
            // GET /approval/flow/preview/{code} - 流程预览（B2：提交审核抽屉/个人中心，仅登录鉴权，无 system:approval:list 要求）
            .route(
                "/flow/preview/{code}",
                web::get().to(flow_preview),
            )
            // POST /approval/flow/toggle/{id} - 启用/停用审批流（需要 system:approval:toggle）
            .route(
                "/flow/toggle/{id}",
                web::post().to(toggle_flow).wrap(require_permission("system:approval:toggle")),
            )
            // POST /approval/flow/delete/{id} - 删除审批流（需要 system:approval:delete）
            .route(
                "/flow/delete/{id}",
                web::post().to(delete_flow).wrap(require_permission("system:approval:delete")),
            )
            // POST /approval/submit - 提交审批（登录鉴权 + 业务权限由业务模块控制器校验）
            .route(
                "/submit",
                web::post().to(submit_approval),
            )
            // POST /approval/process - 处理审批（登录鉴权 + 候选池校验，业务权限由业务模块控制器校验）
            .route(
                "/process",
                web::post().to(process_approval),
            )
            // GET /approval/detail/{id} - 审批实例详情（B3：发起人本人放行；审批人/管理员需 system:approval:todo）
            .route(
                "/detail/{id}",
                web::get().to(approval_detail),
            )
            // GET /approval/list - 审批实例列表（需要 system:approval:todo）
            .route(
                "/list",
                web::get().to(approval_list).wrap(require_permission("system:approval:todo")),
            )
            // POST /approval/cancel - 发起人撤回审批（登录鉴权 + 发起人校验）
            .route(
                "/cancel",
                web::post().to(cancel_approval),
            )
            // POST /approval/reject-to - 退回（退回到发起人或指定节点）
            .route(
                "/reject-to",
                web::post().to(reject_to_approval),
            )
            // POST /approval/transfer - 转办
            .route(
                "/transfer",
                web::post().to(transfer_approval),
            )
            // POST /approval/delegate - 委派
            .route(
                "/delegate",
                web::post().to(delegate_approval),
            )
            // POST /approval/add-sign - 加签
            .route(
                "/add-sign",
                web::post().to(add_sign_approval),
            )
            // POST /approval/cc/add - 添加抄送
            .route(
                "/cc/add",
                web::post().to(add_cc_approval),
            )
            // GET /approval/cc/list - 我的抄送列表
            .route(
                "/cc/list",
                web::get().to(cc_list).wrap(require_permission("system:approval:cc:list")),
            )
            // POST /approval/cc/read/{id} - 标记抄送已读
            .route(
                "/cc/read/{id}",
                web::post().to(cc_mark_read).wrap(require_permission("system:approval:cc:list")),
            ),
    );
}
