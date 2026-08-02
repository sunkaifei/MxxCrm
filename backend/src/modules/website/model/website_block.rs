//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::website::entity::{website_block, website_block::Entity as WebsiteBlock};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};
use sea_orm::*;


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct BlockSaveRequest {
    /// 区块编码
    pub block_code: Option<String>,
    /// 区块名称
    pub block_name: Option<String>,
    /// 区块类型：1=文本, 2=HTML, 3=图片, 4=链接
    pub block_type: Option<i32>,
    /// 内容
    pub content: Option<String>,
    /// 图片地址
    pub image_url: Option<String>,
    /// 链接地址
    pub link_url: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
}

impl From<BlockSaveRequest> for BlockSaveDTO {
    fn from(req: BlockSaveRequest) -> Self {
        BlockSaveDTO {
            id: None,
            block_code: req.block_code,
            block_name: req.block_name,
            block_type: req.block_type,
            content: req.content,
            image_url: req.image_url,
            link_url: req.link_url,
            sort: req.sort,
            status: req.status,
        }
    }
}


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct BlockUpdateRequest {
    /// 主键ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 区块编码
    pub block_code: Option<String>,
    /// 区块名称
    pub block_name: Option<String>,
    /// 区块类型：1=文本, 2=HTML, 3=图片, 4=链接
    pub block_type: Option<i32>,
    /// 内容
    pub content: Option<String>,
    /// 图片地址
    pub image_url: Option<String>,
    /// 链接地址
    pub link_url: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
}

impl From<BlockUpdateRequest> for BlockSaveDTO {
    fn from(req: BlockUpdateRequest) -> Self {
        BlockSaveDTO {
            id: req.id,
            block_code: req.block_code,
            block_name: req.block_name,
            block_type: req.block_type,
            content: req.content,
            image_url: req.image_url,
            link_url: req.link_url,
            sort: req.sort,
            status: req.status,
        }
    }
}


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlockSaveDTO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 区块编码
    pub block_code: Option<String>,
    /// 区块名称
    pub block_name: Option<String>,
    /// 区块类型：1=文本, 2=HTML, 3=图片, 4=链接
    pub block_type: Option<i32>,
    /// 内容
    pub content: Option<String>,
    /// 图片地址
    pub image_url: Option<String>,
    /// 链接地址
    pub link_url: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlockListVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 区块编码
    pub block_code: Option<String>,
    /// 区块名称
    pub block_name: Option<String>,
    /// 区块类型：1=文本, 2=HTML, 3=图片, 4=链接
    pub block_type: Option<i32>,
    /// 内容
    pub content: Option<String>,
    /// 图片地址
    pub image_url: Option<String>,
    /// 链接地址
    pub link_url: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 更新时间
    pub update_time: Option<String>,
}

