//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

#[cfg(feature = "enable-es")]
use std::sync::LazyLock;
#[cfg(feature = "enable-es")]
use elasticsearch::http::StatusCode;
#[cfg(feature = "enable-es")]
use elasticsearch::{BulkOperation, BulkParts, DeleteParts, IndexParts, SearchParts};
#[cfg(feature = "enable-es")]
use elasticsearch::indices::{IndicesCreateParts, IndicesDeleteParts, IndicesExistsParts};
#[cfg(feature = "enable-es")]
use serde_json::{json, Value};
use crate::core::errors::error::{Error, Result};
#[cfg(feature = "enable-es")]
use crate::core::kit::elasticsearch::ES_CLIENT;
#[cfg(feature = "enable-es")]
use crate::core::web::entity::page::Page;
use crate::modules::search::entity::search_model::{IndexSaveRequest, RelatedQueryRequest, SearchPageRequest, SearchPageVO};
#[cfg(feature = "enable-es")]
use crate::core::kit::config;
#[cfg(feature = "enable-es")]
use crate::utils::string_utils::filter_html_tags;

#[cfg(feature = "enable-es")]
pub static INDEX_NAME: LazyLock<String> = LazyLock::new(|| { format!("{}", config::section::<String>("server", "index_name", "".to_string())) });

/// 单条信息创建索引
#[cfg(feature = "enable-es")]
pub async fn create_index(index: IndexSaveRequest) -> Result<bool> {
    //检查是否有索引，没有就创建
    if let Err(err) = create_index_mappings().await {
        eprintln!("Failed to create index mappings: {}", err);
        return Err(err.into());
    }

    let content_string = filter_html_tags(index.content);
    // 定义要插入的文档内容
    let doc = json!({
        "infoType": index.info_type,
        "regionIds": index.region_ids,
        "columnIds": index.column_ids,
        "title": index.title,
        "content": Some(content_string),
        "basics": index.basics,
        "createTime": index.create_time,
    });
    
    let response = ES_CLIENT
        .index(IndexParts::IndexId(&*INDEX_NAME, &*index.id.unwrap_or_default()))
        .body(doc)
        .send()
        .await
        .map_err(|e| Error::from(format!("msg={},code=500", e)))?;

    let status = response.status_code();
    Ok(status == StatusCode::CREATED || status == StatusCode::OK)
}

/// 批量创建索引
#[cfg(feature = "enable-es")]
pub async fn batch_index(index_request: Vec<IndexSaveRequest>) -> Result<bool> {
    let mut bulk_operations: Vec<BulkOperation<_>> = Vec::new();
    
    for index in index_request {
        let content_string = filter_html_tags(index.content.clone());
        let doc = json!({
            "infoType": index.info_type,
            "regionIds": index.region_ids,
            "columnIds": index.column_ids,
            "title": index.title,
            "content": Some(content_string),
            "basics": index.basics,
            "createTime": index.create_time,
        });
        
        let operation = BulkOperation::from(
            BulkOperation::index(doc)
                .index(&*INDEX_NAME)
                .id(index.id.unwrap_or_default())
        );
        bulk_operations.push(operation);
    }

    let response = ES_CLIENT
        .bulk(BulkParts::Index(&*INDEX_NAME))
        .body(bulk_operations)
        .send()
        .await
        .map_err(|e| Error::from(format!("msg={},code=500", e)))?;

    let status = response.status_code();
    Ok(status == StatusCode::OK)
}

/// 检查索引是否存在
#[cfg(feature = "enable-es")]
pub async fn index_exists() -> Result<bool> {
    let response = ES_CLIENT
        .indices()
        .exists(IndicesExistsParts::Index(&[&*INDEX_NAME]))
        .send()
        .await
        .map_err(|e| Error::from(format!("msg={},code=500", e)))?;

    Ok(response.status_code() == StatusCode::OK)
}

