//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! PDF 生成模块核心服务
//!
//! 负责：
//! 1. 组装各单据（报价单/订单/合同）的业务数据为 serde_json::Value
//! 2. 调用 pdf_compiler_service 生成 PDF
//! 3. 保存文件并写入数据库记录
//! 4. 提供审批通过后的自动生成 Hook

use crate::core::errors::error::{Error, Result};
use crate::modules::company::entity::company_account::{self as account_entity, Entity as CompanyAccount};
use crate::modules::company::entity::company_info::{self as company_entity, Entity as CompanyInfo};
use crate::modules::crm::entity::contract::{self as contract_entity, Entity as Contract};
use crate::modules::crm::entity::contract_payment_plan::{self as plan_entity, Entity as ContractPaymentPlan};
use crate::modules::crm::entity::customer::{self as customer_entity, Entity as Customer};
use crate::modules::sale::entity::order::{self as order_entity, Entity as Order};
use crate::modules::sale::entity::order_item::{self as order_item_entity, Entity as OrderItem};
use crate::modules::sale::entity::quotation::{self as quo_entity, Entity as Quotation};
use crate::modules::sale::entity::quotation_item::{self as quo_item_entity, Entity as QuotationItem};
use crate::modules::system::entity::pdf_template;
use crate::modules::system::model::pdf::{PdfRecordModel, PdfRecordSaveDTO, PdfTemplateModel};
use crate::modules::system::service::html_to_typst;
use crate::modules::system::service::pdf_compiler_service::{self, PdfPageOptions};

use chrono::Local;
use rust_decimal::Decimal;
use sea_orm::{ActiveValue::Set, ColumnTrait, DbConn, EntityTrait, QueryFilter, QueryOrder};
use serde_json::{json, Value};

// ============================ 结果结构体 ============================

/// PDF 生成结果
#[derive(Debug, Clone)]
pub struct PdfGenerateResult {
    /// PDF 记录ID
    pub record_id: i64,
    /// 文件访问URL
    pub file_url: String,
    /// 服务器存储路径
    pub file_path: String,
    /// 文件大小（字节）
    pub file_size: i64,
}

// ============================ 辅助函数 ============================

/// 从 PDF 模板 Model 构建页面配置
pub fn page_opts_from_template(t: &pdf_template::Model) -> PdfPageOptions {
    PdfPageOptions {
        paper_size: t.paper_size.clone().unwrap_or_else(|| "a4".to_string()),
        orientation: t.orientation.clone().unwrap_or_else(|| "portrait".to_string()),
        margin_top: t.margin_top.unwrap_or(20),
        margin_bottom: t.margin_bottom.unwrap_or(20),
        margin_left: t.margin_left.unwrap_or(40),
        margin_right: t.margin_right.unwrap_or(40),
        font_family: t.font_family.clone().unwrap_or_else(|| "Source Han Sans SC".to_string()),
    }
}

/// 币种代码转中文名称
pub fn currency_name(currency: i32) -> String {
    match currency {
        1 => "人民币".to_string(),
        2 => "美元".to_string(),
        3 => "欧元".to_string(),
        4 => "英镑".to_string(),
        5 => "日元".to_string(),
        6 => "港币".to_string(),
        _ => "人民币".to_string(),
    }
}

