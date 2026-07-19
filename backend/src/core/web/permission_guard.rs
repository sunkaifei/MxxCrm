//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//!

//! 权限校验中间件
//!
//! 用于替代 `actix-web-grants::protect` 宏，在路由层集中声明权限码，
//! 未授权时返回 HTTP 403 Forbidden（与原 `#[protect]` 行为一致）。
//!
//! ## 用法
//!
//! ```ignore
//! use actix_web::web;
//! use crate::core::web::permission_guard::require_permission;
//!
//! web::resource("/sale/shipment/save")
//!     .route(web::post()
//!         .to(handler::save)
//!         .wrap(require_permission("sale:shipment:create")))
//! ```
//!
//! ## 重要：Route::to() 与 wrap() 的调用顺序
//!
//! `Route::to()` 会覆盖之前 `wrap()` 设置的中间件，所以**必须先调用 `to()`
//! 再调用 `wrap()`**，否则权限中间件不会生效：
//!
//! ```ignore
//! // 正确 ✓
//! web::post().to(handler).wrap(require_permission("perm"))
//!
//! // 错误 ✗ - to() 会覆盖 wrap()，权限检查不生效
//! web::post().wrap(require_permission("perm")).to(handler)
//! ```
//!
//! ## 权限数据来源
//!
//! 权限集合由 [`crate::routes::admin_routes::extract`] 中间件从 JWT 中解析，
//! 通过 `actix-web-grants` 的 `GrantsMiddleware` 注入到请求扩展中，
//! 类型为 `AuthDetails<String>`（内部包装了 `Arc<HashSet<String>>`）。
//! 本中间件从 `AuthDetails` 读取权限集合并做 `contains` 判断。

use std::collections::HashSet;

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::{Error, HttpMessage, error};
use actix_web_grants::authorities::AuthDetails;
use futures_util::future::LocalBoxFuture;

/// 权限校验中间件：未通过返回 403 Forbidden
///
/// 创建一个中间件实例，传入权限码（字符串字面量，编译期确定）。
pub fn require_permission(code: &'static str) -> PermissionGuard {
    PermissionGuard { code }
}

#[derive(Clone, Debug)]
pub struct PermissionGuard {
    code: &'static str,
}

impl<S, B> Transform<S, ServiceRequest> for PermissionGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = PermissionGuardMiddleware<S>;
    type InitError = ();
    type Future = LocalBoxFuture<'static, Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let code = self.code;
        Box::pin(async move { Ok(PermissionGuardMiddleware { service, code }) })
    }
}

pub struct PermissionGuardMiddleware<S> {
    service: S,
    code: &'static str,
}

