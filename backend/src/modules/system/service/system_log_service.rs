//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use actix_web::HttpRequest;
use sea_orm::DbConn;
use crate::core::web::response::ResultPage;
use crate::core::errors::error::Result;
use crate::modules::system::model::system_log::{ListQuery, LogListVO, PageWhere, SystemLogModel, SystemLogSaveDTO, SystemLogSaveRequest};
use crate::SNOWFLAKE;

/// #添加系统日志
/// * `db` - 数据库连接对象
/// * `request`  -  请求对象
/// * `title` - 日志标题
/// * `business_type` -业务类型（0查看 1新增 2修改 3删除）
/// * `method` - 方法名称
/// * `request_method` - 请求方式
/// * `operator_type` - 操作类别（0其它 1后台用户 2手机端用户）
///
pub async fn save_system_log(db: &DbConn, 
                             request: &HttpRequest,
                             title: Option<String>,
                             business_type: Option<i32>,
                             method: Option<String>,
                             request_method: Option<String>,
                             operator_type: Option<i32>,
) -> Result<i64> {

    //获取请求的接口
    let uri = request.uri();
    let url = format!("{}{}", uri.path(), uri.query().map(|q| format!("?{}", q)).unwrap_or_default());

    let system_entity = SystemLogSaveRequest {
        id: Some(SNOWFLAKE.generate() as i64),
        title,
        business_type,
        method,
        request_method,
        operator_type,
        oper_url: Option::from(url),
        oper_ip: Option::from(request.connection_info().realip_remote_addr().unwrap_or_default().to_string()),
    };
    let system_log = SystemLogSaveDTO::from(system_entity);
    
    let result = SystemLogModel::insert(&db, system_log).await?;
    Ok(result)
}


/// 查询地区列表
pub async fn get_by_page(db: &DbConn, query : ListQuery) -> Result<ResultPage<Vec<LogListVO>>> {

    let select_where = PageWhere {
        title: query.title,
        business_type: query.business_type,
        status: query.status,
        operator_type: query.operator_type,
        begin_time: None,
        end_time: None,
    };
    let search_where = select_where.format();

    let (list, _num_pages) = SystemLogModel::select_in_page(
        &db,
        query.page_num.unwrap_or(0),
        query.page_size.unwrap_or(10),
        search_where.clone()
    ).await?;

    let list_data: Vec<LogListVO> = list.into_iter().map(|item| LogListVO::from(item)).collect();

    let count = SystemLogModel::select_count(db, select_where.clone()).await.unwrap_or(0);

    let page_data = ResultPage::new_simple(list_data, count);

    Ok(page_data)
}