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
use crate::modules::system::service::admin_service::build_admin_name_map;
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

/// # 发布通知
///
/// 完整发布流程：
/// 1. 更新通知状态为"已发布"（publish_status=1）
/// 2. 根据 target_type 收集目标用户ID集合
///    - target_type=1（全体）：查询所有启用状态的后台用户
///    - target_type=2（指定）：解析 target_user_ids 字段
/// 3. 为每个目标用户创建/重置 mxx_system_admin_notice_merge 记录（未读）
/// 4. 通过 WebSocket 向在线用户推送 `notice_publish` 事件，触发前端铃铛提醒
///
/// 事务保证：状态更新 + merge 记录创建原子性
///
/// * `db` - 数据库连接
/// * `id` - 需要发布的公告id
/// * `user_id` - 操作人id（用于记录 update_by）
///
/// 返回值: 更新数量（>0 表示发布成功）
pub async fn update_by_id_publish(db: &DbConn, id: &Option<i64>, user_id: &Option<i64>) -> std::result::Result<i64, DbErr> {
    use crate::modules::message::websocket::ConnectionRegistry;
    use crate::modules::system::model::admin_notice_merge::{AdminNoticeMergeModel, AdminNoticeMergeSaveDTO};
    use sea_orm::TransactionTrait;

    let notice_id = match id {
        Some(v) if *v > 0 => *v,
        _ => return Err(DbErr::Custom("公告ID无效".to_string())),
    };

    // 1. 开启事务：更新发布状态 + 创建 merge 记录
    let txn = db.begin().await?;

    // 1.1 更新通知为已发布
    let update_count = NoticeModel::update_by_id_publish(&txn, id, user_id).await?;
    if update_count == 0 {
        txn.rollback().await?;
        return Ok(0);
    }

    // 1.2 查询通知详情（获取目标类型、目标用户、标题、内容等）
    let notice = NoticeModel::find_by_id(&txn, id)
        .await?
        .ok_or_else(|| DbErr::Custom("公告不存在".to_string()))?;

    let target_type = notice.target_type.unwrap_or(1);
    let target_user_ids_str = notice.target_user_ids.clone().unwrap_or_default();

    // 1.3 根据 target_type 收集目标用户ID
    let target_user_ids: Vec<i64> = if target_type == 1 {
        // 全体用户：查询所有启用状态的后台用户
        AdminModel::find_all_options(&txn)
            .await?
            .into_iter()
            .map(|a| a.id)
            .collect()
    } else {
        // 指定用户：解析 target_user_ids 字段（逗号分隔）
        target_user_ids_str
            .split(',')
            .filter_map(|s| s.trim().parse::<i64>().ok())
            .collect()
    };

    if target_user_ids.is_empty() {
        log::warn!("[notice] 公告 id={} 的目标用户列表为空", notice_id);
        txn.commit().await?;
        return Ok(update_count);
    }

    // 1.4 幂等处理：检查是否已存在 merge 记录
    let existing_merges = AdminNoticeMergeModel::find_by_notice_id(&txn, id).await?;
    let existing_user_ids: std::collections::HashSet<i64> = existing_merges.iter()
        .filter_map(|m| m.user_id)
        .collect();

    if existing_merges.is_empty() {
        // 首次发布：批量创建 merge 记录
        let merge_list: Vec<AdminNoticeMergeSaveDTO> = target_user_ids.iter()
            .map(|uid| AdminNoticeMergeSaveDTO {
                id: None,
                notice_id: Some(notice_id),
                user_id: Some(*uid),
                is_read: Some(0),
                read_time: None,
                create_time: None,
                update_time: None,
                deleted: Some(0),
            })
            .collect();
        AdminNoticeMergeModel::insert_batch(&txn, &merge_list).await?;
        log::info!("[notice] 公告 id={} 首次发布，创建 {} 条 merge 记录", notice_id, merge_list.len());
    } else {
        // 重新发布（撤回后重发）：重置已有记录为未读，并为新用户创建记录
        AdminNoticeMergeModel::reset_read_by_notice_id(&txn, id).await?;

        let new_user_ids: Vec<i64> = target_user_ids.iter()
            .filter(|uid| !existing_user_ids.contains(uid))
            .copied()
            .collect();

        if !new_user_ids.is_empty() {
            let merge_list: Vec<AdminNoticeMergeSaveDTO> = new_user_ids.iter()
                .map(|uid| AdminNoticeMergeSaveDTO {
                    id: None,
                    notice_id: Some(notice_id),
                    user_id: Some(*uid),
                    is_read: Some(0),
                    read_time: None,
                    create_time: None,
                    update_time: None,
                    deleted: Some(0),
                })
                .collect();
            AdminNoticeMergeModel::insert_batch(&txn, &merge_list).await?;
            log::info!("[notice] 公告 id={} 重新发布，重置 {} 条记录，新增 {} 条记录",
                notice_id, existing_merges.len(), merge_list.len());
        } else {
            log::info!("[notice] 公告 id={} 重新发布，重置 {} 条已有记录", notice_id, existing_merges.len());
        }
    }

    // 1.5 提交事务
    txn.commit().await?;

    // 2. 事务提交后：通过 WebSocket 推送实时通知（非关键路径，失败不影响发布）
    let payload = serde_json::json!({
        "type": "notice_publish",
        "data": {
            "noticeId": notice_id,
            "title": notice.title,
            "content": notice.content,
            "noticeType": notice.r#type,
            "level": notice.level,
            "targetType": notice.target_type,
            "publisherId": notice.publisher_id,
            "publishTime": notice.publish_time.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    });
    let payload_str = payload.to_string();

    let registry = ConnectionRegistry::global();
    let mut pushed = 0;
    for uid in &target_user_ids {
        registry.send_to_user(*uid, payload_str.clone());
        if registry.is_online(*uid) {
            pushed += 1;
        }
    }
    log::info!("[notice] 公告 id={} 推送 WebSocket 通知：目标 {} 人，在线 {} 人",
        notice_id, target_user_ids.len(), pushed);

    Ok(update_count)
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

/// 标记公告为已读（专用接口，直接更新 merge 表）
/// 如果找不到关联记录（用户在公告发布后才创建），主动创建一条已读关联记录
pub async fn mark_notice_read(db: &DbConn, notice_id: &Option<i64>, user_id: &Option<i64>) -> Result<i64> {
    use crate::modules::system::model::admin_notice_merge::AdminNoticeMergeSaveDTO;
    log::info!("[mark_notice_read] notice_id={:?}, user_id={:?}", notice_id, user_id);

    // 先查找关联记录，确认存在且未读
    let merge = AdminNoticeMergeModel::find_merge_by_notice_and_user(db, notice_id, user_id).await
        .map_err(|e| Error::from(e.to_string()))?;

    if let Some(m) = merge {
        log::info!("[mark_notice_read] 找到 merge 记录, is_read={:?}", m.is_read);
        // 使用 unwrap_or(0) 处理 is_read 可能为 None 的情况（数据库默认值为0，但ORM可能返回None）
        if m.is_read.unwrap_or(0) == 0 {
            log::info!("[mark_notice_read] 执行更新: notice_id={:?}, user_id={:?}", notice_id, user_id);
            // 更新为已读
            let result = AdminNoticeMergeModel::update_by_read(db, notice_id, user_id).await
                .map_err(|e| Error::from(e.to_string()))?;
            log::info!("[mark_notice_read] 更新结果: rows_affected={}", result);
            return Ok(result);
        }
        // 已经是已读状态，返回1表示成功
        log::info!("[mark_notice_read] 公告已是已读状态，跳过更新");
        return Ok(1);
    }
    // 没有关联记录：用户端主动提交，创建已读关联记录
    log::info!("[mark_notice_read] 未找到关联记录，主动创建已读关联");
    let merge_dto = AdminNoticeMergeSaveDTO {
        id: None,
        notice_id: notice_id.clone(),
        user_id: user_id.clone(),
        is_read: Some(1),
        read_time: Some(chrono::Local::now().naive_local().to_owned()),
        create_time: None,
        update_time: None,
        deleted: Some(0),
    };
    match AdminNoticeMergeModel::insert_batch(db, &vec![merge_dto]).await {
        Ok(_) => {
            log::info!("[mark_notice_read] 已成功创建已读关联记录");
            Ok(1)
        }
        Err(e) => {
            log::error!("[mark_notice_read] 创建已读关联记录失败: {:?}", e);
            Err(Error::from(e.to_string()))
        }
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

    // 页码 1-indexed，默认 1；若上游误传 0/负数，max(1) 兜底，防止 (page-1) as u64 下溢
    let page_num = query.page_num.unwrap_or(1).max(1);
    let (list, _num_pages) = NoticeModel::select_my_in_page(
        &db,
        page_num,
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

    // 页码 1-indexed，默认 1；若上游误传 0/负数，max(1) 兜底，防止 (page-1) as u64 下溢
    let page_num = query.page_num.unwrap_or(1).max(1);
    let (list, _num_pages) = NoticeModel::select_in_page(
        &db,
        page_num,
        query.page_size.unwrap_or(10),
        search_where.clone()
    ).await?;

    // 批量查询发布人名称（避免 N+1 循环查询）
    let publisher_ids: Vec<i64> = list.iter().filter_map(|n| n.publisher_id).collect();
    let publisher_name_map = build_admin_name_map(db, publisher_ids).await;

    let list_data: Vec<NoticeListVO> = list.into_iter().map(|item| {
        let publisher_name = item.publisher_id.as_ref().and_then(|pid| publisher_name_map.get(pid).cloned());
        NoticeListVO {
            id: Option::from(item.id),
            title: item.title,
            content: item.content,
            r#type: item.r#type,
            level: item.level,
            target_type: item.target_type,
            target_user_ids: item.target_user_ids,
            publisher_id: item.publisher_id,
            publisher_name,
            publish_status: item.publish_status,
            publish_time: item.publish_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            revoke_time: item.revoke_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            create_by: item.create_by,
            create_time: item.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_by: item.update_by,
            update_time: item.update_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }).collect();
    let count = NoticeModel::select_count(db, select_where.clone()).await.unwrap_or(0);

    let page_data = ResultPage::new_simple(list_data, count);

    Ok(page_data)
}