/// 人民币金额大写转换
///
/// 将 Decimal 金额转换为中文大写形式，如 `1234.56` → `壹仟贰佰叁拾肆元伍角陆分`
pub fn amount_to_chinese(amount: &Decimal) -> String {
    if amount.is_zero() {
        return "零元整".to_string();
    }

    let is_negative = amount.is_sign_negative();
    let abs = amount.abs();
    let s = abs.to_string();

    let parts: Vec<&str> = s.split('.').collect();
    let int_str = parts[0];
    let dec_str = if parts.len() > 1 {
        let mut d = parts[1].to_string();
        if d.len() == 1 {
            d.push('0');
        }
        d[..2].to_string()
    } else {
        "00".to_string()
    };

    let digits = ["零", "壹", "贰", "叁", "肆", "伍", "陆", "柒", "捌", "玖"];
    let units = ["", "拾", "佰", "仟"];
    let big_units = ["", "万", "亿", "兆"];

    // 转换整数部分
    let mut result = String::new();
    let int_chars: Vec<char> = int_str.chars().collect();
    let int_len = int_chars.len();

    // 每4位为一组，从右到左处理
    let group_count = (int_len + 3) / 4;

    for g in (0..group_count).rev() {
        let end = int_len - g * 4;
        let start = if end > 4 { end - 4 } else { 0 };
        let group: &[char] = &int_chars[start..end];
        let group_len = group.len();

        let mut group_str = String::new();
        let mut prev_zero = false;
        let mut all_zero = true;

        for (i, ch) in group.iter().enumerate() {
            let digit = ch.to_digit(10).unwrap_or(0);
            let pos = group_len - 1 - i;

            if digit == 0 {
                prev_zero = true;
            } else {
                all_zero = false;
                if prev_zero {
                    group_str.push_str("零");
                }
                group_str.push_str(digits[digit as usize]);
                group_str.push_str(units[pos]);
                prev_zero = false;
            }
        }

        if !all_zero {
            group_str.push_str(big_units[g]);
            result.push_str(&group_str);
        }
    }

    result.push_str("元");

    // 转换小数部分
    let dec_chars: Vec<char> = dec_str.chars().collect();
    let jiao = dec_chars.get(0).and_then(|c| c.to_digit(10)).unwrap_or(0);
    let fen = dec_chars.get(1).and_then(|c| c.to_digit(10)).unwrap_or(0);

    if jiao == 0 && fen == 0 {
        result.push_str("整");
    } else {
        if jiao > 0 {
            result.push_str(digits[jiao as usize]);
            result.push_str("角");
        } else if fen > 0 {
            result.push_str("零");
        }
        if fen > 0 {
            result.push_str(digits[fen as usize]);
            result.push_str("分");
        }
    }

    if is_negative {
        result = format!("负{}", result);
    }

    result
}

/// 将 HTML 富文本转换为 Typst 语法字符串
///
/// 委托给 `html_to_typst` 模块的 `convert_html_to_typst` 函数，
/// 支持常见 HTML 标签的 typst 转换，失败时回退为空字符串。
fn convert_html(html: &str) -> String {
    html_to_typst::convert_html_to_typst(html).unwrap_or_default()
}

/// Decimal 转 String（Option 安全处理）
fn decimal_to_string(d: &Option<Decimal>) -> String {
    d.map(|v| v.to_string()).unwrap_or_default()
}

/// NaiveDate 转 String（格式 %Y-%m-%d）
fn date_to_string(d: &Option<chrono::NaiveDate>) -> String {
    d.map(|v| v.format("%Y-%m-%d").to_string()).unwrap_or_default()
}

// ============================ 上下文构建 ============================

