//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! A-2.1: 密码强度统一校验
//!
//! 口径：8-64 位，必须同时包含字母和数字。admin 端（注册/改密/重置）与
//! website 前台用户端共用本工具，避免两侧口径漂移（原 admin 仅要求 ≥6 位）。

use crate::core::errors::error::{Error, Result};

/// 校验密码强度：8-64 位，须同时包含字母与数字
pub fn validate_password_strength(password: &str) -> Result<()> {
    let len = password.chars().count();
    if len < 8 || len > 64 {
        return Err(Error::from("密码长度必须在8-64个字符之间"));
    }
    if !password.chars().any(|c| c.is_ascii_alphabetic()) {
        return Err(Error::from("密码必须包含字母"));
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(Error::from("密码必须包含数字"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_strength() {
        assert!(validate_password_strength("abc12345").is_ok());
        assert!(validate_password_strength("short1a").is_err()); // 长度不足
        assert!(validate_password_strength("12345678").is_err()); // 纯数字
        assert!(validate_password_strength("abcdefgh").is_err()); // 纯字母
        assert!(validate_password_strength(&"a1".repeat(33)).is_err()); // 超长
    }
}
