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
use chrono::Local;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, DbConn, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect, Set, Statement, TransactionTrait};
use std::collections::{HashMap, HashSet};
use crate::core::errors::error::{Error, Result};
use crate::core::kit::app::is_demo_mode;
use crate::core::web::response::ResultPage;
use crate::modules::system::entity::admin;
use crate::modules::system::entity::{admin_dept_merge, admin_post_merge, dept, post};
use crate::modules::system::entity::config::{ActiveModel as ConfigActiveModel, Column as ConfigColumn, Entity as ConfigEntity};
use crate::modules::system::model::admin::{AdminDetailVO, AdminListVO, AdminModel, AdminOptionVO, AdminSaveDTO, AdminSaveRequest, AdminUpdateRequest, DeptNameDTO, ListQuery, PageWhere, PostNameDTO, RoleNameDTO, UpdateAdminPasswordRequest, UpdateAdminStatusRequest, UpdateLoginRequest};
use crate::modules::system::model::admin_dept_merge::{AdminDeptMergeModel, AdminDeptMergeSaveDTO};
use crate::modules::system::model::admin_post_merge::{AdminPostMergeModel, AdminPostMergeSaveDTO};
use crate::modules::system::model::admin_role_merge::{AdminRoleMergeModel, AdminRolesMergeSaveDTO};
use crate::modules::system::model::role::RoleModel;
use crate::modules::system::service::{config_service, dept_service, post_service, role_service};
use crate::utils::string_utils::{convert_vec_option_string_to_vec_u64, normalize_and_validate_cn_mobile};

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

/// 统一校验手机号：必填 + 格式 + 唯一性
/// - id=None：用于新增，全表唯一性
/// - id=Some(x)：用于更新，排除自身记录
///
/// 返回成功时返回 trim + 正则校验后的标准 11 位手机号字符串；失败返回业务 Error。
pub async fn validate_mobile(db: &DbConn, mobile: Option<&String>, id: Option<i64>) -> Result<String> {
    let normalized = normalize_and_validate_cn_mobile(mobile)
        .map_err(Error::from)?;
    if find_by_mobile_unique(db, &Some(normalized.clone()), &id).await.unwrap_or_default() {
        return Err(Error::from("手机号已存在，请更换其他手机号"));
    }
    Ok(normalized)
}