/// 组装报价单上下文数据
pub async fn build_quotation_context(db: &DbConn, quotation_id: i64) -> Result<Value> {
    // 1. 查询报价单
    let quotation = Quotation::find_by_id(quotation_id)
        .filter(quo_entity::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("报价单不存在"))?;

    // 2. 查询明细
    let items = QuotationItem::find()
        .filter(quo_item_entity::Column::QuotationId.eq(quotation_id))
        .filter(quo_item_entity::Column::Deleted.eq(0))
        .order_by_asc(quo_item_entity::Column::Sort)
        .all(db)
        .await?;

    // 3. 查询企业信息
    let company = CompanyInfo::find()
        .filter(company_entity::Column::Deleted.eq(0))
        .one(db)
        .await?;

    // 4. 查询默认银行账户
    let bank_account = CompanyAccount::find()
        .filter(account_entity::Column::IsDefault.eq(1))
        .filter(account_entity::Column::Deleted.eq(0))
        .one(db)
        .await?;

    // 5. 查询客户
    let customer = if let Some(cid) = quotation.customer_id {
        Customer::find_by_id(cid)
            .filter(customer_entity::Column::Deleted.eq(0))
            .one(db)
            .await?
    } else {
        None
    };

    // 组装 JSON
    let currency = quotation.currency.unwrap_or(1);
    let grand_total = quotation.grand_total.unwrap_or_default();
    let grand_total_cn = amount_to_chinese(&grand_total);

    let items_json: Vec<Value> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            json!({
                "index": i + 1,
                "product_name": item.product_name,
                "product_code": item.product_code,
                "spec": item.spec,
                "unit": item.unit,
                "quantity": decimal_to_string(&item.quantity),
                "unit_price": decimal_to_string(&item.unit_price),
                "discount_rate": decimal_to_string(&item.discount_rate),
                "tax_rate": decimal_to_string(&item.tax_rate),
                "subtotal": decimal_to_string(&item.subtotal),
            })
        })
        .collect();

    let company_json = company
        .as_ref()
        .map(|c| {
            json!({
                "company_name": c.company_name,
                "credit_code": c.credit_code,
                "legal_person": c.legal_person,
                "legal_phone": c.legal_phone,
                "register_address": c.register_address,
                "contact_phone": c.contact_phone,
                "contact_email": c.contact_email,
                "logo_url": c.logo_url,
            })
        })
        .unwrap_or(Value::Null);

    let bank_json = bank_account
        .as_ref()
        .map(|b| {
            json!({
                "bank_name": b.bank_name,
                "account_name": b.account_name,
                "account_number": b.account_number,
            })
        })
        .unwrap_or(Value::Null);

    let customer_json = customer
        .as_ref()
        .map(|c| {
            json!({
                "company_name": c.company_name,
                "person_name": c.person_name,
                "address": c.address,
                "personal_mobile": c.personal_mobile,
            })
        })
        .unwrap_or(Value::Null);

    let quotation_json = json!({
        "id": quotation.id,
        "quotation_no": quotation.quotation_no,
        "customer_id": quotation.customer_id,
        "customer_name": quotation.customer_name,
        "contact_name": quotation.contact_name,
        "title": quotation.title,
        "total_amount": decimal_to_string(&quotation.total_amount),
        "currency": currency_name(currency),
        "currency_code": currency,
        "tax_amount": decimal_to_string(&quotation.tax_amount),
        "discount_amount": decimal_to_string(&quotation.discount_amount),
        "grand_total": decimal_to_string(&quotation.grand_total),
        "valid_until": date_to_string(&quotation.valid_until),
        "quotation_date": date_to_string(&quotation.quotation_date),
        "payment_terms": quotation.payment_terms,
        "delivery_terms": quotation.delivery_terms,
        "delivery_date": date_to_string(&quotation.delivery_date),
        "bank_info": quotation.bank_info,
        "remark": quotation.remark,
    });

    Ok(json!({
        "quotation": quotation_json,
        "items": items_json,
        "company": company_json,
        "bank_account": bank_json,
        "customer": customer_json,
        "grand_total_cn": grand_total_cn,
    }))
}

