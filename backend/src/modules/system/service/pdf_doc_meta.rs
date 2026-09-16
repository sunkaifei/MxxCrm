//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 可视化 PDF 模板设计器 —— 单据类型、字段树元数据、样例数据。
//!
//! 设计依据：设计文档 §6（数据源与字段树）、§32（设计时数据策略）。
//!
//! 关键约定：
//! - **硬规则（§32.4）**：样例数据绝不来源于真实客户单据，一律使用专用演示单据
//!   （客户名「示例客户 A」、商品名「示例商品 001」）。
//! - 样例数据必须满足五项质量要求（§32.5）：同构 / 压迫（长度 ≥ 真实典型值 120%）/
//!   稳定（禁随机）/ 可辨 / 可控。

use serde::Serialize;
use serde_json::{json, Map, Value};

// ============================================================================
// 单据类型
// ============================================================================

/// (code, name, 是否已接入真实数据上下文)
pub const DOC_TYPES: &[(&str, &str, bool)] = &[
    ("quotation", "报价单", true),
    ("order", "销售订单", true),
    ("contract", "合同", true),
    ("shipment", "发货单", true),
    ("delivery", "送货单", false),
    ("outbound", "出库单", true),
    ("inbound", "入库单", true),
    ("purchase", "采购单", true),
    ("invoice", "已开具发票", false),
    ("payment", "收款单", true),
    ("packing_list", "装箱单", false),
    ("proforma_invoice", "形式发票", false),
];

pub fn doc_name(code: &str) -> &'static str {
    DOC_TYPES
        .iter()
        .find(|(c, _, _)| *c == code)
        .map(|(_, n, _)| *n)
        .unwrap_or("未知单据")
}

pub fn doc_has_real_context(code: &str) -> bool {
    DOC_TYPES
        .iter()
        .find(|(c, _, _)| *c == code)
        .map(|(_, _, r)| *r)
        .unwrap_or(false)
}

/// 单据头节点 key（context 中单据主体的键名）
fn doc_root_key(code: &str) -> &'static str {
    match code {
        "quotation" => "quotation",
        "contract" => "contract",
        "shipment" => "shipment",
        "delivery" => "delivery",
        "outbound" => "outbound",
        "inbound" => "inbound",
        "purchase" => "purchase",
        "invoice" => "invoice",
        "payment" => "payment",
        "packing_list" => "packing_list",
        "proforma_invoice" => "proforma_invoice",
        _ => "order",
    }
}

// ============================================================================
// 字段树元数据（内建定义 = 单一真值；DB 表为可编辑覆盖层）
// ============================================================================

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub doc_type: String,
    pub group_code: &'static str,
    pub group_name: &'static str,
    pub field_path: String,
    pub field_name: &'static str,
    pub data_type: &'static str,
    pub sample: &'static str,
    pub sample_long: &'static str,
    pub sort: i32,
}

macro_rules! f {
    ($g:expr, $gn:expr, $p:expr, $n:expr, $t:expr, $s:expr, $sl:expr, $o:expr) => {
        FieldDef {
            doc_type: String::new(),
            group_code: $g,
            group_name: $gn,
            field_path: $p.to_string(),
            field_name: $n,
            data_type: $t,
            sample: $s,
            sample_long: $sl,
            sort: $o,
        }
    };
}

