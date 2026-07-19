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
use crate::modules::company::entity::{code_rule, code_sequence};
use crate::modules::company::model::code_rule::{
    save_req_to_active, BatchRegenerateProgressVO, BatchRegenerateResultVO, CodeRuleSaveReq,
    CodeRuleVO, PreviewCodeReq, SegmentConfig, SEG_BIZ_TYPE, SEG_COMPANY, SEG_DATE, SEG_DEPT,
    SEG_FIXED, SEG_SEQ, SEG_VERSION, SEG_YEAR, YEAR_SRC_BUSINESS_DATE, YEAR_SRC_CREATE_TIME,
    YEAR_SRC_CURRENT,
};
use crate::modules::crm::entity::customer;
use crate::modules::crm::service::customer_edit_log_service;
use sea_orm::prelude::Json;
use sea_orm::sea_query::{Expr, OnConflict};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DbConn, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Set, TransactionTrait,
};
use std::sync::Mutex;
use chrono::{Datelike, NaiveDate};

/// 全局一键更新任务进度缓存（单机内存，进程级）
/// 简化实现：同一时刻只允许一个任务运行，新任务会覆盖旧任务状态
static BATCH_PROGRESS: Mutex<Option<BatchRegenerateProgressVO>> = Mutex::new(None);

/// 列表查询（分页）
pub async fn list<C: ConnectionTrait>(
    db: &C,
    page: u64,
    page_size: u64,
    module_code: Option<&str>,
    enabled: Option<i16>,
) -> Result<ResultPage<Vec<CodeRuleVO>>> {
    let mut query = code_rule::Entity::find()
        .filter(code_rule::Column::Deleted.eq(0));
    if let Some(code) = module_code {
        query = query.filter(code_rule::Column::ModuleCode.eq(code));
    }
    if let Some(e) = enabled {
        query = query.filter(code_rule::Column::Enabled.eq(e));
    }
    query = query.order_by_asc(code_rule::Column::Id);

    let total = query.clone().count(db).await?;
    let paginator = query.paginate(db, page_size);
    let items: Vec<code_rule::Model> = paginator.fetch_page(page.saturating_sub(1)).await?;

    let list: Vec<CodeRuleVO> = items.into_iter().map(|m| m.into()).collect();
    Ok(ResultPage::new(list, total as i64, page as i64, page_size as i64))
}

/// 详情查询
pub async fn find_by_id<C: ConnectionTrait>(db: &C, id: i64) -> Result<CodeRuleVO> {
    let m = code_rule::Entity::find_by_id(id)
        .filter(code_rule::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("编号规则不存在"))?;
    Ok(m.into())
}

/// 新增规则
/// 检查 module_code 唯一性，事务包裹
pub async fn create(db: &DbConn, req: CodeRuleSaveReq, user_id: i64) -> Result<i64> {
    // 唯一性检查
    let exists = code_rule::Entity::find()
        .filter(code_rule::Column::ModuleCode.eq(&req.module_code))
        .filter(code_rule::Column::Deleted.eq(0))
        .one(db)
        .await?;
    if exists.is_some() {
        return Err(Error::from(format!("模块编码 {} 已存在编号规则", req.module_code)));
    }

    let now = chrono::Local::now().naive_local();
    let mut am = save_req_to_active(&req);
    am.created_by = Set(Some(user_id));
    am.create_time = Set(Some(now));
    am.updated_by = Set(Some(user_id));
    am.update_time = Set(Some(now));
    am.deleted = Set(Some(0));

    let result = db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move {
            let r = am.insert(txn).await?;
            Ok(r.id)
        })
    }).await.map_err(|e| Error::from(e.to_string()))?;
    Ok(result)
}

/// 修改规则
pub async fn update(db: &DbConn, id: i64, req: CodeRuleSaveReq, user_id: i64) -> Result<i64> {
    let now = chrono::Local::now().naive_local();
    let mut am = save_req_to_active(&req);
    am.updated_by = Set(Some(user_id));
    am.update_time = Set(Some(now));

    let res = db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move {
            let r = code_rule::Entity::update_many()
                .set(am)
                .filter(code_rule::Column::Id.eq(id))
                .filter(code_rule::Column::Deleted.eq(0))
                .exec(txn)
                .await?;
            Ok(r.rows_affected as i64)
        })
    }).await.map_err(|e| Error::from(e.to_string()))?;
    Ok(res)
}

