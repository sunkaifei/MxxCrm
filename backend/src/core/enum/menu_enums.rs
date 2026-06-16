//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use std::fmt;

pub enum MenuTypeEnum{
    CATALOG(String),
    MENU(String),
    BUTTON(String),
    EXTLINK(String),
}


impl fmt::Display for MenuTypeEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MenuTypeEnum::CATALOG(_) => write!(f, "CATALOG"),
            MenuTypeEnum::MENU(_) => write!(f, "MENU"),
            MenuTypeEnum::BUTTON(_) => write!(f, "BUTTON"),
            MenuTypeEnum::EXTLINK(_) => write!(f, "EXTLINK"),
        }
    }
}