/// 公共分组（公司 / 客户 / 系统 / 自定义），所有单据类型共享
fn common_fields() -> Vec<FieldDef> {
    vec![
        f!("company", "公司", "company.name", "公司名称", "string", "示例科技有限公司", "示例科技有限公司（华东区总部）", 10),
        f!("company", "公司", "company.address", "公司地址", "string", "北京市朝阳区示例路 1 号", "北京市朝阳区示例路 1 号示例科技大厦 A 座 18 层 1801 室", 11),
        f!("company", "公司", "company.phone", "公司电话", "string", "010-88880000", "010-88880000 转 6666 / 010-88880001", 12),
        f!("company", "公司", "company.bank_name", "开户银行", "string", "中国银行北京分行", "中国银行股份有限公司北京市朝阳区示例支行营业部", 13),
        f!("company", "公司", "company.bank_account", "银行账号", "string", "1234 5678 9012 3456", "1234 5678 9012 3456 7890", 14),
        f!("company", "公司", "company.tax_no", "税号", "string", "91110105MA0000000X", "91110105MA0000000X", 15),
        f!("company", "公司", "company.logo_url", "公司 Logo", "image", "", "", 16),
        f!("customer", "客户", "customer.name", "客户名称", "string", "示例客户 A", "示例客户 A（华东区）国际贸易有限公司", 20),
        f!("customer", "客户", "customer.contact", "联系人", "string", "示例联系人 张三", "示例联系人 张三（采购部经理）", 21),
        f!("customer", "客户", "customer.phone", "联系电话", "string", "13800000000", "13800000000 / 021-66668888", 22),
        f!("customer", "客户", "customer.address", "客户地址", "string", "上海市浦东新区示例路 2 号", "上海市浦东新区示例路 2 号示例大厦 B 座 25 层", 23),
        f!("customer", "客户", "customer.tax_no", "客户税号", "string", "91310115MA1111111Y", "91310115MA1111111Y", 24),
        f!("sys", "系统", "sys.print_time", "打印时间", "date", "2026-09-15 14:30", "2026-09-15 14:30:59", 90),
        f!("sys", "系统", "sys.print_user", "打印人", "string", "示例用户", "示例用户（销售部）", 91),
        f!("sys", "系统", "sys.page", "当前页码", "number", "1", "999", 92),
        f!("sys", "系统", "sys.total_pages", "总页数", "number", "1", "99", 93),
        f!("sys", "系统", "sys.verify_url", "验真地址", "string", "https://example.com/verify", "https://example.com/verify/order/1024/a1b2c3d4", 94),
        f!("custom", "自定义字段", "custom.demo_text", "自定义文本", "string", "自定义内容", "自定义内容（这是一段用于验证超长换行的自定义字段文本）", 99),
    ]
}

/// 单据头字段（按 doc_type 生成）
fn header_fields(doc_type: &str) -> Vec<FieldDef> {
    let root = doc_root_key(doc_type);
    let p = |suffix: &str| -> String { format!("{}.{}", root, suffix) };
    let no_key = match doc_type {
        "quotation" => "quotation_no",
        "contract" => "contract_no",
        "shipment" => "shipment_no",
        "delivery" => "delivery_no",
        "outbound" => "outbound_no",
        "inbound" => "inbound_no",
        "purchase" => "po_no",
        "invoice" => "invoice_no",
        "payment" => "payment_no",
        "packing_list" => "packing_no",
        "proforma_invoice" => "proforma_no",
        _ => "order_no",
    };
    let title = match doc_type {
        "quotation" => "报价单编号",
        "contract" => "合同编号",
        "shipment" => "发货单号",
        "delivery" => "送货单号",
        "outbound" => "出库单号",
        "inbound" => "入库单号",
        "purchase" => "采购单号",
        "invoice" => "发票号码",
        "payment" => "收款单号",
        "packing_list" => "装箱单号",
        "proforma_invoice" => "形式发票号",
        _ => "订单编号",
    };

    let mk = |path: String, name: &'static str, dt: &'static str, s: &'static str, sl: &'static str, sort: i32| FieldDef {
        doc_type: String::new(),
        group_code: "header",
        group_name: "单据头",
        field_path: path,
        field_name: name,
        data_type: dt,
        sample: s,
        sample_long: sl,
        sort,
    };

    vec![
        mk(p(no_key), title, "string", "SO20260915001", "SO2026091500123456789-XYZ", 1),
        mk(p("doc_date"), "单据日期", "date", "2026-09-15", "2026-09-15", 2),
        mk(p("total_amount"), "单据总额", "money", "12345.67", "1234567890.12", 3),
        mk(p("total_amount_cn"), "总额（中文大写）", "string", "壹万贰仟叁佰肆拾伍元陆角柒分", "壹拾贰亿叁仟肆佰伍拾陆万柒仟捌佰玖拾元壹角贰分", 4),
        mk(p("currency"), "币种", "string", "CNY", "USD", 5),
        mk(p("total_qty"), "总数量", "number", "10", "999999", 6),
        mk(p("remark"), "备注", "string", "示例备注", "示例备注：本单据为设计器演示数据，用于验证超长备注在多行容器中的换行与溢出表现，请勿作为真实凭证使用。", 7),
        mk(p("status_text"), "单据状态", "string", "已审核", "已审核（待发货）", 8),
    ]
}

