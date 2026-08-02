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
use sea_orm::{ConnectionTrait, DatabaseConnection, Value};
use sea_query::{Alias, ColumnDef, TableCreateStatement};

/// 动态表服务
/// 封装内容模型动态表的创建、CRUD操作
pub struct DynamicTableService;

impl DynamicTableService {
    /// 获取动态表名
    pub fn get_table_name(model_code: &str) -> String {
        format!("mxx_model_{}", model_code)
    }

    /// 创建动态表
    /// * `db` - 数据库连接
    /// * `model_code` - 模型编码（如 article, product, download）
    /// * `fields` - 字段定义列表（来自 mxx_content_model_field）
    pub async fn create_table(
        db: &DatabaseConnection,
        model_code: &str,
        fields: &[(String, String, bool)], // (field_name, field_type, is_required)
    ) -> Result<()> {
        let table_name = Self::get_table_name(model_code);

        let mut table_create = TableCreateStatement::new();
        table_create.table(Alias::new(&table_name)).if_not_exists();

        // 固定字段
        table_create.col(ColumnDef::new(Alias::new("id")).big_integer().not_null().primary_key().auto_increment());
        table_create.col(ColumnDef::new(Alias::new("title")).string_len(255).not_null());
        table_create.col(ColumnDef::new(Alias::new("short_url")).string_len(160));
        table_create.col(ColumnDef::new(Alias::new("category_id")).big_integer());
        table_create.col(ColumnDef::new(Alias::new("cover_image")).string_len(500));
        table_create.col(ColumnDef::new(Alias::new("author")).string_len(100));
        table_create.col(ColumnDef::new(Alias::new("summary")).string_len(500));
        table_create.col(ColumnDef::new(Alias::new("content")).text());
        table_create.col(ColumnDef::new(Alias::new("seo_title")).string_len(255));
        table_create.col(ColumnDef::new(Alias::new("seo_keywords")).string_len(255));
        table_create.col(ColumnDef::new(Alias::new("seo_description")).string_len(500));
        table_create.col(ColumnDef::new(Alias::new("sort")).integer().default(0));
        table_create.col(ColumnDef::new(Alias::new("status")).integer().default(1));
        table_create.col(ColumnDef::new(Alias::new("deleted")).integer().default(0));
        table_create.col(ColumnDef::new(Alias::new("create_time")).date_time());
        table_create.col(ColumnDef::new(Alias::new("update_time")).date_time());

        // 自定义字段
        for (field_name, field_type, is_required) in fields {
            let mut col = ColumnDef::new(Alias::new(field_name));
            match field_type.as_str() {
                "text" | "richtext" => {
                    col.text();
                }
                "number" => {
                    col.big_integer();
                }
                "date" => {
                    col.date_time();
                }
                "image" | "file" => {
                    col.string_len(500);
                }
                _ => {
                    col.string_len(500); // 默认字符串
                }
            }
            if *is_required {
                col.not_null();
            }
            table_create.col(col);
        }

        db.execute(&table_create)
            .await
            .map_err(|e| Error::from(format!("创建动态表失败: {:?}", e)))?;

        Ok(())
    }

    /// 删除动态表（DROP TABLE）
    /// * `db` - 数据库连接
    /// * `model_code` - 模型编码
    pub async fn drop_table(db: &DatabaseConnection, model_code: &str) -> Result<()> {
        let table_name = Self::get_table_name(model_code);
        let sql = format!("DROP TABLE IF EXISTS \"{}\"", table_name);
        db.execute_unprepared(&sql)
            .await
            .map_err(|e| Error::from(format!("删除动态表失败: {:?}", e)))?;
        Ok(())
    }

    /// 检查表是否存在
    pub async fn table_exists(db: &DatabaseConnection, model_code: &str) -> Result<bool> {
        let table_name = Self::get_table_name(model_code);
        let sql = format!(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = '{}')",
            table_name
        );
        let stmt = sea_orm::Statement::from_sql_and_values(
            db.get_database_backend(),
            &sql,
            [],
        );
        let result = db.query_one_raw(stmt).await?;