impl<S, B> Service<ServiceRequest> for PermissionGuardMiddleware<S>
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
        // 从请求扩展中取出权限集合（由 GrantsMiddleware::with_extractor 注入）
        // actix-web-grants 把权限存储在 AuthDetails<String> 包装类型中，
        // 不是直接存 HashSet<String>，所以必须用 AuthDetails 类型来读取。
        let has_perm = req
            .extensions()
            .get::<AuthDetails<String>>()
            .map(|details| details.authorities.contains(self.code))
            .unwrap_or(false);

        if has_perm {
            // 通过：继续后续 service 调用
            let fut = self.service.call(req);
            Box::pin(async move { fut.await })
        } else {
            // 拒绝：返回 403 Forbidden
            let err: Error = error::ErrorForbidden(format!("缺少权限: {}", self.code));
            Box::pin(async { Err(err) })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::http::StatusCode;
    use actix_web::{test, web, App, HttpResponse, ResponseError};

    async fn ok_handler() -> HttpResponse {
        HttpResponse::Ok().body("success")
    }

    /// 调用 service 并返回状态码：Err(Error) 时通过 error_response() 提取状态码
    /// （模拟 HTTP server 层把 Error 转换成 HttpResponse 的行为）
    macro_rules! call_and_status {
        ($app:expr, $req:expr) => {{
            match $app.call($req).await {
                Ok(resp) => resp.status(),
                Err(err) => err.error_response().status(),
            }
        }};
    }

    /// 测试用辅助中间件：向请求扩展中注入 AuthDetails（模拟 GrantsMiddleware 的行为）
    struct InjectPermissions {
        perms: HashSet<String>,
    }

    impl<S, B> Transform<S, ServiceRequest> for InjectPermissions
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
        S::Future: 'static,
        B: 'static,
    {
        type Response = ServiceResponse<B>;
        type Error = Error;
        type Transform = InjectPermissionsMiddleware<S>;
        type InitError = ();
        type Future = LocalBoxFuture<'static, Result<Self::Transform, Self::InitError>>;

        fn new_transform(&self, service: S) -> Self::Future {
            let perms = self.perms.clone();
            Box::pin(async move { Ok(InjectPermissionsMiddleware { service, perms }) })
        }
    }

    struct InjectPermissionsMiddleware<S> {
        service: S,
        perms: HashSet<String>,
    }

    impl<S, B> Service<ServiceRequest> for InjectPermissionsMiddleware<S>
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
        S::Future: 'static,
        B: 'static,
    {
        type Response = ServiceResponse<B>;
        type Error = Error;
        type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

        forward_ready!(service);

        fn call(&self, mut req: ServiceRequest) -> Self::Future {
            // 注入 AuthDetails<String>（与 GrantsMiddleware 的行为一致）
            req.extensions_mut()
                .insert(AuthDetails::new(self.perms.clone()));
            let fut = self.service.call(req);
            Box::pin(async move { fut.await })
        }
    }

    /// 构造测试 app 并注入指定权限集合
    macro_rules! build_app_with_perms {
        ($perms:expr) => {
            test::init_service(
                App::new().service(
                    web::scope("/api")
                        .wrap(InjectPermissions { perms: $perms })
                        .route(
                            "/test",
                            web::get().to(ok_handler).wrap(require_permission("test:perm")),
                        ),
                ),
            )
            .await
        };
    }

    /// Test 4a：有权限 → 返回 200 OK
    #[actix_rt::test]
    async fn test_permission_granted_returns_200() {
        let mut perms = HashSet::new();
        perms.insert("test:perm".to_string());

        let app = build_app_with_perms!(perms);

        let req = test::TestRequest::get().uri("/api/test").to_request();
        let status = call_and_status!(&app, req);
        assert_eq!(status, StatusCode::OK, "有权限时应返回 200");
    }

    /// Test 4b：无权限（空集合）→ 返回 403 Forbidden
    #[actix_rt::test]
    async fn test_permission_denied_returns_403() {
        let perms = HashSet::<String>::new();

        let app = build_app_with_perms!(perms);

        let req = test::TestRequest::get().uri("/api/test").to_request();
        let status = call_and_status!(&app, req);
        assert_eq!(status, StatusCode::FORBIDDEN, "无权限时应返回 403 Forbidden");
    }

    /// Test 4c：extensions 中没有权限集合（GrantsMiddleware 未注入）→ 返回 403
    #[actix_rt::test]
    async fn test_permission_extension_missing_returns_403() {
        // 不 wrap InjectPermissions，模拟权限中间件未执行的场景
        let app = test::init_service(
            App::new().service(
                web::scope("/api").route(
                    "/test",
                    web::get().to(ok_handler).wrap(require_permission("test:perm")),
                ),
            ),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/test").to_request();
        let status = call_and_status!(&app, req);
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "权限集合缺失时应返回 403 Forbidden（unwrap_or(false) 生效）"
        );
    }

    /// Test 4d：有权限但不是所需的那个 → 返回 403
    #[actix_rt::test]
    async fn test_wrong_permission_returns_403() {
        let mut perms = HashSet::new();
        perms.insert("other:perm".to_string()); // 拥有别的权限，但没有 test:perm

        let app = build_app_with_perms!(perms);

        let req = test::TestRequest::get().uri("/api/test").to_request();
        let status = call_and_status!(&app, req);
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "拥有其他权限但不包含所需权限时应返回 403"
        );
    }
}