/// 明细字段（items 数组）
fn item_fields() -> Vec<FieldDef> {
    vec![
        f!("item", "明细", "item.index", "序号", "number", "1", "999", 40),
        f!("item", "明细", "item.product_name", "商品名称", "string", "示例商品 001", "示例商品 001 蓝牙无线降噪耳机 头戴式 长续航商务版 深空灰", 41),
        f!("item", "明细", "item.product_code", "商品编码", "string", "SKU-DEMO-001", "SKU-DEMO-001-2026-VIP", 42),
        f!("item", "明细", "item.spec", "规格型号", "string", "标准版", "标准版 / 商务定制版 / 出口包装版", 43),
        f!("item", "明细", "item.unit", "单位", "string", "个", "千克", 44),
        f!("item", "明细", "item.quantity", "数量", "number", "2", "999999", 45),
        f!("item", "明细", "item.price", "单价", "money", "100.00", "1234567.89", 46),
        f!("item", "明细", "item.amount", "金额", "money", "200.00", "1234567890.12", 47),
        f!("item", "明细", "item.tax_rate", "税率", "string", "13%", "13%", 48),
        f!("item", "明细", "item.remark", "明细备注", "string", "—", "示例明细备注：定制包装，需在出库前完成质检并附检验报告。", 49),
        f!("item", "明细", "item.hs_code", "HS Code", "string", "8518300000", "8518300000", 50),
        f!("item", "明细", "item.net_weight", "净重(kg)", "number", "1.2", "99999.99", 51),
        f!("item", "明细", "item.gross_weight", "毛重(kg)", "number", "1.5", "99999.99", 52),
        f!("item", "明细", "item.volume", "体积(CBM)", "number", "0.02", "999.999", 53),
    ]
}

/// 外贸单据专属字段（§21.4）
fn export_fields() -> Vec<FieldDef> {
    vec![
        f!("header", "单据头", "trade.terms", "贸易术语", "string", "FOB SHANGHAI", "CIF LOS ANGELES", 60),
        f!("header", "单据头", "trade.origin_country", "原产国", "string", "CHINA", "CHINA", 61),
        f!("header", "单据头", "trade.destination_port", "目的港", "string", "LOS ANGELES", "LOS ANGELES, CA, USA", 62),
        f!("header", "单据头", "trade.payment_terms", "付款方式", "string", "T/T 30% 预付", "T/T 30% DEPOSIT, BALANCE AGAINST B/L COPY", 63),
        f!("header", "单据头", "trade.lc_no", "信用证号", "string", "LC-DEMO-001", "LC-DEMO-001-2026-LONG", 64),
        f!("header", "单据头", "trade.shipping_mark", "唛头", "string", "MXX/DEMO/001", "MXX/DEMO/001\nLOS ANGELES\nC/NO.1-100\nMADE IN CHINA", 65),
        f!("header", "单据头", "trade.packages", "件数", "number", "100", "99999", 66),
        f!("header", "单据头", "trade.total_net_weight", "总净重(kg)", "number", "120.00", "999999.99", 67),
        f!("header", "单据头", "trade.total_gross_weight", "总毛重(kg)", "number", "150.00", "999999.99", 68),
        f!("header", "单据头", "trade.total_volume", "总体积(CBM)", "number", "2.000", "9999.999", 69),
        f!("header", "单据头", "trade.amount_en", "英文金额大写", "string", "SAY US DOLLARS ONE THOUSAND ONLY", "SAY US DOLLARS ONE BILLION TWO HUNDRED THIRTY-FOUR MILLION FIVE HUNDRED SIXTY-SEVEN THOUSAND EIGHT HUNDRED NINETY AND CENTS TWELVE ONLY", 70),
    ]
}

