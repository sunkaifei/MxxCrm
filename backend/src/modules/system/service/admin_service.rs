//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use bcrypt::{hash, DEFAULT_COST};
use sea_orm::{ColumnTrait, ConnectionTrait, DbConn, EntityTrait, QueryFilter};
use sea_orm::TransactionTrait;
use std::collections::{HashMap, HashSet};
use crate::core::errors::error::{Error, Result};
use crate::core::kit::app::is_demo_mode;
use crate::core::web::response::ResultPage;
use crate::modules::system::entity::admin;
use crate::modules::system::model::admin::{AdminDetailVO, AdminListVO, AdminModel, AdminOptionVO, AdminSaveDTO, AdminSaveRequest, AdminUpdateRequest, DeptNameDTO, ListQuery, PageWhere, RoleNameDTO, UpdateAdminPasswordRequest, UpdateAdminStatusRequest, UpdateLoginRequest};
use crate::modules::system::model::admin_dept_merge::{AdminDeptMergeModel, AdminDeptMergeSaveDTO};
use crate::modules::system::model::admin_post_merge::{AdminPostMergeModel, AdminPostMergeSaveDTO};
use crate::modules::system::model::admin_role_merge::{AdminRoleMergeModel, AdminRolesMergeSaveDTO};
use crate::modules::system::model::role::RoleModel;
use crate::modules::system::service::{config_service, dept_service, role_service};
use crate::utils::string_utils::{convert_vec_option_string_to_vec_u64};

/// 批量查询 admin 用户名映射：admin_id -> 显示名（nick_name 优先，回退 user_name）
///
/// 统一所有列表/详情查询中"用户名回填"的实现，避免循环 N+1 查询。
///
/// - 入参 `admin_ids` 无需调用方去重（内部用 HashSet 去重）
/// - 空 `Vec` 直接返回空 HashMap，不发 SQL
/// - 自动过滤 `deleted=0`
/// - 名称解析统一为 `nick_name.or(user_name).unwrap_or_default()`
/// - 泛型 `ConnectionTrait`，兼容 `&DbConn` 和事务 `&txn`
///
/// # 示例
/// ```ignore
/// let ids: Vec<i64> = list.iter().flat_map(|c| [c.assigned_to, c.created_by]).flatten().collect();
/// let name_map = build_admin_name_map(db, ids).await;
/// vo.assignee = assigned_to.and_then(|id| name_map.get(&id).cloned());
/// ```
pub async fn build_admin_name_map<C: ConnectionTrait>(
    db: &C,
    admin_ids: Vec<i64>,
) -> HashMap<i64, String> {
    if admin_ids.is_empty() {
        return HashMap::new();
    }
    // 内部去重，避免 IN 列表过大或重复
    let unique_ids: Vec<i64> = admin_ids.into_iter().collect::<HashSet<_>>().into_iter().collect();
    admin::Entity::find()
        .filter(admin::Column::Id.is_in(unique_ids))
        .filter(admin::Column::Deleted.eq(0))
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|a| (a.id, a.nick_name.or(a.user_name).unwrap_or_default()))
        .collect()
}