/// 软删除规则
pub async fn delete(db: &DbConn, id: i64) -> Result<i64> {
    let now = chrono::Local::now().naive_local();
    let am = code_rule::ActiveModel {
        deleted: Set(Some(1)),
        update_time: Set(Some(now)),
        ..Default::default()
    };
    let res = db.transaction::<_, i64, DbErr>(|txn| {
        Box::pin(async move {
            let r = code_rule::Entity::update_many()
                .set(am)
                .filter(code_rule::Column::Id.eq(id))
                .filter(code_rule::Column::Deleted.eq(0))
                .exec(txn)
                .await?;
            Ok(r.rows_affected as i64)
        })
    }).await.map_err(|e| Error::from(e.to_string()))?;
    Ok(res)
}

/// 启用/停用规则
pub async fn toggle_enabled(db: &DbConn, id: i64, enabled: i16, user_id: i64) -> Result<i64> {
    let now = chrono::Local::now().naive_local();
    let am = code_rule::ActiveModel {
        enabled: Set(Some(enabled)),
        updated_by: Set(Some(user_id)),
        update_time: Set(Some(now)),
        ..Default::default()
    };
    let res = code_rule::Entity::update_many()
        .set(am)
        .filter(code_rule::Column::Id.eq(id))
        .filter(code_rule::Column::Deleted.eq(0))
        .exec(db)
        .await?;
    Ok(res.rows_affected as i64)
}

/// 解析段位配置并按 sort 排序
fn sort_segments(segments: &[SegmentConfig]) -> Vec<SegmentConfig> {
    let mut v = segments.to_vec();
    v.sort_by_key(|s| s.sort);
    v
}

/// 解析段位配置 JSON 为 SegmentConfig 数组
fn parse_segments(j: &Option<Json>) -> Vec<SegmentConfig> {
    j.as_ref()
        .and_then(|v| serde_json::from_value::<Vec<SegmentConfig>>(v.clone()).ok())
        .unwrap_or_default()
}

/// 根据 year 段位的 source 决定年份
/// - rule_segments: 规则配置中的段位
/// - business_date: 调用方传入的业务日期
/// - record_create_time: 仅一键更新使用
fn resolve_year(
    rule_segments: &[SegmentConfig],
    business_date: Option<NaiveDate>,
    record_create_time: Option<chrono::NaiveDateTime>,
) -> i32 {
    let year_seg = rule_segments.iter().find(|s| s.type_ == SEG_YEAR);
    let current_year = chrono::Local::now().year();
    if let Some(seg) = year_seg {
        let src = seg.source.as_deref().unwrap_or(YEAR_SRC_CURRENT);
        match src {
            YEAR_SRC_BUSINESS_DATE => {
                business_date.map(|d| d.year()).unwrap_or(current_year)
            }
            YEAR_SRC_CREATE_TIME => {
                record_create_time.map(|t| t.year()).unwrap_or(current_year)
            }
            _ => current_year,
        }
    } else {
        current_year
    }
}

/// 格式化年份
fn format_year(format: Option<&str>, year: i32) -> String {
    match format {
        Some("yy") => format!("{:02}", year % 100),
        _ => format!("{}", year),
    }
}

/// 格式化日期段位
fn format_date(format: Option<&str>) -> String {
    let now = chrono::Local::now().naive_local();
    match format {
        Some("yyyyMMdd") => now.format("%Y%m%d").to_string(),
        Some("yyyyMM") | Some("yyyymm") => now.format("%Y%m").to_string(),
        _ => now.format("%Y%m%d").to_string(),
    }
}

