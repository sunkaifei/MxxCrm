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
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_article")]
pub struct Model {
    //文章ID
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 用户id
    pub user_id: Option<i64>,
    //短网址
    pub short_url: Option<String>,
    //文章分类ID
    pub category_id: Option<i64>,
    //文章标题
    pub title: Option<String>,
    //简短标题
    pub short_title: Option<String>,
    //文章主图
    pub title_image: Option<String>,
    //文章作者
    pub author: Option<String>,
    //原文链接
    pub original_link: Option<String>,
    //文章摘要
    pub description: Option<String>,
    //文章内容
    pub content: Option<String>,
    //回复统计
    pub count_comment: Option<i64>,
    //帖子展示数
    pub count_view: Option<i64>,
    //喜欢数
    pub count_love: Option<i64>,
    //支持数
    pub count_digg: Option<i64>,
    //反对次数
    pub count_burys: Option<i64>,
    //关注话题统计数
    pub count_follow: Option<i64>,
    //是否置顶
    pub istop: Option<i32>,
    //是否关闭帖子
    pub isclose: Option<i32>,
    //是否允许评论
    pub iscomment: Option<i32>,
    //是否评论后显示内容
    pub iscommentshow: Option<i32>,
    //是否精华帖子
    pub isposts: Option<i32>,
    //0不审核，1审核，2未通过
    pub isaudit: Option<i32>,
    //0不删除1删除
    pub deleted: Option<i32>,
    //1为推荐
    pub isrecommend: Option<i32>,
    //0未审核，1审核，2未通过
    pub status: Option<i32>,
    //创建时间
    pub create_time: Option<DateTime>,
    //更新时间
    pub update_time: Option<DateTime>,
}


#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
