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
use crate::modules::system::entity::dept::Model;
use crate::modules::system::model::admin_dept_merge::{AdminDeptMergeModel, AdminDeptMergeSaveDTO};
use crate::modules::system::model::dept::{DeptAdminByName, DeptDetailVO, DeptModel, DeptOptionVO, DeptOptionsTreeVO, DeptSaveDTO, DeptTreeListVO, ListQuery, PageWhere};
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;
use sea_orm::{DbConn, DbErr, TransactionTrait};
use std::collections::{HashMap, HashSet};


/// 新增部门
pub async fn insert(db: &DbConn, form_data: &DeptSaveDTO) -> Result<i64> {
    let result = DeptModel::insert(&db, form_data).await?;
    Ok(result)
}

/// 批量删除部门
pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<Option<String>>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let ids = convert_vec_option_string_to_vec_u64(ids_vec.clone());
    let result = DeptModel::batch_delete_by_ids(&db, ids).await?;
    Ok(result)
}

/// 更新部门
pub async fn update_by_id(db: &DbConn, form_data: &DeptSaveDTO) -> Result<i64> {
    let result = DeptModel::update_by_id(&db, form_data.id.unwrap(), form_data).await?;
    Ok(result)
}

/// 根据管理员ID查询关联的部门列表
pub async fn batch_update_dept(
    db: &DbConn,
    dept_ids: &Option<Vec<i64>>,
    admin_id: &Option<i64>,
) -> Result<i64> {
    // 1. 前置校验：admin_id 必须存在
    let admin_id = match admin_id {
        Some(id) => *id,
        None => return Ok(0), // 或根据需求返回错误 Err(Error::BadRequest("admin_id required"))
    };

    // 2. 预处理 dept_ids，构造插入数据
    let sys_dept_admin_list: Vec<AdminDeptMergeSaveDTO> = match dept_ids {
        Some(ids) if !ids.is_empty() => {
            let valid_dept_ids: Vec<i64> = ids
                .iter()
                .filter(|&&id| id != 0)
                .copied()
                .collect();

            valid_dept_ids
                .into_iter()
                .map(|dept_id| AdminDeptMergeSaveDTO {
                    id: None,
                    create_time: None,
                    dept_id: Some(dept_id),
                    admin_id: Some(admin_id),
                })
                .collect()
        }
        _ => Vec::new(),
    };

    // 3. 删除旧关联 + 插入新关联需原子执行，避免中途失败丢失全部部门关联
    let result = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move {
                AdminDeptMergeModel::delete_by_admin_id(txn, &Some(admin_id)).await?;
                if sys_dept_admin_list.is_empty() {
                    return Ok(0);
                }
                AdminDeptMergeModel::insert_batch(txn, &sys_dept_admin_list).await
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(result)
}




/// # 根据管理员ID查询关联的部门列表
/// ## admin_id: 用户id
///
/// * 返回值: 部门列表
///
pub async fn select_by_admin_id(db: &DbConn, admin_id: &Option<i64>) -> Result<Vec<DeptDetailVO>> {
    let result_merge = AdminDeptMergeModel::find_by_admin_id(&db, admin_id.clone().unwrap_or_default()).await?;
    let id_list: Vec<Option<i64>> = result_merge.iter().map(|data| data.dept_id).collect();
    if !id_list.is_empty() {
        let vec_u64: Vec<i64> = id_list.into_iter()
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect();
        let dept_data = DeptModel::find_by_ids(&db, vec_u64).await?;
        let mut dept_vo: Vec<DeptDetailVO> = Vec::new();
        for dept in dept_data {
            dept_vo.push(DeptDetailVO {
                id: Option::from(dept.id),
                parent_id: Option::from(dept.parent_id),
                ancestors: None,
                dept_name: dept.dept_name,
                code: dept.code,
                sort: dept.sort,
                leader: dept.leader,
                leader_id: dept.leader_id,
                phone: dept.phone,
                email: dept.email,
                status: dept.status,
                deleted: dept.deleted,
                create_by: dept.create_by,
                create_time: dept.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
                update_by: dept.update_by,
                update_time: dept.update_time,
            })
        }
        Ok(dept_vo)
    }else{
        Ok(vec![])
    }
}


pub async fn select_by_ids(db: &DbConn, admin_ids: Vec<i64>) -> Result<Vec<DeptAdminByName>> {
    let result_merge = AdminDeptMergeModel::find_by_admin_ids(db,admin_ids).await?;
    let mut list_data: Vec<DeptAdminByName> = Vec::new();
    for merge in result_merge {
        let result_dept = DeptModel::find_by_id(db,merge.dept_id.unwrap_or_default()).await?;
        list_data.push(DeptAdminByName {
            admin_id: merge.admin_id,
            dept_name: result_dept.unwrap_or_default().dept_name,
        })
    }
    Ok(list_data)
}

/// 部门下拉树形结构
pub fn dept_options_tree(re_list: &mut Vec<DeptOptionsTreeVO>, ori_arr: Vec<Model>, pid: Option<i64>) {
    for (_, it) in ori_arr.iter().enumerate() {
        if pid == it.parent_id {
            let mut children = Vec::<DeptOptionsTreeVO>::new();
            dept_options_tree(&mut children, ori_arr.clone(), Option::from(it.id));

            let temp_router = DeptOptionsTreeVO {
                children: (|| -> Option<Vec<DeptOptionsTreeVO>> {
                    if children.len() > 0 {
                        Some(children)
                    } else {
                        None
                    }
                })(),
                value: it.id.clone(),
                label: it.dept_name.clone(),
            };
            re_list.push(temp_router)
        }
    }
}

pub async fn get_dept_options(db: &DbConn) -> Result<Vec<DeptOptionVO>> {
    let list = DeptModel::find_all(db).await?;
    let mut list_data: Vec<DeptOptionVO> = Vec::new();
    for data in list {
        list_data.push(DeptOptionVO {
            value: Option::from(data.id),
            label: data.dept_name,
        });
    }
    Ok(list_data)
}

pub async fn get_dept_tree(db: &DbConn) -> Result<Vec<DeptOptionsTreeVO>> {
    let list = DeptModel::find_all(db).await?;
    let mut router_list = Vec::<DeptOptionsTreeVO>::new();
    dept_options_tree(&mut router_list, list, Some(0));
    Ok(router_list)
}


/// 部门所有树形结构
pub fn dept_all_tree(re_list: &mut Vec<DeptTreeListVO>, ori_arr: &[Model]) {
    // 构建从 id 到节点的映射
    let mut id_to_node: HashMap<i64, &Model> = HashMap::new();
    // 构建从 parent_id 到子节点列表的映射
    let mut parent_to_children: HashMap<i64, Vec<i64>> = HashMap::new();
    // 用于存储所有已知的 id
    let mut all_ids: HashSet<i64> = HashSet::new();

    for it in ori_arr.iter() {
        id_to_node.insert(it.id, it);
        all_ids.insert(it.id);
        if let Some(parent_id) = it.parent_id {
            parent_to_children.entry(parent_id).or_insert_with(Vec::new).push(it.id);
        }
    }

    // 找到所有根节点（即 parent_id 不存在于 all_ids 中的节点）
    let root_nodes: Vec<i64> = ori_arr.iter()
        .filter_map(|it| {
            if it.parent_id.is_none() || !all_ids.contains(&it.parent_id.unwrap()) {
                Some(it.id)
            } else {
                None
            }
        })
        .collect();

    // 递归构建树
    for root_id in root_nodes {
        if let Some(root_node) = id_to_node.get(&root_id) {
            let mut children = Vec::<DeptTreeListVO>::new();
            build_tree(&mut children, &id_to_node, &parent_to_children, root_id);

            let temp_router = DeptTreeListVO {
                id: Some(root_node.id),
                parent_id: root_node.parent_id,
                ancestors: root_node.ancestors.clone(),
                dept_name: root_node.dept_name.clone(),
                code: root_node.code.clone(),
                sort: root_node.sort,
                leader: root_node.leader.clone(),
                leader_id: root_node.leader_id,
                phone: root_node.phone.clone(),
                email: root_node.email.clone(),
                status: root_node.status.clone(),
                children: if children.is_empty() { None } else { Some(children) },
            };
            re_list.push(temp_router);
        }
    }
}

#[allow(dead_code)]
fn build_tree(re_list: &mut Vec<DeptTreeListVO>, id_to_node: &HashMap<i64, &Model>, parent_to_children: &HashMap<i64, Vec<i64>>, pid: i64) {
    if let Some(children_ids) = parent_to_children.get(&pid) {
        for child_id in children_ids {
            if let Some(child_node) = id_to_node.get(child_id) {
                let mut children = Vec::<DeptTreeListVO>::new();
                build_tree(&mut children, id_to_node, parent_to_children, *child_id);

                let temp_router = DeptTreeListVO {
                    id: Some(child_node.id),
                    parent_id: child_node.parent_id,
                    ancestors: child_node.ancestors.clone(),
                    dept_name: child_node.dept_name.clone(),
                    code: child_node.code.clone(),
                    sort: child_node.sort,
                    leader: child_node.leader.clone(),
                    leader_id: child_node.leader_id,
                    phone: child_node.phone.clone(),
                    email: child_node.email.clone(),
                    status: child_node.status.clone(),
                    children: if children.is_empty() { None } else { Some(children) },
                };
                re_list.push(temp_router);
            }
        }
    }
}

pub async fn get_all_tree(db: &DbConn, query : ListQuery) -> Result<Vec<DeptTreeListVO>> {
    let select_where = PageWhere {
        dept_name: query.keywords,
        code: query.code,
        status: query.status,
    };
    let search_where = select_where.format();
    let list = DeptModel::select_all(&db, search_where).await?;
    let mut router_list = Vec::<DeptTreeListVO>::new();
    dept_all_tree(&mut router_list, &list);
    Ok(router_list)
}