/// 组装订单上下文数据
pub async fn build_order_context(db: &DbConn, order_id: i64) -> Result<Value> {
    // 1. 查询订单
    let order = Order::find_by_id(order_id)
        .filter(order_entity::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("订单不存在"))?;

    // 2. 查询明细
    let items = OrderItem::find()
        .filter(order_item_entity::Column::OrderId.eq(order_id))
        .filter(order_item_entity::Column::Deleted.eq(0))
        .order_by_asc(order_item_entity::Column::Sort)
        .all(db)
        .await?;

    // 3. 查询企业信息
    let company = CompanyInfo::find()
        .filter(company_entity::Column::Deleted.eq(0))
        .one(db)
        .await?;

    // 4. 查询默认银行账户
    let bank_account = CompanyAccount::find()
        .filter(account_entity::Column::IsDefault.eq(1))
        .filter(account_entity::Column::Deleted.eq(0))
        .one(db)
        .await?;

    // 5. 查询客户
    let customer = if let Some(cid) = order.customer_id {
        Customer::find_by_id(cid)
            .filter(customer_entity::Column::Deleted.eq(0))
            .one(db)
            .await?
    } else {
        None
    };

    // 组装 JSON
    let currency = order.currency.unwrap_or(1);
    let total_amount = order.total_amount.unwrap_or_default();
    let grand_total_cn = amount_to_chinese(&total_amount);

    let items_json: Vec<Value> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            json!({
                "index": i + 1,
                "product_name": item.product_name,
                "product_code": item.product_code,
                "spec": item.spec,
                "unit": item.unit,
                "quantity": decimal_to_string(&item.quantity),
                "unit_price": decimal_to_string(&item.unit_price),
                "discount_amount": decimal_to_string(&item.discount_amount),
                "tax_rate": decimal_to_string(&item.tax_rate),
                "amount": decimal_to_string(&item.amount),
            })
        })
        .collect();

    let company_json = company
        .as_ref()
        .map(|c| {
            json!({
                "company_name": c.company_name,
                "credit_code": c.credit_code,
                "legal_person": c.legal_person,
                "legal_phone": c.legal_phone,
                "register_address": c.register_address,
                "contact_phone": c.contact_phone,
                "contact_email": c.contact_email,
                "logo_url": c.logo_url,
            })
        })
        .unwrap_or(Value::Null);

    let bank_json = bank_account
        .as_ref()
        .map(|b| {
            json!({
                "bank_name": b.bank_name,
                "account_name": b.account_name,
                "account_number": b.account_number,
            })
        })
        .unwrap_or(Value::Null);

    let customer_json = customer
        .as_ref()
        .map(|c| {
            json!({
                "company_name": c.company_name,
                "person_name": c.person_name,
                "address": c.address,
                "personal_mobile": c.personal_mobile,
            })
        })
        .unwrap_or(Value::Null);

    let order_json = json!({
        "id": order.id,
        "order_no": order.order_no,
        "title": order.title,
        "customer_id": order.customer_id,
        "customer_name": order.customer_name,
        "contact_name": order.contact_name,
        "order_date": date_to_string(&order.order_date),
        "delivery_date": date_to_string(&order.delivery_date),
        "currency": currency_name(currency),
        "currency_code": currency,
        "product_amount": decimal_to_string(&order.product_amount),
        "discount_amount": decimal_to_string(&order.discount_amount),
        "tax_amount": decimal_to_string(&order.tax_amount),
        "total_amount": decimal_to_string(&order.total_amount),
        "buyer_company_name": order.buyer_company_name,
        "buyer_account_name": order.buyer_account_name,
        "buyer_bank_name": order.buyer_bank_name,
        "buyer_account_number": order.buyer_account_number,
        "seller_company_name": order.seller_company_name,
        "seller_bank_name": order.seller_bank_name,
        "seller_account_name": order.seller_account_name,
        "seller_account_number": order.seller_account_number,
        "remark": order.remark,
    });

    Ok(json!({
        "order": order_json,
        "items": items_json,
        "company": company_json,
        "bank_account": bank_json,
        "customer": customer_json,
        "grand_total_cn": grand_total_cn,
    }))
}