/// 生成某单据类型的完整字段定义
pub fn builtin_fields(doc_type: &str) -> Vec<FieldDef> {
    let mut all: Vec<FieldDef> = Vec::new();
    for mut d in header_fields(doc_type) {
        d.doc_type = doc_type.to_string();
        all.push(d);
    }
    for mut d in item_fields() {
        d.doc_type = doc_type.to_string();
        all.push(d);
    }
    for mut d in common_fields() {
        d.doc_type = doc_type.to_string();
        all.push(d);
    }
    if matches!(doc_type, "packing_list" | "proforma_invoice" | "invoice") {
        for mut d in export_fields() {
            d.doc_type = doc_type.to_string();
            all.push(d);
        }
    }
    all.sort_by_key(|d| (group_rank(d.group_code), d.sort));
    all
}

pub fn group_rank(code: &str) -> i32 {
    match code {
        "header" => 0,
        "item" => 1,
        "company" => 2,
        "customer" => 3,
        "trade" => 4,
        "sys" => 8,
        "custom" => 9,
        _ => 5,
    }
}

/// 支持样本预设的覆盖（§32.6）
pub const PRESETS: &[(&str, &str, &str)] = &[
    ("typical", "典型", "常规单据，中等长度、格式规范"),
    ("long", "超长", "商品名 40+ 字、地址 3 行、金额 1,234,567,890.12"),
    ("empty", "缺省", "可选字段全空"),
    ("zero", "零值", "金额 0.00、数量 0、折扣 0%"),
    ("mixed", "混合", "部分有值部分为空，最接近生产"),
    ("unicode", "特殊字符", "中英阿混排、全角括号、emoji、长英文单词"),
    ("longlist", "长明细", "明细 50 行，强制多页"),
];

// ============================================================================
// 字段树（供前端左侧数据源树）
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldNode {
    pub path: String,
    pub name: String,
    #[serde(rename = "type")]
    pub data_type: String,
    pub sample: String,
    pub sample_long: String,
    pub nullable: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldGroup {
    pub code: String,
    pub name: String,
    pub is_array: bool,
    // 前端数据源面板契约键名为 children（PanelDataSource 按 children 渲染）
    #[serde(rename = "children")]
    pub fields: Vec<FieldNode>,
}

/// 构建字段树。`overrides` 来自 `mxx_system_pdf_field_meta`（可为空）。
pub fn build_field_tree(
    doc_type: &str,
    overrides: &[(String, String, String, String, bool)], // (field_path, field_name, data_type, sample, nullable)
) -> Vec<FieldGroup> {
    let defs = builtin_fields(doc_type);
    let mut groups: Vec<FieldGroup> = Vec::new();
    for d in defs {
        let ov = overrides.iter().find(|o| o.0 == d.field_path);
        let mut g = groups
            .iter_mut()
            .find(|g| g.code == d.group_code);
        let node = FieldNode {
            path: d.field_path.to_string(),
            name: ov.map(|o| o.1.clone()).unwrap_or_else(|| d.field_name.to_string()),
            data_type: ov.map(|o| o.2.clone()).unwrap_or_else(|| d.data_type.to_string()),
            sample: ov
                .map(|o| o.3.clone())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| d.sample.to_string()),
            sample_long: d.sample_long.to_string(),
            nullable: ov.map(|o| o.4).unwrap_or(true),
        };
        match g {
            Some(ref mut g) => g.fields.push(node),
            None => groups.push(FieldGroup {
                code: d.group_code.to_string(),
                name: d.group_name.to_string(),
                is_array: d.group_code == "item",
                fields: vec![node],
            }),
        }
    }
    groups
}

// ============================================================================
// 样例数据（§32）
// ============================================================================

/// 生成样例上下文：完整 context 形状，可直接喂渲染器。
///
/// `preset` 见 §32.6；`item_rows` 控制明细行数（§32.7）。
pub fn sample_context(doc_type: &str, preset: &str, item_rows: i32) -> Value {
    let preset = if preset.is_empty() { "typical" } else { preset };
    let rows = if preset == "longlist" {
        item_rows.max(50)
    } else {
        item_rows.clamp(0, 200)
    };
    sample_context_with_rows(doc_type, preset, rows)
}

