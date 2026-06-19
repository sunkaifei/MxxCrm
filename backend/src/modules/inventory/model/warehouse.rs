use crate::modules::inventory::entity::warehouse::Model;
use serde::{Deserialize, Serialize};

/// 仓库列表查询参数
#[derive(Debug, Serialize, Deserialize)]
pub struct WarehouseListQuery {
    /// 当前页码
    pub page_num: Option<i64>,
    /// 每页条数
    pub page_size: Option<i64>,
    /// 仓库名称（模糊搜索）
    pub warehouse_name: Option<String>,
}

/// 仓库列表响应
#[derive(Debug, Serialize, Deserialize)]
pub struct WarehouseListVO {
    /// 数据总数
    pub total: i64,
    /// 仓库列表
    pub list: Vec<WarehouseVO>,
}

/// 仓库列表项
#[derive(Debug, Serialize, Deserialize)]
pub struct WarehouseVO {
    /// 仓库ID
    pub id: i64,
    /// 仓库名称
    #[serde(rename = "warehouseName")]
    pub name: Option<String>,
    /// 仓库编码
    pub code: Option<String>,
    /// 仓库地址
    pub address: Option<String>,
    /// 负责人
    pub manager: Option<String>,
    /// 状态
    pub status: Option<String>,
    /// 创建时间
    pub created_at: Option<String>,
}

/// 仓库详情响应项
#[derive(Debug, Serialize, Deserialize)]
pub struct WarehouseDetailVO {
    /// 仓库ID
    pub id: i64,
    /// 仓库名称
    pub name: Option<String>,
    /// 仓库编码
    pub code: Option<String>,
    /// 仓库地址
    pub address: Option<String>,
    /// 联系人
    pub contact_person: Option<String>,
    /// 联系电话
    pub contact_phone: Option<String>,
    /// 是否启用
    pub is_active: Option<bool>,
}

/// 仓库新增请求
#[derive(Debug, Serialize, Deserialize)]
pub struct WarehouseSaveRequest {
    /// 仓库名称
    pub name: Option<String>,
    /// 仓库编码
    pub code: Option<String>,
    /// 仓库地址
    pub address: Option<String>,
    /// 联系人
    pub contact_person: Option<String>,
    /// 联系电话
    pub contact_phone: Option<String>,
    /// 是否启用
    pub is_active: Option<bool>,
}

/// 仓库更新请求
#[derive(Debug, Serialize, Deserialize)]
pub struct WarehouseUpdateRequest {
    /// 仓库ID
    pub id: Option<i64>,
    /// 仓库名称
    pub name: Option<String>,
    /// 仓库编码
    pub code: Option<String>,
    /// 仓库地址
    pub address: Option<String>,
    /// 联系人
    pub contact_person: Option<String>,
    /// 联系电话
    pub contact_phone: Option<String>,
    /// 是否启用
    pub is_active: Option<bool>,
}

impl From<Model> for WarehouseVO {
    fn from(model: Model) -> Self {
        WarehouseVO {
            id: model.id,
            name: model.name,
            code: model.code,
            address: model.address,
            manager: model.contact_person,
            status: Some(if model.is_active.unwrap_or(true) { "启用".to_string() } else { "禁用".to_string() }),
            created_at: model.created_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

impl From<Model> for WarehouseDetailVO {
    fn from(model: Model) -> Self {
        WarehouseDetailVO {
            id: model.id,
            name: model.name,
            code: model.code,
            address: model.address,
            contact_person: model.contact_person,
            contact_phone: model.contact_phone,
            is_active: model.is_active,
        }
    }
}
