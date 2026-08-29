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
use crate::modules::system::model::admin_perm_set_merge::{AdminPermSetMergeModel, AdminPermSetMergeSaveDTO};
use crate::modules::system::model::perm_set::{ListQuery, PageWhere, PermSetDetailVO, PermSetListVO, PermSetModel, PermSetOptionVO, PermSetSaveDTO, UpdatePermSetMenuRequest};
use crate::modules::system::model::perm_set_menu_merge::{PermSetMenuMergeModel, PermSetMenuMergeSaveDTO};
use sea_orm::{ColumnTrait, DbConn, DbErr, EntityTrait, QueryFilter, Set, TransactionTrait};
use crate::modules::system::entity::perm_set;
use crate::modules::system::model::menu::MenuModel;
use crate::modules::system::service::menu_service;

pub async fn insert(db: &DbConn, form_data: &PermSetSaveDTO) -> Result<i64> {
    let result = PermSetModel::insert(&db, form_data).await?;
    Ok(result)
}

/// 复制权限集（P3-1 一键复制）：深拷贝权限集基本信息 + 菜单授权
///
/// - 名称追加"副本"、perm_set_key 追加"-copy"后缀，冲突时追加序号防唯一约束冲突
/// - 主表与菜单关联表在同一事务内写入，中途失败不产生半成品数据
pub async fn copy_perm_set(db: &DbConn, source_id: i64, operator: Option<String>) -> Result<i64> {
    let source = PermSetModel::find_by_id(db, source_id).await?
        .ok_or_else(|| Error::from("源权限集不存在或已被删除"))?;

    // 复制前读取源权限集的菜单授权快照，事务内只做写入
    let src_menus = PermSetMenuMergeModel::find_by_perm_set_id(db, &Some(source_id)).await?;

    // 名称加"副本"，已存在则追加序号
    let base_name = format!("{}副本", source.perm_set_name.clone().unwrap_or_default());
    let mut new_name = base_name.clone();
    let mut name_seq = 1;
    while PermSetModel::find_by_name_unique(db, &Some(new_name.clone()), &None).await? > 0 {
        name_seq += 1;
        new_name = format!("{}{}", base_name, name_seq);
    }

    // perm_set_key 加"-copy"后缀，已存在则追加序号
    let base_key = format!("{}-copy", source.perm_set_key.clone().unwrap_or_default());
    let mut new_key = base_key.clone();
    let mut key_seq = 1;
    while PermSetModel::find_by_perm_set_key(db, &new_key).await?.is_some() {
        key_seq += 1;
        new_key = format!("{}{}", base_key, key_seq);
    }

    let menu_rows: Vec<PermSetMenuMergeSaveDTO> = src_menus.iter()
        .map(|m| PermSetMenuMergeSaveDTO {
            id: None,
            menu_id: m.menu_id,
            perm_set_id: None,
            create_time: None,
            update_time: None,
        })
        .collect();

    let now = chrono::Local::now().naive_local();
    let new_perm_set = perm_set::ActiveModel {
        perm_set_name: Set(Some(new_name)),
        perm_set_key: Set(Some(new_key)),
        sort: Set(source.sort),
        status: Set(source.status),
        remark: Set(source.remark.clone()),
        create_by: Set(operator.clone()),
        update_by: Set(operator),
        create_time: Set(Some(now.clone())),
        update_time: Set(Some(now)),
        ..Default::default()
    };

    // 主表 + 菜单授权原子写入
    let new_id = db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move {
            let pid = perm_set::Entity::insert(new_perm_set).exec(txn).await?.last_insert_id;
            let mut menu_rows = menu_rows;
            for row in menu_rows.iter_mut() {
                row.perm_set_id = Some(pid);
            }
            if !menu_rows.is_empty() {
                PermSetMenuMergeModel::insert_batch(txn, &menu_rows).await?;
            }
            Ok(pid)
        })
    }).await.map_err(|e| Error::from(e.to_string()))?;

    Ok(new_id)
}