pub fn sample_context_with_rows(doc_type: &str, preset: &str, rows: i32) -> Value {
    let root = doc_root_key(doc_type);
    let long_mode = preset == "long";
    let pick = |tv: &str, lv: &str| -> String {
        if long_mode {
            lv.to_string()
        } else {
            tv.to_string()
        }
    };

    let no = match doc_type {
        "quotation" => "QT20260915001",
        "contract" => "HT20260915001",
        "shipment" => "FH20260915001",
        "delivery" => "SH20260915001",
        "outbound" => "CK20260915001",
        "inbound" => "RK20260915001",
        "purchase" => "PO20260915001",
        "invoice" => "FP20260915001",
        "payment" => "SK20260915001",
        "packing_list" => "PL20260915001",
        "proforma_invoice" => "PI20260915001",
        _ => "SO20260915001",
    };

    let mut items: Vec<Value> = Vec::new();
    let mut total = 0.0f64;
    let mut total_qty = 0.0f64;
    for i in 1..=rows.max(0) {
        let qty = if preset == "zero" { 0.0 } else { 2.0 };
        let price = if preset == "zero" { 0.0 } else { 100.0 * (i as f64) };
        let amount = qty * price;
        total += amount;
        total_qty += qty;
        let name = if preset == "empty" {
            String::new()
        } else {
            pick(
                &format!("示例商品 {:03}", i),
                &format!("示例商品 {:03} 蓝牙无线降噪耳机 头戴式 长续航商务版 深空灰", i),
            )
        };
        items.push(json!({
            "index": i,
            "product_name": name,
            "product_code": format!("SKU-DEMO-{:03}", i),
            "spec": pick("标准版", "标准版 / 商务定制版 / 出口包装版"),
            "unit": "个",
            "quantity": qty,
            "price": price,
            "amount": amount,
            "tax_rate": "13%",
            "remark": if preset == "empty" { String::new() } else { "—".to_string() },
            "hs_code": "8518300000",
            "net_weight": 1.2,
            "gross_weight": 1.5,
            "volume": 0.02
        }));
    }

    let amount_val = if preset == "zero" { 0.0 } else if long_mode { 1234567890.12 } else { total };
    let amount_cn = crate::modules::system::service::pdf_generator_service::amount_to_chinese(
        &rust_decimal::Decimal::try_from(amount_val).unwrap_or_default(),
    );

    let mut root_obj = Map::new();
    root_obj.insert("doc_date".to_string(), json!("2026-09-15"));
    root_obj.insert("total_amount".to_string(), json!(amount_val));
    root_obj.insert("total_amount_cn".to_string(), json!(amount_cn));
    root_obj.insert(
        "currency".to_string(),
        json!(if matches!(doc_type, "packing_list" | "proforma_invoice") { "USD" } else { "CNY" }),
    );
    root_obj.insert("total_qty".to_string(), json!(total_qty));
    root_obj.insert(
        "remark".to_string(),
        json!(if preset == "empty" {
            ""
        } else {
            "示例备注：本单据为设计器演示数据，请勿作为真实凭证使用。"
        }),
    );
    root_obj.insert("status_text".to_string(), json!("已审核"));
    root_obj.insert(
        match doc_type {
            "quotation" => "quotation_no",
            "contract" => "contract_no",
            "shipment" => "shipment_no",
            "delivery" => "delivery_no",
            "outbound" => "outbound_no",
            "inbound" => "inbound_no",
            "purchase" => "po_no",
            "invoice" => "invoice_no",
            "payment" => "payment_no",
            "packing_list" => "packing_no",
            "proforma_invoice" => "proforma_no",
            _ => "order_no",
        }
        .to_string(),
        json!(no),
    );

    // 预设收尾：empty / zero / mixed / unicode
    let mut ctx = json!({
        "company": {
            "name": pick("示例科技有限公司", "示例科技有限公司（华东区总部）"),
            "address": pick("北京市朝阳区示例路 1 号", "北京市朝阳区示例路 1 号示例科技大厦 A 座 18 层 1801 室"),
            "phone": pick("010-88880000", "010-88880000 转 6666 / 010-88880001"),
            "bank_name": "中国银行北京分行",
            "bank_account": "1234 5678 9012 3456",
            "tax_no": "91110105MA0000000X",
            "logo_url": ""
        },
        "customer": {
            "name": pick("示例客户 A", "示例客户 A（华东区）国际贸易有限公司"),
            "contact": pick("示例联系人 张三", "示例联系人 张三（采购部经理）"),
            "phone": "13800000000",
            "address": pick("上海市浦东新区示例路 2 号", "上海市浦东新区示例路 2 号示例大厦 B 座 25 层"),
            "tax_no": "91310115MA1111111Y"
        },
        "items": items,
        "sys": {
            "print_time": "2026-09-15 14:30",
            "print_user": "示例用户",
            "page": 1,
            "total_pages": 1,
            "verify_url": "https://example.com/verify"
        },
        "custom": { "demo_text": "自定义内容" }
    });
    ctx[root] = Value::Object(root_obj);

    // 外贸字段
    if matches!(doc_type, "packing_list" | "proforma_invoice" | "invoice") {
        ctx["trade"] = json!({
            "terms": pick("FOB SHANGHAI", "CIF LOS ANGELES"),
            "origin_country": "CHINA",
            "destination_port": pick("LOS ANGELES", "LOS ANGELES, CA, USA"),
            "payment_terms": pick("T/T 30% 预付", "T/T 30% DEPOSIT, BALANCE AGAINST B/L COPY"),
            "lc_no": "LC-DEMO-001",
            "shipping_mark": pick("MXX/DEMO/001", "MXX/DEMO/001\nLOS ANGELES\nC/NO.1-100\nMADE IN CHINA"),
            "packages": if preset == "zero" { 0 } else { 100 },
            "total_net_weight": 120.0,
            "total_gross_weight": 150.0,
            "total_volume": 2.0,
            "amount_en": "SAY US DOLLARS ONE THOUSAND TWO HUNDRED THIRTY-FOUR AND CENTS FIFTY-SIX ONLY"
        });
    }

    apply_preset(&mut ctx, preset);
    ctx
}

