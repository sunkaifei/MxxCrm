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
use crate::modules::shop::model::spu::*;
use sea_orm::DbConn;

pub async fn create_spu(db: &DbConn, req: SpuRequest) -> Result<i64> {
    let dto: SpuDTO = req.into();
    SpuModel::insert(db, &dto).await
        .map_err(|e| Error::from(format!("创建商品失败: {:?}", e)))
}

pub async fn get_spu_by_id(db: &DbConn, id: i64) -> Result<Option<SpuVO>> {
    let model = SpuModel::find_by_id(db, id).await
        .map_err(|e| Error::from(format!("查询商品失败: {:?}", e)))?;
    Ok(model.map(|m| m.into()))
}

pub async fn get_spu_page(
    db: &DbConn,
    page_num: i64,
    page_size: i64,
    shop_id: Option<i64>,
    category_id: Option<i64>,
    status: Option<i16>,
) -> Result<(Vec<SpuVO>, i64)> {
    let (list, total) = SpuModel::find_page(db, page_num, page_size, shop_id, category_id, status).await
        .map_err(|e| Error::from(format!("查询商品分页失败: {:?}", e)))?;

    let vo_list: Vec<SpuVO> = list.into_iter().map(|m| m.into()).collect();
    Ok((vo_list, total))
}

pub async fn update_spu_status(db: &DbConn, id: i64, status: i16) -> Result<i64> {
    SpuModel::update_status(db, id, status).await
        .map_err(|e| Error::from(format!("更新商品状态失败: {:?}", e)))
}

pub async fn get_skus_by_spu_id(db: &DbConn, spu_id: i64) -> Result<Vec<SkuVO>> {
    let list = SkuModel::find_by_spu_id(db, spu_id).await
        .map_err(|e| Error::from(format!("查询SKU列表失败: {:?}", e)))?;
    Ok(list.into_iter().map(|m| m.into()).collect())
}

pub async fn get_specs_by_spu_id(db: &DbConn, spu_id: i64) -> Result<Vec<SpecVO>> {
    let specs = SpecModel::find_by_spu_id(db, spu_id).await
        .map_err(|e| Error::from(format!("查询规格列表失败: {:?}", e)))?;

    let mut spec_vos = Vec::new();
    for spec in specs {
        let spec_id = spec.id;
        let values = SpecValueModel::find_by_spec_id(db, spec_id).await
            .map_err(|e| Error::from(format!("查询规格值列表失败: {:?}", e)))?;

        let value_vos: Vec<SpecValueVO> = values.into_iter().map(|v| SpecValueVO {
            id: Some(v.id),
            spec_id: Some(v.spec_id),
            spec_value: Some(v.spec_value),
            sort_order: Some(v.sort_order),
        }).collect();

        spec_vos.push(SpecVO {
            id: Some(spec.id),
            spu_id: Some(spec.spu_id),
            spec_name: Some(spec.spec_name),
            sort_order: Some(spec.sort_order),
            values: Some(value_vos),
        });
    }
    Ok(spec_vos)
}
