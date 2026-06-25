//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use std::error::Error as StdError;
use std::fmt::{self, Debug, Display};
use std::io;
use std::path::StripPrefixError;
use actix_web::{HttpResponse, ResponseError};
use chrono::ParseError;
use serde::de::Visitor;
use serde::ser::{Serialize, Serializer};
use serde::{Deserialize, Deserializer};
use crate::core::web::response::MetaResp;

pub type Result<T> = std::result::Result<T, Error>;

/// A generic error that represents all the ways a method can fail inside expr::core.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    E(String),
    Database(String),
    NotFound(String),
    BadRequest(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::E(error) => write!(f, "{}", error),
            Error::Database(error) => write!(f, "Database error: {}", error),
            Error::NotFound(error) => write!(f, "Not found: {}", error),
            Error::BadRequest(error) => write!(f, "Bad request: {}", error),
        }
    }
}

impl StdError for Error {}

impl From<io::Error> for Error {
    #[inline]
    fn from(err: io::Error) -> Self {
        Error::from(err.to_string())
    }
}

impl From<walkdir::Error> for Error {
    #[inline]
    fn from(err: walkdir::Error) -> Self {
        Error::from(err.to_string())
    }
}

impl From<&str> for Error {
    fn from(arg: &str) -> Self {
        Error::E(arg.to_string())
    }
}

impl From<String> for Error {
    fn from(arg: String) -> Self {
        Error::E(arg)
    }
}

impl From<actix_web::error::Error> for Error {
    fn from(arg: actix_web::error::Error) -> Self {
        Error::E(arg.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(arg: serde_json::Error) -> Self {
        Error::E(arg.to_string())
    }
}

///模版错误处理
impl From<minijinja::Error> for Error {
    fn from(arg: minijinja::Error) -> Self {
        Error::E(arg.to_string())
    }
}

impl From<&dyn std::error::Error> for Error {
    fn from(arg: &dyn std::error::Error) -> Self {
        Error::E(arg.to_string())
    }
}

impl From<sea_orm::DbErr> for Error {
    fn from(arg: sea_orm::DbErr) -> Self {
        Error::E(arg.to_string())
    }
}
impl From<ParseError> for Error {
    fn from(arg: ParseError) -> Self {
        Error::E(arg.to_string())
    }
}

impl From<StripPrefixError> for Error {
    fn from(arg: StripPrefixError) -> Self {
        Error::E(arg.to_string())
    }
}
impl ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Error::NotFound(_) => actix_web::http::StatusCode::NOT_FOUND,
            Error::BadRequest(_) | Error::E(_) | Error::Database(_) => {
                actix_web::http::StatusCode::BAD_REQUEST
            }
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let code = status.as_u16() as i32;
        let body = MetaResp::<()>::fail(code, &self.to_string(), "local");
        HttpResponse::build(status)
            .content_type("application/msgpack")
            .body(body)
    }
}

impl From<Box<dyn ResponseError>> for Error {
    fn from(err: Box<dyn ResponseError>) -> Self {
        Error::E(err.to_string())
    }
}

impl Clone for Error {
    fn clone(&self) -> Self {
        Error::from(self.to_string())
    }

    fn clone_from(&mut self, source: &Self) {
        *self = Self::from(source.to_string());
    }
}

// This is what #[derive(Serialize)] would generate.
impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

struct ErrorVisitor;

impl<'de> Visitor<'de> for ErrorVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
        where
            E: std::error::Error,
    {
        Ok(v.to_string())
    }

    fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
        where
            E: std::error::Error,
    {
        Ok(v)
    }
}

impl<'de> Deserialize<'de> for Error {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        let r = deserializer.deserialize_string(ErrorVisitor)?;
        Ok(Error::from(r))
    }
}
