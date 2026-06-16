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
use crate::core::kit::jwt_util::JWTToken;
use crate::modules::user::model::user::{UserModel, UserSaveDTO};
use crate::modules::user::model::user_platform::UserInfo;
use crate::modules::user::model::sms_verification::{SmsVerificationModel, SmsSendResult, VerificationResult, SmsLoginResult};
use crate::modules::user::service::tencent_sms_service::TencentSmsService;
use crate::config;
use sea_orm::DbConn;
use chrono::{Duration, Local};

pub struct SmsService;

impl SmsService {
    pub async fn send_code(
        db: &DbConn,
        phone: &str,
        scene: &str,
        ip: &str,
        sms_service: &TencentSmsService,
    ) -> Result<SmsSendResult> {
        let now = Local::now().naive_local();

        let today_count = SmsVerificationModel::get_today_send_count(db, phone).await?;
        let max_send_count = config::section::<i32>("tencent_sms", "max_send_count", 10);
        if today_count >= max_send_count {
            return Err(Error::from("今日发送次数已达上限"));
        }

        let opt_record = SmsVerificationModel::find_latest(db, phone, scene).await?;
        if let Some(record) = opt_record {
            let send_interval = config::section::<i64>("tencent_sms", "send_interval", 60);
            let last_send_time = record.create_time.and_utc().timestamp();
            let current_time = now.and_utc().timestamp();
            if current_time - last_send_time < send_interval {
                return Err(Error::from("发送频率过快，请稍后再试"));
            }

            if let Some(lock_until) = record.lock_until {
                if now < lock_until {
                    return Err(Error::from(format!(
                        "号码已锁定，请于 {} 后再试",
                        lock_until.format("%Y-%m-%d %H:%M:%S")
                    )));
                }
            }

            let code = Self::generate_code();
            let request_id = sms_service.send_sms(phone, &code, scene).await?;

            SmsVerificationModel::update_send_count(db, record.id).await?;
            SmsVerificationModel::update_status(db, record.id, 0).await?;

            return Ok(SmsSendResult {
                request_id,
                phone: Self::mask_phone(phone),
                expire_seconds: 300,
            });
        }

        let code = Self::generate_code();
        let request_id = sms_service.send_sms(phone, &code, scene).await?;

        SmsVerificationModel::insert(db, phone, &code, scene, Some(ip)).await?;

        Ok(SmsSendResult {
            request_id,
            phone: Self::mask_phone(phone),
            expire_seconds: 300,
        })
    }

    pub async fn verify_code(
        db: &DbConn,
        phone: &str,
        code: &str,
        scene: &str,
    ) -> Result<VerificationResult> {
        let verification = SmsVerificationModel::find_latest(db, phone, scene)
            .await?
            .ok_or(Error::from("验证码不存在"))?;

        if verification.status == 1 {
            return Err(Error::from("验证码已使用"));
        }
        if verification.status == 2 {
            return Err(Error::from("验证码已失效"));
        }

        let now = Local::now().naive_local();
        if now > verification.expire_time {
            return Err(Error::from("验证码已过期"));
        }

        if let Some(lock_until) = verification.lock_until {
            if now < lock_until {
                return Err(Error::from(format!(
                    "验证错误次数过多，号码已锁定，请{}后再试",
                    lock_until.format("%Y-%m-%d %H:%M:%S")
                )));
            }
        }

        if verification.code == code {
            SmsVerificationModel::update_status(db, verification.id, 1).await?;

            return Ok(VerificationResult {
                success: true,
                message: "验证成功".to_string(),
                remain_count: None,
            });
        } else {
            let new_error_count = verification.error_count + 1;

            if new_error_count >= 3 {
                let lock_until = now + Duration::hours(24);
                SmsVerificationModel::update_error_and_lock(
                    db,
                    verification.id,
                    new_error_count,
                    lock_until,
                )
                .await?;

                return Err(Error::from(format!(
                    "验证错误次数过多，号码已锁定24小时，锁定结束时间：{}",
                    lock_until.format("%Y-%m-%d %H:%M:%S")
                )));
            } else {
                SmsVerificationModel::update_error_count(db, verification.id, new_error_count).await?;

                let remain = 3 - new_error_count;
                return Err(Error::from(format!(
                    "验证码错误，还剩{}次机会",
                    remain
                )));
            }
        }
    }

    pub async fn sms_login(
        db: &DbConn,
        phone: &str,
        code: &str,
        ip: &str,
    ) -> Result<SmsLoginResult> {
        Self::verify_code(db, phone, code, "login").await?;

        let user = match UserModel::find_by_mobile(db, phone).await? {
            Some(u) => {
                let update_dto = UserSaveDTO {
                    id: Some(u.id),
                    username: None,
                    nickname: None,
                    avatar: None,
                    email: None,
                    mobile: None,
                    loginfailure: None,
                    lastlogintime: Some(Local::now().naive_local().to_string()),
                    lastloginip: Some(ip.to_string()),
                    password: None,
                    salt: None,
                    motto: None,
                    status: None,
                };
                UserModel::update_by_id(db, &Some(u.id), &update_dto).await?;
                u
            }
            None => {
                let save_dto = UserSaveDTO {
                    id: None,
                    username: None,
                    nickname: None,
                    avatar: None,
                    email: None,
                    mobile: Some(phone.to_string()),
                    loginfailure: None,
                    lastlogintime: None,
                    lastloginip: None,
                    password: None,
                    salt: None,
                    motto: None,
                    status: Some("1".to_string()),
                };
                let new_user_id = UserModel::insert(db, &save_dto).await?;
                UserModel::find_by_id(db, &Some(new_user_id)).await?
                    .ok_or(Error::from("创建用户失败"))?
            }
        };

        let jwt_secret = config::section::<String>("server", "jwt_secret_user", "mxx_secret_key".to_string());
        let jwt_token = JWTToken::new(
            Some(user.id),
            None,
            vec![],
            Some("mxx_B2B_user"),
        );
        let token_str = jwt_token.create_token(&jwt_secret)?;

        Ok(SmsLoginResult {
            token: token_str,
            expire_time: jwt_token.exp as i64 * 1000,
            user_info: UserInfo {
                id: user.id,
                nick_name: user.nickname.unwrap_or_default(),
                avatar_url: user.avatar.unwrap_or_default(),
                gender: 0,
            },
        })
    }

    fn generate_code() -> String {
        let mut rng = rand::thread_rng();
        format!(
            "{:06}",
            rand::Rng::gen_range(&mut rng, 0..1000000)
        )
    }

    fn mask_phone(phone: &str) -> String {
        if phone.len() >= 11 {
            format!("{}****{}", &phone[0..3], &phone[7..11])
        } else {
            phone.to_string()
        }
    }
}