impl From<website_block::Model> for BlockListVO {
    fn from(model: website_block::Model) -> Self {
        BlockListVO {
            id: Option::from(model.id),
            block_code: model.block_code,
            block_name: model.block_name,
            block_type: model.block_type,
            content: model.content,
            image_url: model.image_url,
            link_url: model.link_url,
            sort: model.sort,
            status: model.status,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlockDetailVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 区块编码
    pub block_code: Option<String>,
    /// 区块名称
    pub block_name: Option<String>,
    /// 区块类型：1=文本, 2=HTML, 3=图片, 4=链接
    pub block_type: Option<i32>,
    /// 内容
    pub content: Option<String>,
    /// 图片地址
    pub image_url: Option<String>,
    /// 链接地址
    pub link_url: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 更新时间
    pub update_time: Option<String>,
}

impl From<website_block::Model> for BlockDetailVO {
    fn from(model: website_block::Model) -> Self {
        BlockDetailVO {
            id: Option::from(model.id),
            block_code: model.block_code,
            block_name: model.block_name,
            block_type: model.block_type,
            content: model.content,
            image_url: model.image_url,
            link_url: model.link_url,
            sort: model.sort,
            status: model.status,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_time: model.update_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub keywords: Option<String>,
    /// 区块类型
    pub block_type: Option<i32>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Clone)]
pub struct PageWhere {
    pub keywords: Option<String>,
    /// 区块类型
    pub block_type: Option<i32>,
    /// 状态：0停用，1正常
    pub status: Option<i32>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut keywords = None;
        if self.keywords != Some("".to_string()) {
            keywords = self.keywords.clone();
        }

        let mut block_type = None;
        if self.block_type > Some(0) {
            block_type = self.block_type;
        }

        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        Self {
            keywords,
            block_type,
            status,
        }
    }
}

pub struct WebsiteBlockModel;

impl WebsiteBlockModel {
    pub async fn insert(db: &DbConn, dto: &BlockSaveDTO) -> Result<i64, DbErr> {
        let model = website_block::ActiveModel {
            block_code: Set(dto.block_code.to_owned()),
            block_name: Set(dto.block_name.to_owned()),
            block_type: Set(dto.block_type.to_owned()),
            content: Set(dto.content.to_owned()),
            image_url: Set(dto.image_url.to_owned()),
            link_url: Set(dto.link_url.to_owned()),
            sort: Set(dto.sort.to_owned()),
            status: Set(dto.status.to_owned()),
            create_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let res = WebsiteBlock::insert(model).exec(db).await?;
        Ok(res.last_insert_id)
    }

    /// 按id批量软删除
    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        let result: UpdateResult = WebsiteBlock::update_many()
            .set(website_block::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(website_block::Column::Id.is_in(ids))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, dto: &BlockSaveDTO) -> Result<i64, DbErr> {
        let model = website_block::ActiveModel {
            block_code: Set(dto.block_code.to_owned()),
            block_name: Set(dto.block_name.to_owned()),
            block_type: Set(dto.block_type.to_owned()),
            content: Set(dto.content.to_owned()),
            image_url: Set(dto.image_url.to_owned()),
            link_url: Set(dto.link_url.to_owned()),
            sort: Set(dto.sort.to_owned()),
            status: Set(dto.status.to_owned()),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let result: UpdateResult = WebsiteBlock::update_many()
            .set(model)
            .filter(website_block::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;
        Ok(result.rows_affected as i64)
    }

    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<website_block::Model>, DbErr> {
        let result = website_block::Entity::find_by_id(id.clone().unwrap_or_default())
            .filter(website_block::Column::Deleted.eq(0))
            .one(db)
            .await?;
        Ok(result)
    }

    pub async fn find_by_code(
        db: &DbConn,
        code: &Option<String>,
    ) -> Result<Option<website_block::Model>, DbErr> {
        let result = website_block::Entity::find()
            .filter(website_block::Column::BlockCode.eq(code.clone().unwrap_or_default()))
            .filter(website_block::Column::Deleted.eq(0))
            .filter(website_block::Column::Status.eq(1))
            .one(db)
            .await?;
        Ok(result)
    }

    pub async fn select_in_page(
        db: &DbConn,
        page_num: i64,
        page_size: i64,
        where_case: PageWhere,
    ) -> Result<(Vec<website_block::Model>, u64), DbErr> {
        let paginator = WebsiteBlock::find()
            .filter(website_block::Column::Deleted.eq(0))
            .apply_if(where_case.keywords.clone(), |query, v| {
                query.filter(website_block::Column::BlockName.contains(v))
            })
            .apply_if(where_case.block_type, |query, v| {
                query.filter(website_block::Column::BlockType.eq(v))
            })
            .apply_if(where_case.status, |query, v| {
                query.filter(website_block::Column::Status.eq(v))
            })
            .order_by_asc(website_block::Column::Sort)
            .paginate(db, page_size as u64);
        let total = paginator.num_items().await?;
        let list = paginator.fetch_page(page_num as u64).await?;
        Ok((list, total))
    }

    pub async fn select_count(db: &DbConn, where_case: PageWhere) -> Result<i64, DbErr> {
        let count = WebsiteBlock::find()
            .filter(website_block::Column::Deleted.eq(0))
            .apply_if(where_case.keywords.clone(), |query, v| {
                query.filter(website_block::Column::BlockName.contains(v))
            })
            .apply_if(where_case.block_type, |query, v| {
                query.filter(website_block::Column::BlockType.eq(v))
            })
            .apply_if(where_case.status, |query, v| {
                query.filter(website_block::Column::Status.eq(v))
            })
            .count(db)
            .await? as i64;
        Ok(count)
    }
}