        if let Some(row) = result {
            let exists: bool = row.try_get("", "exists")?;
            Ok(exists)
        } else {
            Ok(false)
        }
    }

    /// 插入记录
    /// * `db` - 数据库连接
    /// * `model_code` - 模型编码
    /// * `data` - 键值对数据
    pub async fn insert(
        db: &DatabaseConnection,
        model_code: &str,
        data: &serde_json::Value,
    ) -> Result<i64> {
        let table_name = Self::get_table_name(model_code);

        let obj = data.as_object().ok_or_else(|| Error::from("数据必须是JSON对象"))?;

        let mut columns: Vec<String> = Vec::new();
        let mut placeholders: Vec<String> = Vec::new();
        let mut values: Vec<Value> = Vec::new();

        // 固定字段
        columns.push("create_time".to_string());
        placeholders.push("CURRENT_TIMESTAMP".to_string());

        for (key, val) in obj {
            // 安全检查：字段名只允许字母数字下划线
            if !key.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(Error::from(format!("非法字段名: {}", key)));
            }
            columns.push(key.clone());
            placeholders.push(format!("${}", values.len() + 1));
            values.push(Self::json_to_sea_value(val));
        }

        let col_list = columns.join(", ");
        let val_list = placeholders.join(", ");
        let sql = format!(
            "INSERT INTO \"{}\" ({}) VALUES ({}) RETURNING id",
            table_name, col_list, val_list
        );

        let stmt = sea_orm::Statement::from_sql_and_values(
            db.get_database_backend(),
            &sql,
            values,
        );

        let result = db.query_one_raw(stmt).await?;
        if let Some(row) = result {
            let id: i64 = row.try_get("", "id")?;
            Ok(id)
        } else {
            Err(Error::from("插入失败"))
        }
    }

    /// 根据ID查询记录
    pub async fn find_by_id(
        db: &DatabaseConnection,
        model_code: &str,
        id: i64,
    ) -> Result<Option<serde_json::Value>> {
        let table_name = Self::get_table_name(model_code);
        let sql = format!(
            "SELECT * FROM \"{}\" WHERE id = $1 AND deleted = 0",
            table_name
        );
        let stmt = sea_orm::Statement::from_sql_and_values(
            db.get_database_backend(),
            &sql,
            [id.into()],
        );
        let result = db.query_one_raw(stmt).await?;

        if let Some(row) = result {
            Ok(Some(Self::row_to_json(&row)?))
        } else {
            Ok(None)
        }
    }

    /// 根据 short_url 查询记录
    pub async fn find_by_short_url(
        db: &DatabaseConnection,
        model_code: &str,
        short_url: &str,
    ) -> Result<Option<serde_json::Value>> {
        let table_name = Self::get_table_name(model_code);
        let sql = format!(
            "SELECT * FROM \"{}\" WHERE short_url = $1 AND deleted = 0",
            table_name
        );
        let stmt = sea_orm::Statement::from_sql_and_values(
            db.get_database_backend(),
            &sql,
            [short_url.into()],
        );
        let result = db.query_one_raw(stmt).await?;

        if let Some(row) = result {
            Ok(Some(Self::row_to_json(&row)?))
        } else {
            Ok(None)
        }
    }

    /// 分页查询
    pub async fn paginate(
        db: &DatabaseConnection,
        model_code: &str,
        page_num: u64,
        page_size: u64,
        category_id: Option<i64>,
        keywords: Option<&str>,
    ) -> Result<(Vec<serde_json::Value>, u64)> {
        let table_name = Self::get_table_name(model_code);

        // 构建WHERE条件
        let mut conditions: Vec<String> = vec!["deleted = 0".to_string()];
        let mut values: Vec<Value> = Vec::new();
        let mut param_idx = 1;

        if let Some(cat_id) = category_id {
            conditions.push(format!("category_id = ${}", param_idx));
            values.push(cat_id.into());
            param_idx += 1;
        }

        if let Some(kw) = keywords {
            if !kw.is_empty() {
                conditions.push(format!("title LIKE ${}", param_idx));
                values.push(format!("%{}%", kw).into());
                param_idx += 1;
            }
        }

        let where_clause = conditions.join(" AND ");

        // 查询总数
        let count_sql = format!("SELECT COUNT(*) as total FROM \"{}\" WHERE {}", table_name, where_clause);
        let count_stmt = sea_orm::Statement::from_sql_and_values(
            db.get_database_backend(),
            &count_sql,
            values.clone(),
        );
        let count_result = db.query_one_raw(count_stmt).await?;
        let total: i64 = if let Some(row) = count_result {
            row.try_get::<i64>("", "total").unwrap_or(0)
        } else {
            0
        };

        // 查询列表
        let offset = (page_num - 1) * page_size;
        let list_sql = format!(
            "SELECT * FROM \"{}\" WHERE {} ORDER BY sort ASC, create_time DESC LIMIT ${} OFFSET ${}",
            table_name, where_clause, param_idx, param_idx + 1
        );
        values.push((page_size as i64).into());
        values.push((offset as i64).into());

        let list_stmt = sea_orm::Statement::from_sql_and_values(
            db.get_database_backend(),
            &list_sql,
            values,
        );
        let rows = db.query_all_raw(list_stmt).await?;

        let mut list = Vec::new();
        for row in rows {
            list.push(Self::row_to_json(&row)?);
        }

        Ok((list, total as u64))
    }

    /// 更新记录
    pub async fn update(
        db: &DatabaseConnection,
        model_code: &str,
        id: i64,
        data: &serde_json::Value,
    ) -> Result<i64> {
        let table_name = Self::get_table_name(model_code);
        let obj = data.as_object().ok_or_else(|| Error::from("数据必须是JSON对象"))?;

        let mut set_clauses: Vec<String> = Vec::new();
        let mut values: Vec<Value> = Vec::new();

        set_clauses.push("update_time = CURRENT_TIMESTAMP".to_string());

        for (key, val) in obj {
            if !key.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(Error::from(format!("非法字段名: {}", key)));
            }
            set_clauses.push(format!("{} = ${}", key, values.len() + 1));
            values.push(Self::json_to_sea_value(val));
        }

        let set_clause = set_clauses.join(", ");
        values.push(id.into());

        let sql = format!(
            "UPDATE \"{}\" SET {} WHERE id = ${} AND deleted = 0",
            table_name, set_clause, values.len()
        );

        let stmt = sea_orm::Statement::from_sql_and_values(
            db.get_database_backend(),
            &sql,
            values,
        );

        let result = db.execute_raw(stmt).await?;
        Ok(result.rows_affected() as i64)
    }

    /// 软删除记录
    pub async fn soft_delete(
        db: &DatabaseConnection,
        model_code: &str,
        id: i64,
    ) -> Result<i64> {
        let table_name = Self::get_table_name(model_code);
        let sql = format!(
            "UPDATE \"{}\" SET deleted = 1, update_time = CURRENT_TIMESTAMP WHERE id = $1",
            table_name
        );
        let stmt = sea_orm::Statement::from_sql_and_values(
            db.get_database_backend(),
            &sql,
            [id.into()],
        );
        let result = db.execute_raw(stmt).await?;
        Ok(result.rows_affected() as i64)
    }

    /// JSON值转换为SeaORM Value
    fn json_to_sea_value(val: &serde_json::Value) -> Value {
        match val {
            serde_json::Value::Null => Value::String(None),
            serde_json::Value::Bool(b) => Value::Bool(Some(*b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::BigInt(Some(i))
                } else if let Some(f) = n.as_f64() {
                    Value::Double(Some(f))
                } else {
                    Value::String(None)
                }
            }
            serde_json::Value::String(s) => Value::String(Some(s.clone())),
            _ => Value::String(Some(val.to_string())),
        }
    }

    /// 查询结果行转换为JSON
    fn row_to_json(row: &sea_orm::QueryResult) -> Result<serde_json::Value> {
        let json = serde_json::json!({
            "id": row.try_get::<i64>("", "id").unwrap_or(0),
            "title": row.try_get::<String>("", "title").unwrap_or_default(),
            "shortUrl": row.try_get::<String>("", "short_url").unwrap_or_default(),
            "categoryId": row.try_get::<i64>("", "category_id").unwrap_or(0),
            "coverImage": row.try_get::<String>("", "cover_image").unwrap_or_default(),
            "author": row.try_get::<String>("", "author").unwrap_or_default(),
            "summary": row.try_get::<String>("", "summary").unwrap_or_default(),
            "content": row.try_get::<String>("", "content").unwrap_or_default(),
            "seoTitle": row.try_get::<String>("", "seo_title").unwrap_or_default(),
            "seoKeywords": row.try_get::<String>("", "seo_keywords").unwrap_or_default(),
            "seoDescription": row.try_get::<String>("", "seo_description").unwrap_or_default(),
            "sort": row.try_get::<i32>("", "sort").unwrap_or(0),
            "status": row.try_get::<i32>("", "status").unwrap_or(0),
            "createTime": row.try_get::<chrono::NaiveDateTime>("", "create_time")
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default(),
        });
        Ok(json)
    }
}
