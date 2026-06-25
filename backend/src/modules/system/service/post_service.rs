//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::{DbConn, DbErr, TransactionTrait};
use crate::core::errors::error::{Error, Result};
use crate::core::web::response::ResultPage;
use crate::modules::system::model::admin_post_merge::{AdminPostMergeModel, AdminPostMergeSaveDTO};
use crate::modules::system::model::post::{ListQuery, PageWhere, PostDetailVO, PostListVO, PostModel, PostSaveDTO, PostSaveRequest, PostUpdateRequest};
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;

/// ### 添加岗位
/// * `db` 数据库链接
/// * `post` 岗位信息
///
/// * 返回值: 添加成功数量
pub async fn save(db: &DbConn, post: &PostSaveRequest) -> Result<i64> {
    let form_data = PostSaveDTO::from(post.clone());
    let result = PostModel::insert(&db, form_data).await?;
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
    let result = PostModel::batch_delete_by_ids(&db, ids).await?;
    Ok(result)
}

/// ### 更新岗位信息
/// * `db` 数据库链接
/// * `post` 岗位信息
///
/// * 返回值: 更新成功数量
pub async fn update_by_id(db: &DbConn, post: &PostUpdateRequest) -> Result<i64> {
    let form_data = PostSaveDTO::from(post.clone());
    let result = PostModel::update_by_id(&db, form_data.id.unwrap_or_default(), form_data).await?;
    Ok(result)
}


/// 批量更新职位和管理员关联关系
pub async fn batch_update_post(
    db: &DbConn,
    post_ids: &Option<Vec<i64>>,
    admin_id: &Option<i64>,
) -> Result<i64> {
    // 1. 前置校验：admin_id 必须存在
    let admin_id = match admin_id {
        Some(id) => *id,
        None => return Ok(0), // 或返回错误 Err(Error::BadRequest("admin_id required"))
    };

    // 2. 预处理 post_ids，构造插入数据
    let sys_post_admin_list: Vec<AdminPostMergeSaveDTO> = match post_ids {
        Some(ids) if !ids.is_empty() => {
            let valid_post_ids: Vec<i64> = ids
                .iter()
                .filter(|&&id| id != 0)
                .copied()
                .collect();

            valid_post_ids
                .into_iter()
                .map(|post_id| AdminPostMergeSaveDTO {
                    id: None,
                    create_time: None,
                    post_id: Some(post_id),
                    admin_id: Some(admin_id),
                })
                .collect()
        }
        _ => Vec::new(),
    };

    // 3. 删除旧关联 + 插入新关联需原子执行，避免中途失败丢失全部岗位关联
    let result = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move {
                AdminPostMergeModel::delete_by_admin_id(txn, &Some(admin_id)).await?;
                if sys_post_admin_list.is_empty() {
                    return Ok(0);
                }
                AdminPostMergeModel::insert_batch(txn, &sys_post_admin_list).await
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(result)
}


/// ### 根据岗位名称查询岗位是否存在
/// * `db` 数据库链接
/// * `post_name` 岗位名称
/// * `id` 岗位id
///
/// * 返回值: 岗位是否存在
///
pub async fn find_by_post_name_unique(db: &DbConn, post_name: &Option<String>, id: &Option<i64>) -> Result<bool>{
    let result_count = PostModel::find_by_post_name_unique(&db, &post_name, id).await?;
    if result_count > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// ### 根据岗位编码查询岗位是否存在
/// * `db` 数据库链接
/// * `post_code` 岗位编码
/// * `id` 岗位id
///
/// * 返回值: 岗位是否存在
///
pub async fn find_by_post_code_unique(db: &DbConn, post_code: &Option<String>, id: &Option<i64>) -> Result<bool>{
    let result_count = PostModel::find_by_post_code_unique(&db, post_code, id).await?;
    if result_count > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}


/// ### 根据管理员ID查询关联的岗位列表
/// * `db` 数据库链接
/// * `admin_id` 用户id
///
/// * 返回值: 岗位列表
///
pub async fn select_by_admin_id(db: &DbConn, admin_id: &Option<i64>) -> Result<Vec<PostDetailVO>> {
    let result_merge = AdminPostMergeModel::find_by_admin_id(&db, admin_id.clone().unwrap_or_default()).await?;
    let id_list: Vec<Option<i64>> = result_merge.iter().map(|data| data.post_id).collect();
    if !id_list.is_empty() {
        let vec_u64: Vec<i64> = id_list.into_iter()
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect();
        let post_data = PostModel::find_by_ids(&db, vec_u64).await?;
        let mut post_vo: Vec<PostDetailVO> = Vec::new();
        for post in post_data {
            post_vo.push(PostDetailVO {
                id: Option::from(post.id),
                post_code: post.post_code,
                post_name: post.post_name,
                sort: post.sort,
                status: post.status,
                remark: post.remark,
                create_time: Option::from(post.create_time.unwrap_or_default().format("%Y-%m-%d %H:%M:%S").to_string()),
                update_time: Option::from(post.update_time.unwrap_or_default().format("%Y-%m-%d %H:%M:%S").to_string()),
            })
        }
        Ok(post_vo)
    } else {
        Ok(vec![])
    }
}

pub async fn get_by_id(db: &DbConn, id: &Option<i64>) -> Result<PostDetailVO> {
    let post_data = PostModel::find_by_id(&db, id).await?;
    match post_data {
        None => {
            Err(Error::from(format!(
                "{}={}",
                "岗位信息不存在，id".to_string(),
                &id.unwrap_or_default()
            )))
        }
        Some(post) => {
            Ok(PostDetailVO {
                id: Option::from(post.id),
                post_code: post.post_code,
                post_name: post.post_name,
                sort: post.sort,
                status: post.status,
                remark: post.remark,
                create_time: Option::from(post.create_time.unwrap_or_default().format("%Y-%m-%d %H:%M:%S").to_string()),
                update_time: Option::from(post.update_time.unwrap_or_default().format("%Y-%m-%d %H:%M:%S").to_string()),
            })
        }
    }
}


pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<PostDetailVO> {
    let post_data = PostModel::find_by_id(&db, id).await?;
    
    match post_data {
        None => {
            Err(Error::from(format!(
                "{}={}",
                "岗位信息不存在，id".to_string(),
                &id.unwrap_or_default()
            )))
        }
        Some(post) => {
            Ok(PostDetailVO {
                id: Option::from(post.id),
                post_code: post.post_code,
                post_name: post.post_name,
                sort: post.sort,
                status: post.status,
                remark: post.remark,
                create_time: Option::from(post.create_time.unwrap_or_default().format("%Y-%m-%d %H:%M:%S").to_string()),
                update_time: Option::from(post.update_time.unwrap_or_default().format("%Y-%m-%d %H:%M:%S").to_string()),
            })
        }
    }
}


/// 查询岗位列表
pub async fn get_by_page(db: &DbConn, query : ListQuery) -> Result<ResultPage<Vec<PostListVO>>> {

    let select_where = PageWhere {
        post_code: query.post_code,
        post_name: query.post_name,
        status: query.status,
    };
    let search_where = select_where.format();

    let (list, _num_pages) = PostModel::select_in_page(
        &db,
        query.page_num.unwrap_or(0),
        query.page_size.unwrap_or(10),
        search_where.clone()
    ).await?;

    let list_data: Vec<PostListVO> = list.into_iter().map(|item| PostListVO::from(item)).collect();

    let count = PostModel::select_count(db, select_where.clone()).await.unwrap_or(0);

    let page_data = ResultPage::new_simple(list_data, count);

    Ok(page_data)
}