/// 组装合同上下文数据
pub async fn build_contract_context(db: &DbConn, contract_id: i64) -> Result<Value> {
    // 1. 查询合同
    let contract = Contract::find_by_id(contract_id)
        .filter(contract_entity::Column::Deleted.eq(0))
        .one(db)
        .await?
        .ok_or_else(|| Error::from("合同不存在"))?;

    // 2. 查询付款计划
    let payment_plans = ContractPaymentPlan::find()
        .filter(plan_entity::Column::ContractId.eq(contract_id))
        .filter(plan_entity::Column::Deleted.eq(0))
        .order_by_asc(plan_entity::Column::Sort)
        .all(db)
        .await?;

    // 3. 查询企业信息（甲方）
    let company = CompanyInfo::find()
        .filter(company_entity::Column::Deleted.eq(0))
        .one(db)
        .await?;

    // 4. 查询默认银行账户
    let bank_account = CompanyAccount::find()
        .filter(account_entity::Column::IsDefault.eq(1))
        .filter(account_entity::Column::Deleted.eq(0))
        .one(db)
        .await?;

    // 5. 查询客户（乙方）
    let customer = if let Some(cid) = contract.customer_id {
        Customer::find_by_id(cid)
            .filter(customer_entity::Column::Deleted.eq(0))
            .one(db)
            .await?
    } else {
        None
    };

    // 组装 JSON
    let total_amount = contract.total_amount.unwrap_or_default();
    let grand_total_cn = amount_to_chinese(&total_amount);

    // 合同描述 HTML → Typst
    let description_typst = contract
        .description
        .as_ref()
        .filter(|html| !html.trim().is_empty())
        .map(|html| convert_html(html))
        .unwrap_or_default();

    let payment_plans_json: Vec<Value> = payment_plans
        .iter()
        .enumerate()
        .map(|(i, p)| {
            json!({
                "index": i + 1,
                "stage_name": p.stage_name,
                "plan_amount": decimal_to_string(&p.plan_amount),
                "plan_date": date_to_string(&p.plan_date),
                "actual_date": date_to_string(&p.actual_date),
                "status": p.status,
                "remark": p.remark,
            })
        })
        .collect();

    let company_json = company
        .as_ref()
        .map(|c| {
            json!({
                "company_name": c.company_name,
                "credit_code": c.credit_code,
                "legal_person": c.legal_person,
                "legal_phone": c.legal_phone,
                "register_address": c.register_address,
                "contact_phone": c.contact_phone,
                "contact_email": c.contact_email,
                "logo_url": c.logo_url,
            })
        })
        .unwrap_or(Value::Null);

    let bank_json = bank_account
        .as_ref()
        .map(|b| {
            json!({
                "bank_name": b.bank_name,
                "account_name": b.account_name,
                "account_number": b.account_number,
            })
        })
        .unwrap_or(Value::Null);

    let customer_json = customer
        .as_ref()
        .map(|c| {
            json!({
                "company_name": c.company_name,
                "person_name": c.person_name,
                "address": c.address,
                "personal_mobile": c.personal_mobile,
            })
        })
        .unwrap_or(Value::Null);

    let contract_json = json!({
        "id": contract.id,
        "contract_no": contract.contract_no,
        "customer_id": contract.customer_id,
        "title": contract.title,
        "amount": decimal_to_string(&contract.amount),
        "total_amount": decimal_to_string(&contract.total_amount),
        "tax_amount": decimal_to_string(&contract.tax_amount),
        "sign_date": date_to_string(&contract.sign_date),
        "start_date": date_to_string(&contract.start_date),
        "end_date": date_to_string(&contract.end_date),
        "payment_terms": contract.payment_terms,
        "delivery_terms": contract.delivery_terms,
        "payment_method_type": contract.payment_method_type,
        "our_signer_name": contract.our_signer_name,
        "their_signer_name": contract.their_signer_name,
        "their_signer_phone": contract.their_signer_phone,
        "remark": contract.remark,
    });

    Ok(json!({
        "contract": contract_json,
        "company": company_json,
        "bank_account": bank_json,
        "customer": customer_json,
        "payment_plans": payment_plans_json,
        "grand_total_cn": grand_total_cn,
        "contract_description_typst": description_typst,
    }))
}

// ============================ PDF 生成 ============================

