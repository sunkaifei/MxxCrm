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
    /// HTTP 响应状态码
    pub status_code: Option<i32>,
    /// 接口耗时（毫秒）
    pub elapsed: Option<i64>,
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
            status_code: None,
            elapsed: None,
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
    /// HTTP 响应状态码
    pub status_code: Option<i32>,
    /// 接口耗时（毫秒）
    pub elapsed: Option<i64>,
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
            status_code: model.status_code,
            elapsed: model.elapsed,
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
    /// HTTP 响应状态码
    pub status_code: Option<i32>,
    /// 接口耗时（毫秒）
    pub elapsed: Option<i64>,
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
            status_code: model.status_code,
            elapsed: model.elapsed,
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
    pub oper_name: Option<String>,
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
    pub oper_name: Option<String>,
    pub status: Option<i32>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

impl PageWhere {
    /// 格式化：清理空字符串与无效值
    pub fn format(&self) -> Self {
        let title = self.title.as_ref().filter(|s| !s.is_empty()).cloned();

        let business_type = self.business_type.filter(|v| (0..=3).contains(v));

        let operator_type = self.operator_type.filter(|v| (0..=2).contains(v));

        let oper_name = self.oper_name.as_ref().filter(|s| !s.is_empty()).cloned();

        let status = self.status.filter(|v| (0..=1).contains(v));

        let begin_time = self.begin_time.as_ref().filter(|s| !s.is_empty()).cloned();
        let end_time = self.end_time.as_ref().filter(|s| !s.is_empty()).cloned();

        Self {
            title,
            business_type,
            status,
            operator_type,
            oper_name,
            begin_time,
            end_time,
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
            oper_name:         Set(form_data.oper_name.to_owned()),
            dept_name:         Set(form_data.dept_name.to_owned()),
            oper_url:          Set(form_data.oper_url.to_owned()),
            oper_ip:           Set(form_data.oper_ip.to_owned()),
            oper_location:     Set(form_data.oper_location.to_owned()),
            oper_param:        Set(form_data.oper_param.to_owned()),
            json_result:       Set(form_data.json_result.to_owned()),
            status:            Set(form_data.status.to_owned()),
            error_msg:         Set(form_data.error_msg.to_owned()),
            status_code:       Set(form_data.status_code.to_owned()),
            elapsed:           Set(form_data.elapsed.to_owned()),
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
            .apply_if(wheres.oper_name, |query, v| {
                query.filter(system_log::Column::OperName.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(system_log::Column::Status.eq(v))
            })
            .apply_if(wheres.begin_time, |query, v| {
                query.filter(system_log::Column::CreateTime.gte(parse_naive_from_str(&v)))
            })
            .apply_if(wheres.end_time, |query, v| {
                query.filter(system_log::Column::CreateTime.lte(parse_naive_from_str(&v)))
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
            .apply_if(wheres.oper_name, |query, v| {
                query.filter(system_log::Column::OperName.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(system_log::Column::Status.eq(v))
            })
            .apply_if(wheres.begin_time, |query, v| {
                query.filter(system_log::Column::CreateTime.gte(parse_naive_from_str(&v)))
            })
            .apply_if(wheres.end_time, |query, v| {
                query.filter(system_log::Column::CreateTime.lte(parse_naive_from_str(&v)))
            })
            .order_by_desc(system_log::Column::Id)
            .paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }

}

/// 将 "yyyy-MM-dd HH:mm:ss" 或 "yyyy-MM-dd" 字符串解析为 NaiveDateTime
/// 解析失败时返回 chrono::Local::now()（保证 filter 不会 panic，但仍是有效约束）
fn parse_naive_from_str(s: &str) -> chrono::NaiveDateTime {
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d",
        "%Y/%m/%d %H:%M:%S",
        "%Y/%m/%d",
    ];
    for fmt in formats {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, fmt) {
            return dt;
        }
        if let Ok(d) = chrono::NaiveDate::parse_from_str(s, fmt) {
            return d.and_hms_opt(0, 0, 0).unwrap_or_else(|| chrono::Local::now().naive_local());
        }
    }
    chrono::Local::now().naive_local()
}