/// 按预设对上下文做整体变换（§32.6）
fn apply_preset(ctx: &mut Value, preset: &str) {
    match preset {
        "empty" => {
            // 可选字段全空（保留单据编号等必填项，避免完全失去版式参照）
            null_out(ctx, &["customer.address", "customer.contact", "customer.phone", "customer.tax_no"]);
            null_out(ctx, &["company.phone", "company.bank_name", "company.bank_account", "company.address"]);
            if let Some(items) = ctx.get_mut("items").and_then(|v| v.as_array_mut()) {
                for it in items.iter_mut() {
                    null_out(it, &["spec", "unit", "remark", "hs_code"]);
                }
            }
        }
        "zero" => {
            zero_out(ctx, &["customer.phone"]);
        }
        "mixed" => {
            if let Some(items) = ctx.get_mut("items").and_then(|v| v.as_array_mut()) {
                for (i, it) in items.iter_mut().enumerate() {
                    if i % 3 == 0 {
                        null_out(it, &["spec", "remark"]);
                    }
                    if i % 5 == 4 {
                        null_out(it, &["product_code"]);
                    }
                }
            }
            null_out(ctx, &["customer.address"]);
        }
        "unicode" => {
            ctx["customer"]["name"] = json!("示例客户 A（华东区）国際貿易有限公司 ＡＢＣ");
            ctx["customer"]["contact"] = json!("示例联系人 张三 🙂");
            ctx["company"]["address"] =
                json!("北京市朝阳区示例路 1 号　Ａ座 18F（东侧）— 近地铁 3 号线");
            if let Some(items) = ctx.get_mut("items").and_then(|v| v.as_array_mut()) {
                for (i, it) in items.iter_mut().enumerate() {
                    it["product_name"] = json!(format!(
                        "示例商品 {:03} SuperLongEnglishWordWithoutSpaces-AntiAliasing-Test",
                        i + 1
                    ));
                }
            }
        }
        _ => {}
    }
}