/// 新增管理员
pub async fn insert(db: &DbConn, form_data: &AdminSaveRequest) -> Result<i64> {
    if is_demo_mode() {
        return Err(Error::from("演示站模式下禁止新增用户"));
    }
    // 手机号统一校验（必填 + 格式 + 唯一），并回写 trim 后的标准值
    let mobile_norm = validate_mobile(db, form_data.mobile.as_ref(), None).await?;
    // 部门必选校验：所有用户必须归属至少一个部门（用于审批流向上查找领导和数据权限隔离）
    if form_data.dept_ids.as_ref().map_or(true, |ids| ids.is_empty()) {
        return Err(Error::from("部门为必选项，请至少选择一个部门"));
    }
    // 角色必选校验：所有用户必须分配至少一个角色（用于功能权限控制）
    if form_data.role_ids.as_ref().map_or(true, |ids| ids.is_empty()) {
        return Err(Error::from("角色为必选项，请至少选择一个角色"));
    }
    // 岗位必选校验：所有用户必须分配至少一个岗位（用于审批流按岗位解析审批人）
    if form_data.post_ids.as_ref().map_or(true, |ids| ids.is_empty()) {
        return Err(Error::from("岗位为必选项，请至少选择一个岗位"));
    }
    // 直属上级自引用校验
    if let Some(mid) = form_data.direct_manager_id {
        if mid > 0 {
            // 新增时无用户ID，跳过自引用检查；调用方应保证不会自引用
            // 校验上级用户是否存在且启用
            let manager = AdminModel::find_by_id(db, &Some(mid)).await
                .map_err(|e| Error::from(format!("查询直属上级失败: {}", e)))?
                .ok_or_else(|| Error::from("直属上级用户不存在"))?;
            if manager.status.unwrap_or(0) != 1 {
                return Err(Error::from("直属上级用户已停用，请选择其他用户"));
            }
        }
    }
    // 试用期工资比例校验：必须在 0~1 范围内
    if let Some(ratio) = form_data.probation_ratio {
        if ratio < rust_decimal::Decimal::ZERO || ratio > rust_decimal::Decimal::ONE {
            return Err(Error::from("试用期工资比例必须在 0~1 范围内（如 0.6 表示 60%）"));
        }
    }
    let mut dto_data = AdminSaveDTO::from(form_data.clone());
    // 使用通过校验并标准化后的手机号（覆盖原始可能含空格/非法字符的值）
    dto_data.mobile = Some(mobile_norm);

    if let Some(password) = &form_data.password {
        if !password.is_empty() {
            let hashed = hash(password, DEFAULT_COST)
                .map_err(|e| Error::from(format!("密码加密失败: {}", e)))?;
            dto_data.password = Option::from(hashed);
        } else {
            let config = config_service::select_by_key(db, &"initPassword".to_string()).await?;
            let init_pwd = config.config_value.unwrap_or_default();
            if init_pwd.is_empty() {
                return Err(Error::from("系统未配置初始密码（initPassword），请联系管理员"));
            }
            let hashed = hash(init_pwd, DEFAULT_COST)
                .map_err(|e| Error::from(format!("密码加密失败: {}", e)))?;
            dto_data.password = Option::from(hashed);
        }
    } else {
        let config = config_service::select_by_key(db, &"initPassword".to_string()).await?;
        let init_pwd = config.config_value.unwrap_or_default();
        if init_pwd.is_empty() {
            return Err(Error::from("系统未配置初始密码（initPassword），请联系管理员"));
        }
        let hashed = hash(init_pwd, DEFAULT_COST)
            .map_err(|e| Error::from(format!("密码加密失败: {}", e)))?;
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

    // 注册用户不经过 insert 的部门/角色/岗位必选校验（注册时为文本填写，审核后管理员分配）
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

    let admin_id = AdminModel::insert(db, &dto_data).await
        .map_err(|e| Error::from(format!("插入注册用户失败: {}", e)))?;

    // 注册用户标记为待审核（audit_status=0），status 已在 form_data 中设为 0
    let _ = AdminModel::update_audit_status(db, admin_id, 0).await;

    // 绑定默认角色（受限角色，管理员审核后可调整）
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
    let admin_id = match form_data.id {
        Some(id) => id,
        None => return Err(Error::from("管理员ID不能为空")),
    };
    // 部门必选校验：若前端显式传入 dept_ids（包括空数组），不允许清空所有部门
    if let Some(ref ids) = form_data.dept_ids {
        if ids.is_empty() {
            return Err(Error::from("部门为必选项，不能清空所有部门"));
        }
    }
    // 角色必选校验：若前端显式传入 role_ids（包括空数组），不允许清空所有角色
    if let Some(ref ids) = form_data.role_ids {
        if ids.is_empty() {
            return Err(Error::from("角色为必选项，不能清空所有角色"));
        }
    }
    // 岗位必选校验：若前端显式传入 post_ids（包括空数组），不允许清空所有岗位
    if let Some(ref ids) = form_data.post_ids {
        if ids.is_empty() {
            return Err(Error::from("岗位为必选项，不能清空所有岗位"));
        }
    }
    // 直属上级自引用校验：不允许将自己设为直属上级
    if let Some(mid) = form_data.direct_manager_id {
        if mid > 0 {
            if mid == admin_id {
                return Err(Error::from("不能将自己设为直属上级"));
            }
            let manager = AdminModel::find_by_id(db, &Some(mid)).await
                .map_err(|e| Error::from(format!("查询直属上级失败: {}", e)))?
                .ok_or_else(|| Error::from("直属上级用户不存在"))?;
            if manager.status.unwrap_or(0) != 1 {
                return Err(Error::from("直属上级用户已停用，请选择其他用户"));
            }
        }
    }
    // 试用期工资比例校验：必须在 0~1 范围内
    if let Some(ratio) = form_data.probation_ratio {
        if ratio < rust_decimal::Decimal::ZERO || ratio > rust_decimal::Decimal::ONE {
            return Err(Error::from("试用期工资比例必须在 0~1 范围内（如 0.6 表示 60%）"));
        }
    }
    // 审核进行中锁定档案编辑（方案 3.1）：存在 user 类型待审/审批中实例（status∈{1,2}）时禁止编辑；
    // 已驳回(4)/已撤回(5)/待修改(6)不锁定，允许修改档案后重新提交
    let ongoing = crate::modules::approval::entity::approval_instance::Entity::find()
        .filter(crate::modules::approval::entity::approval_instance::Column::BusinessType.eq("user"))
        .filter(crate::modules::approval::entity::approval_instance::Column::BusinessId.eq(admin_id))
        .filter(crate::modules::approval::entity::approval_instance::Column::Status.is_in([1, 2]))
        .count(db)
        .await
        .map_err(|e| Error::from(format!("查询审核状态失败: {}", e)))?;
    if ongoing > 0 {
        return Err(Error::from(
            "该员工审核进行中，档案已锁定，请待审核完成（或驳回/撤回）后再编辑",
        ));
    }
    // 手机号：如果前端显式传入（含空字符串），则统一校验（必填 + 格式 + 唯一）
    // 并把 trim/标准化后的结果写回 dto_data
    let mobile_norm = if form_data.mobile.is_some() {
        Some(validate_mobile(db, form_data.mobile.as_ref(), Some(admin_id)).await?)
    } else {
        None
    };
    let mut dto_data = AdminSaveDTO::from(form_data.clone());
    if let Some(m) = mobile_norm {
        dto_data.mobile = Some(m);
    }
    
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

/// 审核注册用户（通过时 status=1, audit_status=1；拒绝时保持 status=0, audit_status=0）
pub async fn update_audit_status(db: &DbConn, user_id: i64, audit_status: i32) -> Result<i64> {
    let result = AdminModel::update_audit_status(db, user_id, audit_status).await?;
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
    let mut result = AdminDetailVO::from(result_data);
    // 补充直属上级姓名
    if let Some(mid) = result.direct_manager_id {
        if mid > 0 {
            if let Some(manager) = AdminModel::find_by_id(db, &Some(mid)).await? {
                result.direct_manager_name = manager.nick_name.or(manager.user_name);
            }
        }
    }
    // 补充部门名称
    let mut dept_names: Vec<Option<String>> = Vec::new();
    if let Some(uid) = id {
        let dept_merges = admin_dept_merge::Entity::find()
            .filter(admin_dept_merge::Column::AdminId.eq(*uid))
            .all(db).await?;
        for m in dept_merges {
            let did = m.dept_id.unwrap_or_default();
            if did > 0 {
                if let Some(d) = dept::Entity::find_by_id(did).one(db).await? {
                    dept_names.push(d.dept_name);
                }
            }
        }
    }
    result.dept_names = Some(dept_names);
    // 补充岗位名称
    let mut post_names: Vec<Option<String>> = Vec::new();
    if let Some(uid) = id {
        let post_merges = admin_post_merge::Entity::find()
            .filter(admin_post_merge::Column::AdminId.eq(*uid))
            .all(db).await?;
        for m in post_merges {
            let pid = m.post_id.unwrap_or_default();
            if pid > 0 {
                if let Some(p) = post::Entity::find_by_id(pid).one(db).await? {
                    post_names.push(p.post_name);
                }
            }
        }
    }
    result.post_names = Some(post_names);
    Ok(result)
}


/// 查询管理员列表
pub async fn get_by_page(db: &DbConn, query: ListQuery, current_user_id: i64) -> Result<ResultPage<Vec<AdminListVO>>> {

    let select_where = PageWhere {
        user_name: query.user_name.clone(),
        nick_name: query.nick_name.clone(),
        email: query.email.clone(),
        mobile: query.mobile.clone(),
        user_type: query.user_type.clone(),
        status: query.status.clone(),
        dept_id: query.dept_id.clone(),
        biz_enabled: query.biz_enabled.clone(),
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
    let result_post = post_service::select_by_ids(db, id_list.clone()).await;

    // 档案完善度：简历/紧急联系人批量计数（原生 SQL GROUP BY，避免 N+1）
    let mut resume_counts: HashMap<i64, i64> = HashMap::new();
    let mut contact_counts: HashMap<i64, i64> = HashMap::new();
    if !id_list.is_empty() {
        let placeholders = (1..=id_list.len()).map(|i| format!("${i}")).collect::<Vec<_>>().join(",");
        let values = id_list.iter().map(|i| sea_orm::Value::from(*i)).collect::<Vec<_>>();
        let resume_sql = format!(
            r#"SELECT admin_id AS id, COUNT(*)::int8 AS cnt FROM mxx_hr_resume WHERE deleted = 0 AND admin_id IN ({placeholders}) GROUP BY admin_id"#
        );
        let rows = db.query_all_raw(Statement::from_sql_and_values(
            db.get_database_backend(), resume_sql, values.clone(),
        )).await?;
        for row in rows {
            let id: i64 = row.try_get("", "id")?;
            let cnt: i64 = row.try_get("", "cnt")?;
            resume_counts.insert(id, cnt);
        }
        let contact_sql = format!(
            r#"SELECT admin_id AS id, COUNT(*)::int8 AS cnt FROM mxx_hr_emergency_contact WHERE deleted = 0 AND admin_id IN ({placeholders}) GROUP BY admin_id"#
        );
        let rows = db.query_all_raw(Statement::from_sql_and_values(
            db.get_database_backend(), contact_sql, values,
        )).await?;
        for row in rows {
            let id: i64 = row.try_get("", "id")?;
            let cnt: i64 = row.try_get("", "cnt")?;
            contact_counts.insert(id, cnt);
        }
    }

    // 批量查询用户入职审批（business_type='user'）最新实例，防 N+1（方案 B1）
    // approval_instance_id：最新实例ID；approval_status：最新实例状态；
    // need_audit：status=2 且当前登录人在当前节点候选审批人池中
    let mut approval_instance_id_map: HashMap<i64, i64> = HashMap::new();
    let mut approval_status_map: HashMap<i64, i32> = HashMap::new();
    let mut need_audit_map: HashMap<i64, bool> = HashMap::new();
    if !id_list.is_empty() {
        let placeholders = (1..=id_list.len()).map(|i| format!("${i}")).collect::<Vec<_>>().join(",");
        let values = id_list.iter().map(|i| sea_orm::Value::from(*i)).collect::<Vec<_>>();
        let inst_sql = format!(
            r#"SELECT DISTINCT ON (business_id) id, business_id, status, candidate_approvers
               FROM mxx_system_approval_instance
               WHERE business_type = 'user' AND business_id IN ({placeholders})
               ORDER BY business_id, id DESC"#
        );
        if let Ok(rows) = db.query_all_raw(Statement::from_sql_and_values(
            db.get_database_backend(), inst_sql, values,
        )).await {
            for row in rows {
                let inst_id: i64 = match row.try_get("", "id") { Ok(v) => v, Err(_) => continue };
                let biz_id: i64 = match row.try_get("", "business_id") { Ok(v) => v, Err(_) => continue };
                let status: i32 = match row.try_get("", "status") { Ok(v) => v, Err(_) => continue };
                approval_instance_id_map.insert(biz_id, inst_id);
                approval_status_map.insert(biz_id, status);
                if status == 2 {
                    if let Ok(candidates) = row.try_get::<serde_json::Value>("", "candidate_approvers") {
                        let hit = candidates.as_array().map(|arr| {
                            arr.iter().any(|x| x.as_i64() == Some(current_user_id))
                        }).unwrap_or(false);
                        if hit {
                            need_audit_map.insert(biz_id, true);
                        }
                    }
                }
            }
        }
    }

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

        let mut posts_data: Vec<PostNameDTO> = Vec::new();
        match result_post {
            Ok(ref post_list) => {
                for post_entity in post_list {
                    if post_entity.admin_id == Some(data.id) {
                        posts_data.push(PostNameDTO { post_name: post_entity.post_name.clone() });
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

        let dept_name_str = if depts_data.is_empty() {
            None
        } else {
            Some(depts_data.iter()
                .filter_map(|d| d.dept_name.clone())
                .collect::<Vec<_>>()
                .join(", "))
        };

        let post_name_str = if posts_data.is_empty() {
            None
        } else {
            Some(posts_data.iter()
                .filter_map(|p| p.post_name.clone())
                .collect::<Vec<_>>()
                .join(", "))
        };

        // 档案六项完善度
        let id_filled = data.id_card_no.as_deref().map_or(false, |s| !s.is_empty());
        let bank_filled = data.bank_card_no.as_deref().map_or(false, |s| !s.is_empty());
        let email_filled = data.email.as_deref().map_or(false, |s| !s.is_empty());
        let hire_filled = data.hire_date.is_some();
        let resume_filled = resume_counts.get(&data.id).copied().unwrap_or(0) > 0;
        let contact_filled = contact_counts.get(&data.id).copied().unwrap_or(0) > 0;
        let filled_score = u32::from(id_filled) + u32::from(bank_filled) + u32::from(email_filled)
            + u32::from(hire_filled) + u32::from(resume_filled) + u32::from(contact_filled);

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
            dept_name: dept_name_str,
            posts: Option::from(posts_data),
            post_name: post_name_str,
            remark: data.remark,
            status: data.status,
            sort: data.sort,
            login_ip: data.login_ip,
            login_date: data.login_date.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
            create_time: Option::from(data.create_time.unwrap_or_default().format("%Y-%m-%d %H:%M:%S").to_string()),
            direct_manager_id: data.direct_manager_id,
            direct_manager_name: None,
            online: false,
            audit_status: data.audit_status.unwrap_or(0),
            approval_instance_id: approval_instance_id_map.get(&data.id).copied(),
            approval_status: approval_status_map.get(&data.id).copied(),
            need_audit: need_audit_map.get(&data.id).copied().unwrap_or(false),
            hire_date: data.hire_date,
            salary_enabled: data.salary_enabled.unwrap_or(1),
            biz_enabled: data.biz_enabled.unwrap_or(1),
            probation_months: data.probation_months,
            probation_ratio: data.probation_ratio,
            id_filled,
            bank_filled,
            email_filled,
            hire_filled,
            resume_filled,
            contact_filled,
            completeness: (filled_score * 100 / 6) as i32,
        });

    }

    // 批量查询在线状态，避免逐条查缓存
    let online_sessions = crate::modules::system::service::permission_cache_service::list_online_sessions().await;
    let online_ids: std::collections::HashSet<i64> = online_sessions.into_iter().map(|(uid, _)| uid).collect();
    for item in list_data.iter_mut() {
        item.online = online_ids.contains(&item.id);
    }

    // 批量查询直属上级姓名，避免 N+1
    let manager_ids: Vec<i64> = list_data.iter()
        .filter_map(|v| v.direct_manager_id.filter(|&id| id > 0))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    if !manager_ids.is_empty() {
        let managers = AdminModel::find_by_id_in(db, manager_ids).await.unwrap_or_default();
        let manager_map: std::collections::HashMap<i64, String> = managers.into_iter()
            .map(|m| (m.id, m.nick_name.or(m.user_name).unwrap_or_default()))
            .collect();
        for item in list_data.iter_mut() {
            if let Some(mid) = item.direct_manager_id {
                if let Some(name) = manager_map.get(&mid) {
                    item.direct_manager_name = Some(name.clone());
                }
            }
        }
    }

    let count = AdminModel::select_count(db, select_where.clone()).await.unwrap_or(0);

    let page_data = ResultPage::new_simple(list_data, count);
    
    Ok(page_data)
}

pub async fn get_admin_options(db: &DbConn, biz_only: bool) -> Result<Vec<AdminOptionVO>> {
    let result = if biz_only {
        AdminModel::find_biz_options(db).await?
    } else {
        AdminModel::find_all_options(db).await?
    };
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

/// 员工列表列显示配置键
pub const COLUMNS_CONFIG_KEY: &str = "system:admin:list:columns";

/// 获取员工列表列显示配置（无配置返回 None）
pub async fn get_columns_config(db: &DbConn) -> Result<Option<String>> {
    let config = ConfigEntity::find()
        .filter(ConfigColumn::ConfigKey.eq(COLUMNS_CONFIG_KEY))
        .one(db)
        .await?;
    Ok(config.and_then(|c| c.config_value))
}

/// 保存员工列表列显示配置（upsert：存在则更新，不存在则插入，用事务）
pub async fn save_columns_config(db: &DbConn, value: &str) -> Result<i64> {
    let now = Local::now().naive_local();
    let existing = ConfigEntity::find()
        .filter(ConfigColumn::ConfigKey.eq(COLUMNS_CONFIG_KEY))
        .one(db)
        .await?;

    let value_owned = value.to_string();

    let result_id = db
        .transaction::<_, i64, sea_orm::DbErr>(|txn| {
            Box::pin(async move {
                let id = if let Some(m) = existing {
                    let mut active: ConfigActiveModel = m.into();
                    active.config_value = Set(Some(value_owned.clone()));
                    active.update_time = Set(Some(now));
                    let updated = active.update(txn).await?;
                    updated.id
                } else {
                    let active = ConfigActiveModel {
                        config_name: Set(Some("员工列表列显示配置".to_string())),
                        config_key: Set(Some(COLUMNS_CONFIG_KEY.to_string())),
                        config_value: Set(Some(value_owned.clone())),
                        config_type: Set(Some("N".to_string())),
                        remark: Set(Some("员工列表各角色可见列配置".to_string())),
                        sort: Set(Some(1)),
                        create_time: Set(Some(now)),
                        update_time: Set(Some(now)),
                        ..Default::default()
                    };
                    let inserted = ConfigEntity::insert(active).exec(txn).await?;
                    inserted.last_insert_id
                };
                Ok(id)
            })
        })
        .await
        .map_err(|e| match e {
            sea_orm::TransactionError::Connection(err) => err,
            sea_orm::TransactionError::Transaction(err) => err,
        })?;
    Ok(result_id)
}