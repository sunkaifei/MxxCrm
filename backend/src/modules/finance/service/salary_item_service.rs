//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 工资项目自定义引擎服务
//! 支持固定值、公式计算、手动输入三种模式
//! 公式语法：{variable} 变量引用，支持 + - * / 四则运算与括号
//!

use sea_orm::*;
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::collections::HashMap;

use crate::modules::finance::entity::{salary_item, salary_item_value};

// ==================== DTO ====================

/// 工资项目新增/更新 DTO
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalaryItemUpsertDTO {
    pub id: Option<i64>,
    pub item_code: String,
    pub item_name: String,
    pub item_type: Option<i32>,
    pub calc_mode: Option<i32>,
    pub formula: Option<String>,
    pub default_value: Option<f64>,
    pub is_taxable: Option<i32>,
    pub is_pretax: Option<i32>,
    pub sort: Option<i32>,
    pub enabled: Option<i32>,
}

/// 单个工资项值
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalaryItemValueDTO {
    pub item_id: i64,
    pub amount: f64,
}

/// 保存工资项值请求
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveItemValuesDTO {
    pub salary_record_id: i64,
    pub values: Vec<SalaryItemValueDTO>,
}

// ==================== CRUD ====================

/// 查询所有启用的工资项目（按 sort 排序）
pub async fn get_item_list(db: &DatabaseConnection) -> Result<Vec<salary_item::Model>, String> {
    salary_item::Entity::find()
        .filter(salary_item::Column::Enabled.eq(1))
        .order_by_asc(salary_item::Column::Sort)
        .all(db)
        .await
        .map_err(|e| e.to_string())
}

/// 新增/更新工资项目
pub async fn upsert_item(db: &DatabaseConnection, dto: SalaryItemUpsertDTO) -> Result<i64, String> {
    let now = Utc::now().naive_utc();
    let default_value = Decimal::from_f64(dto.default_value.unwrap_or_default()).unwrap_or_default();
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    let id = if let Some(id) = dto.id {
        let existing = salary_item::Entity::find_by_id(id)
            .one(&txn)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "工资项目不存在".to_string())?;
        let mut active: salary_item::ActiveModel = existing.into();
        active.item_code = Set(dto.item_code);
        active.item_name = Set(dto.item_name);
        active.item_type = Set(dto.item_type);
        active.calc_mode = Set(dto.calc_mode);
        active.formula = Set(dto.formula);
        active.default_value = Set(default_value);
        active.is_taxable = Set(dto.is_taxable);
        active.is_pretax = Set(dto.is_pretax);
        active.sort = Set(dto.sort);
        active.enabled = Set(dto.enabled);
        let updated = active.update(&txn).await.map_err(|e| e.to_string())?;
        updated.id
    } else {
        let active = salary_item::ActiveModel {
            item_code: Set(dto.item_code),
            item_name: Set(dto.item_name),
            item_type: Set(dto.item_type),
            calc_mode: Set(dto.calc_mode),
            formula: Set(dto.formula),
            default_value: Set(default_value),
            is_taxable: Set(dto.is_taxable),
            is_pretax: Set(dto.is_pretax),
            sort: Set(dto.sort),
            enabled: Set(dto.enabled),
            create_time: Set(Some(now)),
            ..Default::default()
        };
        let inserted = active.insert(&txn).await.map_err(|e| e.to_string())?;
        inserted.id
    };

    txn.commit().await.map_err(|e| e.to_string())?;
    Ok(id)
}

