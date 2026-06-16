//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

pub mod user;
pub mod user_group;
pub mod user_invite;
pub mod user_menu;
pub mod user_platform;
pub mod sms_verification;
pub mod user_level;

pub use user::Entity as User;
pub use user_group::Entity as UserGroup;
pub use user_invite::Entity as UserInvite;
pub use user_menu::Entity as UserMenu;
pub use user_platform::Entity as UserPlatform;
pub use sms_verification::Entity as SmsVerification;
pub use user_level::Entity as UserLevel;