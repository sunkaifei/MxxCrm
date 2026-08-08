//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//! 

//! 演示模式拦截中间件
//!
//! 开启 demo_mode 后，仅拦截核心系统数据的写操作（POST/PUT/DELETE/PATCH），
//! 包括：员工信息、角色、系统设置、上传功能。
//! 其余业务模块（CRM、销售、采购等）正常使用，访客可自由体验。

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::{Error, error};
use futures_util::future::LocalBoxFuture;

/// 演示模式下禁止修改的路径前缀
const DEMO_PROTECTED: &[&str] = &[
    "/api/system/admin",     // 员工信息管理
    "/api/system/role",      // 角色管理
    "/api/system/setting",   // 系统设置
    "/api/system/attachment", // 附件/文件上传
];

/// 演示模式拦截中间件
pub struct DemoGuard;

impl DemoGuard {
    pub fn new() -> Self {
        DemoGuard
    }
}

impl<S, B> Transform<S, ServiceRequest> for DemoGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = DemoGuardMiddleware<S>;
    type InitError = ();
    type Future = LocalBoxFuture<'static, Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        Box::pin(async move { Ok(DemoGuardMiddleware { service }) })
    }
}

pub struct DemoGuardMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for DemoGuardMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // 非演示模式直接放行
        if !crate::core::kit::app::is_demo_mode() {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await });
        }

        let method = req.method().clone();
        let path = req.path().to_string();

        // 非写操作直接放行
        let is_write = matches!(
            method,
            actix_web::http::Method::POST
                | actix_web::http::Method::PUT
                | actix_web::http::Method::DELETE
                | actix_web::http::Method::PATCH
        );

        if !is_write {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await });
        }

        // 仅拦截受保护的系统数据路径
        let is_protected = DEMO_PROTECTED
            .iter()
            .any(|prefix| path.starts_with(prefix));

        if is_protected {
            let err: Error = error::ErrorForbidden(
                "演示站模式下，员工、角色、系统设置等核心数据为只读，无法修改",
            );
            Box::pin(async { Err(err) })
        } else {
            // 其他模块正常放行
            let fut = self.service.call(req);
            Box::pin(async move { fut.await })
        }
    }
}