/// 创建索引映射
#[cfg(feature = "enable-es")]
pub async fn create_index_mappings() -> Result<bool> {
    if index_exists().await? {
        return Ok(true);
    }

    let mappings = json!({
        "mappings": {
            "properties": {
                "infoType": { "type": "keyword" },
                "regionIds": { "type": "keyword" },
                "columnIds": { "type": "keyword" },
                "title": { 
                    "type": "text",
                    "analyzer": "ik_max_word",
                    "search_analyzer": "ik_smart"
                },
                "content": { 
                    "type": "text",
                    "analyzer": "ik_max_word",
                    "search_analyzer": "ik_smart"
                },
                "basics": { "type": "keyword" },
                "createTime": { "type": "date" }
            }
        }
    });

    let response = ES_CLIENT
        .indices()
        .create(IndicesCreateParts::Index(&*INDEX_NAME))
        .body(mappings)
        .send()
        .await
        .map_err(|e| Error::from(format!("msg={},code=500", e)))?;

    let status = response.status_code();
    Ok(status == StatusCode::OK)
}

/// 删除索引
#[cfg(feature = "enable-es")]
pub async fn delete_index() -> Result<bool> {
    if !index_exists().await? {
        return Ok(true);
    }

    let response = ES_CLIENT
        .indices()
        .delete(IndicesDeleteParts::Index(&[&*INDEX_NAME]))
        .send()
        .await
        .map_err(|e| Error::from(format!("msg={},code=500", e)))?;

    let status = response.status_code();
    Ok(status == StatusCode::OK)
}

/// 删除文档
#[cfg(feature = "enable-es")]
pub async fn delete_doc(id: String) -> Result<bool> {
    let response = ES_CLIENT
        .delete(DeleteParts::IndexId(&*INDEX_NAME, &id))
        .send()
        .await
        .map_err(|e| Error::from(format!("msg={},code=500", e)))?;

    let status = response.status_code();
    Ok(status == StatusCode::OK)
}

/// 搜索列表
#[cfg(feature = "enable-es")]
pub async fn search_list(item: SearchPageRequest) ->Result<Page<SearchPageVO>> {
    let mut query_conditions: Vec<Value> = Vec::new();

    if let Some(title) = &item.title {
        if !title.is_empty() {
            query_conditions.push(json!({
                "multi_match": {
                    "query": title,
                    "fields": ["title^2", "content"]
                }
            }));
        }
    }

    if let Some(info_type) = &item.info_type {
        if !info_type.is_empty() {
            query_conditions.push(json!({
                "term": { "infoType": info_type }
            }));
        }
    }

    if let Some(region_id) = &item.region_id {
        if !region_id.is_empty() {
            query_conditions.push(json!({
                "term": { "regionIds": region_id }
            }));
        }
    }

    if let Some(column_id) = &item.column_id {
        if !column_id.is_empty() {
            query_conditions.push(json!({
                "term": { "columnIds": column_id }
            }));
        }
    }

    let query = if query_conditions.is_empty() {
        json!({ "match_all": {} })
    } else if query_conditions.len() == 1 {
        query_conditions.remove(0)
    } else {
        json!({ "bool": { "must": query_conditions } })
    };

    let search_body = json!({
        "query": query,
        "from": (item.page.unwrap_or(1) - 1) * item.limit.unwrap_or(10),
        "size": item.limit.unwrap_or(10),
        "sort": [
            { "createTime": { "order": "desc" } }
        ],
        "_source": ["id", "infoType", "title", "basics", "createTime"]
    });

    let response = ES_CLIENT
        .search(SearchParts::Index(&[&*INDEX_NAME]))
        .body(search_body)
        .send()
        .await
        .map_err(|e| Error::from(format!("msg={},code=500", e)))?;

    let response_body = response.json::<Value>().await.map_err(|e| Error::from(format!("msg={},code=500", e)))?;
    
    let hits = response_body["hits"]["hits"].as_array().unwrap_or(&vec![]);
    let total = response_body["hits"]["total"]["value"].as_i64().unwrap_or(0);
    
    let mut result_list: Vec<SearchPageVO> = Vec::new();
    
    for hit in hits {
        let source = &hit["_source"];
        let vo = SearchPageVO {
            id: Some(hit["_id"].as_str().unwrap_or("").to_string()),
            info_type: source["infoType"].as_str().map(|s| s.to_string()),
            title: source["title"].as_str().map(|s| s.to_string()),
            basics: source["basics"].as_str().map(|s| s.to_string()),
            create_time: source["createTime"].as_str().map(|s| s.to_string()),
        };
        result_list.push(vo);
    }

    Ok(Page {
        data: result_list,
        current_page: item.page.unwrap_or(1),
        page_size: item.limit.unwrap_or(10),
        total,
    })
}