/// 批量删除权限集
pub async fn batch_delete_by_ids(db: &DbConn, ids_vec: &Vec<i64>) -> Result<i64> {
    if ids_vec.is_empty() {
        return Ok(0);
    }

    let ids_clone = ids_vec.clone();
    // 权限集主表与关联表需原子删除，避免产生孤儿关联数据
    let result = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move {
                let affected = PermSetModel::batch_delete_by_ids(txn, &ids_clone).await?;
                if affected > 0 {
                    for &perm_set_id in &ids_clone {
                        AdminPermSetMergeModel::delete_by_perm_set_id(txn, &Some(perm_set_id)).await?;
                        PermSetMenuMergeModel::delete_by_perm_set_id(txn, &Some(perm_set_id)).await?;
                    }
                }
                Ok(affected)
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(result)
}

/// 批量更新用户和权限集关联关系
pub async fn batch_update_admin_perm_set(
    db: &DbConn,
    perm_set_ids: &Option<Vec<i64>>,
    admin_id: &Option<i64>,
) -> Result<i64> {
    // 1. 参数校验前置（提前处理空值情况）
    let admin_id = match admin_id {
        Some(id) => *id,
        None => return Ok(0), // 如果 admin_id 为空，直接返回 0
    };

    // 预先处理 perm_set_ids，构造插入数据
    let merge_list: Vec<AdminPermSetMergeSaveDTO> = match perm_set_ids {
        Some(ids) if !ids.is_empty() => ids
            .iter()
            .filter(|&&id| id != 0)
            .copied()
            .map(|perm_set_id| AdminPermSetMergeSaveDTO {
                id: None,
                create_time: None,
                perm_set_id: Some(perm_set_id),
                admin_id: Some(admin_id),
            })
            .collect(),
        _ => Vec::new(),
    };

    // 删除旧关联 + 插入新关联需原子执行，避免中途失败丢失全部权限集关联
    let result = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move {
                AdminPermSetMergeModel::delete_by_admin_id(txn, &Some(admin_id)).await?;
                if merge_list.is_empty() {
                    return Ok(0);
                }
                AdminPermSetMergeModel::insert_batch(txn, &merge_list).await
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(result)
}

/// 更新权限集id和菜单id进行绑定
/// * `db` 数据库链接
/// * `form_data` 权限集id和菜单id进行绑定
///
/// 返回更新条数
pub async fn update_perm_set_menus(
    db: &DbConn,
    form_data: &UpdatePermSetMenuRequest,
) -> Result<i64> {
    // 链路完整性过滤：父级未勾选的菜单不保存（如勾选了子级页面但未勾选父级菜单），
    // 与读端 get_user_router_tree 的菜单可见性规则保持一致，避免产生"有子无父"的脏权限数据
    // 注意：权限范围校验（授权人是否有权授予这些菜单）由 controller 层在调用前完成
    let requested_ids = form_data.menu_ids.clone().unwrap_or_default();
    let valid_ids: Vec<i64> = if requested_ids.is_empty() {
        requested_ids
    } else {
        let all_menus = MenuModel::find_all(db).await?;
        let parent_map: std::collections::HashMap<i64, i64> = all_menus.iter().map(|m| (m.id, m.parent_id)).collect();
        let authorized: std::collections::HashSet<i64> = requested_ids.iter().cloned().collect();
        requested_ids.into_iter()
            .filter(|&mid| menu_service::is_menu_chain_complete(mid, &parent_map, &authorized))
            .collect()
    };

    // 构建新的关联列表
    let merge_list: Vec<PermSetMenuMergeSaveDTO> = valid_ids
        .into_iter()
        .map(|menu_id| PermSetMenuMergeSaveDTO {
            id: None,
            menu_id: Some(menu_id),
            perm_set_id: form_data.perm_set_id.clone(),
            create_time: None,
            update_time: None,
        })
        .collect();

    let perm_set_id = form_data.perm_set_id;
    // 删除旧关联 + 插入新关联需原子执行，避免中途失败丢失权限集全部菜单权限
    let result = db
        .transaction::<_, i64, DbErr>(|txn| {
            Box::pin(async move {
                PermSetMenuMergeModel::delete_by_perm_set_id(txn, &perm_set_id).await?;
                if merge_list.is_empty() {
                    // 清空权限集授权（menu_ids 为空数组）属合法操作：旧关联已删除，返回成功
                    return Ok(1);
                }
                PermSetMenuMergeModel::insert_batch(txn, &merge_list).await
            })
        })
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(result)
}

/// 更新权限集信息
pub async fn update_by_id(db: &DbConn, perm_set_data: &PermSetSaveDTO) -> Result<i64> {
    let result = PermSetModel::update_by_id(&db, &perm_set_data.id, &perm_set_data).await?;
    Ok(result)
}

pub async fn find_by_name_unique(db: &DbConn, perm_set_name: &Option<String>, id: &Option<i64>) -> Result<bool> {
    let result = PermSetModel::find_by_name_unique(&db, perm_set_name, id).await?;
    if result > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// 根据用户id查询权限集信息
pub async fn select_by_admin_id(db: &DbConn, admin_id: &Option<i64>) -> Result<Vec<PermSetDetailVO>> {
    let result_merge = AdminPermSetMergeModel::find_by_admin_id(&db, admin_id).await?;
    let id_list: Vec<Option<i64>> = result_merge.iter().map(|data| data.perm_set_id).collect();
    if !id_list.is_empty() {
        let vec_u64: Vec<i64> = id_list.into_iter()
            .flatten()
            .collect();
        let perm_set_data = PermSetModel::find_by_ids(&db, vec_u64).await?;
        let mut perm_set_vo: Vec<PermSetDetailVO> = Vec::new();
        for data in perm_set_data {
            perm_set_vo.push(PermSetDetailVO {
                id: Option::from(data.id),
                perm_set_name: data.perm_set_name,
                perm_set_key: data.perm_set_key,
                status: data.status,
                remark: data.remark,
                sort: data.sort,
            })
        }
        Ok(perm_set_vo)
    } else {
        Ok(vec![])
    }
}

/// 获取权限集id关联的所有菜单id，菜单id为字符串类型
/// * `db`: 数据库连接
/// * `perm_set_id`: 权限集id
///
/// * 返回值：`Vec<Option<String>>` 关联的菜单id列表
pub async fn get_perm_set_menu_list_by_perm_set_id(db: &DbConn, perm_set_id: &Option<i64>) -> Result<Vec<Option<String>>> {
    let result = PermSetMenuMergeModel::find_by_perm_set_id(&db, perm_set_id).await?;
    let ids: Vec<Option<String>> = result.iter().map(|merge| merge.menu_id.map(|menu_id| menu_id.to_string())).collect();
    Ok(ids)
}

/// 获取权限集下拉列表
pub async fn get_perm_set_options(db: &DbConn) -> Result<Vec<PermSetOptionVO>> {
    let result = PermSetModel::find_all(&db).await?;
    let mut list_data: Vec<PermSetOptionVO> = Vec::new();
    for data in result {
        list_data.push(PermSetOptionVO {
            value: Option::from(data.id),
            label: data.perm_set_name,
        });
    }
    Ok(list_data)
}

/// 查询权限集列表
pub async fn get_by_page(db: &DbConn, query: ListQuery) -> Result<ResultPage<Vec<PermSetListVO>>> {
    let select_where = PageWhere {
        perm_set_name: query.keywords,
        status: query.status,
    };
    let search_where = select_where.format();

    let (list, _num_pages) = PermSetModel::select_in_page(
        &db,
        query.page_num.unwrap_or(0),
        query.page_size.unwrap_or(10),
        search_where.clone()
    ).await?;

    let list_data: Vec<PermSetListVO> = list.into_iter().map(|item| PermSetListVO::from(item)).collect();

    let count = PermSetModel::select_count(db, select_where.clone()).await.unwrap_or(0);

    let page_data = ResultPage::new_simple(list_data, count);

    Ok(page_data)
}

pub async fn get_by_detail(db: &DbConn, id: &Option<i64>) -> Result<PermSetDetailVO> {
    let result = PermSetModel::find_by_id(db, id.clone().unwrap_or_default()).await?.ok_or_else(|| {
        Error::from(format!(
            "{}={}",
            "权限集信息不存在，id".to_string(),
            &id.unwrap_or_default()
        ))
    })?;
    let vo = PermSetDetailVO::from(result);
    Ok(vo)
}
