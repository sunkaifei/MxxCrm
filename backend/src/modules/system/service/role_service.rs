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
use crate::modules::system::model::admin_role_merge::{AdminRoleMergeModel, AdminRolesMergeSaveDTO};
use crate::modules::system::model::role::{AdminRoleByName, ListQuery, PageWhere, RoleDetailVO, RoleListVO, RoleModel, RoleOptionVO, RoleSaveDTO, UpdateRoleMenuRequest};
use crate::modules::system::model::role_menu_merge::{RoleMenuMergeModel, RoleMenuMergeSaveDTO};
use sea_orm::{DbConn, DbErr, TransactionTrait};
use crate::modules::system::model::menu::MenuModel;

pub async fn insert(db: &DbConn, form_data: &RoleSaveDTO) -> Result<i64> {
    let result = RoleModel::insert(&db, form_data).await?;
    Ok(result)
}

/// 批量删除角色
pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }

    // 检查是否包含超级管理员角色ID (1)
    if ids_vec.contains(&1) {
        return Err(Error::from("不能删除超级管理员角色".to_string()));
    }

    let ids_clone = ids_vec.clone();
    // 角色主表与关联表需原子删除，避免产生孤儿关联数据
    let result = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move {
                let affected = RoleModel::batch_delete_by_ids(txn, &ids_clone).await?;
                if affected > 0 {
                    for &role_id in &ids_clone {
                        AdminRoleMergeModel::delete_by_role_id(txn, &Some(role_id)).await?;
                        RoleMenuMergeModel::delete_by_role_id(txn, &Some(role_id)).await?;
                    }
                }
                Ok(affected)
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(result)
}

