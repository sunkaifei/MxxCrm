//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mxx_member_fee")]
pub struct Model {
    /// 涓婚敭ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 鐢ㄦ埛ID
    pub user_id: i64,

    /// 浼氬憳绫诲瀷: 1=鏈堝害浼氬憳, 2=瀛ｅ害浼氬憳, 3=骞村害浼氬憳
    pub member_type: Option<i32>,

    /// 鏀粯閲戦
    pub amount: Decimal,

    /// 浼氬憳鏈夋晥鏈熷紑濮嬫椂闂?
    pub valid_start_time: Option<DateTime>,

    /// 浼氬憳鏈夋晥鏈熺粨鏉熸椂闂?
    pub valid_end_time: Option<DateTime>,

    /// 鏀粯鐘舵€? 0=寰呮敮浠? 1=宸叉敮浠? 2=宸茶繃鏈?
    pub status: Option<i32>,

    /// 鍏宠仈鏀粯璁板綍ID
    pub payment_record_id: Option<i64>,

    /// 澶囨敞
    pub remark: Option<String>,

    /// 鍒涘缓鏃堕棿
    pub create_time: Option<DateTime>,

    /// 鏇存柊鏃堕棿
    pub update_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
