//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::system::entity::{dept, dept::Entity as Dept};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string, u64_to_string};
use sea_orm::prelude::DateTime;
use sea_orm::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct DeptSaveRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub parent_id: Option<i64>,
    pub dept_name: Option<String>,
    /// 部门编码
    pub code: Option<String>,
    pub sort: Option<i32>,
    pub leader: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub status: Option<i32>,
}

impl From<DeptSaveRequest> for DeptSaveDTO {
    fn from(value: DeptSaveRequest) -> Self {
        DeptSaveDTO {
            id: None,
            parent_id: value.parent_id,
            ancestors: None,
            dept_name: value.dept_name,
            code: value.code,
            sort: value.sort,
            leader: value.leader,
            phone: value.phone,
            email: value.email,
            status: value.status,
            deleted: None,
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct DeptUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub parent_id: Option<i64>,
    pub dept_name: Option<String>,
    /// 部门编码
    pub code: Option<String>,
    pub sort: Option<i32>,
    pub leader: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub status: Option<i32>,
}

impl From<DeptUpdateRequest> for DeptSaveDTO {
    fn from(value: DeptUpdateRequest) -> Self {
        DeptSaveDTO {
            id: value.id,
            parent_id: value.parent_id,
            ancestors: None,
            dept_name: value.dept_name,
            code: value.code,
            sort: value.sort,
            leader: value.leader,
            phone: value.phone,
            email: value.email,
            status: value.status,
            deleted: None,
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
        }
    }
}


pub struct DeptSaveDTO {
    pub id: Option<i64>,
    pub parent_id: Option<i64>,
    pub ancestors: Option<String>,
    pub dept_name: Option<String>,
    /// 部门编码
    pub code: Option<String>,
    pub sort: Option<i32>,
    ///负责人
    pub leader: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub status: Option<i32>,
    pub deleted: Option<i32>,
    pub create_by: Option<String>,
    pub create_time: Option<DateTime>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeptTreeListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub parent_id: Option<i64>,
    pub ancestors: Option<String>,
    pub dept_name: Option<String>,
    /// 部门编码
    pub code: Option<String>,
    /// 显示顺序
    pub sort: Option<i32>,
    ///负责人
    pub leader: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub status: Option<i32>,
    /// 子菜单
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<DeptTreeListVO>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeptDetailVO {
    pub id: Option<i64>,
    pub parent_id: Option<i64>,
    pub ancestors: Option<String>,
    pub dept_name: Option<String>,
    /// 部门编码
    pub code: Option<String>,
    /// 显示顺序
    pub sort: Option<i32>,
    ///负责人
    pub leader: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub status: Option<i32>,
    pub deleted: Option<i32>,
    pub create_by: Option<String>,
    pub create_time: Option<String>,
    pub update_by: Option<String>,
    pub update_time: Option<DateTime>,
}

/// 菜单下拉树形结构
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeptOptionsTreeVO {
    /// 菜单ID
    #[serde(serialize_with = "u64_to_string")]
    pub value: i64,
    /// 父菜单ID
    pub label: Option<String>,
    /// 子菜单
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<DeptOptionsTreeVO>>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct DeptOptionVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub value: Option<i64>,
    pub label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeptAdminByName {
    pub admin_id: Option<i64>,
    pub dept_name: Option<String>,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ListQuery {
    pub keywords: Option<String>,
    /// 部门编码
    pub code: Option<String>,
    pub status: Option<i32>,
}

#[derive(Clone)]
pub struct PageWhere {
    pub dept_name: Option<String>,
    pub code: Option<String>,
    pub status: Option<i32>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut dept_name = None;
        if self.dept_name != Some("".to_string()) {
            dept_name = self.dept_name.clone();
        }

        let mut code = None;
        if self.code != Some("".to_string()) {
            code = self.code.clone();
        }

        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        Self {
            dept_name,
            code,
            status,
        }
    }
}

pub struct DeptModel;

impl DeptModel {

    pub async fn insert(db: &DbConn, form_data: &DeptSaveDTO) -> Result<i64, DbErr> {
        let payload = dept::ActiveModel{
            parent_id:         Set(form_data.parent_id.to_owned()),
            ancestors:         Set(form_data.ancestors.to_owned()),
            dept_name:         Set(form_data.dept_name.to_owned()),
            sort:              Set(form_data.sort.to_owned()),
            leader:            Set(form_data.leader.to_owned()),
            phone:             Set(form_data.phone.to_owned()),
            email:             Set(form_data.email.to_owned()),
            status:            Set(form_data.status.to_owned()),
            create_by:         Set(form_data.create_by.to_owned()),
            create_time:       Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            update_by:         Set(form_data.update_by.to_owned()),
            update_time:       Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        Dept::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

     pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        Dept::delete_many()
            .filter(dept::Column::Id.is_in(ids))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

     pub async fn update_by_id(db: &DbConn, id: i64, form_data: &DeptSaveDTO) -> Result<i64, DbErr> {
         let payload = dept::ActiveModel{
             parent_id:      Set(form_data.parent_id.to_owned()),
             ancestors:      Set(form_data.ancestors.to_owned()),
             dept_name:      Set(form_data.dept_name.to_owned()),
             sort:           Set(form_data.sort.to_owned()),
             leader:         Set(form_data.leader.to_owned()),
             phone:          Set(form_data.phone.to_owned()),
             email:          Set(form_data.email.to_owned()),
             status:         Set(form_data.status.to_owned()),
             update_by:      Set(form_data.update_by.to_owned()),
             update_time:    Set(Option::from(chrono::Local::now().naive_local().to_owned())),
             ..Default::default()
         };

         let update_result: UpdateResult = Dept::update_many()
            .set(payload)
            .filter(dept::Column::Id.eq(id))
            .exec(db).await?;
        Ok(update_result.rows_affected as i64)
    }

    pub async fn find_by_name_unique(db: &DbConn, 
                                     dept_name: &Option<String>, 
                                     parent_id: &Option<i64>, 
                                     id: &Option<i64>
    ) -> Result<bool, DbErr> {
        let result_num = Dept::find()
            .filter(dept::Column::DeptName.eq(dept_name.clone().unwrap_or_default()))
            .apply_if(id.clone(), |query, v| {
                query.filter(dept::Column::Id.ne(v))
            })
            .apply_if(parent_id.clone(), |query, v| {
                query.filter(dept::Column::ParentId.eq(v))
            })
            .count(db).await? as i64;
        Ok(result_num > 0)
    }

    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<dept::Model>, DbErr> {
        Dept::find_by_id(id)
            .one(db)
            .await
    }

    /// 根据id集合查询
    pub async fn find_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<Vec<dept::Model>, DbErr> {
        Dept::find()
            .filter(dept::Column::Id.is_in(ids))
            .all(db)
            .await
    }
    
    pub async fn find_all(db: &DbConn) -> Result<Vec<dept::Model>, DbErr> {
        Dept::find()
            .order_by_asc(dept::Column::Sort)
            .all(db)
            .await
    }

    pub async fn select_all(db: &DbConn, wheres: PageWhere) -> Result<Vec<dept::Model>, DbErr> {
        let query = Dept::find()
            .apply_if(wheres.dept_name, |query, v| {
                query.filter(dept::Column::DeptName.contains(v))
            })
            .apply_if(wheres.code, |query, v| {
                query.filter(dept::Column::Code.eq(v))
            })
            .apply_if(wheres.status, |query, v| {
                match v {
                    0 => query.filter(dept::Column::Status.gte(0)),
                    1 => query.filter(dept::Column::Status.eq(0)),
                    2 => query.filter(dept::Column::Status.eq(1)),
                    _ => query.filter(dept::Column::Status.eq(v)),
                }
            })
            .order_by_asc(dept::Column::Sort);

        // 手动生成并打印完整的 SQL 查询字符串
        //let sql = query.build(DbBackend::MySql);
        //log::info!("===================sql={}", sql.to_string());

        query.all(db)
            .await
    }

}