/// 删除工资项目
pub async fn delete_item(db: &DatabaseConnection, id: i64) -> Result<(), String> {
    let txn = db.begin().await.map_err(|e| e.to_string())?;
    salary_item::Entity::delete_by_id(id)
        .exec(&txn)
        .await
        .map_err(|e| e.to_string())?;
    txn.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 查询某工资记录的自定义项值
pub async fn get_item_values(
    db: &DatabaseConnection,
    salary_record_id: i64,
) -> Result<Vec<salary_item_value::Model>, String> {
    salary_item_value::Entity::find()
        .filter(salary_item_value::Column::SalaryRecordId.eq(salary_record_id))
        .all(db)
        .await
        .map_err(|e| e.to_string())
}

/// 保存自定义项值
/// 采用"先删后插"策略，保证原子性
pub async fn save_item_values(
    db: &DatabaseConnection,
    salary_record_id: i64,
    values: Vec<SalaryItemValueDTO>,
) -> Result<(), String> {
    let now = Utc::now().naive_utc();
    let txn = db.begin().await.map_err(|e| e.to_string())?;

    // 1. 删除该工资记录的所有自定义项值
    salary_item_value::Entity::delete_many()
        .filter(salary_item_value::Column::SalaryRecordId.eq(salary_record_id))
        .exec(&txn)
        .await
        .map_err(|e| e.to_string())?;

    // 2. 收集所有 item_id 用于查询项目元数据
    let item_ids: Vec<i64> = values.iter().map(|v| v.item_id).collect();
    let mut item_map: HashMap<i64, salary_item::Model> = HashMap::new();
    if !item_ids.is_empty() {
        let items = salary_item::Entity::find()
            .filter(salary_item::Column::Id.is_in(item_ids))
            .all(&txn)
            .await
            .map_err(|e| e.to_string())?;
        for it in items {
            item_map.insert(it.id, it);
        }
    }

    // 3. 逐条插入
    for v in values {
        let item = item_map.get(&v.item_id);
        let item_code = item.map(|i| i.item_code.clone());
        let item_name = item.map(|i| i.item_name.clone());
        let is_taxable = item.and_then(|i| i.is_taxable);
        let amount = Decimal::from_f64(v.amount).unwrap_or_default();

        let active = salary_item_value::ActiveModel {
            salary_record_id: Set(salary_record_id),
            item_id: Set(v.item_id),
            item_code: Set(item_code),
            item_name: Set(item_name),
            amount: Set(amount),
            is_taxable: Set(is_taxable),
            ..Default::default()
        };
        let _ = now; // 暂未设置创建时间字段（表无该字段）
        active.insert(&txn).await.map_err(|e| e.to_string())?;
    }

    txn.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

// ==================== 公式计算 ====================

/// 简单公式计算
///
/// - `formula`：包含 {variable} 变量引用与 + - * / 四则运算及括号的表达式
/// - `context`：变量名 → 数值 的映射（如 baseSalary, commission, bonus, deduction）
/// - 计算失败或公式为空时返回 `default_value`
pub fn calculate_formula(
    formula: &str,
    context: &HashMap<String, Decimal>,
    default_value: Decimal,
) -> Decimal {
    let formula = formula.trim();
    if formula.is_empty() {
        return default_value;
    }

    // 1. 替换 {variable} 为对应的数值字符串
    let mut expr = String::with_capacity(formula.len());
    let mut i = 0;
    let bytes = formula.as_bytes();
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == '{' {
            // 查找闭合 }
            if let Some(end) = formula[i + 1..].find('}') {
                let var_name = &formula[i + 1..i + 1 + end];
                match context.get(var_name.trim()) {
                    Some(v) => {
                        expr.push_str(&v.to_string());
                    }
                    None => {
                        // 未知变量 → 用 0 替换，避免解析失败
                        expr.push_str("0");
                    }
                }
                i = i + 1 + end + 1;
                continue;
            } else {
                // 没有闭合 }，公式非法
                return default_value;
            }
        }
        expr.push(c);
        i += 1;
    }

    // 2. 解析并计算
    // P2-5: 使用 parse_value 入口，支持 IF/CASE/比较运算
    let mut parser = ExprParser::new(&expr);
    match parser.parse_value() {
        Ok(value) => {
            // 解析完成后应该没有剩余 token
            if parser.peek().is_some() {
                default_value
            } else {
                value
            }
        }
        Err(_) => default_value,
    }
}

// ==================== P2-5: 表达式解析器（支持 IF/CASE/比较运算）====================

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(Decimal),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Comma,
    Gt,
    Lt,
    Ge,
    Le,
    Eq,
    Ne,
    /// 标识符：IF, CASE, WHEN, THEN, ELSE, END, AND, OR 等
    Ident(String),
}

struct ExprParser {
    tokens: Vec<Token>,
    pos: usize,
}

