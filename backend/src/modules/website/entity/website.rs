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
#[sea_orm(table_name = "mxx_website")]
pub struct Model {
    /// 主键
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 站点名字
    pub site_name: Option<String>,
    /// 用户id
    pub user_id: Option<i64>,
    /// 是否在首页显示Banner  1显示 0不显示
    pub show_banner: Option<i32>,
    /// 模版id
    pub template_id: Option<i64>,
    /// 二级域名
    pub domain: Option<String>,
    /// PC端的LOGO
    pub logo: Option<String>,
    /// 客户端类型，1:PC，  2:WAP，3:CMS
    pub client: Option<i32>,
    /// 搜索的关键词
    pub keywords: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 绑定的域名
    pub bind_domain: Option<String>,
    /// 站点类型，1=企业官网，2=商城，3=其他
    pub site_type: Option<i32>,
    /// 站点状态，1正常；2冻结
    pub status: Option<i32>,
    /// 是否是默认站点，1是默认，0不是默认,一个用户只能有一个默认的网站
    pub is_default: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 站点备注，代理商给网站的备注，方便代理商记录这个网站干嘛的
    pub remark: Option<String>,
    /// 是否开启水印 0=关闭 1=开启
    pub watermark_enable: Option<i32>,
    /// 水印类型 1=文字水印 2=图片水印
    pub watermark_type: Option<i32>,
    /// 水印文字内容
    pub watermark_text: Option<String>,
    /// 水印图片地址
    pub watermark_image: Option<String>,
    /// 水印位置 1-9 九宫格
    pub watermark_position: Option<i32>,
    /// 水印透明度 0-100
    pub watermark_opacity: Option<i32>,
    /// 允许上传的文件类型，逗号分隔
    pub upload_allowed_types: Option<String>,
    /// 单文件最大上传大小(MB)
    pub upload_max_size: Option<i32>,
    /// 公司名称
    pub company_name: Option<String>,
    /// 联系电话
    pub company_phone: Option<String>,
    /// 联系邮箱
    pub company_email: Option<String>,
    /// 公司地址
    pub company_address: Option<String>,
    /// 上班时间
    pub work_time_start: Option<String>,
    /// 下班时间
    pub work_time_end: Option<String>,
    /// 工作日期
    pub work_days: Option<String>,
    /// 客服QQ
    pub qq: Option<String>,
    /// 微信号
    pub wechat: Option<String>,
    /// 微信二维码
    pub wechat_qrcode: Option<String>,
    /// 备案号
    pub icp: Option<String>,
    /// 版权信息
    pub copyright: Option<String>,
    /// 统计代码
    pub statistics_code: Option<String>,
    /// 自定义CSS
    pub custom_css: Option<String>,
    /// 自定义JS
    pub custom_js: Option<String>,
    /// 网站关闭原因
    pub close_reason: Option<String>,
    /// 分享标题
    pub share_title: Option<String>,
    /// 分享描述
    pub share_desc: Option<String>,
    /// 分享图片
    pub share_image: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 删除状态，1删除，0未删除
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}