/// PDF 生成入口函数
///
/// 流程：
/// 1. 查询模板（template_id 为 None 时查默认模板）
/// 2. 根据 doc_type 调用对应的 build_xxx_context
/// 3. 调用 pdf_compiler_service::generate_pdf_bytes 生成 PDF 字节
/// 4. 构建文件路径并写入文件
/// 5. 写入 mxx_system_pdf_record 记录
/// 6. 更新业务单据的 pdf_file_url 字段
/// 7. 返回 PdfGenerateResult
pub async fn generate_pdf(
    db: &DbConn,
    doc_type: &str,
    doc_id: i64,
    template_id: Option<i64>,
    trigger_type: &str,
    operator_id: Option<i64>,
) -> Result<PdfGenerateResult> {
    // 1. 查询模板
    let template = if let Some(tid) = template_id {
        PdfTemplateModel::find_by_id(db, tid)
            .await?
            .ok_or_else(|| Error::from("PDF模板不存在"))?
    } else {
        PdfTemplateModel::find_default(db, doc_type)
            .await?
            .ok_or_else(|| Error::from(format!("未找到 {} 类型的默认PDF模板", doc_type)))?
    };

    // 2. 构建上下文
    let context = match doc_type {
        "quotation" => build_quotation_context(db, doc_id).await?,
        "order" => build_order_context(db, doc_id).await?,
        "contract" => build_contract_context(db, doc_id).await?,
        _ => return Err(Error::from(format!("不支持的单据类型: {}", doc_type))),
    };

    // 从上下文中提取单据编号
    let doc_no = match doc_type {
        "quotation" => context["quotation"]["quotation_no"]
            .as_str()
            .unwrap_or("quotation")
            .to_string(),
        "order" => context["order"]["order_no"]
            .as_str()
            .unwrap_or("order")
            .to_string(),
        "contract" => context["contract"]["contract_no"]
            .as_str()
            .unwrap_or("contract")
            .to_string(),
        _ => doc_type.to_string(),
    };

    // 3. 获取模板内容和页面配置
    let content = template.content.clone().unwrap_or_default();
    let header_content = template.header_content.clone();
    let footer_content = template.footer_content.clone();
    let opts = page_opts_from_template(&template);

    // 4. 生成 PDF 字节
    let pdf_bytes = pdf_compiler_service::generate_pdf_bytes(
        &content,
        &header_content,
        &footer_content,
        &context,
        &opts,
    )?;

    // 5. 构建文件路径
    let now = Local::now();
    let yyyymm = now.format("%Y%m").to_string();
    let timestamp = now.format("%Y%m%d%H%M%S").to_string();
    let safe_doc_no = sanitize_filename::sanitize(&doc_no);
    let file_name = format!("{}_{}.pdf", safe_doc_no, timestamp);

    let dir_path = format!("storage/upload/pdf/{}/{}", doc_type, yyyymm);
    let file_path = format!("{}/{}", dir_path, file_name);
    let file_url = format!("/upload/pdf/{}/{}/{}", doc_type, yyyymm, file_name);

    // 创建目录
    std::fs::create_dir_all(&dir_path)
        .map_err(|e| Error::from(format!("创建目录失败: {}", e)))?;

    // 写入文件
    std::fs::write(&file_path, &pdf_bytes)
        .map_err(|e| Error::from(format!("写入PDF文件失败: {}", e)))?;

    let file_size = pdf_bytes.len() as i64;

    // 6. 写入 PDF 记录
    let record_dto = PdfRecordSaveDTO {
        doc_type: Some(doc_type.to_string()),
        doc_id: Some(doc_id),
        doc_no: Some(doc_no.clone()),
        template_id: Some(template.id),
        template_name: template.name.clone(),
        file_name: Some(file_name.clone()),
        file_path: Some(file_path.clone()),
        file_url: Some(file_url.clone()),
        file_size: Some(file_size),
        page_count: None,
        trigger_type: Some(trigger_type.to_string()),
        status: Some(1),
        error_msg: None,
        create_by: operator_id,
    };
    let record_id = PdfRecordModel::insert(db, &record_dto).await?;

    // 7. 更新业务单据的 PDF URL 字段
    let _ = update_doc_pdf_url(db, doc_type, doc_id, &file_url, Some(template.id)).await;

    // 8. 返回结果
    Ok(PdfGenerateResult {
        record_id,
        file_url,
        file_path,
        file_size,
    })
}

