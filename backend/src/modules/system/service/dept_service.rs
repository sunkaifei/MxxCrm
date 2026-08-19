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
use crate::modules::system::entity::dept::{Column as DeptColumn, Entity as DeptEntity, Model};
use crate::modules::system::model::admin_dept_merge::{AdminDeptMergeModel, AdminDeptMergeSaveDTO};
use crate::modules::system::model::dept::{DeptAdminByName, DeptDetailVO, DeptModel, DeptOptionVO, DeptOptionsTreeVO, DeptSaveDTO, DeptTreeListVO, ListQuery, PageWhere};
use crate::utils::string_utils::convert_vec_option_string_to_vec_u64;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, QueryFilter, Set, TransactionTrait};
use std::collections::{HashMap, HashSet};


/// ==================== 唯一性/合法性校验 ====================

/// 同一父级下部门名称唯一（exclude_id：更新时排除自身）
async fn check_name_unique(db: &DbConn, parent_id: i64, dept_name: &str, exclude_id: Option<i64>) -> Result<()> {
    let mut qr = DeptEntity::find()
        .filter(DeptColumn::Deleted.eq(0))
        .filter(DeptColumn::ParentId.eq(parent_id))
        .filter(DeptColumn::DeptName.eq(dept_name.trim()));
    if let Some(id) = exclude_id {
        qr = qr.filter(DeptColumn::Id.ne(id));
    }
    if qr.one(db).await?.is_some() {
        return Err(Error::from(format!("同级下已存在同名部门【{}】", dept_name.trim())));
    }
    Ok(())
}

/// 部门编码全局唯一（非空时校验；exclude_id：更新时排除自身）
async fn check_code_unique(db: &DbConn, code: &Option<String>, exclude_id: Option<i64>) -> Result<()> {
    let code = match code {
        Some(c) if !c.trim().is_empty() => c.trim().to_string(),
        _ => return Ok(()),
    };
    let mut qr = DeptEntity::find()
        .filter(DeptColumn::Deleted.eq(0))
        .filter(DeptColumn::Code.eq(code.clone()));
    if let Some(id) = exclude_id {
        qr = qr.filter(DeptColumn::Id.ne(id));
    }
    if qr.one(db).await?.is_some() {
        return Err(Error::from(format!("部门编码【{}】已被使用", code)));
    }
    Ok(())
}

/// 顶级部门唯一（审批流以唯一根节点规划，顶级只允许存在一个）
async fn check_top_level_unique(db: &DbConn, exclude_id: Option<i64>) -> Result<()> {
    let mut qr = DeptEntity::find()
        .filter(DeptColumn::Deleted.eq(0))
        .filter(DeptColumn::ParentId.eq(0));
    if let Some(id) = exclude_id {
        qr = qr.filter(DeptColumn::Id.ne(id));
    }
    if qr.one(db).await?.is_some() {
        return Err(Error::from("顶级部门已存在，组织架构只允许一个顶级（根）部门，其余部门请挂在顶级部门之下"));
    }
    Ok(())
}

/// 校验目标父级不能是自身或自身的子孙（防环）
async fn check_not_descendant(db: &DbConn, self_id: i64, new_parent_id: i64) -> Result<()> {
    if new_parent_id == 0 {
        return Ok(());
    }
    // 沿目标父级向上找，若遇到自己则构成环
    let mut cursor = new_parent_id;
    for _ in 0..50 {
        if cursor == self_id {
            return Err(Error::from("上级部门不能选择自己或自己的下级部门"));
        }
        let parent = DeptEntity::find_by_id(cursor)
            .filter(DeptColumn::Deleted.eq(0))
            .one(db)
            .await?;
        match parent {
            Some(p) => {
                cursor = p.parent_id.unwrap_or(0);
                if cursor == 0 {
                    break;
                }
            }
            None => return Err(Error::from("所选上级部门不存在")),
        }
    }
    Ok(())
}

/// 计算部门的祖先链（0,100,101 形式，顶级为 "0"）
async fn build_ancestors(db: &DbConn, parent_id: i64) -> Result<String> {
    if parent_id == 0 {
        return Ok("0".to_string());
    }
    let mut chain: Vec<String> = Vec::new();
    let mut cursor = parent_id;
    for _ in 0..50 {
        let p = DeptEntity::find_by_id(cursor)
            .filter(DeptColumn::Deleted.eq(0))
            .one(db)
            .await?
            .ok_or_else(|| Error::from("所选上级部门不存在"))?;
        chain.insert(0, cursor.to_string());
        cursor = p.parent_id.unwrap_or(0);
        if cursor == 0 {
            break;
        }
    }
    Ok(format!("0,{}", chain.join(",")))
}