/// 新增管理员
pub async fn insert(db: &DbConn, form_data: &AdminSaveRequest) -> Result<i64> {
    if is_demo_mode() {
        return Err(Error::from("演示站模式下禁止新增用户"));
    }
    let mut dto_data = AdminSaveDTO::from(form_data.clone());

    if let Some(password) = &form_data.password {
        if !password.is_empty() {
            let hashed = hash(password, DEFAULT_COST).unwrap_or_default();
            dto_data.password = Option::from(hashed);
        } else {
            let config = config_service::select_by_key(db, &"initPassword".to_string()).await?;
            let hashed = hash(config.config_value.unwrap_or_default(), DEFAULT_COST).unwrap_or_default();
            dto_data.password = Option::from(hashed);
        }
    } else {
        let config = config_service::select_by_key(db, &"initPassword".to_string()).await?;
        let hashed = hash(config.config_value.unwrap_or_default(), DEFAULT_COST).unwrap_or_default();
        dto_data.password = Option::from(hashed);
    }
    
    let form_data_clone = form_data.clone();
    let result = (*db).transaction::<_, _, Error>(|tx| {
        Box::pin(async move {
            let admin_id = AdminModel::insert(tx, &dto_data).await
                .map_err(|e| Error::from(format!("插入管理员失败: {}", e)))?;
            if admin_id > 0 {
                // 插入关联表数据
                if let Some(dept_ids) = form_data_clone.dept_ids.clone() {
                    let dept_merge_list: Vec<AdminDeptMergeSaveDTO> = dept_ids
                        .into_iter()
                        .map(|id| {
                            AdminDeptMergeSaveDTO {
                                id: None,
                                admin_id: Some(admin_id),
                                dept_id: Option::from(id),
                                create_time: None,
                            }
                        })
                        .collect();
                    AdminDeptMergeModel::insert_batch(tx, &dept_merge_list).await
                        .map_err(|e| Error::from(format!("插入部门关联失败: {}", e)))?;
                }
                if let Some(post_ids) = form_data_clone.post_ids.clone() {
                    let post_merge_list: Vec<AdminPostMergeSaveDTO> = post_ids.into_iter().map(|id| {
                        AdminPostMergeSaveDTO {
                            id: None,
                            admin_id: Some(admin_id),
                            post_id: Option::from(id),
                            create_time: None,
                        }
                    }).collect();
                    AdminPostMergeModel::insert_batch(tx, &post_merge_list).await
                        .map_err(|e| Error::from(format!("插入岗位关联失败: {}", e)))?;
                }
                if let Some(role_ids) = form_data_clone.role_ids.clone() {
                    let role_merge_list: Vec<AdminRolesMergeSaveDTO> = role_ids.into_iter().map(|id| {
                        AdminRolesMergeSaveDTO {
                            id: None,
                            admin_id: Some(admin_id),
                            role_id: Option::from(id),
                            create_time: None,
                        }
                    }).collect();
                    AdminRoleMergeModel::insert_batch(tx, &role_merge_list).await
                        .map_err(|e| Error::from(format!("插入角色关联失败: {}", e)))?;
                }
            }
            Ok(admin_id)
        })
    }).await.map_err(|e| Error::from(format!("事务执行失败: {}", e)))?;

    Ok(result)
}

/// 用户注册（自动分配默认角色）
pub async fn register(db: &DbConn, form_data: &AdminSaveRequest) -> Result<i64> {
    if is_demo_mode() {
        return Err(Error::from("演示站模式下禁止注册用户"));
    }
    
    let admin_id = insert(db, form_data).await?;
    
    let default_role = RoleModel::find_by_role_key(db, "admin").await?.unwrap_or_default();
    if default_role.id > 0 {
        let role_merge = vec![AdminRolesMergeSaveDTO {
            id: None,
            admin_id: Some(admin_id),
            role_id: Some(default_role.id),
            create_time: None,
        }];
        let _ = AdminRoleMergeModel::insert_batch(db, &role_merge).await;
    }
    
    Ok(admin_id)
}

pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<Option<String>>) -> Result<i64> {
    if is_demo_mode() {
        return Err(Error::from("演示站模式下禁止删除用户"));
    }
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let ids = convert_vec_option_string_to_vec_u64(ids_vec.clone());
    let result = (*db).transaction::<_, _, Error>(|tx| {
        Box::pin(async move {
            let del_count = AdminModel::batch_delete_by_ids(tx, &ids).await
                .map_err(|e| Error::from(format!("批量删除管理员失败: {}", e)))?;
            if del_count > 0 {
                for id in ids {
                    AdminDeptMergeModel::delete_by_admin_id(tx, &Option::from(id.clone())).await
                        .map_err(|e| Error::from(format!("删除部门关联失败: {}", e)))?;
                    AdminPostMergeModel::delete_by_admin_id(tx, &Option::from(id.clone())).await
                        .map_err(|e| Error::from(format!("删除岗位关联失败: {}", e)))?;
                    AdminRoleMergeModel::delete_by_admin_id(tx, &Option::from(id.clone())).await
                        .map_err(|e| Error::from(format!("删除角色关联失败: {}", e)))?;
                }
            }
            Ok(del_count)
        })
    }).await.map_err(|e| Error::from(format!("事务执行失败: {}", e)))?;
    Ok(result)
}

