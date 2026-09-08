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

/// 内容模型字段类型枚举（与前端 field-drawer 一致）
/// 1 单行文本 / 2 多行文本 / 3 富文本 / 4 数字 / 5 日期
/// 6 下拉 / 7 单选 / 8 多选 / 9 图片 / 10 文件
pub const FT_SINGLE_TEXT: i32 = 1;
pub const FT_MULTI_TEXT: i32 = 2;
pub const FT_RICH_TEXT: i32 = 3;
pub const FT_NUMBER: i32 = 4;
pub const FT_DATE: i32 = 5;
pub const FT_SELECT: i32 = 6;
pub const FT_RADIO: i32 = 7;
pub const FT_CHECKBOX: i32 = 8;
pub const FT_IMAGE: i32 = 9;
pub const FT_FILE: i32 = 10;

/// 校验 SQL 标识符：首字符字母/下划线，其余字母数字下划线，长度 ≤ 60
pub fn is_valid_identifier(name: &str) -> bool {
    if name.is_empty() || name.len() > 60 {
        return false;
    }
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// 字段类型（int）→ PostgreSQL 列类型 SQL 片段
pub fn field_type_to_col_sql(field_type: i32) -> &'static str {
    match field_type {
        FT_MULTI_TEXT | FT_RICH_TEXT => "TEXT",
        FT_NUMBER => "BIGINT",
        FT_DATE => "TIMESTAMP",
        FT_CHECKBOX => "VARCHAR(1000)",
        FT_IMAGE | FT_FILE => "VARCHAR(1000)",
        FT_SINGLE_TEXT | FT_SELECT => "VARCHAR(500)",
        FT_RADIO => "VARCHAR(255)",
        _ => "VARCHAR(500)",
    }
}

/// 固定列：snake_case 列名 → camelCase 输出键
const FIXED_COLUMNS: &[(&str, &str)] = &[
    ("id", "id"),
    ("title", "title"),
    ("short_url", "shortUrl"),
    ("category_id", "categoryId"),
    ("cover_image", "coverImage"),
    ("author", "author"),
    ("summary", "summary"),
    ("content", "content"),
    ("seo_title", "seoTitle"),
    ("seo_keywords", "seoKeywords"),
    ("seo_description", "seoDescription"),
    ("sort", "sort"),
    ("status", "status"),
    ("create_time", "createTime"),
    ("update_time", "updateTime"),
];

/// 固定列 camelCase 入参键 → snake_case 列名
fn camel_to_snake_key(key: &str) -> String {
    match key {
        "shortUrl" => "short_url".into(),
        "categoryId" => "category_id".into(),
        "coverImage" => "cover_image".into(),
        "seoTitle" => "seo_title".into(),
        "seoKeywords" => "seo_keywords".into(),
        "seoDescription" => "seo_description".into(),
        _ => key.to_string(),
    }
}

