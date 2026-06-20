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
use crate::modules::system::entity::{system_log, system_log::Entity as SystemLog};
use sea_orm::*;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemLogSaveRequest {
    ///id
    pub id: Option<i64>,
    /// 模块标题
    pub title: Option<String>,
    /// 业务类型（0其它 1新增 2修改 3删除）
    pub business_type: Option<i32>,
    /// 方法名称
    pub method: Option<String>,
    /// 请求方式
    pub request_method: Option<String>,
    /// 操作类别（0其它 1后台用户 2手机端用户）
    pub operator_type: Option<i32>,
    /// 请求URL
    pub oper_url: Option<String>,
    /// 主机地址
    pub oper_ip: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemLogSaveDTO {
    /// 日志主键
    pub id: Option<i64>,
    /// 模块标题
    pub title: Option<String>,
    /// 业务类型（0其它 1新增 2修改 3删除）
    pub business_type: Option<i32>,
    /// 方法名称
    pub method: Option<String>,
    /// 请求方式(POST, PUT, DELETE)
    pub request_method: Option<String>,
    /// 操作类别（0其它 1后台用户 2手机端用户）
    pub operator_type: Option<i32>,
    /// 操作人员
    pub oper_name: Option<String>,
    /// 部门名称
    pub dept_name: Option<String>,
    /// 请求URL
    pub oper_url: Option<String>,
    /// 主机地址
    pub oper_ip: Option<String>,
    /// 操作地点
    pub oper_location: Option<String>,
    /// 请求参数
    pub oper_param: Option<String>,
    /// 返回参数
    pub json_result: Option<String>,
    /// 操作状态（0正常 1异常）
    pub status: Option<i32>,
    /// 错误消息
    pub error_msg: Option<String>,
}

impl From<SystemLogSaveRequest> for SystemLogSaveDTO {
    fn from(request: SystemLogSaveRequest) -> Self {
        Self {
            id: None,
            title: request.title,
            business_type: request.business_type,
            method: request.method,
            request_method: request.request_method,
            operator_type: request.operator_type,
            oper_name: None,
            dept_name: None,
            oper_url: request.oper_url,
            oper_ip: request.oper_ip,
            oper_location: None,
            oper_param: None,
            json_result: None,
            status: None,
            error_msg: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LogListVO {
    /// 日志主键
    pub id: Option<i64>,
    /// 模块标题
    pub title: Option<String>,
    /// 业务类型（0其它 1新增 2修改 3删除）
    pub business_type: Option<i32>,
    /// 方法名称
    pub method: Option<String>,
    /// 请求方式(POST, PUT, DELETE)
    pub request_method: Option<String>,
    /// 操作类别（0其它 1后台用户 2手机端用户）
    pub operator_type: Option<i32>,
    /// 操作人员
    pub oper_name: Option<String>,
    /// 部门名称
    pub dept_name: Option<String>,
    /// 请求URL
    pub oper_url: Option<String>,
    /// 主机地址
    pub oper_ip: Option<String>,
    /// 操作地点
    pub oper_location: Option<String>,
    /// 请求参数
    pub oper_param: Option<String>,
    /// 返回参数
    pub json_result: Option<String>,
    /// 操作状态（0正常 1异常）
    pub status: Option<i32>,
    /// 错误消息
    pub error_msg: Option<String>,
    /// 操作时间
    pub create_time: Option<String>,
}

impl From<system_log::Model> for LogListVO {
    fn from(model: system_log::Model) -> Self {
        Self {
            id: Option::from(model.id),
            title: model.title,
            business_type: model.business_type,
            method: model.method,
            request_method: model.request_method,
            operator_type: model.operator_type,
            oper_name: model.oper_name,
            dept_name: model.dept_name,
            oper_url: model.oper_url,
            oper_ip: model.oper_ip,
            oper_location: model.oper_location,
            oper_param: model.oper_param,
            json_result: model.json_result,
            status: model.status,
            error_msg: model.error_msg,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LogDetailVO {
    /// 日志主键
    pub id: Option<i64>,
    /// 模块标题
    pub title: Option<String>,
    /// 业务类型（0其它 1新增 2修改 3删除）
    pub business_type: Option<i32>,
    /// 方法名称
    pub method: Option<String>,
    /// 请求方式(POST, PUT, DELETE)
    pub request_method: Option<String>,
    /// 操作类别（0其它 1后台用户 2手机端用户）
    pub operator_type: Option<i32>,
    /// 操作人员
    pub oper_name: Option<String>,
    /// 部门名称
    pub dept_name: Option<String>,
    /// 请求URL
    pub oper_url: Option<String>,
    /// 主机地址
    pub oper_ip: Option<String>,
    /// 操作地点
    pub oper_location: Option<String>,
    /// 请求参数
    pub oper_param: Option<String>,
    /// 返回参数
    pub json_result: Option<String>,
    /// 操作状态（0正常 1异常）
    pub status: Option<i32>,
    /// 错误消息
    pub error_msg: Option<String>,
    /// 操作时间
    pub create_time: Option<String>,
}

impl From<system_log::Model> for LogDetailVO {
    fn from(model: system_log::Model) -> Self {
        Self {
            id: Option::from(model.id),
            title: model.title,
            business_type: model.business_type,
            method: model.method,
            request_method: model.request_method,
            operator_type: model.operator_type,
            oper_name: model.oper_name,
            dept_name: model.dept_name,
            oper_url: model.oper_url,
            oper_ip: model.oper_ip,
            oper_location: model.oper_location,
            oper_param: model.oper_param,
            json_result: model.json_result,
            status: model.status,
            error_msg: model.error_msg,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub title: Option<String>,
    pub business_type: Option<i32>,
    pub operator_type: Option<i32>,
    pub status: Option<i32>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
}


#[derive(Clone)]
pub struct PageWhere {
    pub title: Option<String>,
    pub business_type: Option<i32>,
    pub operator_type: Option<i32>,
    pub status: Option<i32>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut title = None;
        if self.title != Some("".to_string()) {
            title = self.title.clone();
        }

        let mut business_type = None;
        if self.business_type == Some(1) || self.business_type == Some(0) {
            business_type = self.business_type;
        }

        let mut operator_type = None;
        if self.operator_type == Some(1) || self.operator_type == Some(0) {
            operator_type = self.operator_type;
        }

        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        Self {
            title,
            business_type,
            status,
            operator_type,
            begin_time: None,
            end_time: None,
        }
    }
}




pub  struct SystemLogModel;

impl SystemLogModel{

    pub async fn insert(db: &DbConn, form_data: SystemLogSaveDTO) -> Result<i64, DbErr> {
        let payload = system_log::ActiveModel {
            id:                Set(form_data.id.unwrap_or_default()),
            title:             Set(form_data.title.to_owned()),
            business_type:     Set(form_data.business_type.to_owned()),
            method:            Set(form_data.method.to_owned()),
            request_method:    Set(form_data.request_method.to_owned()),
            operator_type:     Set(form_data.operator_type.to_owned()),
            create_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        SystemLog::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }
    
    
    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        SystemLog::delete_many()
            .filter(system_log::Column::Id.is_in(ids))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }


    /// 根据主键查询
    pub async fn find_by_id(db: &DbConn, id:  &Option<i64>) -> Result<Option<system_log::Model>, DbErr> {
        SystemLog::find_by_id(id.clone().unwrap_or_default()).one(db).await
    }


    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        system_log::Entity::find()
            .apply_if(wheres.title, |query, v| {
                query.filter(system_log::Column::Title.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.business_type, |query, v| {
                query.filter(system_log::Column::BusinessType.eq(v))
            })
            .apply_if(wheres.operator_type, |query, v| {
                query.filter(system_log::Column::OperatorType.eq(v))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(system_log::Column::Status.eq(v))
            })
            .count(db)
            .await
            .map(|c| c as i64)
    }

    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        wheres: PageWhere,
    ) -> Result<(Vec<system_log::Model>, i64), DbErr> {
        let paginator = system_log::Entity::find()
            .apply_if(wheres.title, |query, v| {
                query.filter(system_log::Column::Title.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.business_type, |query, v| {
                query.filter(system_log::Column::BusinessType.eq(v))
            })
            .apply_if(wheres.operator_type, |query, v| {
                query.filter(system_log::Column::OperatorType.eq(v))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(system_log::Column::Status.eq(v))
            })
            .order_by_desc(system_log::Column::Id)
            .paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
    
}