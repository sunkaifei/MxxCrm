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
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "mxx_payment_record")]
pub struct Model {
    /// 涓婚敭ID
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    /// 鐢ㄦ埛ID
    pub user_id: i64,

    /// 浼氬憳浜у搧ID锛堝叧鑱斾細鍛樹骇鍝侊級
    pub member_product_id: Option<i64>,

    /// 璁㈠崟ID锛堝叧鑱斾笟鍔¤鍗曪級
    pub order_id: Option<String>,

    /// 鏀粯绫诲瀷: 1=浼氬憳璐圭敤, 2=鍟嗗搧璐拱, 3=鍏呭€? 4=鍏朵粬
    pub payment_type: Option<i32>,

    /// 鏀粯閲戦
    pub amount: Decimal,

    /// 鏀粯鏂瑰紡: 1=寰俊鏀粯, 2=鏀粯瀹? 3=閾惰鍗?
    pub pay_method: Option<i32>,

    /// 鏀粯鐘舵€? 0=寰呮敮浠? 1=鏀粯鎴愬姛, 2=鏀粯澶辫触, 3=宸查€€娆?
    pub status: Option<i32>,

    /// 绗笁鏂规敮浠樹氦鏄撳彿
    pub transaction_id: Option<String>,

    /// 鏀粯鏃堕棿
    pub pay_time: Option<DateTime<Utc>>,

    /// 澶囨敞
    pub remark: Option<String>,

    /// 鍒涘缓鏃堕棿
    pub create_time: Option<DateTime<Utc>>,

    /// 鏇存柊鏃堕棿
    pub update_time: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