/// 写入时应忽略的内部键（新增/更新均不接受）
fn is_internal_key(key: &str) -> bool {
    matches!(
        key,
        "id" | "createTime" | "updateTime" | "deleted" | "create_time" | "update_time"
    )
}

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
    /// * `fields` - 字段定义列表 (field_name, field_type(int 1-10), is_required)
    pub async fn create_table(
        db: &DatabaseConnection,
        model_code: &str,
        fields: &[(String, i32, bool)],
    ) -> Result<()> {
        if !is_valid_identifier(model_code) {
            return Err(Error::from(format!("非法模型编码: {}", model_code)));
        }
        let table_name = Self::get_table_name(model_code);

        let mut cols: Vec<String> = vec![
            "id BIGSERIAL PRIMARY KEY".to_string(),
            "title VARCHAR(255) NOT NULL".to_string(),
            "short_url VARCHAR(160)".to_string(),
            "category_id BIGINT".to_string(),
            "cover_image VARCHAR(500)".to_string(),
            "author VARCHAR(100)".to_string(),
            "summary VARCHAR(500)".to_string(),
            "content TEXT".to_string(),
            "seo_title VARCHAR(255)".to_string(),
            "seo_keywords VARCHAR(255)".to_string(),
            "seo_description VARCHAR(500)".to_string(),
            "sort INTEGER DEFAULT 0".to_string(),
            "status INTEGER DEFAULT 1".to_string(),
            "deleted INTEGER DEFAULT 0".to_string(),
            "create_time TIMESTAMP".to_string(),
            "update_time TIMESTAMP".to_string(),
        ];

        // 自定义字段
        for (field_name, field_type, is_required) in fields {
            if !is_valid_identifier(field_name) {
                log::warn!("[dynamic_table] 跳过非法字段名: {}", field_name);
                continue;
            }
            let col_sql = field_type_to_col_sql(*field_type);
            let not_null = if *is_required { " NOT NULL" } else { "" };
            cols.push(format!("\"{}\" {}{}", field_name, col_sql, not_null));
        }

        let sql = format!(
            "CREATE TABLE IF NOT EXISTS \"{}\" ({})",
            table_name,
            cols.join(", ")
        );
        db.execute_unprepared(&sql)
            .await
            .map_err(|e| Error::from(format!("创建动态表失败: {:?}", e)))?;

        Ok(())
    }

    /// 为动态表新增列（幂等：ADD COLUMN IF NOT EXISTS）
    /// 用于「给模型加字段」时同步物理表结构（T-P0.2）
    pub async fn add_column_if_not_exists(
        db: &DatabaseConnection,
        model_code: &str,
        field_name: &str,
        field_type: i32,
    ) -> Result<()> {
        if !is_valid_identifier(model_code) || !is_valid_identifier(field_name) {
            return Err(Error::from(format!(
                "非法标识符: model_code={}, field_name={}",
                model_code, field_name
            )));
        }
        // 表不存在则跳过（避免半成品报错打断字段新增）
        if !Self::table_exists(db, model_code).await.unwrap_or(false) {
            log::warn!("[dynamic_table] 动态表 mxx_model_{} 不存在，跳过加列", model_code);
            return Ok(());
        }
        let col_sql = field_type_to_col_sql(field_type);
        let sql = format!(
            "ALTER TABLE \"mxx_model_{}\" ADD COLUMN IF NOT EXISTS \"{}\" {}",
            model_code, field_name, col_sql
        );
        db.execute_unprepared(&sql)
            .await
            .map_err(|e| Error::from(format!("动态表加列失败: {:?}", e)))?;
        Ok(())
    }

    /// 删除动态表（DROP TABLE）
    pub async fn drop_table(db: &DatabaseConnection, model_code: &str) -> Result<()> {
        if !is_valid_identifier(model_code) {
            return Err(Error::from(format!("非法模型编码: {}", model_code)));
        }
        let table_name = Self::get_table_name(model_code);
        let sql = format!("DROP TABLE IF EXISTS \"{}\"", table_name);
        db.execute_unprepared(&sql)
            .await
            .map_err(|e| Error::from(format!("删除动态表失败: {:?}", e)))?;
        Ok(())
    }

    /// 确保动态表存在（自愈）：不存在则依据模型字段定义自动建表。
    ///
    /// 存量模型（本模块上线前 seed 的 article/product/download 等）没有物理表，
    /// 首次访问其内容时会经由此方法补建，保证「通用内容页」对所有模型（含未来新增）开箱即用。
    pub async fn ensure_table(db: &DatabaseConnection, model_code: &str) -> Result<()> {
        if !is_valid_identifier(model_code) {
            return Err(Error::from(format!("非法模型编码: {}", model_code)));
        }
        if Self::table_exists(db, model_code).await.unwrap_or(false) {
            return Ok(());
        }

        // 依据模型编码查模型 id
        let model_sql =
            "SELECT id FROM mxx_content_model WHERE model_code = $1 AND deleted = 0 LIMIT 1";
        let stmt = sea_orm::Statement::from_sql_and_values(
            db.get_database_backend(),
            model_sql,
            [model_code.into()],
        );
        let model_row = db.query_one_raw(stmt).await?;
        let model_id: i64 = match model_row {
            Some(r) => r.try_get("", "id")?,
            None => {
                return Err(Error::from(format!("内容模型不存在: {}", model_code)));
            }
        };

        // 载入字段定义
        let field_sql = "SELECT field_name, field_type, is_required FROM mxx_content_model_field \
                         WHERE model_id = $1 AND deleted = 0 ORDER BY sort ASC, id ASC";
        let stmt = sea_orm::Statement::from_sql_and_values(
            db.get_database_backend(),
            field_sql,
            [model_id.into()],
        );
        let rows = db.query_all_raw(stmt).await?;
        let mut fields: Vec<(String, i32, bool)> = Vec::new();
        for r in rows {
            let name: String = r.try_get("", "field_name")?;
            let ftype: i32 = r.try_get("", "field_type").unwrap_or(FT_SINGLE_TEXT);
            let req: i32 = r.try_get("", "is_required").unwrap_or(0);
            fields.push((name, ftype, req == 1));
        }

        Self::create_table(db, model_code, &fields).await?;
        log::info!(
            "[dynamic_table] 已为模型 {} 自动建表 mxx_model_{}（字段数 {}）",
            model_code,
            model_code,
            fields.len()
        );
        Ok(())
    }

    /// 检查表是否存在
    pub async fn table_exists(db: &DatabaseConnection, model_code: &str) -> Result<bool> {
        let table_name = Self::get_table_name(model_code);
        let sql = format!(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = '{}') AS exists",
            table_name
        );
        let stmt = sea_orm::Statement::from_sql_and_values(db.get_database_backend(), &sql, []);
        let result = db.query_one_raw(stmt).await?;
        if let Some(row) = result {
            let exists: bool = row.try_get("", "exists").unwrap_or(false);
            Ok(exists)
        } else {
            Ok(false)
        }
    }

    /// 插入记录
    pub async fn insert(
        db: &DatabaseConnection,
        model_code: &str,
        data: &serde_json::Value,
    ) -> Result<i64> {
        if !is_valid_identifier(model_code) {
            return Err(Error::from(format!("非法模型编码: {}", model_code)));
        }
        let table_name = Self::get_table_name(model_code);
        let obj = data.as_object().ok_or_else(|| Error::from("数据必须是JSON对象"))?;

        let mut columns: Vec<String> = Vec::new();
        let mut placeholders: Vec<String> = Vec::new();
        let mut values: Vec<Value> = Vec::new();

        for (key, val) in obj {
            if is_internal_key(key) || key == "delete_by" || key == "delete_time" {
                continue;
            }
            let col = camel_to_snake_key(key);
            if !is_valid_identifier(&col) {
                return Err(Error::from(format!("非法字段名: {}", key)));
            }
            columns.push(format!("\"{}\"", col));
            placeholders.push(format!("${}", values.len() + 1));
            values.push(Self::json_to_sea_value(val));
        }

        // 时间戳
        columns.push("create_time".to_string());
        placeholders.push("CURRENT_TIMESTAMP".to_string());
        columns.push("update_time".to_string());
        placeholders.push("CURRENT_TIMESTAMP".to_string());

        let col_list = columns.join(", ");
        let val_list = placeholders.join(", ");
        let sql = format!(
            "INSERT INTO \"{}\" ({}) VALUES ({}) RETURNING id",
            table_name, col_list, val_list
        );

        let stmt =
            sea_orm::Statement::from_sql_and_values(db.get_database_backend(), &sql, values);
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
        if !is_valid_identifier(model_code) {
            return Err(Error::from(format!("非法模型编码: {}", model_code)));
        }
        let table_name = Self::get_table_name(model_code);
        let sql = format!(
            "SELECT row_to_json(t)::text AS data FROM (SELECT * FROM \"{}\" WHERE id = $1 AND deleted = 0) t",
            table_name
        );
        let stmt =
            sea_orm::Statement::from_sql_and_values(db.get_database_backend(), &sql, [id.into()]);
        let result = db.query_one_raw(stmt).await?;
        match result {
            Some(row) => {
                let txt: String = row.try_get("", "data")?;
                let v: serde_json::Value =
                    serde_json::from_str(&txt).unwrap_or(serde_json::Value::Null);
                Ok(Some(Self::normalize_row(v)))
            }
            None => Ok(None),
        }
    }

    /// 根据 short_url 查询记录
    pub async fn find_by_short_url(
        db: &DatabaseConnection,
        model_code: &str,
        short_url: &str,
    ) -> Result<Option<serde_json::Value>> {
        if !is_valid_identifier(model_code) {
            return Err(Error::from(format!("非法模型编码: {}", model_code)));
        }
        let table_name = Self::get_table_name(model_code);
        let sql = format!(
            "SELECT row_to_json(t)::text AS data FROM (SELECT * FROM \"{}\" WHERE short_url = $1 AND deleted = 0) t",
            table_name
        );
        let stmt = sea_orm::Statement::from_sql_and_values(
            db.get_database_backend(),
            &sql,
            [short_url.into()],
        );
        let result = db.query_one_raw(stmt).await?;
        match result {
            Some(row) => {
                let txt: String = row.try_get("", "data")?;
                let v: serde_json::Value =
                    serde_json::from_str(&txt).unwrap_or(serde_json::Value::Null);
                Ok(Some(Self::normalize_row(v)))
            }
            None => Ok(None),
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
        if !is_valid_identifier(model_code) {
            return Err(Error::from(format!("非法模型编码: {}", model_code)));
        }
        let table_name = Self::get_table_name(model_code);
        let page_num = page_num.max(1);
        let page_size = page_size.clamp(1, 200);

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

        let count_sql = format!(
            "SELECT COUNT(*) as total FROM \"{}\" WHERE {}",
            table_name, where_clause
        );
        let count_stmt = sea_orm::Statement::from_sql_and_values(
            db.get_database_backend(),
            &count_sql,
            values.clone(),
        );
        let count_result = db.query_one_raw(count_stmt).await?;
        let total: i64 = count_result
            .and_then(|row| row.try_get::<i64>("", "total").ok())
            .unwrap_or(0);

        let offset = (page_num - 1) * page_size;
        let list_sql = format!(
            "SELECT row_to_json(t)::text AS data FROM (SELECT * FROM \"{}\" WHERE {} ORDER BY sort ASC, id DESC LIMIT ${} OFFSET ${}) t",
            table_name, where_clause, param_idx, param_idx + 1
        );
        values.push((page_size as i64).into());
        values.push((offset as i64).into());

        let list_stmt =
            sea_orm::Statement::from_sql_and_values(db.get_database_backend(), &list_sql, values);
        let rows = db.query_all_raw(list_stmt).await?;

        let mut list = Vec::new();
        for row in rows {
            let txt: String = row.try_get("", "data")?;
            let v: serde_json::Value =
                serde_json::from_str(&txt).unwrap_or(serde_json::Value::Null);
            list.push(Self::normalize_row(v));
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
        if !is_valid_identifier(model_code) {
            return Err(Error::from(format!("非法模型编码: {}", model_code)));
        }
        let table_name = Self::get_table_name(model_code);
        let obj = data.as_object().ok_or_else(|| Error::from("数据必须是JSON对象"))?;

        let mut set_clauses: Vec<String> = Vec::new();
        let mut values: Vec<Value> = Vec::new();

        for (key, val) in obj {
            if is_internal_key(key) || key == "delete_by" || key == "delete_time" {
                continue;
            }
            let col = camel_to_snake_key(key);
            if !is_valid_identifier(&col) {
                return Err(Error::from(format!("非法字段名: {}", key)));
            }
            set_clauses.push(format!("\"{}\" = ${}", col, values.len() + 1));
            values.push(Self::json_to_sea_value(val));
        }
        set_clauses.push("update_time = CURRENT_TIMESTAMP".to_string());

        let set_clause = set_clauses.join(", ");
        values.push(id.into());

        let sql = format!(
            "UPDATE \"{}\" SET {} WHERE id = ${} AND deleted = 0",
            table_name,
            set_clause,
            values.len()
        );

        let stmt =
            sea_orm::Statement::from_sql_and_values(db.get_database_backend(), &sql, values);
        let result = db.execute_raw(stmt).await?;
        Ok(result.rows_affected() as i64)
    }

    /// 软删除记录
    pub async fn soft_delete(
        db: &DatabaseConnection,
        model_code: &str,
        id: i64,
    ) -> Result<i64> {
        if !is_valid_identifier(model_code) {
            return Err(Error::from(format!("非法模型编码: {}", model_code)));
        }
        let table_name = Self::get_table_name(model_code);
        let sql = format!(
            "UPDATE \"{}\" SET deleted = 1, update_time = CURRENT_TIMESTAMP WHERE id = $1",
            table_name
        );
        let stmt =
            sea_orm::Statement::from_sql_and_values(db.get_database_backend(), &sql, [id.into()]);
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

    /// 把动态表原始行（snake_case）规范化为对外 JSON：
    /// 固定列 → camelCase；自定义列 → 原样保留；剔除 deleted
    fn normalize_row(raw: serde_json::Value) -> serde_json::Value {
        let obj = match raw.as_object() {
            Some(o) => o,
            None => return raw,
        };
        let fixed_snake: std::collections::HashSet<&str> =
            FIXED_COLUMNS.iter().map(|(k, _)| *k).collect();

        let mut out = serde_json::Map::new();
        for (snake, camel) in FIXED_COLUMNS {
            if let Some(v) = obj.get(*snake) {
                out.insert((*camel).to_string(), v.clone());
            }
        }
        for (k, v) in obj {
            if fixed_snake.contains(k.as_str()) || k == "deleted" {
                continue;
            }
            out.insert(k.clone(), v.clone());
        }
        serde_json::Value::Object(out)
    }
}