/// 软删除用户
pub async fn soft_delete_by_id(db: &DbConn, id: i64) -> Result<i64> {
    if is_demo_mode() {
        return Err(Error::from("演示站模式下禁止删除用户"));
    }
    // 关联表删除与主表软删除需原子执行，避免产生孤儿关联或残留主记录
    let result = (*db).transaction::<_, _, Error>(|tx| {
        Box::pin(async move {
            AdminDeptMergeModel::delete_by_admin_id(tx, &Option::from(id)).await
                .map_err(|e| Error::from(format!("删除部门关联失败: {}", e)))?;
            AdminPostMergeModel::delete_by_admin_id(tx, &Option::from(id)).await
                .map_err(|e| Error::from(format!("删除岗位关联失败: {}", e)))?;
            AdminRoleMergeModel::delete_by_admin_id(tx, &Option::from(id)).await
                .map_err(|e| Error::from(format!("删除角色关联失败: {}", e)))?;
            AdminModel::soft_delete(tx, id).await
                .map_err(|e| Error::from(format!("软删除管理员失败: {}", e)))
        })
    }).await.map_err(|e| Error::from(format!("事务执行失败: {}", e)))?;
    Ok(result)
}

pub async fn update_admin(db: &DbConn, form_data: &AdminUpdateRequest) -> Result<i64> {
    if is_demo_mode() {
        return Err(Error::from("演示站模式下禁止修改用户信息"));
    }
    let dto_data = AdminSaveDTO::from(form_data.clone());
    let admin_id = match form_data.id {
        Some(id) => id,
        None => return Err(Error::from("管理员ID不能为空")),
    };
    
    let result = (*db).transaction::<_, _, Error>(|tx| {
        let dept_ids = form_data.dept_ids.clone();
        let post_ids = form_data.post_ids.clone();
        let role_ids = form_data.role_ids.clone();
        Box::pin(async move {
            // 更新关联表：部门
            if dept_ids.is_some() {
                AdminDeptMergeModel::delete_by_admin_id(tx, &Some(admin_id)).await
                    .map_err(|e| Error::from(format!("删除部门关联失败: {}", e)))?;
                if let Some(ref ids) = dept_ids {
                    let dept_merge_list: Vec<AdminDeptMergeSaveDTO> = ids.iter().map(|id| {
                        AdminDeptMergeSaveDTO {
                            id: None,
                            admin_id: Some(admin_id),
                            dept_id: Option::from(*id),
                            create_time: None,
                        }
                    }).collect();
                    if !dept_merge_list.is_empty() {
                        AdminDeptMergeModel::insert_batch(tx, &dept_merge_list).await
                            .map_err(|e| Error::from(format!("插入部门关联失败: {}", e)))?;
                    }
                }
            }
            // 更新关联表：岗位
            if post_ids.is_some() {
                AdminPostMergeModel::delete_by_admin_id(tx, &Some(admin_id)).await
                    .map_err(|e| Error::from(format!("删除岗位关联失败: {}", e)))?;
                if let Some(ref ids) = post_ids {
                    let post_merge_list: Vec<AdminPostMergeSaveDTO> = ids.iter().map(|id| {
                        AdminPostMergeSaveDTO {
                            id: None,
                            admin_id: Some(admin_id),
                            post_id: Option::from(*id),
                            create_time: None,
                        }
                    }).collect();
                    if !post_merge_list.is_empty() {
                        AdminPostMergeModel::insert_batch(tx, &post_merge_list).await
                            .map_err(|e| Error::from(format!("插入岗位关联失败: {}", e)))?;
                    }
                }
            }
            // 更新关联表：角色
            if role_ids.is_some() {
                AdminRoleMergeModel::delete_by_admin_id(tx, &Some(admin_id)).await
                    .map_err(|e| Error::from(format!("删除角色关联失败: {}", e)))?;
                if let Some(ref ids) = role_ids {
                    let role_merge_list: Vec<AdminRolesMergeSaveDTO> = ids.iter().map(|id| {
                        AdminRolesMergeSaveDTO {
                            id: None,
                            admin_id: Some(admin_id),
                            role_id: Option::from(*id),
                            create_time: None,
                        }
                    }).collect();
                    if !role_merge_list.is_empty() {
                        AdminRoleMergeModel::insert_batch(tx, &role_merge_list).await
                            .map_err(|e| Error::from(format!("插入角色关联失败: {}", e)))?;
                    }
                }
            }
            // 更新管理员主表
            let update_count = AdminModel::update_admin(tx, admin_id, &dto_data).await
                .map_err(|e| Error::from(format!("更新管理员失败: {}", e)))?;
            Ok(update_count)
        })
    }).await.map_err(|e| Error::from(format!("事务执行失败: {}", e)))?;
    Ok(result)
}