///批量更新用户和角色关联关系
pub async fn batch_update_role(
    db: &DbConn,
    role_ids: &Option<Vec<i64>>,
    admin_id: &Option<i64>,
) -> Result<i64> {
    // 1. 参数校验前置（提前处理空值情况）
    let admin_id = match admin_id {
        Some(id) => *id,
        None => return Ok(0), // 如果 admin_id 为空，直接返回 0
    };

    // 预先处理 role_ids，构造插入数据
    let sys_role_admin_list: Vec<AdminRolesMergeSaveDTO> = match role_ids {
        Some(role_ids) if !role_ids.is_empty() => role_ids
            .iter()
            .filter(|&&id| id != 0)
            .copied()
            .map(|role_id| AdminRolesMergeSaveDTO {
                id: None,
                create_time: None,
                role_id: Some(role_id),
                admin_id: Some(admin_id),
            })
            .collect(),
        _ => Vec::new(),
    };

    // 删除旧关联 + 插入新关联需原子执行，避免中途失败丢失全部角色关联
    let result = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move {
                AdminRoleMergeModel::delete_by_admin_id(txn, &Some(admin_id)).await?;
                if sys_role_admin_list.is_empty() {
                    return Ok(0);
                }
                AdminRoleMergeModel::insert_batch(txn, &sys_role_admin_list).await
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(result)
}

/// 更新角色id和菜单id进行绑定
/// * `db` 数据库链接
/// * `form_data` 角色id和菜单id进行绑定
///
/// 范湖更新条数
pub async fn update_role_menus(
    db: &DbConn,
    form_data: &UpdateRoleMenuRequest,
) -> Result<i64> {
    // 构建新的关联列表
    let sys_role_menu_list: Vec<RoleMenuMergeSaveDTO> = form_data.menu_ids.clone().unwrap_or_default()
        .into_iter()
        .map(|menu_id| RoleMenuMergeSaveDTO {
            id: None,
            menu_id: Some(menu_id),
            role_id: form_data.role_id.clone(),
            create_time: None,
            update_time: None,
        })
        .collect();

    let role_id = form_data.role_id;
    // 删除旧关联 + 插入新关联需原子执行，避免中途失败丢失角色全部菜单权限
    let result = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move {
                RoleMenuMergeModel::delete_by_role_id(txn, &role_id).await?;
                if sys_role_menu_list.is_empty() {
                    return Ok(0);
                }
                RoleMenuMergeModel::insert_batch(txn, &sys_role_menu_list).await
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(result)
}



///更新角色信息
pub async fn update_by_id(db: &DbConn, role_data: &RoleSaveDTO) -> Result<i64> {
    let result = RoleModel::update_by_id(&db, &role_data.id, &role_data).await?;
    Ok(result)
}

pub async fn find_by_name_unique(db: &DbConn, role_name: &Option<String>, id: &Option<i64>) -> Result<bool> {
    let result = RoleModel::find_by_name_unique(&db, role_name, id).await?;
    if result > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}


///根据用户id查询角色信息
pub async fn select_by_admin_id(db: &DbConn, admin_id: &Option<i64>) -> Result<Vec<RoleDetailVO>> {
    let result_merge = AdminRoleMergeModel::find_by_admin_id(&db, admin_id).await?;
    let id_list: Vec<Option<i64>> = result_merge.iter().map(|data| data.role_id).collect();
    if !id_list.is_empty() {
        let vec_u64: Vec<i64> = id_list.into_iter()
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect();
        let role_data = RoleModel::find_by_ids(&db, vec_u64).await?;
            let mut role_vo: Vec<RoleDetailVO> = Vec::new();
        for data in role_data {
            role_vo.push(RoleDetailVO {
                id: Option::from(data.id),
                role_name: data.role_name,
                role_key: data.role_key,
                status: data.status,
                remark: data.remark,
                sort: data.sort,
                data_scope: data.data_scope,
            })
        }
        Ok(role_vo)
    }else{
        Ok(vec![])
    }
}

pub async fn select_by_ids(db: &DbConn, admin_ids: Vec<i64>) -> Result<Vec<AdminRoleByName>> {
    let result_merge = AdminRoleMergeModel::find_by_admin_ids(db,admin_ids).await?;
    let mut list_data: Vec<AdminRoleByName> = Vec::new();
    for merge in result_merge {
        let result_role = RoleModel::find_by_id(db, merge.role_id.unwrap_or_default()).await?;
        list_data.push(AdminRoleByName {
            admin_id: merge.admin_id,
            role_name: result_role.unwrap_or_default().role_name,
        });
    }
    
    Ok(list_data)
}

/// 获取用户角色组
pub async fn user_by_role_group(db: &DbConn, admin_id: &Option<i64>) -> Result<Vec<String>> {
    let result_merge = AdminRoleMergeModel::find_by_admin_id(db,admin_id).await?;
    let mut list_data: Vec<String> = Vec::new();
    for merge in result_merge {
        let result_role = RoleModel::find_by_id(db, merge.role_id.unwrap_or_default()).await?;
        if result_role.clone().unwrap_or_default().role_key.is_some() {
            list_data.push(result_role.unwrap_or_default().role_key.unwrap_or_default());
        }
    }
    Ok(list_data)
}

pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<RoleDetailVO> {
    let result = RoleModel::find_by_id(db, id.clone().unwrap_or_default()).await?.ok_or_else(|| {
        Error::from(format!(
            "{}={}",
            "角色信息不存在，id".to_string(),
            &id.unwrap_or_default()
        ))
    })?;
    Ok(RoleDetailVO::from(result))
}

/// 获取角色id关联的所有菜单id，菜单id为字符串类型
/// * `db`: 数据库连接
/// * `is_admin`: 是否是超级管理员, true为超级管理员，false为普通管理员
/// * `role_id`: 角色id
///
/// * 返回值：`Vec<Option<String>>` 关联的菜单id列表
pub async fn get_role_menu_list_by_role_id(db: &DbConn ,role_id: &Option<i64>) -> Result<Vec<Option<String>>> {
    if role_id == &Some(1) {
        let result = MenuModel::find_all(&db).await?;
        let ids: Vec<Option<String>> = result.iter().map(|menu| Some(menu.id).map(|id| id.to_string())).collect();
        Ok(ids)
    } else {
        let result = RoleMenuMergeModel::find_by_role_id(&db,role_id).await?;
        let ids: Vec<Option<String>> = result.iter().map(|merge| merge.menu_id.map(|menu_id| menu_id.to_string())).collect();
        Ok(ids)
    }
}

/// 根据角色id查询菜单id
pub async fn find_role_menu_merge_by_role_id(db: &DbConn,role_id: &Option<i64>) -> Result<Vec<Option<String>>> {
    let result = RoleMenuMergeModel::find_by_role_id(&db,role_id).await?;
    let ids: Vec<Option<String>> = result.iter().map(|merge| merge.menu_id.map(|menu_id| menu_id.to_string())).collect();
    Ok(ids)
}

/// 获取角色下拉列表
pub async fn get_role_options(db: &DbConn) -> Result<Vec<RoleOptionVO>> {
    let result = RoleModel::find_all(&db).await?;
    let mut list_data: Vec<RoleOptionVO> = Vec::new();
    for data in result {
        list_data.push(RoleOptionVO {
            value: Option::from(data.id),
            label: data.role_name,
        });
    }
    Ok(list_data)
}


/// 查询角色列表
pub async fn get_by_page(db: &DbConn, query : ListQuery) -> Result<ResultPage<Vec<RoleListVO>>> {

    let select_where = PageWhere {
        role_name: query.keywords,
        status: query.status,
    };
    let search_where = select_where.format();

    let (list, _num_pages) = RoleModel::select_in_page(
        &db,
        query.page_num.unwrap_or(0),
        query.page_size.unwrap_or(10),
        search_where.clone()
    ).await?;

    let list_data: Vec<RoleListVO> = list.into_iter().map(|item| RoleListVO::from(item)).collect();

    let count = RoleModel::select_count(db, select_where.clone()).await.unwrap_or(0);

    let page_data = ResultPage::new_simple(list_data, count);

    Ok(page_data)
}