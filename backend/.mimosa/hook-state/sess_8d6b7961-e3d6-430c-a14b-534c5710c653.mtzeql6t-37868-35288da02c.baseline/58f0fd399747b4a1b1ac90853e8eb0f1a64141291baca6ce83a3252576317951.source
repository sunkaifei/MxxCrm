//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::HttpRequest;
use sea_orm::DbConn;
use crate::core::web::response::ResultPage;
use crate::core::errors::error::Result;
use crate::modules::system::model::system_log::{ListQuery, LogListVO, PageWhere, SystemLogModel, SystemLogSaveDTO, SystemLogSaveRequest};
use crate::SNOWFLAKE;

/// 完整的日志保存参数
///
/// 由中间件或 controller 在已收集完所有上下文（含响应结果、耗时）后调用。
pub struct SaveLogContext<'a> {
    pub request: &'a HttpRequest,
    pub title: Option<String>,
    /// 0=其它 1=新增 2=修改 3=删除
    pub business_type: Option<i32>,
    /// 方法名（一般是 controller path 或 handler 名）
    pub method: Option<String>,
    /// HTTP 方法
    pub request_method: Option<String>,
    /// 0=其它 1=后台用户 2=手机端用户
    pub operator_type: Option<i32>,
    /// 操作人账号
    pub oper_name: Option<String>,
    /// 操作人部门名
    pub dept_name: Option<String>,
    /// 请求参数（JSON 字符串，已截断）
    pub oper_param: Option<String>,
    /// 响应结果（JSON 字符串，已截断）
    pub json_result: Option<String>,
    /// 0=正常 1=异常
    pub status: Option<i32>,
    pub error_msg: Option<String>,
    /// HTTP 响应状态码
    pub status_code: Option<i32>,
    /// 接口耗时（毫秒）
    pub elapsed: Option<i64>,
}

/// #保存系统日志（完整字段）
///
/// 中间件推荐使用此版本：调用方一次性把已收集的请求/响应/用户信息传进来。
pub async fn save_log(db: &DbConn, ctx: SaveLogContext<'_>) -> Result<i64> {
    let uri = ctx.request.uri();
    let url = format!(
        "{}{}",
        uri.path(),
        uri.query().map(|q| format!("?{}", q)).unwrap_or_default()
    );
    let oper_ip = ctx
        .request
        .connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string());

    let system_entity = SystemLogSaveRequest {
        id: Some(SNOWFLAKE.generate() as i64),
        title: ctx.title,
        business_type: ctx.business_type,
        method: ctx.method,
        request_method: ctx.request_method,
        operator_type: ctx.operator_type,
        oper_url: Some(url),
        oper_ip,
    };

    let mut dto: SystemLogSaveDTO = system_entity.into();
    dto.oper_name = ctx.oper_name;
    dto.dept_name = ctx.dept_name;
    dto.oper_param = ctx.oper_param;
    dto.json_result = ctx.json_result;
    dto.status = ctx.status;
    dto.error_msg = ctx.error_msg;
    dto.status_code = ctx.status_code;
    dto.elapsed = ctx.elapsed;

    let result = SystemLogModel::insert(&db, dto).await?;
    Ok(result)
}

/// #保存系统日志（带预设 IP）
///
/// 与 `save_log` 区别：调用方已提取好 `oper_ip`，避免重复读取 `connection_info`。
/// 用于登录场景：登录前 IP 已读取用于其他用途，这里直接复用。
pub async fn save_log_with_ip(db: &DbConn, ctx: SaveLogContext<'_>, oper_ip_override: Option<String>) -> Result<i64> {
    let uri = ctx.request.uri();
    let url = format!(
        "{}{}",
        uri.path(),
        uri.query().map(|q| format!("?{}", q)).unwrap_or_default()
    );
    let oper_ip = oper_ip_override.or_else(|| {
        ctx.request
            .connection_info()
            .realip_remote_addr()
            .map(|s| s.to_string())
    });

    let system_entity = SystemLogSaveRequest {
        id: Some(SNOWFLAKE.generate() as i64),
        title: ctx.title,
        business_type: ctx.business_type,
        method: ctx.method,
        request_method: ctx.request_method,
        operator_type: ctx.operator_type,
        oper_url: Some(url),
        oper_ip,
    };

    let mut dto: SystemLogSaveDTO = system_entity.into();
    dto.oper_name = ctx.oper_name;
    dto.dept_name = ctx.dept_name;
    dto.oper_param = ctx.oper_param;
    dto.json_result = ctx.json_result;
    dto.status = ctx.status;
    dto.error_msg = ctx.error_msg;
    dto.status_code = ctx.status_code;
    dto.elapsed = ctx.elapsed;

    let result = SystemLogModel::insert(&db, dto).await?;
    Ok(result)
}

/// #兼容旧调用：仅根据请求记录"操作发生"日志（不含响应结果）
///
/// 保留给已有的 `system_admin_controller::login` 等手写调用点使用。
/// 新代码请改用 `save_log` + `SaveLogContext`。
pub async fn save_system_log(
    db: &DbConn,
    request: &HttpRequest,
    title: Option<String>,
    business_type: Option<i32>,
    method: Option<String>,
    request_method: Option<String>,
    operator_type: Option<i32>,
) -> Result<i64> {
    let ctx = SaveLogContext {
        request,
        title,
        business_type,
        method,
        request_method,
        operator_type,
        oper_name: None,
        dept_name: None,
        oper_param: None,
        json_result: None,
        status: None,
        error_msg: None,
        status_code: None,
        elapsed: None,
    };
    save_log(db, ctx).await
}


/// 查询系统日志分页列表
pub async fn get_by_page(db: &DbConn, query: ListQuery) -> Result<ResultPage<Vec<LogListVO>>> {
    let select_where = PageWhere {
        title: query.title,
        business_type: query.business_type,
        status: query.status,
        operator_type: query.operator_type,
        oper_name: query.oper_name,
        begin_time: query.begin_time,
        end_time: query.end_time,
    };
    let search_where = select_where.format();

    let (list, _num_pages) = SystemLogModel::select_in_page(
        &db,
        query.page_num.unwrap_or(0),
        query.page_size.unwrap_or(10),
        search_where.clone()
    ).await?;

    let list_data: Vec<LogListVO> = list.into_iter().map(LogListVO::from).collect();

    let count = SystemLogModel::select_count(db, select_where).await.unwrap_or(0);

    let page_data = ResultPage::new_simple(list_data, count);
    Ok(page_data)
}
