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
use crate::modules::articles::model::article::ArticleModel;
use crate::modules::articles::model::category::CategoryModel;
use crate::modules::website::model::website::{ListQuery, PageWhere, SiteAdminListVO, SiteDetailVO, SiteModel, SiteSaveDTO, UpdateDefaultDTO};
use sea_orm::{DbConn, DbErr, TransactionTrait};
use crate::modules::system::model::admin::AdminModel;

/// 保存站点（单站模式精简版）
///
/// 单站模式下不再：
/// - 写入 `mxx_system_admin_template_merge`（多租户模板权限遗留）
/// - 复制系统模板到用户模板（改为按需复制，见 7.0.7）
/// - 动态建文章表（统一使用共享 `mxx_article` 表）
///
/// 仅保留站点基本信息写入和默认站点切换逻辑。
pub async fn save_site(db: &DbConn, form_data: &SiteSaveDTO) -> Result<i64> {
    let insert_id = SiteModel::insert(db, &form_data).await?;
    if insert_id > 0 {
        // 处理默认站点逻辑
        if form_data.is_default.unwrap_or_default() == 1 {
            set_default_site(db, &form_data.user_id, &Some(insert_id)).await?;
        }
        Ok(insert_id)
    } else {
        Err(Error::from("添加失败"))
    }
}


/// 设置默认站点
async fn set_default_site(db: &DbConn, user_id: &Option<i64>, site_id: &Option<i64>) -> Result<()> {
    let user_id = user_id.clone();
    let site_id = site_id.clone();
    // 重置旧默认 + 设置新默认需原子执行，避免出现多个或零个默认站点
    db.transaction::<_, (), DbErr>(|txn| {
        Box::pin(async move {
            SiteModel::update_by_reset_default(txn, &user_id).await?;
            SiteModel::update_by_default_id(txn, &site_id).await?;
            Ok(())
        })
    }).await.map_err(|e| Error::from(e.to_string()))?;
    Ok(())
}



/// 删除网站和对应的表
pub async fn delete_by_id(db: &DbConn, id: i64) -> Result<i64> {
    let rows = SiteModel::delete_by_id(&db, id).await?;
    if rows > 0 {
        ArticleModel::delete_table(&db, id).await?;
        CategoryModel::batch_delete_by_id(&db, &Some(id)).await?;
        Ok(rows)
    } else {
        Err(Error::from("删除失败"))
    }
}


/// 批量删除网站和对应的网站表
pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64> {
    // 删除的数量
    let mut cont = 0;
    for id in ids {
        // 按id删除网站
        let result = SiteModel::delete_by_id(&db,id).await?;
        if result > 0 {
            cont = cont + 1;
            // 删除文章表
            ArticleModel::delete_table(&db,id).await?;
            // 删除网站关联的分类
            CategoryModel::batch_delete_by_id(&db, &Some(id)).await?;
        }
    }
    Ok(cont)
}

/// 更新网站信息
pub async fn update_by_id(db: &DbConn, form_data: &SiteSaveDTO) -> Result<i64> {
    
    if form_data.is_default.unwrap_or_default() == 1 {
        set_default_site(db, &form_data.user_id, &form_data.id).await?;
    } else {
        // 判断是否还有默认站点,没有就设置当前站点为默认
        if !find_by_default_count(db, &form_data.user_id, &Some(1)).await? {
            // 重新按ID设置默认站点
            SiteModel::update_by_default_id(db, &form_data.id).await?;
        }
    }
    let rows = SiteModel::update_by_id(&db, &form_data.id, form_data).await?;
    if rows > 0 {
        Ok(rows)
    } else {
        Err(Error::from("更新失败"))
    }
}


pub async fn update_by_default_id(db: &DbConn, form_data: &UpdateDefaultDTO) -> Result<i64> {
    if form_data.is_default.unwrap_or_default() == 1 {
        let user_id = form_data.user_id.clone();
        let site_id = form_data.id.clone();
        // 重置旧默认 + 设置新默认需原子执行，避免出现多个或零个默认站点
        let rows = db
            .transaction::<_, i64, DbErr>(|txn| {
                Box::pin(async move {
                    SiteModel::update_by_reset_default(txn, &user_id).await?;
                    SiteModel::update_by_default_id(txn, &site_id).await
                })
            })
            .await
            .map_err(|e| Error::from(e.to_string()))?;
        if rows > 0 {
            Ok(rows)
        } else {
            Err(Error::from("更新失败"))
        }
    } else {
        // 检查用户是否有默认站点
        if !find_by_default_count(&db, &form_data.user_id, &Some(1)).await? {
            // 如果没有默认站点，则设置当前站点为默认
            let rows = SiteModel::update_by_default_id(db, &form_data.id).await?;
            if rows > 0 {
                Ok(rows)
            } else {
                Err(Error::from("更新失败"))
            }
        } else {
            // 如果已经有默认站点，则不需要做任何操作
            Ok(0)
        }
    }
}