/// 相关列表
#[cfg(feature = "enable-es")]
pub async fn related_list(item: RelatedQueryRequest)->Result<Vec<SearchPageVO>> {
    let mut query_conditions: Vec<Value> = Vec::new();

    if let Some(title) = &item.title {
        if !title.is_empty() {
            query_conditions.push(json!({
                "multi_match": {
                    "query": title,
                    "fields": ["title^2", "content"]
                }
            }));
        }
    }

    if let Some(info_type) = &item.info_type {
        if !info_type.is_empty() {
            query_conditions.push(json!({
                "term": { "infoType": info_type }
            }));
        }
    }

    let query = if query_conditions.is_empty() {
        json!({ "match_all": {} })
    } else if query_conditions.len() == 1 {
        query_conditions.remove(0)
    } else {
        json!({ "bool": { "must": query_conditions } })
    };

    let search_body = json!({
        "query": query,
        "size": item.limit.unwrap_or(10),
        "sort": [
            { "createTime": { "order": "desc" } }
        ],
        "_source": ["id", "infoType", "title", "basics", "createTime"]
    });

    let response = ES_CLIENT
        .search(SearchParts::Index(&[&*INDEX_NAME]))
        .body(search_body)
        .send()
        .await
        .map_err(|e| Error::from(format!("msg={},code=500", e)))?;

    let response_body = response.json::<Value>().await.map_err(|e| Error::from(format!("msg={},code=500", e)))?;
    
    let hits = response_body["hits"]["hits"].as_array().unwrap_or(&vec![]);
    
    let mut result_list: Vec<SearchPageVO> = Vec::new();
    
    for hit in hits {
        let source = &hit["_source"];
        let vo = SearchPageVO {
            id: Some(hit["_id"].as_str().unwrap_or("").to_string()),
            info_type: source["infoType"].as_str().map(|s| s.to_string()),
            title: source["title"].as_str().map(|s| s.to_string()),
            basics: source["basics"].as_str().map(|s| s.to_string()),
            create_time: source["createTime"].as_str().map(|s| s.to_string()),
        };
        result_list.push(vo);
    }

    Ok(result_list)
}

// 当 Elasticsearch 功能未启用时的空实现
#[cfg(not(feature = "enable-es"))]
pub async fn create_index(_index: IndexSaveRequest) -> Result<bool> {
    Err(Error::from("Elasticsearch 功能未启用"))
}

#[cfg(not(feature = "enable-es"))]
pub async fn batch_index(_index_request: Vec<IndexSaveRequest>) -> Result<bool> {
    Err(Error::from("Elasticsearch 功能未启用"))
}

#[cfg(not(feature = "enable-es"))]
pub async fn index_exists() -> Result<bool> {
    Err(Error::from("Elasticsearch 功能未启用"))
}

#[cfg(not(feature = "enable-es"))]
pub async fn create_index_mappings() -> Result<bool> {
    Err(Error::from("Elasticsearch 功能未启用"))
}

#[cfg(not(feature = "enable-es"))]
pub async fn delete_index() -> Result<bool> {
    Err(Error::from("Elasticsearch 功能未启用"))
}

#[cfg(not(feature = "enable-es"))]
pub async fn delete_doc(_id: String) -> Result<bool> {
    Err(Error::from("Elasticsearch 功能未启用"))
}

#[cfg(not(feature = "enable-es"))]
pub async fn search_list(_item: SearchPageRequest) -> Result<Page<SearchPageVO>> {
    Err(Error::from("Elasticsearch 功能未启用"))
}

#[cfg(not(feature = "enable-es"))]
pub async fn related_list(_item: RelatedQueryRequest) -> Result<Vec<SearchPageVO>> {
    Err(Error::from("Elasticsearch 功能未启用"))
}