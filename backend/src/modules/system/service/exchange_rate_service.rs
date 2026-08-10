//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 汇率自动更新 Service
//!
//! 数据来源：欧洲央行（ECB）每日参考汇率。
//! ECB 仅提供以 EUR 为基准的汇率（1 EUR = X currency），因此本服务在存储与查询时
//! 统一基于 EUR 做交叉汇率换算。
//!

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait};

use crate::core::errors::error::{Error, Result};
use crate::modules::system::entity::exchange_rate::{self, Column, Entity};
use crate::modules::system::service::integration_config_service;

/// ECB 每日汇率 XML 接口默认地址（未在 integration_config 配置时使用）
const ECB_DAILY_URL: &str = "https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml";
/// 第三方接口配置编码：ECB 汇率
const INTEGRATION_CODE_ECB: &str = "ecb";
/// EUR 货币代码
const EUR_CURRENCY: &str = "EUR";

/// 获取 ECB 接口地址：优先读取 integration_config 中 code="ecb" 的 api_base_url，否则使用默认值
async fn get_ecb_url(db: &DbConn) -> String {
    match integration_config_service::get_by_code(db, INTEGRATION_CODE_ECB).await {
        Ok(Some(cfg)) => cfg
            .api_base_url
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| ECB_DAILY_URL.to_string()),
        _ => ECB_DAILY_URL.to_string(),
    }
}

/// 从欧洲央行（ECB）拉取最新汇率并保存
///
/// 流程：
/// 1. 请求 ECB 每日汇率 XML
/// 2. 解析 XML 提取日期与各货币对 EUR 的汇率
/// 3. 写入 exchange_rate 表（已存在则跳过）
/// 4. 返回更新的货币对数量
pub async fn fetch_and_save_rates(db: &DbConn) -> Result<i64> {
    // 0. 优先从 integration_config 读取 ECB 接口地址，未配置时使用默认值
    let ecb_url = get_ecb_url(db).await;

    // 1. 请求 ECB XML
    let resp = reqwest::get(&ecb_url)
        .await
        .map_err(|e| Error::from(format!("请求ECB汇率失败: {}", e)))?;
    let xml = resp.text()
        .await
        .map_err(|e| Error::from(format!("读取ECB汇率内容失败: {}", e)))?;

    // 2. 解析 XML
    let (rate_date, pairs) = parse_ecb_xml(&xml)?;
    if pairs.is_empty() {
        return Err(Error::from("ECB汇率XML解析结果为空"));
    }

    // 3. 写入 exchange_rate 表（事务包裹，已存在的货币对+日期跳过）
    let pairs_clone = pairs.clone();
    let rate_date_clone = rate_date;
    let inserted = db.transaction::<_, i64, sea_orm::DbErr>(|txn| {
        Box::pin(async move {
            let mut count: i64 = 0;
            for (currency, rate) in &pairs_clone {
                // 跳过已存在的记录（按 from/to/date 去重）
                let exists = Entity::find()
                    .filter(Column::FromCurrency.eq(EUR_CURRENCY))
                    .filter(Column::ToCurrency.eq(currency))
                    .filter(Column::RateDate.eq(rate_date_clone))
                    .one(txn)
                    .await?;
                if exists.is_some() {
                    continue;
                }

                let active = exchange_rate::ActiveModel {
                    from_currency: Set(Some(EUR_CURRENCY.to_string())),
                    to_currency: Set(Some(currency.clone())),
                    rate: Set(Some(*rate)),
                    rate_date: Set(Some(rate_date_clone)),
                    source: Set(Some("ECB".to_string())),
                    create_time: Set(Some(chrono::Local::now().naive_local())),
                    ..Default::default()
                };
                active.insert(txn).await?;
                count += 1;
            }
            Ok(count)
        })
    })
    .await
    .map_err(|e| Error::from(e.to_string()))?;

    Ok(inserted)
}

/// 查询指定日期的汇率
///
/// 支持交叉汇率换算：
/// - EUR→X：直接查
/// - X→EUR：取倒数
/// - X→Y：(EUR→Y) / (EUR→X)
pub async fn get_rate(
    db: &DbConn,
    from_currency: &str,
    to_currency: &str,
    date: NaiveDate,
) -> Result<Decimal> {
    compute_rate(db, from_currency, to_currency, Some(date)).await
}

/// 查询最新汇率
pub async fn get_latest_rate(
    db: &DbConn,
    from_currency: &str,
    to_currency: &str,
) -> Result<Decimal> {
    compute_rate(db, from_currency, to_currency, None).await
}

/// 金额转换
///
/// `amount` 以 `from_currency` 计价，返回折算为 `to_currency` 后的金额。
/// 使用最新汇率。
pub async fn convert_amount(
    db: &DbConn,
    amount: Decimal,
    from_currency: &str,
    to_currency: &str,
) -> Result<Decimal> {
    let rate = get_latest_rate(db, from_currency, to_currency).await?;
    Ok((amount * rate).round_dp(4))
}

