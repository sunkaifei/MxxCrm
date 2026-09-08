//!
//! 客户公海：查重规则引擎（v57 全量设计 §5.4，吸收金蝶/泛微/纷享）
//!
//! - 规则 CRUD（管理端）
//! - `check`：给定客户录入要素，按启用规则返回命中明细（前端提示级展示）
//! - `enforce_on_create`：新建客户时的阻断级强制（strength=2），阻断返回命中客户

use std::collections::HashSet;

use sea_orm::{ColumnTrait, Condition, DbConn, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set};

use crate::core::errors::error::{Error, Result};
use crate::modules::crm::entity::customer_pool_dup_rule;
use crate::modules::crm::entity::customer;

fn map_db_err(e: sea_orm::DbErr) -> Error {
    Error::from(e.to_string())
}

// ==================== 规则 CRUD ====================

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct DupRuleSaveRequest {
    pub id: Option<i64>,
    pub name: Option<String>,
    /// 查重字段（name/contact/mobile/phone/wechat/credit_code 子集）
    pub fields: Option<Vec<String>>,
    /// 控制强度：1=提示 2=阻断 3=需审批（V2）
    pub strength: Option<i16>,
    /// 有效期天数（按最近跟进时间判定，0/空=永久有效）
    pub valid_days: Option<i32>,
    /// 名称连续字符匹配率阈值（0-100）
    pub match_ratio: Option<i32>,
    pub enabled: Option<i16>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct DupRuleDeleteRequest {
    #[serde(default)]
    pub ids: Vec<i64>,
}

/// 规则分页
pub async fn page(db: &DbConn, page: i64, page_size: i64) -> Result<(i64, Vec<customer_pool_dup_rule::Model>)> {
    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);
    let paginator = customer_pool_dup_rule::Entity::find()
        .filter(customer_pool_dup_rule::Column::Deleted.eq(0))
        .order_by_desc(customer_pool_dup_rule::Column::UpdateTime)
        .paginate(db, page_size as u64);
    let total = paginator.num_items().await.map_err(map_db_err)? as i64;
    let items = paginator
        .fetch_page((page - 1) as u64)
        .await
        .map_err(map_db_err)?;
    Ok((total, items))
}