/// 检查用户是否可以删除默认站点
pub async fn can_remove_default_site(db: &DbConn, user_id: &Option<i64>, site_id: &Option<i64>) -> Result<bool> {
    let default_data = SiteModel::find_by_user_default_list(&db, user_id).await?;
    if default_data.is_empty() {
        return Ok(true);
    }

    let payload_id = site_id.unwrap_or_default();

    let has_other_default = default_data.iter().any(|item| {
        item.id != payload_id && item.is_default.unwrap_or_default() == 1
    });

    Ok(has_other_default)
}


/// 查询站点名称是否已存在
/// * `db` 数据库链接
/// * `site_name` 站点名称
/// * `user_id` 用户id
/// * `id` 是否排除当前id
pub async fn find_by_name_unique(db: &DbConn, site_name: &Option<String>, user_id: &Option<i64>, id: &Option<i64>) -> Result<bool> {
    let result = SiteModel::find_by_name_count(&db, site_name, user_id, id).await?;
    if result > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// 查询该用户下是否已存在默认站点
/// * `db` 数据库链接
/// * `user_id` 用户id
/// * `is_default` 是否默认站点设置，1是默认，0不是默认,一个用户只能有一个默认的网站
pub async fn find_by_default_count(db: &DbConn, user_id: &Option<i64>, is_default: &Option<i32>) -> Result<bool> {
    let result = SiteModel::find_by_default_count(&db, user_id, is_default).await?;
    if result > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// 查询该用户下是否已存在该id的站点
/// * `db` 数据库链接
/// * `id` 站点id
/// * `user_id` 用户id
pub async fn find_by_id_unique(db: &DbConn, id: &Option<i64>, user_id: &Option<i64>) -> Result<bool> {
    let result_op = SiteModel::find_by_id(&db, id).await?;
    match result_op {
        Some(data) => Ok(data.user_id.eq(user_id)),
        None => Ok(false),
    }
}

/// 根据id查询站点信息
/// * `db` 数据库链接
/// * `id` 站点id
pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<SiteDetailVO> {
    let result_data = SiteModel::find_by_id(&db, id).await?.ok_or_else(|| {
        Error::from(format!(
            "{}",
            "该id未获取到站点信息".to_string(),
        ))
    })?;
    
    let result = SiteDetailVO::from(result_data);
    Ok(result)
}

/// 根据用户id查询默认站点信息
pub async fn find_by_user_default(db: &DbConn, user_id: &Option<i64>) -> Result<Option<SiteDetailVO>> {
    let site_model = SiteModel::find_by_user_id(db, user_id).await?;
    let result = site_model.map(SiteDetailVO::from);
    Ok(result)
}


pub async fn find_by_domain(db: &DbConn, domain: &Option<String>) -> Result<SiteDetailVO> {
    let result_data = SiteModel::find_by_domain(&db, &domain).await?.ok_or_else(|| {
        Error::from(format!(
            "{}",
            "该域名未获取到站点信息".to_string(),
        ))
    })?;
    
    let result = SiteDetailVO::from(result_data);
    Ok(result)
}


/// 查询默认站点（单站模式核心方法）
/// 优先查 is_default=1 的站点，没有则取 id 最小的站点
/// * `db` 数据库链接
pub async fn find_default(db: &DbConn) -> Result<SiteDetailVO> {
    let result_data = SiteModel::find_default(&db).await?.ok_or_else(|| {
        Error::from(format!(
            "{}",
            "未获取到默认站点信息，请先在后台创建站点".to_string(),
        ))
    })?;

    let result = SiteDetailVO::from(result_data);
    Ok(result)
}


pub async fn get_by_page(db: &DbConn, query: ListQuery) -> Result<ResultPage<Vec<SiteAdminListVO>>> {
    let select_where = PageWhere{
        site_name: query.keywords.clone(),
        user_id: query.user_id.clone(),
        bind_domain: query.bind_domain.clone(),
        domain: query.domain.clone(),
        status: query.status.clone(),
    };
    let select_where = select_where.format();

    let (list, _num_pages) = SiteModel::select_in_page(
        &db,
        query.page_num.unwrap_or(0),
        query.page_size.unwrap_or(10),
        select_where.clone()
    ).await?;

    //let list_data: Vec<SiteAdminListVO> = list.into_iter().map(|item| SiteAdminListVO::from(item)).collect();
    let list_data: Vec<SiteAdminListVO> = futures::future::join_all(
        list.into_iter().map(|item| async move {
            let admin = AdminModel::find_by_id(db, &item.user_id).await?;
            Ok(SiteAdminListVO {
                id: Option::from(item.id),
                user_id: item.user_id,
                site_name: item.site_name,
                user_name: admin.unwrap_or_default().user_name,
                domain: item.domain,
                template_id: item.template_id,
                logo: item.logo,
                client: item.client,
                site_type: item.site_type,
                status: item.status,
                sort: item.sort,
                is_default: item.is_default,
                remark: item.remark,
                create_time: item.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            })
        })
    ).await.into_iter().collect::<Result<Vec<SiteAdminListVO>>>()?;
    
    
    
    let count = SiteModel::select_count(db, select_where.clone()).await.unwrap_or(0);
    let page_data = ResultPage::new_simple(list_data, count);
    
    Ok(page_data)
}