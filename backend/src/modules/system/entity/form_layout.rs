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

/// 模块表单/详情布局元数据表
/// 每模块每布局类型一行；无行或停用=回落现用默认渲染（保证"现用设计不变"）
#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "mxx_system_form_layout")]
pub struct Model {
    /// 主键ID
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    /// 业务模块标识（取值同 field_def 的 module 白名单：crm_opportunity/crm_contract/sale_quotation/sale_order 等）
    pub module: Option<String>,
    /// 布局类型：1=新建/编辑表单 2=详情页
    pub layout_type: Option<i32>,
    /// 适用角色 key：NULL=默认布局（所有用户兜底）；非空=该角色用户的专属布局（D7 角色差异化）
    pub role_key: Option<String>,
    /// 布局 JSON：{"version","tabs":[{"key","title"}],"fields":[{"key","source","tab","span","sort"}],"unassigned_policy"}
    /// 保存时应用层做 schema 校验与字段存在性校验
    pub layout_json: Option<serde_json::Value>,
    /// 版本号：每次保存 +1，前端乐观锁防互相覆盖
    pub version: Option<i32>,
    /// 状态：1启用 0停用（停用=前端回落默认渲染）
    pub status: Option<i32>,
    /// 创建者
    pub create_by: Option<String>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新者
    pub update_by: Option<String>,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 删除：0正常 1已删除（逻辑删除=恢复默认布局）
    pub deleted: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