/// 修改管理员密码
/// * db 数据库连接
/// * user_id: 管理员ID
/// * password: 新密码
/// 
/// 返回值：受影响的行数
pub async fn update_user_password(db: &DbConn, user_id: &Option<i64>, password: &Option<String>) -> Result<i64> {
    if is_demo_mode() {
        return Err(Error::from("演示站模式下禁止修改密码"));
    }
    let result = AdminModel::update_user_password(&db, &user_id, &password).await?;
    Ok(result)
}

/// 修改管理员状态
pub async fn update_user_status(db: &DbConn, form_data: &UpdateAdminStatusRequest) -> Result<i64> {
    if is_demo_mode() {
        return Err(Error::from("演示站模式下禁止修改用户状态"));
    }
    let result = AdminModel::update_by_status(&db, &form_data.id.unwrap_or_default(), &form_data.status).await?;
    Ok(result)
}

/// 修改登录信息
pub async fn update_login_info(db: &DbConn, form_data: &UpdateLoginRequest) -> Result<i64> {
    let form_data = AdminSaveDTO::from(form_data.clone());
    let result = AdminModel::update_login_info(&db, form_data.id.unwrap_or_default(), &form_data).await?;
    Ok(result)
}


pub async fn find_by_name_unique(db: &DbConn, name: &Option<String>, id: &Option<i64>) -> Result<bool>{
    let result_num = AdminModel::find_by_name_unique(db, &name, id).await?;
    if result_num > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn find_by_mobile_unique(db: &DbConn, mobile: &Option<String>, id: &Option<i64>) -> Result<bool>{
    let result_num = AdminModel::find_by_mobile_unique(db, &mobile, id.clone()).await?;
    if result_num > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn find_by_email_unique(db: &DbConn, email: &Option<String>, id: &Option<i64>) -> Result<bool>{
    let result_num = AdminModel::find_by_email_unique(db, &email, id.clone()).await?;
    if result_num > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// 根据昵称查询管理员
pub async fn find_by_nick_name_unique(db: &DbConn, nick_name: &Option<String>, id: &Option<i64>) -> Result<bool>{
    let result_num = AdminModel::find_by_nick_name_unique(db, &nick_name, id.clone()).await?;
    if result_num > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// 根据id查询管理员
pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<admin::Model>>{
    let result = AdminModel::find_by_id(db, id).await?.ok_or_else(|| {
        Error::from(format!(
            "{}={}",
            "用户信息不存在，id".to_string(),
            &id.unwrap_or_default()
        ))
    })?;
    Ok(Option::from(result))
}

pub async fn find_by_name(db: &DbConn, user_name: &Option<String>) -> Result<Option<admin::Model>> {
    let result_data = AdminModel::find_by_username(db, user_name).await?.ok_or_else(|| {
        Error::from(format!(
            "{}={}",
            "用户信息不存在，id".to_string(),
            user_name.as_deref().unwrap_or_default()
        ))
    })?;
    Ok(Option::from(result_data))
}

/// 获取管理员详情
pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<AdminDetailVO>{
    let result_data = AdminModel::find_by_id(db, id).await?
        .ok_or_else(|| {
            Error::from(format!(
                "{}={}",
                "用户信息不存在，id".to_string(),
                &id.unwrap_or_default()
            ))
        })?;
    let result = AdminDetailVO::from(result_data);
    Ok(result)
}


/// 查询管理员列表
pub async fn get_by_page(db: &DbConn, query : ListQuery) -> Result<ResultPage<Vec<AdminListVO>>> {

    let select_where = PageWhere {
        user_name: query.user_name.clone(),
        nick_name: query.nick_name.clone(),
        email: query.email.clone(),
        mobile: query.mobile.clone(),
        user_type: query.user_type.clone(),
        status: query.status.clone(),
        dept_id: query.dept_id.clone(),
    };
    let search_where = select_where.format();
    
    let (list, _num_pages) = AdminModel::select_in_page(
        &db,
        query.page_num.unwrap_or(1),
        query.page_size.unwrap_or(10), 
        search_where.clone()
    ).await?;

    let id_list: Vec<i64> = list.clone().into_iter().map(|data| data.id).collect();
    let result_role = role_service::select_by_ids(db, id_list.clone()).await;
    let result_dept = dept_service::select_by_ids(db, id_list.clone()).await;
    let mut list_data: Vec<AdminListVO> = Vec::new();
    for data in list.clone() {
        let mut role_data: Vec<RoleNameDTO> = Vec::new();
        match result_role {
            Ok(ref role_list) => {
                for role_entity in role_list {
                    if role_entity.admin_id == Some(data.id) {
                        role_data.push(RoleNameDTO{ role_name: role_entity.role_name.clone() });
                    }
                }
            }
            Err(_) => {}
        }

        let mut depts_data: Vec<DeptNameDTO> = Vec::new();
        match result_dept {
            Ok(ref dept_list) => {
                for dept_entity in dept_list {
                    if dept_entity.admin_id == Some(data.id) {
                        depts_data.push(DeptNameDTO {dept_name: dept_entity.dept_name.clone()});
                    }
                }
            }
            Err(_) => {}
        }

        let role_name_str = if role_data.is_empty() {
            None
        } else {
            Some(role_data.iter()
                .filter_map(|r| r.role_name.clone())
                .collect::<Vec<_>>()
                .join(", "))
        };

        list_data.push(AdminListVO {
            id: data.id,
            user_name: data.user_name,
            nick_name: data.nick_name,
            user_type: data.user_type,
            mobile: data.mobile,
            email: data.email,
            role_name: role_name_str,
            roles: Option::from(role_data),
            depts: Option::from(depts_data),
            remark: data.remark,
            status: data.status,
            sort: data.sort,
            login_ip: data.login_ip,
            login_date: data.login_date.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
            create_time: Option::from(data.create_time.unwrap_or_default().format("%Y-%m-%d %H:%M:%S").to_string()),
        });

    }

    let count = AdminModel::select_count(db, select_where.clone()).await.unwrap_or(0);

    let page_data = ResultPage::new_simple(list_data, count);
    
    Ok(page_data)
}

pub async fn get_admin_options(db: &DbConn) -> Result<Vec<AdminOptionVO>> {
    let result = AdminModel::find_all_options(db).await?;
    let mut list_data: Vec<AdminOptionVO> = Vec::new();
    for data in result {
        let label = data.nick_name
            .clone()
            .filter(|s| !s.is_empty())
            .or(data.user_name.clone());
        list_data.push(AdminOptionVO {
            value: Option::from(data.id),
            label,
        });
    }
    Ok(list_data)
}