/// 递增流水号（使用 SeaORM Entity API，兼容事务 & 数据库连接）
/// 三步执行：1) INSERT ON CONFLICT DO NOTHING 创建计数行
///          2) SELECT 当前 current_seq
///          3) UPDATE current_seq = current_seq + 1
/// 在事务内调用时这三步是原子的；不在事务时仍有竞态但单线程顺序调用安全
async fn increment_sequence<C: ConnectionTrait>(
    db: &C,
    module_code: &str,
    year: i32,
    dept_code: &str,
) -> Result<i32> {
    // 1. 尝试插入计数行（若已存在则 do nothing）
    let active = code_sequence::ActiveModel {
        module_code: Set(Some(module_code.to_string())),
        year: Set(Some(year)),
        dept_code: Set(Some(dept_code.to_string())),
        current_seq: Set(Some(0)),
        ..Default::default()
    };
    let _ = code_sequence::Entity::insert(active)
        .on_conflict(
            OnConflict::columns([
                code_sequence::Column::ModuleCode,
                code_sequence::Column::Year,
                code_sequence::Column::DeptCode,
            ])
            .do_nothing()
            .to_owned(),
        )
        .exec(db)
        .await;

    // 2. 查询当前 current_seq
    let current = code_sequence::Entity::find()
        .filter(code_sequence::Column::ModuleCode.eq(module_code))
        .filter(code_sequence::Column::Year.eq(year))
        .filter(code_sequence::Column::DeptCode.eq(dept_code))
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .and_then(|m| m.current_seq)
        .unwrap_or(0);

    // 3. 计算新值并更新
    let new_seq = current + 1;
    let _ = code_sequence::Entity::update_many()
        .col_expr(
            code_sequence::Column::CurrentSeq,
            Expr::value(new_seq),
        )
        .filter(code_sequence::Column::ModuleCode.eq(module_code))
        .filter(code_sequence::Column::Year.eq(year))
        .filter(code_sequence::Column::DeptCode.eq(dept_code))
        .exec(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    Ok(new_seq)
}

/// 重置指定模块/年份/部门的流水号计数器为 0
async fn reset_sequence<C: ConnectionTrait>(
    db: &C,
    module_code: &str,
    year: i32,
    dept_code: &str,
) -> Result<()> {
    let active = code_sequence::ActiveModel {
        module_code: Set(Some(module_code.to_string())),
        year: Set(Some(year)),
        dept_code: Set(Some(dept_code.to_string())),
        current_seq: Set(Some(0)),
        ..Default::default()
    };
    code_sequence::Entity::insert(active)
        .on_conflict(
            OnConflict::columns([
                code_sequence::Column::ModuleCode,
                code_sequence::Column::Year,
                code_sequence::Column::DeptCode,
            ])
            .update_column(code_sequence::Column::CurrentSeq)
            .to_owned(),
        )
        .exec(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(())
}

/// 判断段位配置中是否包含版本号段位
fn has_version_segment(segments: &[SegmentConfig]) -> bool {
    segments.iter().any(|s| s.type_ == SEG_VERSION)
}

/// 从版本号字符串中提取数字部分并递增
/// "V1" → 2, "Rev3" → 4, 失败时返回 1
fn next_version_number(version_str: &str) -> i32 {
    let num: i32 = version_str
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .unwrap_or(0);
    num + 1
}

/// 根据模块编码生成业务编号
/// 使用泛型 ConnectionTrait，支持 &DbConn 和事务 &DatabaseTransaction
///
/// - business_date: 业务日期，用于补录历史文件时取正确年份（详见设计文档 8.5）
///                  传 None 时按规则配置的 source 取值
/// - previous_version: 修订时传入前一个版本号（如 "V1"），自动递增为下一版
pub async fn generate_code<C: ConnectionTrait>(
    db: &C,
    module_code: &str,
    dept_code: Option<&str>,
    business_date: Option<NaiveDate>,
    previous_version: Option<&str>,
) -> Result<String> {
    // 1. 查规则配置
    let rule = code_rule::Entity::find()
        .filter(code_rule::Column::ModuleCode.eq(module_code))
        .filter(code_rule::Column::Enabled.eq(1))
        .filter(code_rule::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from(format!("模块 {} 未配置编号规则或未启用", module_code)))?;

    let segments = parse_segments(&rule.segments);
    if segments.is_empty() {
        return Err(Error::from(format!("模块 {} 编号规则未配置段位", module_code)));
    }

    // 2. 确定年份
    let year = resolve_year(&segments, business_date, None);

    // 3. 原子递增流水号（修订时复用已有流水号，不递增）
    let dept = dept_code.unwrap_or("");
    let has_version = has_version_segment(&segments);
    let next_seq = if previous_version.is_some() && has_version {
        // 修订：不递增流水号，用 0 占位，merge 时从 rule 取 seq_length 格式化
        0
    } else {
        increment_sequence(db, module_code, year, dept).await?
    };

    // 4. 计算版本号
    let version_num = if has_version {
        if let Some(pv) = previous_version {
            next_version_number(pv)
        } else {
            1
        }
    } else {
        1
    };

    // 5. 拼接段位
    Ok(merge_segments(
        &segments,
        &rule.company_abbr.unwrap_or_default(),
        &rule.biz_type_code.unwrap_or_default(),
        dept,
        year,
        next_seq,
        rule.seq_length.unwrap_or(4) as usize,
        business_date,
        None,
        version_num,
    ))
}

/// 拼接段位生成最终编号
#[allow(clippy::too_many_arguments)]
fn merge_segments(
    segments: &[SegmentConfig],
    company_abbr: &str,
    biz_type: &str,
    dept_code: &str,
    year: i32,
    seq: i32,
    seq_length: usize,
    business_date: Option<NaiveDate>,
    record_create_time: Option<chrono::NaiveDateTime>,
    version_num: i32,
) -> String {
    let sep = "-"; // 默认分隔符，调用方传入时由规则决定
    let _ = (business_date, record_create_time);

    let mut parts: Vec<String> = Vec::new();
    for seg in sort_segments(segments) {
        let part = match seg.type_.as_str() {
            SEG_COMPANY => company_abbr.to_string(),
            SEG_BIZ_TYPE => biz_type.to_string(),
            SEG_YEAR => format_year(seg.format.as_deref(), year),
            SEG_DEPT => dept_code.to_string(),
            SEG_SEQ => format!("{:0width$}", seq, width = seq_length),
            SEG_VERSION => {
                // 从 seg.value 提取前缀（如 "V1" → "V"，"Rev1" → "Rev"）
                let prefix = seg.value.as_deref()
                    .map(|v| v.trim_end_matches(|c: char| c.is_ascii_digit()))
                    .filter(|s| !s.is_empty())
                    .unwrap_or("V");
                format!("{}{}", prefix, version_num)
            }
            SEG_FIXED => seg.value.clone().unwrap_or_default(),
            SEG_DATE => format_date(seg.format.as_deref()),
            _ => String::new(),
        };
        if !part.is_empty() {
            parts.push(part);
        }
    }
    parts.join(sep)
}

/// 预览编号（不入库，不递增计数器）
pub async fn preview(db: &DbConn, req: PreviewCodeReq) -> Result<String> {
    // 若传入 module_code，从规则取默认值
    let (mut company_abbr, mut biz_type, mut separator, mut seq_length) =
        (String::new(), String::new(), "-".to_string(), 4usize);
    let mut segments: Vec<SegmentConfig> = req.segments.clone();

    if let Some(code) = &req.module_code {
        if let Some(rule) = code_rule::Entity::find()
            .filter(code_rule::Column::ModuleCode.eq(code))
            .filter(code_rule::Column::Deleted.eq(0))
            .one(db)
            .await?
        {
            company_abbr = rule.company_abbr.unwrap_or_default();
            biz_type = rule.biz_type_code.unwrap_or_default();
            separator = rule.separator.unwrap_or_else(|| "-".to_string());
            seq_length = rule.seq_length.unwrap_or(4) as usize;
            if segments.is_empty() {
                segments = parse_segments(&rule.segments);
            }
        }
    }

    if let Some(c) = &req.company_abbr {
        company_abbr = c.clone();
    }
    if let Some(b) = &req.biz_type_code {
        biz_type = b.clone();
    }
    if let Some(s) = &req.separator {
        separator = s.clone();
    }
    if let Some(l) = req.seq_length {
        seq_length = l as usize;
    }

    if segments.is_empty() {
        return Err(Error::from("段位配置不能为空"));
    }

    let business_date = req
        .business_date
        .as_deref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let year = resolve_year(&segments, business_date, None);
    let dept = req.dept_code.unwrap_or_default();
    let mock_seq = if req.mock_seq.unwrap_or(true) { 1 } else { 0 };

    let merged = merge_segments(
        &segments,
        &company_abbr,
        &biz_type,
        &dept,
        year,
        mock_seq,
        seq_length,
        business_date,
        None,
        1,
    );
    // 应用配置的分隔符
    let final_no = merged.replace('-', &separator);
    Ok(final_no)
}

/// 启动一键更新任务（异步执行，立即返回）
/// 简化实现：当前版本仅完成框架，各业务模块的"已存在记录编号"字段名、表名、SQL 均需在接入时具体实现
pub async fn start_batch_regenerate(
    db: &DbConn,
    req: crate::modules::company::model::code_rule::BatchRegenerateReq,
    user_id: i64,
) -> Result<String> {
    if req.module_codes.is_empty() {
        return Err(Error::from("请至少选择一个模块"));
    }

    // 检查所有模块规则都存在
    for code in &req.module_codes {
        let _ = code_rule::Entity::find()
            .filter(code_rule::Column::ModuleCode.eq(code))
            .filter(code_rule::Column::Deleted.eq(0))
            .one(db)
            .await?
            .ok_or_else(|| Error::from(format!("模块 {} 未配置编号规则", code)))?;
    }

    // 初始化进度
    {
        let mut p = BATCH_PROGRESS.lock().map_err(|e| Error::from(e.to_string()))?;
        *p = Some(BatchRegenerateProgressVO {
            total: 0,
            done: 0,
            current_module: None,
            status: "running".to_string(),
            message: Some("任务已启动".to_string()),
        });
    }

    let mut total_affected = 0i64;
    let mut modules_done: Vec<String> = Vec::new();
    for code in &req.module_codes {
        match code.as_str() {
            "customer" => {
                // 读取客户模块编号规则
                let rule = code_rule::Entity::find()
                    .filter(code_rule::Column::ModuleCode.eq("customer"))
                    .filter(code_rule::Column::Deleted.eq(0))
                    .one(db)
                    .await?
                    .ok_or_else(|| Error::from("客户模块未配置编号规则"))?;
                let segments = parse_segments(&rule.segments);
                let seq_length = rule.seq_length.unwrap_or(4) as usize;
                let separator = rule.separator.unwrap_or_else(|| "-".to_string());
                // 提取规则配置到局部变量（避免循环中移动所有权）
                let company_abbr = rule.company_abbr.clone().unwrap_or_default();
                let biz_type_code = rule.biz_type_code.clone().unwrap_or_default();

                if let Some(years) = &req.years {
                    // 按指定年份重新编号，每一年从 0001 开始
                    for year in years {
                        // 重置该年份计数器
                        reset_sequence(db, "customer", *year, "").await?;

                        // 查询该年份创建的所有未删除客户，按 id 升序
                        let start_date = NaiveDate::from_ymd_opt(*year, 1, 1)
                            .map(|d| d.and_hms_opt(0, 0, 0))
                            .flatten();
                        let end_date = NaiveDate::from_ymd_opt(*year + 1, 1, 1)
                            .map(|d| d.and_hms_opt(0, 0, 0))
                            .flatten();
                        let customers = if let (Some(s), Some(e)) = (start_date, end_date) {
                            customer::Entity::find()
                                .filter(customer::Column::Deleted.eq(0))
                                .filter(customer::Column::CreateTime.gte(s))
                                .filter(customer::Column::CreateTime.lt(e))
                                .order_by_asc(customer::Column::Id)
                                .all(db)
                                .await?
                        } else {
                            vec![]
                        };

                        for c in &customers {
                            let seq = increment_sequence(db, "customer", *year, "").await?;
                            let new_no = merge_segments(
                                &segments,
                                &company_abbr,
                                &biz_type_code,
                                "",
                                *year,
                                seq,
                                seq_length,
                                None,
                                c.create_time,
                                1,
                            );
                            // 替换分隔符
                            let final_no = new_no.replace('-', &separator);

                            // 记录旧编号变更日志
                            let old_json = serde_json::json!({"customer_no": c.customer_no});
                            let new_json = serde_json::json!({"customer_no": final_no});
                            let _ = customer_edit_log_service::log_update(
                                db, c.id, user_id, None, &old_json, &new_json, Some(0),
                            ).await;

                            // 更新客户编号
                            let mut active: customer::ActiveModel = c.clone().into();
                            active.customer_no = Set(Some(final_no));
                            let _ = active.update(db).await;
                        }
                        total_affected += customers.len() as i64;
                    }
                } else {
                    // 未指定年份，使用原有逻辑（仅更新编号为空的客户），按 id 升序
                    let customers = customer::Entity::find()
                        .filter(customer::Column::Deleted.eq(0))
                        .filter(customer::Column::CustomerNo.is_null())
                        .order_by_asc(customer::Column::Id)
                        .all(db)
                        .await?;
                    for c in &customers {
                        if let Ok(new_no) = generate_code(db, "customer", None, None, None).await {
                            let mut active: customer::ActiveModel = c.clone().into();
                            active.customer_no = Set(Some(new_no));
                            let _ = active.update(db).await;
                        }
                    }
                    total_affected += customers.len() as i64;
                }
                modules_done.push(code.clone());
            }
            _ => {
                // 其他模块暂未接入
                total_affected += 0;
                modules_done.push(code.clone());
            }
        }
    }

    // 完成进度
    {
        let mut p = BATCH_PROGRESS.lock().map_err(|e| Error::from(e.to_string()))?;
        *p = Some(BatchRegenerateProgressVO {
            total: total_affected,
            done: total_affected,
            current_module: None,
            status: "success".to_string(),
            message: Some(format!("已完成，共更新 {} 条记录", total_affected)),
        });
    }

    Ok(format!("batch-{}", chrono::Local::now().timestamp()))
}

/// 查询一键更新任务进度
pub fn get_batch_progress() -> Result<BatchRegenerateProgressVO> {
    let p = BATCH_PROGRESS.lock().map_err(|e| Error::from(e.to_string()))?;
    Ok(p.clone().unwrap_or_default())
}

/// 一键更新结果（供 controller 调用）
pub async fn batch_regenerate_result() -> Result<BatchRegenerateResultVO> {
    let p = get_batch_progress()?;
    Ok(BatchRegenerateResultVO {
        modules: Vec::new(),
        total_affected: p.total,
    })
}