impl ExprParser {
    fn new(source: &str) -> Self {
        let tokens = tokenize(source);
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    /// value := if_expr | case_expr | comparison
    /// 顶层入口：优先识别 IF/CASE 关键字，否则按比较/算术表达式处理
    fn parse_value(&mut self) -> Result<Decimal, String> {
        match self.peek() {
            Some(Token::Ident(s)) if s.eq_ignore_ascii_case("IF") => self.parse_if(),
            Some(Token::Ident(s)) if s.eq_ignore_ascii_case("CASE") => self.parse_case(),
            _ => self.parse_comparison(),
        }
    }

    /// IF '(' cond ',' true_expr ',' false_expr ')'
    fn parse_if(&mut self) -> Result<Decimal, String> {
        // 消费 IF
        self.next();
        // 期望 (
        match self.next() {
            Some(Token::LParen) => {}
            _ => return Err("IF 后缺少左括号".to_string()),
        }
        // 条件
        let cond = self.parse_condition()?;
        // 期望 ,
        match self.next() {
            Some(Token::Comma) => {}
            _ => return Err("IF 条件后缺少逗号".to_string()),
        }
        // 真值表达式
        let true_val = self.parse_value()?;
        // 期望 ,
        match self.next() {
            Some(Token::Comma) => {}
            _ => return Err("IF 真值后缺少逗号".to_string()),
        }
        // 假值表达式
        let false_val = self.parse_value()?;
        // 期望 )
        match self.next() {
            Some(Token::RParen) => {}
            _ => return Err("IF 缺少右括号".to_string()),
        }
        Ok(if cond { true_val } else { false_val })
    }

    /// CASE WHEN cond THEN value [WHEN cond THEN value]* [ELSE value] END
    fn parse_case(&mut self) -> Result<Decimal, String> {
        // 消费 CASE
        self.next();
        loop {
            // 期望 WHEN
            match self.peek() {
                Some(Token::Ident(s)) if s.eq_ignore_ascii_case("WHEN") => {
                    self.next();
                }
                Some(Token::Ident(s)) if s.eq_ignore_ascii_case("ELSE") => {
                    self.next();
                    let else_val = self.parse_value()?;
                    // 期望 END
                    match self.peek() {
                        Some(Token::Ident(s)) if s.eq_ignore_ascii_case("END") => {
                            self.next();
                        }
                        _ => return Err("CASE 缺少 END".to_string()),
                    }
                    return Ok(else_val);
                }
                Some(Token::Ident(s)) if s.eq_ignore_ascii_case("END") => {
                    self.next();
                    // 无 ELSE 且无 WHEN 匹配，返回 0
                    return Ok(Decimal::ZERO);
                }
                _ => return Err("CASE 缺少 WHEN/ELSE/END".to_string()),
            }
            // 条件
            let cond = self.parse_condition()?;
            // 期望 THEN
            match self.peek() {
                Some(Token::Ident(s)) if s.eq_ignore_ascii_case("THEN") => {
                    self.next();
                }
                _ => return Err("CASE WHEN 后缺少 THEN".to_string()),
            }
            // 值
            let val = self.parse_value()?;
            if cond {
                // 匹配成功，跳过后续 WHEN/ELSE，找到 END
                while let Some(t) = self.peek() {
                    match t {
                        Token::Ident(s) if s.eq_ignore_ascii_case("END") => {
                            self.next();
                            return Ok(val);
                        }
                        _ => {
                            self.next();
                        }
                    }
                }
                return Err("CASE 匹配后未找到 END".to_string());
            }
        }
    }

    /// condition := comparison (('AND' | 'OR') comparison)*
    /// 返回 bool
    fn parse_condition(&mut self) -> Result<bool, String> {
        let mut result = self.parse_comparison_bool()?;
        loop {
            match self.peek() {
                Some(Token::Ident(s)) if s.eq_ignore_ascii_case("AND") => {
                    self.next();
                    let right = self.parse_comparison_bool()?;
                    result = result && right;
                }
                Some(Token::Ident(s)) if s.eq_ignore_ascii_case("OR") => {
                    self.next();
                    let right = self.parse_comparison_bool()?;
                    result = result || right;
                }
                _ => break,
            }
        }
        Ok(result)
    }

    /// comparison_bool := add_expr (('>=' | '<=' | '==' | '!=' | '>' | '<') add_expr)?
    /// 返回 bool
    fn parse_comparison_bool(&mut self) -> Result<bool, String> {
        let left = self.parse_expr()?;
        let op = match self.peek() {
            Some(Token::Gt) => { self.next(); ">" }
            Some(Token::Lt) => { self.next(); "<" }
            Some(Token::Ge) => { self.next(); ">=" }
            Some(Token::Le) => { self.next(); "<=" }
            Some(Token::Eq) => { self.next(); "==" }
            Some(Token::Ne) => { self.next(); "!=" }
            _ => return Ok(left > Decimal::ZERO), // 无比较运算符时，非零为真
        };
        let right = self.parse_expr()?;
        let result = match op {
            ">" => left > right,
            "<" => left < right,
            ">=" => left >= right,
            "<=" => left <= right,
            "==" => left == right,
            "!=" => left != right,
            _ => false,
        };
        Ok(result)
    }

    /// comparison := add_expr (('>=' | '<=' | '==' | '!=' | '>' | '<') add_expr)?
    /// 返回 Decimal（true=1, false=0），用于兼容旧式算术上下文
    fn parse_comparison(&mut self) -> Result<Decimal, String> {
        let left = self.parse_expr()?;
        let op_token = match self.peek() {
            Some(Token::Gt) | Some(Token::Lt) | Some(Token::Ge)
            | Some(Token::Le) | Some(Token::Eq) | Some(Token::Ne) => self.next(),
            _ => return Ok(left), // 无比较运算符，直接返回算术结果
        };
        // op_token 是 Option<Token>，需要 unwrap
        let op = op_token.ok_or("比较运算符解析失败")?;
        let right = self.parse_expr()?;
        let result = match op {
            Token::Gt => left > right,
            Token::Lt => left < right,
            Token::Ge => left >= right,
            Token::Le => left <= right,
            Token::Eq => left == right,
            Token::Ne => left != right,
            _ => false,
        };
        Ok(if result { Decimal::ONE } else { Decimal::ZERO })
    }

    /// expr := term (('+' | '-') term)*
    fn parse_expr(&mut self) -> Result<Decimal, String> {
        let mut left = self.parse_term()?;
        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.next();
                    let right = self.parse_term()?;
                    left += right;
                }
                Some(Token::Minus) => {
                    self.next();
                    let right = self.parse_term()?;
                    left -= right;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// term := factor (('*' | '/') factor)*
    fn parse_term(&mut self) -> Result<Decimal, String> {
        let mut left = self.parse_factor()?;
        loop {
            match self.peek() {
                Some(Token::Star) => {
                    self.next();
                    let right = self.parse_factor()?;
                    left *= right;
                }
                Some(Token::Slash) => {
                    self.next();
                    let right = self.parse_factor()?;
                    if right.is_zero() {
                        return Err("除零错误".to_string());
                    }
                    left /= right;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// factor := number | '(' value ')' | '-' factor | '+' factor | IF(...) | CASE...
    fn parse_factor(&mut self) -> Result<Decimal, String> {
        match self.peek() {
            Some(Token::Number(_)) => {
                if let Some(Token::Number(v)) = self.next() {
                    Ok(v)
                } else {
                    Err("数字解析错误".to_string())
                }
            }
            Some(Token::LParen) => {
                self.next();
                let v = self.parse_value()?;
                match self.next() {
                    Some(Token::RParen) => Ok(v),
                    _ => Err("缺少右括号".to_string()),
                }
            }
            Some(Token::Minus) => {
                self.next();
                let v = self.parse_factor()?;
                Ok(-v)
            }
            Some(Token::Plus) => {
                self.next();
                self.parse_factor()
            }
            Some(Token::Ident(s)) if s.eq_ignore_ascii_case("IF") => self.parse_if(),
            Some(Token::Ident(s)) if s.eq_ignore_ascii_case("CASE") => self.parse_case(),
            other => Err(format!("意外的 token: {:?}", other)),
        }
    }
}

/// 词法分析：将字符串转为 token 序列
/// P2-5: 扩展支持标识符（IF/CASE/WHEN/THEN/ELSE/END/AND/OR）、比较运算符、逗号
fn tokenize(source: &str) -> Vec<Token> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        match c {
            '+' => { tokens.push(Token::Plus); i += 1; }
            '-' => { tokens.push(Token::Minus); i += 1; }
            '*' => { tokens.push(Token::Star); i += 1; }
            '/' => { tokens.push(Token::Slash); i += 1; }
            '(' => { tokens.push(Token::LParen); i += 1; }
            ')' => { tokens.push(Token::RParen); i += 1; }
            ',' => { tokens.push(Token::Comma); i += 1; }
            '>' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::Ge);
                    i += 2;
                } else {
                    tokens.push(Token::Gt);
                    i += 1;
                }
            }
            '<' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::Le);
                    i += 2;
                } else {
                    tokens.push(Token::Lt);
                    i += 1;
                }
            }
            '=' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::Eq);
                    i += 2;
                } else {
                    // 单个 = 也当作 == 处理，便于用户输入
                    tokens.push(Token::Eq);
                    i += 1;
                }
            }
            '!' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::Ne);
                    i += 2;
                } else {
                    return Vec::new(); // 非法字符
                }
            }
            _ => {
                if c.is_ascii_digit() || c == '.' {
                    let start = i;
                    let mut has_dot = false;
                    while i < chars.len() {
                        let ch = chars[i];
                        if ch.is_ascii_digit() {
                            i += 1;
                        } else if ch == '.' && !has_dot {
                            has_dot = true;
                            i += 1;
                        } else {
                            break;
                        }
                    }
                    let num_str: String = chars[start..i].iter().collect();
                    match num_str.parse::<Decimal>() {
                        Ok(v) => tokens.push(Token::Number(v)),
                        Err(_) => return Vec::new(),
                    }
                } else if c.is_ascii_alphabetic() || c == '_' {
                    // 标识符：字母开头，含字母/数字/下划线
                    let start = i;
                    while i < chars.len() {
                        let ch = chars[i];
                        if ch.is_ascii_alphanumeric() || ch == '_' {
                            i += 1;
                        } else {
                            break;
                        }
                    }
                    let ident: String = chars[start..i].iter().collect();
                    tokens.push(Token::Ident(ident));
                } else {
                    return Vec::new();
                }
            }
        }
    }
    tokens
}
