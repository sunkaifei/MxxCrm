//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::utils::string_utils::serialize_option_u64_to_string;
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::system::entity::{config, config::Entity as Config};
use crate::utils::string_utils::deserialize_string_to_u64;
use sea_orm::*;
use sea_orm::prelude::DateTime;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ConfigSaveRequest {
    ///参数名称
    pub config_name: Option<String>,
    ///参数键名
    pub config_key: Option<String>,
    ///参数键值
    pub config_value: Option<String>,
    ///系统内置（Y是 N否）
    pub config_type: Option<String>,
    ///备注
    pub remark: Option<String>,
    ///排序
    pub sort: Option<i32>,
}

impl From<ConfigSaveRequest> for ConfigSaveDTO {
    fn from(form_data: ConfigSaveRequest) -> Self {
        Self {
            id: None,
            config_name: form_data.config_name,
            config_key: form_data.config_key,
            config_value: form_data.config_value,
            config_type: form_data.config_type,
            remark: form_data.remark,
            sort: form_data.sort,
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
        }
    }
}


///更新系统配置
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ConfigUpdateRequest {
    ///参数主键
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    ///参数名称
    pub config_name: Option<String>,
    ///参数键名
    pub config_key: Option<String>,
    ///参数键值
    pub config_value: Option<String>,
    ///系统内置（Y是 N否）
    pub config_type: Option<String>,
    ///备注
    pub remark: Option<String>,
    ///排序
    pub sort: Option<i32>,
}

