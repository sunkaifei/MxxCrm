//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 全局 5xx 响应追踪中间件：任何 500/502/503/504 响应都打 error 日志
//! （含请求方法与路径），解决"用户报 500 但日志无痕"的排查盲区。
//! App 级最外层 wrap，覆盖全部路由。

use std::future::{ready, Ready};
use std::rc::Rc;

use actix_web::body::MessageBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::Error;
use futures_util::future::LocalBoxFuture;

pub struct ErrorTrace;

impl<S, B> Transform<S, ServiceRequest> for ErrorTrace
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ErrorTraceMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ErrorTraceMiddleware { service: Rc::new(service) }))
    }
}

pub struct ErrorTraceMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for ErrorTraceMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let method = req.method().to_string();
        let path = req.path().to_string();
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            let status = res.status().as_u16();
            if status >= 500 {
                log::error!(
                    "[5xx追踪] {} {} → {}",
                    method,
                    path,
                    status
                );
            }
            Ok(res)
        })
    }
}