/// 内部：计算汇率（支持交叉汇率与日期过滤）
///
/// `date` 为 None 时取最新一条。
async fn compute_rate(
    db: &DbConn,
    from_currency: &str,
    to_currency: &str,
    date: Option<NaiveDate>,
) -> Result<Decimal> {
    // 相同币种汇率为 1
    if from_currency.eq_ignore_ascii_case(to_currency) {
        return Ok(Decimal::ONE);
    }

    let from_upper = from_currency.to_uppercase();
    let to_upper = to_currency.to_uppercase();

    // EUR → X：直接查
    if from_upper == EUR_CURRENCY {
        let rate = load_eur_rate(db, &to_upper, date).await?;
        return Ok(rate);
    }
    // X → EUR：取倒数
    if to_upper == EUR_CURRENCY {
        let rate = load_eur_rate(db, &from_upper, date).await?;
        if rate.is_zero() {
            return Err(Error::from("汇率为零，无法计算倒数"));
        }
        return Ok(Decimal::ONE / rate);
    }
    // X → Y：(EUR→Y) / (EUR→X)
    let eur_to_to = load_eur_rate(db, &to_upper, date).await?;
    let eur_to_from = load_eur_rate(db, &from_upper, date).await?;
    if eur_to_from.is_zero() {
        return Err(Error::from("基准汇率为零，无法计算交叉汇率"));
    }
    Ok(eur_to_to / eur_to_from)
}

/// 加载 EUR→currency 的汇率（date 为 None 取最新）
async fn load_eur_rate(
    db: &DbConn,
    currency: &str,
    date: Option<NaiveDate>,
) -> Result<Decimal> {
    if currency == EUR_CURRENCY {
        return Ok(Decimal::ONE);
    }

    let mut query = Entity::find()
        .filter(Column::FromCurrency.eq(EUR_CURRENCY))
        .filter(Column::ToCurrency.eq(currency));
    if let Some(d) = date {
        query = query.filter(Column::RateDate.eq(d));
    }
    let record = query
        .order_by_desc(Column::RateDate)
        .one(db)
        .await
        .map_err(|e| Error::from(e.to_string()))?
        .ok_or_else(|| Error::from(format!("未找到 EUR→{} 的汇率", currency)))?;

    Ok(record.rate.unwrap_or_default())
}

/// 解析 ECB 每日汇率 XML
///
/// 返回 (汇率日期, [(货币, 汇率)])。
/// ECB XML 格式（简化）：
/// ```xml
/// <Cube>
///   <Cube time='2024-01-15'>
///     <Cube currency='USD' rate='1.0871'/>
///     ...
///   </Cube>
/// </Cube>
/// ```
fn parse_ecb_xml(xml: &str) -> Result<(NaiveDate, Vec<(String, Decimal)>)> {
    // 提取日期：time='YYYY-MM-DD'
    let rate_date = extract_attribute(xml, "Cube", "time")
        .ok_or_else(|| Error::from("ECB XML 缺少 time 日期"))?
        .parse::<NaiveDate>()
        .map_err(|e| Error::from(format!("ECB 日期解析失败: {}", e)))?;

    // 提取所有 currency='XXX' rate='1.234' 配对
    let mut pairs = Vec::new();
    // 按行扫描所有包含 currency= 和 rate= 的片段
    for chunk in xml.split("currency=").skip(1) {
        // currency 值
        let currency = match extract_quoted_value(chunk) {
            Some(c) => c,
            None => continue,
        };
        // 在当前 chunk 中找 rate=
        let rate_str = match chunk.find("rate=").and_then(|pos| {
            extract_quoted_value(&chunk[pos + "rate=".len()..])
        }) {
            Some(r) => r,
            None => continue,
        };
        let rate = match Decimal::from_str_exact(&rate_str) {
            Ok(d) => d,
            Err(_) => continue,
        };
        pairs.push((currency, rate));
    }

    Ok((rate_date, pairs))
}

/// 从一段文本中提取首个被单引号或双引号包裹的值
fn extract_quoted_value(s: &str) -> Option<String> {
    let trimmed = s.trim_start();
    let bytes = trimmed.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let quote = bytes[0];
    if quote != b'\'' && quote != b'"' {
        return None;
    }
    let rest = &trimmed[1..];
    let end = rest.find(quote as char)?;
    Some(rest[..end].to_string())
}

/// 在 XML 中查找首个匹配 `tag` 且包含 `attr` 的属性值
fn extract_attribute(xml: &str, _tag: &str, attr: &str) -> Option<String> {
    let key = format!("{}=", attr);
    let pos = xml.find(&key)?;
    extract_quoted_value(&xml[pos + key.len()..])
}