impl From<ConfigUpdateRequest> for ConfigSaveDTO {
    fn from(form_data: ConfigUpdateRequest) -> Self {
        Self {
            id: form_data.id,
            config_name: form_data.config_name,
            config_key: form_data.config_key,
            config_value: form_data.config_value,
            config_type: form_data.config_type,
            remark: form_data.remark,
            sort: form_data.sort,
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigSaveDTO {
    ///参数主键
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    ///参数名称
    pub config_name: Option<String>,
    ///参数键名
    pub config_key: Option<String>,
    ///参数键值
    pub config_value: Option<String>,
    ///系统内置（Y是 N否）
    pub config_type: Option<String>,
    ///备注
    pub remark: Option<String>,
    ///排序
    pub sort: Option<i32>,
    ///创建者
    pub create_by: Option<String>,
    ///创建时间
    pub create_time: Option<DateTime>,
    ///更新者
    pub update_by: Option<String>,
    ///更新时间
    pub update_time: Option<DateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfigDetailVO {
    ///配置id
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    ///参数名称
    pub config_name: Option<String>,
    ///参数键名
    pub config_key: Option<String>,
    ///参数键值
    pub config_value: Option<String>,
    ///系统内置（Y是 N否）
    pub config_type: Option<String>,
    ///备注
    pub remark: Option<String>,
    ///排序
    pub sort: Option<i32>,
}

impl From<config::Model> for ConfigDetailVO {
    fn from(config: config::Model) -> Self {
        Self {
            id: Option::from(config.id),
            config_name: config.config_name,
            config_key: config.config_key,
            config_value: config.config_value,
            config_type: config.config_type,
            remark: config.remark,
            sort: config.sort,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfigListVO {
    ///配置id
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    ///参数名称
    pub config_name: Option<String>,
    ///参数键名
    pub config_key: Option<String>,
    ///参数键值
    pub config_value: Option<String>,
    ///系统内置（Y是 N否）
    pub config_type: Option<String>,
    ///备注
    pub remark: Option<String>,
    ///排序
    pub sort: Option<i32>,
}

impl From<config::Model> for ConfigListVO {
    fn from(config: config::Model) -> Self {
        Self {
            id: Option::from(config.id),
            config_name: config.config_name,
            config_key: config.config_key,
            config_value: config.config_value,
            config_type: config.config_type,
            remark: config.remark,
            sort: config.sort,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub config_name: Option<String>,
    pub config_key: Option<String>,
    pub config_type: Option<String>,
}

/// 条件
#[derive(Clone)]
pub struct PageWhere {
    pub config_name: Option<String>,
    pub config_key: Option<String>,
    pub config_type: Option<String>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut config_name = None;
        if self.config_name != Some("".to_string()) {
            config_name = self.config_name.clone();
        }

        let mut config_key = None;
        if self.config_key != Some("".to_string()) {
            config_key = self.config_key.clone();
        }

        let mut config_type = None;
        if self.config_type != Some("".to_string()) {
            config_type = self.config_type.clone();
        }
        

        Self {
            config_name,
            config_key,
            config_type,
        }
    }
}


pub struct ConfigModel;

impl ConfigModel {
    /// ### 新增系统配置
    /// * `db` - 数据库连接对象
    /// * `form_data` - 系统配置表单数据
    /// 
    /// 返回值：插入成功的id
    pub async fn insert(db: &DbConn, form_data: &ConfigSaveDTO) -> Result<i64, DbErr> {
        let payload = config::ActiveModel {
            config_name:       Set(form_data.config_name.to_owned()),
            config_key:        Set(form_data.config_key.to_owned()),
            config_value:      Set(form_data.config_value.to_owned()),
            config_type:       Set(form_data.config_type.to_owned()),
            remark:            Set(form_data.remark.to_owned()),
            sort:              Set(form_data.sort.to_owned()),
            create_by:         Set(form_data.create_by.to_owned()),
            create_time:       Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            update_by:         Set(form_data.update_by.to_owned()),
            update_time:       Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        Config::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// ### 批量删除系统配置
    /// * `db` - 数据库连接对象
    /// * `ids` - 系统配置id数组
    /// 
    /// 返回值：受影响的行数
    pub async fn batch_delete_by_ids(db: &DbConn, ids: Vec<i64>) -> Result<i64, DbErr> {
        let delete_result: DeleteResult = Config::delete_many()
            .filter(config::Column::Id.is_in(ids))
            .exec(db)
            .await?;

        Ok(delete_result.rows_affected as i64)
    }
    
    /// ### 更新系统配置
    /// * `db` - 数据库连接对象
    /// * `id` - 系统配置id
    /// * `form_data` - 系统配置表单数据
    /// 
    /// 返回值：受影响的行数
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, form_data: &ConfigSaveDTO) -> Result<i64, DbErr> {
        let payload = config::ActiveModel {
            config_name:       Set(form_data.config_name.to_owned()),
            config_key:        Set(form_data.config_key.to_owned()),
            config_value:      Set(form_data.config_value.to_owned()),
            config_type:       Set(form_data.config_type.to_owned()),
            remark:            Set(form_data.remark.to_owned()),
            sort:              Set(form_data.sort.to_owned()),
            update_by:         Set(form_data.update_by.to_owned()),
            update_time:       Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        let update_result: UpdateResult = Config::update_many()
            .set(payload)
            .filter(config::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }
    
    /// ### 查询配置名称是否唯一
    /// * `db` - 数据库连接对象
    /// * `name` - 配置名称
    /// * `id` - 要排除的配置id
    ///
    /// 返回值：true表示不唯一，false表示唯一
    pub async fn find_by_name_unique(db: &DbConn, name: &Option<String>, id: &Option<i64>) -> Result<bool, DbErr> {
        let result_num = Config::find()
            .filter(config::Column::ConfigName.eq(name.clone().unwrap_or_default()))
            .apply_if(id.clone(), |query, v| {
                query.filter(config::Column::Id.ne(v))
            })
            .count(db).await? as i64;
        Ok(result_num > 0)
    }

    /// ### 查询配置键名是否唯一
    /// * `db` - 数据库连接对象
    /// * `name` - 配置键名
    /// * `id` - 要排除的配置id
    /// 
    /// 返回值：true表示不唯一，false表示唯一
    pub async fn find_by_key_unique(db: &DbConn, name: &Option<String>, id: &Option<i64>) -> Result<bool, DbErr> {
        let result_num = Config::find()
            .filter(config::Column::ConfigKey.eq(name.clone().unwrap_or_default()))
            .apply_if(id.clone(), |query, v| {
                query.filter(config::Column::Id.ne(v))
            })
            .count(db).await? as i64;
        Ok(result_num > 0)
    }
    
    /// ### 根据id查询
    /// * `db` - 数据库连接对象
    /// * `id` - 系统配置id
    /// 
    /// 返回值：系统配置信息
    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<config::Model>, DbErr> {
        Config::find_by_id(id.unwrap_or_default())
            .one(db)
            .await
    }
    
    /// ### 根据配置名称查询
    /// * `db` - 数据库连接对象
    /// * `name` - 配置名称
    /// 
    /// 返回值：系统配置信息
    pub async fn find_by_config_name(db: &DbConn, name: &str) -> Result<Option<config::Model>, DbErr> {
        Config::find()
            .filter(config::Column::ConfigName.eq(name))
            .one(db)
            .await
    }
    
    /// ### 根据键名查询
    /// * `db` - 数据库连接对象
    /// * `key` - 键名
    /// 
    /// 返回值：系统配置信息
    pub async fn find_by_key(db: &DbConn, key: &str) -> Result<Option<config::Model>, DbErr> {
        Config::find()
            .filter(config::Column::ConfigKey.eq(key))
            .one(db)
            .await
    }
    
    /// ### 查询系统配置记录条数
    /// * `db` - 数据库连接对象
    /// * `page_num` - 页码
    /// * `page_size` - 每页数量
    /// * `wheres` - 查询条件
    /// 
    /// 返回值：系统配置记录条数
    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        Config::find()
            .apply_if(wheres.config_name, |query, v| {
                query.filter(config::Column::ConfigName.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.config_key, |query, v| {
                query.filter(config::Column::ConfigKey.contains(format!("%{}%", v).as_str()))
            })
            .filter(config::Column::ConfigType.eq("N"))
            .count(db)
            .await
            .map(|c| c as i64)
    }

    /// ### 分页查询系统配置列表
    /// * `db` - 数据库连接对象
    /// * `page` - 页码
    /// * `per_page` - 每页数量
    /// * `wheres` - 查询条件
    /// 
    /// 返回值：系统配置列表
    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        wheres: PageWhere,
    ) -> Result<(Vec<config::Model>, i64), DbErr> {
        let paginator = Config::find()
            .apply_if(wheres.config_name, |query, v| {
                query.filter(config::Column::ConfigName.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.config_key, |query, v| {
                query.filter(config::Column::ConfigKey.contains(format!("%{}%", v).as_str()))
            })
            .filter(config::Column::ConfigType.eq("N"))
            .order_by_asc(config::Column::Id)
            .paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}
