//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::system::model::notice::{ListQuery, MyNoticeListVO, NoticeDetailVO, NoticeListVO, NoticeModel, NoticeSaveDTO, PageWhere};
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;
use sea_orm::{DbConn, DbErr};
use crate::modules::system::model::admin::AdminModel;
use crate::modules::system::model::admin_notice_merge::AdminNoticeMergeModel;

pub async fn insert(db: &DbConn, form_data: &NoticeSaveDTO) -> Result<i64> {
    let result = NoticeModel::insert(db, form_data).await?;
    Ok(result)
}

/// ### 批量删除岗位
/// * `db` 数据库链接
/// * `ids_vec` 岗位id列表
///
/// * 返回值: 删除成功数量
pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<Option<String>>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let ids = convert_vec_option_string_to_vec_u64(ids_vec.clone());
    let result = NoticeModel::batch_delete_by_ids(&db, ids).await?;
    Ok(result)
}

pub async fn update_by_id(db: &DbConn, form_data: &NoticeSaveDTO) -> Result<i64> {
    let result = NoticeModel::update_by_id(&db, &form_data.id, form_data).await?;
    Ok(result)
}

/// 用户更新阅读状态为已读
/// * `db` - 数据库连接
/// * `user_id` - 用户id
pub async fn update_by_read_all(db: &DbConn, user_id: &Option<i64>) -> Result<i64> {
    let result = AdminNoticeMergeModel::update_by_read_all(&db, user_id).await?;
    Ok(result)
}

/// # 更新通知发布状态
/// * `db` - 数据库连接
/// * `id` - 需要修改的id
/// * `publish_status` - 发布状态（0: 未发布, 1: 已发布, -1: 已撤回）
/// * `user_id` - 操作人id
///
/// 返回值: 更新数量
pub async fn update_by_id_revoke(
    db: &DbConn, id: &Option<i64>, 
    user_id: &Option<i64>
) -> Result<i64> {
    let result = NoticeModel::update_by_id_revoke(&db, id, user_id).await?;
    Ok(result)
}

/// # 更新通知发布状态
/// * `db` - 数据库连接
/// * `id` - 需要修改的id
/// * `user_id` - 操作人id
///
/// 返回值: 更新数量
pub async fn update_by_id_publish(db: &DbConn, id: &Option<i64>, user_id: &Option<i64>) -> std::result::Result<i64, DbErr> {
    let result = NoticeModel::update_by_id_publish(&db, id, user_id).await?;
    Ok(result)
}

pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<NoticeDetailVO>> {
    let result = NoticeModel::find_by_id(db, id).await?.ok_or_else(|| {
        Error::from(format!(
            "{}",
            "未获该公告信息".to_string(),
        ))
    })?;
    let result = NoticeDetailVO::from(result);
    Ok(Option::from(result))
}

/// ### 根据ID查询详情
/// * `db` - 数据库连接
/// * `id` - 需要查询的id
/// 
/// 返回值: 查询结果`Option<NoticeDetailVO>`，如果不存在则返回None。
pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<Option<NoticeDetailVO>> {
    let result = NoticeModel::find_by_id(db, id).await?;

    match result {
        Some(notice_model) => {
            let notice_detail_vo = NoticeDetailVO::from(notice_model);
            Ok(Some(notice_detail_vo))
        }
        None => Ok(None),
    }
}

/// 用户查询公告详情
pub async fn get_by_user_detail(db: &DbConn, id: &Option<i64>, user_id: &Option<i64>) -> Result<Option<NoticeDetailVO>> {
    let result = NoticeModel::find_by_id(db, id).await?.ok_or_else(|| {
        Error::from("未获该公告信息")
    })?;

    // 查询是否有关联信息
    let merge_result = AdminNoticeMergeModel::find_merge_by_notice_and_user(db, &Some(result.id), user_id).await?;

    if let Some(merge) = merge_result {
        if merge.is_read == Some(0) {
            // 更新阅读状态为已读
            AdminNoticeMergeModel::update_by_read(db, &Some(result.id), user_id).await?;
        }
        let result = NoticeDetailVO::from(result);
        Ok(Some(result))
    } else {
        Ok(None)
    }
}

/// 查询岗位列表
pub async fn get_by_my_page(db: &DbConn, query : ListQuery) -> Result<ResultPage<Vec<MyNoticeListVO>>> {

    let select_where = PageWhere {
        title:  query.title,
        user_id: query.user_id,
        is_read: query.is_read,
        status: query.status,
    };
    let search_where = select_where.format();

    let (list, _num_pages) = NoticeModel::select_my_in_page(
        &db,
        query.page_num.unwrap_or(0),
        query.page_size.unwrap_or(10),
        search_where.clone()
    ).await?;

    let list_data: Vec<MyNoticeListVO> = futures::future::join_all(
        list.into_iter().map(|item| async move {
            let admin = AdminModel::find_by_id(db, &item.publisher_id).await?;
            Ok(MyNoticeListVO {
                id: Option::from(item.id),
                title: item.title,
                content: item.content,
                r#type: item.r#type,
                level: item.level,
                publisher_id: item.publisher_id,
                publish_name: admin.unwrap_or_default().user_name,
                publish_status: item.publish_status,
                publish_time: item.publish_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
                is_read: item.is_read,
            })
        })
    ).await.into_iter().collect::<Result<Vec<MyNoticeListVO>>>()?;

    let count = NoticeModel::select_my_count(db, select_where.clone()).await.unwrap_or(0);

    let page_data = ResultPage::new_simple(list_data, count);

    Ok(page_data)
}

/// 查询岗位列表
pub async fn get_by_page(db: &DbConn, query : ListQuery) -> Result<ResultPage<Vec<NoticeListVO>>> {

    let select_where = PageWhere {
        title:  query.title,
        user_id: query.user_id,
        is_read: query.is_read,
        status: query.status,
    };
    let search_where = select_where.format();

    let (list, _num_pages) = NoticeModel::select_in_page(
        &db,
        query.page_num.unwrap_or(0),
        query.page_size.unwrap_or(10),
        search_where.clone()
    ).await?;

    let list_data: Vec<NoticeListVO> = futures::future::join_all(
        list.into_iter().map(|item| async move {
            let admin = AdminModel::find_by_id(db, &item.publisher_id).await?;
            Ok(NoticeListVO {
                id: Option::from(item.id),
                notice_title: item.title,
                content: item.content,
                notice_type: item.r#type,
                level: item.level,
                target_type: item.target_type,
                target_user_ids: item.target_user_ids,
                publisher_id: item.publisher_id,
                publish_name: admin.unwrap_or_default().user_name,
                status: item.publish_status,
                publish_time: item.publish_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
                revoke_time: item.revoke_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
                create_by: item.create_by,
                create_time: item.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
                update_by: item.update_by,
                update_time: item.update_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            })
        })
    ).await.into_iter().collect::<Result<Vec<NoticeListVO>>>()?;
    let count = NoticeModel::select_count(db, select_where.clone()).await.unwrap_or(0);

    let page_data = ResultPage::new_simple(list_data, count);

    Ok(page_data)
}