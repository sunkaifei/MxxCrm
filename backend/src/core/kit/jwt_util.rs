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
use crate::core::kit::config;
use jsonwebtoken::{decode, encode, errors::ErrorKind, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct JWTToken {
    pub id: Option<i64>,
    pub username: Option<String>,
    pub permissions: Vec<String>,
    pub aud: String,
    // (audience)：受众
    pub exp: usize,
    pub iat: usize,
    // (Issued At)：签发时间
    pub iss: String,
    // (issuer)：签发人
    pub nbf: usize,
    // (Not Before)：生效时间
    pub sub: String,
    // (subject)：主题
    pub jti: String, // (JWT ID)：编号
}

impl JWTToken {
    pub fn new(id: Option<i64>, username: Option<String>, permissions: Vec<String>, issuer: Option<&str>) -> JWTToken {
        let now = SystemTime::now();
        let now = now.duration_since(UNIX_EPOCH).expect("获取系统时间失败");
        
        //根据 issuer 选择不同的过期时间
        let issuer_str = issuer.unwrap_or("mxx_B2B_admin");
        let jwt_expire = if issuer_str == "mxx_B2B_user" {
            let expire = config::section::<i64>("server", "jwt_expire_user", 86400);
            log::debug!("[JWT] 用户端过期时间配置: {} 秒", expire);
            expire
        } else {
            let expire = config::section::<i64>("server", "jwt_expire_admin", 1800);
            log::debug!("[JWT] 管理端过期时间配置: {} 秒", expire);
            expire
        };
        let expire_duration = Duration::from_secs(jwt_expire as u64);

        JWTToken {
            id,
            username,
            permissions,
            aud: String::from("mxx_B2B"), // (audience)：受众
            exp: (now + expire_duration).as_secs() as usize,
            iat: now.as_secs() as usize,  // (Issued At)：签发时间
            iss: String::from(issuer_str),     // (issuer)：签发人
            nbf: now.as_secs() as usize,  // (Not Before)：生效时间
            sub: String::from("mxx_B2B_token"), // (subject)：主题
            jti: String::from("ignore"),  // (JWT ID)：编号
        }
    }

    /// create token
    /// secret: your secret string
    pub fn create_token(&self, secret: &str) -> Result<String> {
        let mut validation = Validation::default();
        validation.validate_nbf = true;
        match encode(
            &Header::default(),
            self,
            &EncodingKey::from_secret(secret.as_ref()),
        ) {
            Ok(t) => Ok(t),
            Err(_) => Err(Error::from(format!("{}", "新建token失败".to_string()))),
        }
    }
    
    /// verify token invalid
    /// secret: your secret string
    pub fn verify(secret: &str, token: &str) -> Result<JWTToken> {
        let mut validation = Validation::default();
        validation.sub = Some("mxx_B2B_token".to_string());
        validation.set_audience(&["mxx_B2B"]);
        validation.validate_nbf = true;
        match decode::<JWTToken>(
            &token,
            &DecodingKey::from_secret(secret.as_ref()),
            &validation,
        ) {
            Ok(c) => Ok(c.claims),

            Err(err) => match *err.kind() {
                ErrorKind::InvalidToken => return Err(Error::from(format!("{}", "InvalidToken".to_string()))), // Example on how to handle a specific error
                ErrorKind::InvalidIssuer => return Err(Error::from(format!("{}", "InvalidIssuer".to_string()))), // Example on how to handle a specific error
                ErrorKind::ExpiredSignature => return Err(Error::from(format!("{}", "token 已经超时了".to_string()))), // Example on how to handle a specific error
                _ => Err(Error::from(format!("{}", "token校验失败".to_string()))),
            },
        }
    }

    pub fn refresh(&self, secret: &str, jwt_exp: usize) -> Result<String> {
        let mut jwt = self.clone();
        jwt.exp = jwt.exp + jwt_exp;
        jwt.create_token(&secret)
    }

    pub fn new_temp(openid: String, expire_secs: i64) -> JWTToken {
        let now = SystemTime::now();
        let now = now.duration_since(UNIX_EPOCH).expect("获取系统时间失败");
        let expire_duration = Duration::from_secs(expire_secs as u64);

        JWTToken {
            id: None,
            username: Some(openid),
            permissions: vec![],
            aud: String::from("mxx_B2B"), // (audience)：受众
            exp: (now + expire_duration).as_secs() as usize,
            iat: now.as_secs() as usize,  // (Issued At)：签发时间
            iss: String::from("mxx_B2B_temp"), // (issuer)：临时token
            nbf: now.as_secs() as usize,  // (Not Before)：生效时间
            sub: String::from("mxx_B2B_temp_token"), // (subject)：主题
            jti: String::from("ignore"),  // (JWT ID)：编号
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::kit::jwt_util::JWTToken;

    #[test]
    fn test_jwt() {
        let jwt = JWTToken::new(Some(1), Some("koobe".to_string()), vec![], None);
        let res = jwt.create_token("123");
        match res {
            Ok(token) => {
                println!("{:?}", token);

                let token_s = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpZCI6MSwidXNlcm5hbWUiOiJrb29iZSIsInBlcm1pc3Npb25zIjpbXSwiYXVkIjoicnVzdF9hZG1pbiIsImV4cCI6MTY5OTc4NDk3MywiaWF0IjoxNjk3OTg0OTczLCJpc3MiOiJrb29iZSIsIm5iZiI6MTY5Nzk4NDk3Mywic3ViIjoicnVzdF9hZG1pbiIsImp0aSI6Imlnbm9yZSJ9.NJ6mIJtBAedyzOgqEnoLk8Cs2GqQ33G6w8V-aVpY-WQ";
                let token_tk = JWTToken::verify("123", &token_s);
                println!("{:?}", token_tk)
            }
            Err(_) => {}
        }
    }
}