/// 更新业务单据的 PDF URL 字段
pub async fn update_doc_pdf_url(
    db: &DbConn,
    doc_type: &str,
    doc_id: i64,
    file_url: &str,
    template_id: Option<i64>,
) -> Result<()> {
    match doc_type {
        "quotation" => {
            let payload = quo_entity::ActiveModel {
                pdf_file_url: Set(Some(file_url.to_string())),
                pdf_template_id: Set(template_id),
                ..Default::default()
            };
            Quotation::update_many()
                .set(payload)
                .filter(quo_entity::Column::Id.eq(doc_id))
                .filter(quo_entity::Column::Deleted.eq(0))
                .exec(db)
                .await?;
        }
        "order" => {
            let payload = order_entity::ActiveModel {
                pdf_file_url: Set(Some(file_url.to_string())),
                pdf_template_id: Set(template_id),
                ..Default::default()
            };
            Order::update_many()
                .set(payload)
                .filter(order_entity::Column::Id.eq(doc_id))
                .filter(order_entity::Column::Deleted.eq(0))
                .exec(db)
                .await?;
        }
        "contract" => {
            let payload = contract_entity::ActiveModel {
                file_url: Set(Some(file_url.to_string())),
                ..Default::default()
            };
            Contract::update_many()
                .set(payload)
                .filter(contract_entity::Column::Id.eq(doc_id))
                .filter(contract_entity::Column::Deleted.eq(0))
                .exec(db)
                .await?;
        }
        _ => return Err(Error::from(format!("不支持的单据类型: {}", doc_type))),
    }
    Ok(())
}

// ============================ 审批 Hook ============================

/// 报价单审批通过后自动生成 PDF
///
/// 使用 tokio::spawn 异步执行，失败只记日志不阻断审批流程
pub fn generate_for_quotation_approval(db: &DbConn, quotation_id: i64, operator_id: Option<i64>) {
    let db = db.clone();
    tokio::spawn(async move {
        match generate_pdf(&db, "quotation", quotation_id, None, "auto", operator_id).await {
            Ok(r) => log::info!(
                "报价单 {} 审批通过后自动生成PDF成功: {}",
                quotation_id,
                r.file_url
            ),
            Err(e) => log::error!(
                "报价单 {} 审批通过后自动生成PDF失败: {}",
                quotation_id,
                e
            ),
        }
    });
}

/// 销售订单审批通过后自动生成 PDF
///
/// 使用 tokio::spawn 异步执行，失败只记日志不阻断审批流程
pub fn generate_for_order_approval(db: &DbConn, order_id: i64, operator_id: Option<i64>) {
    let db = db.clone();
    tokio::spawn(async move {
        match generate_pdf(&db, "order", order_id, None, "auto", operator_id).await {
            Ok(r) => log::info!(
                "订单 {} 审批通过后自动生成PDF成功: {}",
                order_id,
                r.file_url
            ),
            Err(e) => log::error!(
                "订单 {} 审批通过后自动生成PDF失败: {}",
                order_id,
                e
            ),
        }
    });
}

/// 合同审批通过后自动生成 PDF
///
/// 使用 tokio::spawn 异步执行，失败只记日志不阻断审批流程
pub fn generate_for_contract_approval(db: &DbConn, contract_id: i64, operator_id: Option<i64>) {
    let db = db.clone();
    tokio::spawn(async move {
        match generate_pdf(&db, "contract", contract_id, None, "auto", operator_id).await {
            Ok(r) => log::info!(
                "合同 {} 审批通过后自动生成PDF成功: {}",
                contract_id,
                r.file_url
            ),
            Err(e) => log::error!(
                "合同 {} 审批通过后自动生成PDF失败: {}",
                contract_id,
                e
            ),
        }
    });
}