fn null_out(v: &mut Value, paths: &[&str]) {
    for p in paths {
        set_path(v, p, Value::Null);
    }
}

fn zero_out(v: &mut Value, paths: &[&str]) {
    for p in paths {
        set_path(v, p, json!("0"));
    }
}

fn set_path(root: &mut Value, path: &str, value: Value) {
    let mut cur = root;
    let segs: Vec<&str> = path.split('.').collect();
    for (i, s) in segs.iter().enumerate() {
        if i + 1 == segs.len() {
            if let Some(obj) = cur.as_object_mut() {
                obj.insert(s.to_string(), value);
            }
            return;
        }
        match cur.get_mut(*s) {
            Some(next) => cur = next,
            None => return,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_context_shape() {
        let ctx = sample_context("order", "typical", 3);
        assert_eq!(ctx["order"]["order_no"], json!("SO20260915001"));
        assert_eq!(ctx["items"].as_array().unwrap().len(), 3);
        assert!(ctx["company"]["name"].is_string());
    }

    #[test]
    fn test_longlist_preset_forces_50_rows() {
        let ctx = sample_context("order", "longlist", 3);
        assert_eq!(ctx["items"].as_array().unwrap().len(), 50);
    }

    #[test]
    fn test_long_preset_is_longer_than_typical() {
        let t = sample_context("order", "typical", 1);
        let l = sample_context("order", "long", 1);
        let tn = t["items"][0]["product_name"].as_str().unwrap().len();
        let ln = l["items"][0]["product_name"].as_str().unwrap().len();
        assert!(ln > tn * 2, "超长预设应显著长于典型：{} vs {}", ln, tn);
    }

    #[test]
    fn test_empty_preset_clears_optional() {
        let ctx = sample_context("order", "empty", 1);
        assert!(ctx["customer"]["address"].is_null());
        assert!(ctx["items"][0]["spec"].is_null());
    }

    #[test]
    fn test_zero_preset_zeroes_amounts() {
        let ctx = sample_context("order", "zero", 2);
        assert_eq!(ctx["order"]["total_amount"], json!(0.0));
        assert_eq!(ctx["items"][0]["amount"], json!(0.0));
    }

    #[test]
    fn test_sample_is_deterministic() {
        let a = sample_context("order", "typical", 3);
        let b = sample_context("order", "typical", 3);
        assert_eq!(a, b, "样例数据必须稳定，禁止随机（§32.5 第 3 条）");
    }

    #[test]
    fn test_field_tree_groups() {
        let tree = build_field_tree("order", &[]);
        let codes: Vec<&str> = tree.iter().map(|g| g.code.as_str()).collect();
        assert!(codes.contains(&"header"));
        assert!(codes.contains(&"item"));
        assert!(codes.contains(&"sys"));
        let header = tree.iter().find(|g| g.code == "header").unwrap();
        assert!(header.fields.iter().any(|f| f.path == "order.order_no"));
        let item = tree.iter().find(|g| g.code == "item").unwrap();
        assert!(item.is_array);
        assert!(item.fields.iter().any(|f| f.path == "item.product_name"));
    }

    #[test]
    fn test_field_tree_override_applies() {
        let ov = vec![(
            "order.order_no".to_string(),
            "订单号（自定义名）".to_string(),
            "string".to_string(),
            "OVERRIDE-001".to_string(),
            true,
        )];
        let tree = build_field_tree("order", &ov);
        let header = tree.iter().find(|g| g.code == "header").unwrap();
        let f = header.fields.iter().find(|f| f.path == "order.order_no").unwrap();
        assert_eq!(f.name, "订单号（自定义名）");
        assert_eq!(f.sample, "OVERRIDE-001");
    }

    #[test]
    fn test_all_doc_types_build_tree() {
        for (code, _, _) in DOC_TYPES {
            let tree = build_field_tree(code, &[]);
            assert!(!tree.is_empty(), "{} 字段树为空", code);
        }
    }
}