/// 变更上级后，级联重算自身及所有子孙的 ancestors（保持审批链路正确）
async fn cascade_update_ancestors(db: &DbConn, self_id: i64, new_ancestors: &str) -> Result<()> {
    // 收集所有子孙（按 parent_id 递归）
    let mut all = DeptEntity::find()
        .filter(DeptColumn::Deleted.eq(0))
        .all(db)
        .await?;
    let mut children_map: HashMap<i64, Vec<Model>> = HashMap::new();
    for m in all.drain(..) {
        children_map.entry(m.parent_id.unwrap_or(0)).or_default().push(m);
    }
    let mut stack = vec![self_id];
    while let Some(pid) = stack.pop() {
        if let Some(children) = children_map.get(&pid) {
            for c in children {
                let child_anc = format!("{},{}", new_ancestors, c.id);
                let mut am: dept::ActiveModel = c.clone().into();
                am.ancestors = Set(Some(child_anc.clone()));
                am.update(db).await?;
                stack.push(c.id);
            }
        }
    }
    Ok(())
}

use crate::modules::system::entity::dept;

/// 新增部门
pub async fn insert(db: &DbConn, form_data: &DeptSaveDTO) -> Result<i64> {
    let parent_id = form_data.parent_id.unwrap_or(0);
    let dept_name = form_data.dept_name.clone()
        .ok_or_else(|| Error::from("部门名称不能为空"))?;

    // 唯一性校验：同级重名 / 编码全局唯一 / 顶级唯一
    check_name_unique(db, parent_id, &dept_name, None).await?;
    check_code_unique(db, &form_data.code, None).await?;
    if parent_id == 0 {
        check_top_level_unique(db, None).await?;
    }
    // 上级必须存在（非顶级时）
    if parent_id != 0 {
        let parent = DeptEntity::find_by_id(parent_id)
            .filter(DeptColumn::Deleted.eq(0))
            .one(db)
            .await?;
        if parent.is_none() {
            return Err(Error::from("所选上级部门不存在"));
        }
    }

    let mut form_data = form_data.clone();
    form_data.ancestors = Some(build_ancestors(db, parent_id).await?);
    let result = DeptModel::insert(&db, &form_data).await?;
    Ok(result)
}

/// 批量删除部门
pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<Option<String>>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }
    let ids = convert_vec_option_string_to_vec_u64(ids_vec.clone());

    // 1. 检查是否存在下级部门
    let children = DeptModel::find_by_parent_ids(&db, ids.clone()).await?;
    if !children.is_empty() {
        let names: Vec<String> = children.iter()
            .filter_map(|c| c.dept_name.clone())
            .collect();
        return Err(Error::from(format!("存在下级部门【{}】，请先删除下级部门后再删除", names.join("、"))));
    }

    // 2. 检查是否存在关联员工
    let admin_merge_list = AdminDeptMergeModel::find_by_dept_id(&db, ids.clone()).await?;
    if !admin_merge_list.is_empty() {
        return Err(Error::from("该部门目前有员工，需要转移员工后才能删除"));
    }

    let result = DeptModel::batch_delete_by_ids(&db, ids).await?;
    Ok(result)
}

/// 更新部门
pub async fn update_by_id(db: &DbConn, form_data: &DeptSaveDTO) -> Result<i64> {
    let self_id = form_data.id.unwrap();
    let old = DeptEntity::find_by_id(self_id)
        .filter(DeptColumn::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("部门不存在"))?;

    let new_parent_id = form_data.parent_id.unwrap_or(0);
    let dept_name = form_data.dept_name.clone()
        .ok_or_else(|| Error::from("部门名称不能为空"))?;

    // 唯一性校验（排除自身）：同级重名 / 编码全局唯一 / 顶级唯一
    check_name_unique(db, new_parent_id, &dept_name, Some(self_id)).await?;
    check_code_unique(db, &form_data.code, Some(self_id)).await?;
    if new_parent_id == 0 {
        check_top_level_unique(db, Some(self_id)).await?;
    }
    // 防环：上级不能是自己或自己的子孙
    check_not_descendant(db, self_id, new_parent_id).await?;

    // 上级发生变更：重算自身及子孙的 ancestors
    let parent_changed = old.parent_id.unwrap_or(0) != new_parent_id;
    let new_ancestors = build_ancestors(db, new_parent_id).await?;
    let mut form_data = form_data.clone();
    form_data.ancestors = Some(new_ancestors.clone());

    let result = DeptModel::update_by_id(&db, self_id, &form_data).await?;
    if parent_changed {
        cascade_update_ancestors(db, self_id, &new_ancestors).await?;
    }
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
            .flatten()
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
