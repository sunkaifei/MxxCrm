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
#[sea_orm(table_name = "mxx_refund_record")]
pub struct Model {
    /// 涓婚敭ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 鐢ㄦ埛ID
    pub user_id: i64,

    /// 鍏宠仈鏀粯璁板綍ID
    pub payment_record_id: i64,

    /// 閫€娆鹃噾棰?
    pub amount: Decimal,

    /// 閫€娆剧姸鎬? 0=寰呴€€娆? 1=閫€娆惧鐞嗕腑, 2=閫€娆炬垚鍔? 3=閫€娆惧け璐? 4=閫€娆惧叧闂?
    pub status: Option<i32>,

    /// 绗笁鏂归€€娆句氦鏄撳彿
    pub transaction_id: Option<String>,

    /// 閫€娆炬椂闂?
    pub refund_time: Option<DateTime>,

    /// 閫€娆惧師鍥?
    pub reason: Option<String>,

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