/// 保存规则（新增/更新）
pub async fn save(db: &DbConn, req: &DupRuleSaveRequest, operator_id: i64) -> Result<i64> {
    if req.name.as_deref().map(|s| s.trim().is_empty()).unwrap_or(true) {
        return Err(Error::from("请输入规则名称".to_string()));
    }
    let fields = serde_json::to_value(req.fields.clone().unwrap_or_default()).map_err(|e| Error::from(e.to_string()))?;
    let ts = chrono::Local::now().naive_local();
    if let Some(id) = req.id {
        let payload = customer_pool_dup_rule::ActiveModel {
            updated_by: Set(Some(operator_id)),
            update_time: Set(Some(ts)),
            name: Set(req.name.clone()),
            fields: Set(Some(fields)),
            strength: Set(req.strength),
            valid_days: Set(req.valid_days),
            match_ratio: Set(req.match_ratio),
            enabled: Set(req.enabled),
            ..Default::default()
        };
        let res = customer_pool_dup_rule::Entity::update_many()
            .set(payload)
            .filter(customer_pool_dup_rule::Column::Id.eq(id))
            .filter(customer_pool_dup_rule::Column::Deleted.eq(0))
            .exec(db)
            .await
            .map_err(map_db_err)?;
        if res.rows_affected == 0 {
            return Err(Error::from("规则不存在或已删除".to_string()));
        }
        Ok(id)
    } else {
        let payload = customer_pool_dup_rule::ActiveModel {
            name: Set(req.name.clone()),
            fields: Set(Some(fields)),
            strength: Set(req.strength),
            valid_days: Set(req.valid_days),
            match_ratio: Set(req.match_ratio),
            enabled: Set(req.enabled),
            created_by: Set(Some(operator_id)),
            create_time: Set(Some(ts.clone())),
            updated_by: Set(Some(operator_id)),
            update_time: Set(Some(ts)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        let inserted = customer_pool_dup_rule::Entity::insert(payload).exec(db).await.map_err(map_db_err)?;
        Ok(inserted.last_insert_id)
    }
}

/// 删除规则（逻辑删）
pub async fn batch_delete(db: &DbConn, ids: &[i64], operator_id: i64) -> Result<u64> {
    if ids.is_empty() {
        return Ok(0);
    }
    let ts = chrono::Local::now().naive_local();
    let payload = customer_pool_dup_rule::ActiveModel {
        deleted: Set(Some(1)),
        delete_time: Set(Some(ts.clone())),
        delete_by: Set(Some(operator_id)),
        update_time: Set(Some(ts)),
        ..Default::default()
    };
    let res = customer_pool_dup_rule::Entity::update_many()
        .set(payload)
        .filter(customer_pool_dup_rule::Column::Id.is_in(ids.to_vec()))
        .exec(db)
        .await
        .map_err(map_db_err)?;
    Ok(res.rows_affected)
}

// ==================== 查重执行 ====================

#[derive(Debug, serde::Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct DupHitVO {
    pub customer_id: i64,
    pub customer_name: String,
    /// 命中字段
    pub matched_field: String,
    pub matched_value: String,
    pub rule_name: String,
    /// 规则强度：1=提示 2=阻断
    pub strength: i16,
}

/// 名称连续字符匹配率：最长公共连续子串长度 / 输入长度（%）
fn name_match_ratio(input: &str, existing: &str) -> f64 {
    if input.is_empty() || existing.is_empty() {
        return 0.0;
    }
    let a: Vec<char> = input.chars().collect();
    let b: Vec<char> = existing.chars().collect();
    let mut best = 0usize;
    for i in 0..a.len() {
        for j in 0..b.len() {
            let mut k = 0usize;
            while i + k < a.len() && j + k < b.len() && a[i + k] == b[j + k] {
                k += 1;
            }
            best = best.max(k);
        }
    }
    (best as f64 / a.len() as f64) * 100.0
}

/// 按启用规则执行查重，返回命中明细（全部规则聚合）
///
/// `input`：新建客户时录入的要素；规则字段以录入值非空为参与条件。
/// 有效期：`last_follow_up_at` 距今 ≤ valid_days 天才参与（NULL 视为无效）。
pub async fn check(
    db: &DbConn,
    name: Option<&str>,
    mobile: Option<&str>,
    phone: Option<&str>,
    wechat: Option<&str>,
    email: Option<&str>,
) -> Result<Vec<DupHitVO>> {
    let rules = customer_pool_dup_rule::Entity::find()
        .filter(customer_pool_dup_rule::Column::Deleted.eq(0))
        .filter(customer_pool_dup_rule::Column::Enabled.eq(1))
        .all(db)
        .await
        .map_err(map_db_err)?;

    let mut hits: Vec<DupHitVO> = Vec::new();
    let mut seen: HashSet<i64> = HashSet::new();
    for rule in rules {
        let fields: Vec<String> = rule
            .fields
            .as_ref()
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();
        let strength = rule.strength.unwrap_or(1);
        let ratio_threshold = rule.match_ratio.unwrap_or(80) as f64;
        let valid_days = rule.valid_days.unwrap_or(0);
        for field in &fields {
            let (input, col_cond): (Option<String>, Option<Condition>) = match field.as_str() {
                "name" => {
                    let input = name.filter(|s| !s.trim().is_empty()).map(|s| s.trim().to_string());
                    if input.is_none() {
                        (None, None)
                    } else {
                        // 名称走连续字符匹配率，宽匹配后按阈值过滤
                        (input, Some(Condition::any()))
                    }
                },
                "mobile" | "phone" | "wechat" | "email" => {
                    let input = match field.as_str() {
                        "mobile" => mobile.filter(|s| !s.trim().is_empty()),
                        "phone" => phone.filter(|s| !s.trim().is_empty()),
                        "wechat" => wechat.filter(|s| !s.trim().is_empty()),
                        _ => email.filter(|s| !s.trim().is_empty()),
                    }
                    .map(|s| s.trim().to_string());
                    if input.is_none() {
                        (None, None)
                    } else {
                        let cond = match field.as_str() {
                            "mobile" => Condition::all().add(customer::Column::PersonalMobile.eq(input.clone())),
                            "phone" => Condition::all().add(customer::Column::PersonalMobile.eq(input.clone())),
                            "wechat" => Condition::all().add(customer::Column::Wechat.eq(input.clone())),
                            _ => Condition::all().add(customer::Column::PersonalEmail.eq(input.clone())),
                        };
                        (Some(input.unwrap()), Some(cond))
                    }
                },
                _ => (None, None),
            };
            let Some(value) = input else { continue };
            let Some(cond) = col_cond else { continue };

            let mut query = customer::Entity::find()
                .filter(customer::Column::Deleted.eq(0))
                .filter(cond);
            if valid_days > 0 {
                let cutoff = chrono::Local::now().naive_local() - chrono::Duration::days(valid_days as i64);
                query = query.filter(customer::Column::LastFollowUpAt.gte(cutoff));
            }
            let rows = query.all(db).await.map_err(map_db_err)?;
            for c in rows {
                if field == "name" {
                    let existing = c
                        .company_name
                        .clone()
                        .filter(|s| !s.trim().is_empty())
                        .or_else(|| c.person_name.clone().filter(|s| !s.trim().is_empty()))
                        .unwrap_or_default();
                    let ratio = name_match_ratio(&value, &existing);
                    if ratio < ratio_threshold {
                        continue;
                    }
                }
                if seen.insert(c.id) {
                    hits.push(DupHitVO {
                        customer_id: c.id,
                        customer_name: c
                            .company_name
                            .or_else(|| c.person_name.clone())
                            .unwrap_or_else(|| format!("客户#{}", c.id)),
                        matched_field: field.clone(),
                        matched_value: value.clone(),
                        rule_name: rule.name.clone().unwrap_or_default(),
                        strength,
                    });
                }
                break; // 同字段命中一条即足够提示
            }
        }
    }
    Ok(hits)
}

/// 新建客户阻断级校验：任一启用规则 strength=2 命中 → 返回错误（含命中客户名）
pub async fn enforce_on_create(
    db: &DbConn,
    name: Option<&str>,
    mobile: Option<&str>,
    phone: Option<&str>,
    wechat: Option<&str>,
    email: Option<&str>,
) -> Result<()> {
    let hits = check(db, name, mobile, phone, wechat, email).await?;
    let blocking: Vec<&DupHitVO> = hits.iter().filter(|h| h.strength == 2).collect();
    if !blocking.is_empty() {
        let names = blocking
            .iter()
            .map(|h| format!("「{}」", h.customer_name))
            .collect::<Vec<_>>()
            .join("、");
        return Err(Error::from(format!(
            "查重命中阻断规则：与既有客户 {} 重复，请先处理后再保存",
            names
        )));
    }
    Ok(())
}
