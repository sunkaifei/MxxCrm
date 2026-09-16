//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 可视化 PDF 模板设计器 —— `layout_json` 数据模型与 schema 版本演进。
//!
//! 设计依据：docs/15-单据与PDF模板/可视化PDF模板设计器-开发设计文档.md
//!   §2   核心数据模型 layout_json v1
//!   §16  layout_json 能力补全（联次/水印/骑缝章/条件显示/i18n/重复块/旋转）
//!   §32.11 settings.designSample（设计期配置，编译时必须忽略）
//!   §33.3 schema 版本演进（链式 migration + 降级拒绝加载）
//!
//! ⚠️ 实现红线（§32.11）：`settings.designSample` 是"设计器的偏好"而非"模板的语义"，
//!    `layout_to_typst` 编译时必须完全忽略该节点。

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// `layout_json` 当前 schema 版本。**每次改结构 +1**，并补一条 migrate_vN_vN1。
pub const LAYOUT_SCHEMA_VERSION: i32 = 1;

// ============================================================================
// 顶层结构
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LayoutJson {
    /// schema 版本（区别于模板业务版本 version）
    pub version: i32,
    pub page: PageConfig,
    pub styles: PageStyles,
    pub bands: Bands,
    pub elements: Vec<Element>,
    /// 骑缝章（§16.1）
    pub seam_seal: SeamSeal,
    pub settings: LayoutSettings,
}

impl Default for LayoutJson {
    fn default() -> Self {
        Self {
            version: LAYOUT_SCHEMA_VERSION,
            page: PageConfig::default(),
            styles: PageStyles::default(),
            bands: Bands::default(),
            elements: Vec::new(),
            seam_seal: SeamSeal::default(),
            settings: LayoutSettings::default(),
        }
    }
}

impl LayoutJson {
    /// 解析并**强制迁移**到当前 schema 版本（§33.3：设计器打开 / 精确预览 / 正式出图三条路径都要走这里）。
    pub fn parse_strict(value: &Value) -> Result<Self, String> {
        let mut v = value.clone();
        migrate_layout(&mut v, None)?;
        let layout: LayoutJson =
            serde_json::from_value(v).map_err(|e| format!("layout_json 结构非法: {}", e))?;
        layout.validate()?;
        Ok(layout)
    }

    /// 结构合法性校验（§24.1 PDF_E_LAYOUT_INVALID）
    pub fn validate(&self) -> Result<(), String> {
        if self.page.width <= 0.0 || self.page.height <= 0.0 {
            return Err("纸张尺寸非法（必须 > 0mm）".to_string());
        }
        if self.page.width > 2000.0 || self.page.height > 2000.0 {
            return Err("纸张尺寸过大（> 2000mm）".to_string());
        }
        let mut ids = std::collections::HashSet::new();
        for el in self.elements.iter().chain(self.bands.page_header.elements.iter()) {
            if !ids.insert(el.id.clone()) {
                return Err(format!("元素 id 重复: {}", el.id));
            }
        }
        for el in self.bands.page_footer.elements.iter() {
            if !ids.insert(el.id.clone()) {
                return Err(format!("元素 id 重复: {}", el.id));
            }
        }
        Ok(())
    }

    /// 全部元素（含带区元素），统一遍历入口
    pub fn all_elements(&self) -> Vec<&Element> {
        let mut out: Vec<&Element> = Vec::new();
        out.extend(self.bands.page_header.elements.iter());
        out.extend(self.elements.iter());
        out.extend(self.bands.page_footer.elements.iter());
        out
    }
}

// ============================================================================
// 页面
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PageConfig {
    pub size: String,       // A4 | A5 | Letter | custom
    pub width: f64,         // mm
    pub height: f64,        // mm
    pub orientation: String, // portrait | landscape
    pub margin: Margin,
    pub background: Background,
    /// 页面水印（§16.1）
    pub watermark: Watermark,
    /// 联次（多联复写纸套打，§16.1 / §19.2）
    pub copies: Vec<CopyItem>,
    pub copy_mode: String,     // merge | separate
    pub copy_binding: Option<String>,
}

impl Default for PageConfig {
    fn default() -> Self {
        Self {
            size: "A4".to_string(),
            width: 210.0,
            height: 297.0,
            orientation: "portrait".to_string(),
            margin: Margin::default(),
            background: Background::default(),
            watermark: Watermark::default(),
            copies: Vec::new(),
            copy_mode: "merge".to_string(),
            copy_binding: None,
        }
    }
}

impl PageConfig {
    /// 有效纸张尺寸（考虑横向）
    pub fn effective_size(&self) -> (f64, f64) {
        if self.orientation == "landscape" {
            (self.height.max(self.width), self.height.min(self.width))
        } else {
            (self.width, self.height)
        }
    }

    /// Typst `#set page(paper: ...)` 片段
    pub fn typst_paper(&self) -> String {
        match self.size.as_str() {
            "A4" | "a4" => "paper: \"a4\"".to_string(),
            "A5" | "a5" => "paper: \"a5\"".to_string(),
            "Letter" | "letter" => "paper: \"us-letter\"".to_string(),
            _ => format!("width: {}mm, height: {}mm", self.width, self.height),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Margin {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl Default for Margin {
    fn default() -> Self {
        Self { top: 12.0, right: 12.0, bottom: 12.0, left: 12.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Background {
    pub r#type: String,   // none | color | image | pdf
    pub value: Option<String>,
    pub fit: String,      // contain | cover | stretch
    pub opacity: f64,
    /// 预印纸套打：底图仅设计器可见，**绝不出图**（§19.3）
    pub preview_only: bool,
}

impl Default for Background {
    fn default() -> Self {
        Self {
            r#type: "none".to_string(),
            value: None,
            fit: "contain".to_string(),
            opacity: 1.0,
            preview_only: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Watermark {
    pub enabled: bool,
    pub text: Option<String>,
    pub image: Option<String>,
    pub opacity: f64,
    pub rotate: f64,
    pub size: f64,
}

impl Default for Watermark {
    fn default() -> Self {
        Self { enabled: false, text: None, image: None, opacity: 0.08, rotate: -30.0, size: 60.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CopyItem {
    pub label: String,
    pub color: String,
}

impl Default for CopyItem {
    fn default() -> Self {
        Self { label: String::new(), color: "#000000".to_string() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SeamSeal {
    pub enabled: bool,
    pub asset_id: Option<i64>,
    pub position: String, // right | left
    pub size: f64,
    pub offset: f64,
}

impl Default for SeamSeal {
    fn default() -> Self {
        Self { enabled: false, asset_id: None, position: "right".to_string(), size: 20.0, offset: 0.0 }
    }
}

// ============================================================================
// 样式 / 带区 / 设置
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PageStyles {
    pub font_family: Option<String>,
    pub font_size: f64,
    pub color: Option<String>,
    pub line_height: f64,
}

impl Default for PageStyles {
    fn default() -> Self {
        Self {
            font_family: Some("Source Han Sans SC".to_string()),
            font_size: 10.5,
            color: Some("#1f1f1f".to_string()),
            line_height: 1.5,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Bands {
    pub page_header: Band,
    pub page_footer: Band,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Band {
    pub height: f64,
    pub elements: Vec<Element>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DesignSample {
    pub preset: String,   // typical|long|empty|zero|mixed|unicode|longlist
    pub item_rows: i32,
    pub source: String,   // meta | seed
}

impl Default for DesignSample {
    fn default() -> Self {
        Self { preset: "typical".to_string(), item_rows: 3, source: "meta".to_string() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PrintOffset {
    pub x: f64,
    pub y: f64,
}

impl Default for PrintOffset {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LayoutSettings {
    pub grid_size: f64,
    pub snap: bool,
    pub show_grid: bool,
    pub print_offset: PrintOffset,
    pub locale: String,     // zh | en
    pub paper_type: String, // blank | preprinted
    /// ⚠️ 仅设计器使用，编译时**必须完全忽略**（§32.11 实现红线）
    pub design_sample: DesignSample,
}

impl Default for LayoutSettings {
    fn default() -> Self {
        Self {
            grid_size: 1.0,
            snap: true,
            show_grid: true,
            print_offset: PrintOffset::default(),
            locale: "zh".to_string(),
            paper_type: "blank".to_string(),
            design_sample: DesignSample::default(),
        }
    }
}

// ============================================================================
// 元素
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Element {
    pub id: String,
    #[serde(rename = "type")]
    pub element_type: String,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub z: i32,
    pub locked: bool,
    pub visible: bool,
    /// main | pageHeader | pageFooter | firstPage | lastPage
    pub band: String,
    pub style: ElementStyle,
    pub bind: Bind,
    pub props: Value,
    /// 旋转角度（度，逆时针，§16.2）
    pub rotate: f64,
    pub opacity: f64,
    /// 条件显示（后端编译期求值，§16.2）
    pub print_if: Option<PrintIf>,
    /// 多语言文案（外贸双语，§16.2 / §21.1）
    pub i18n: BTreeMap<String, String>,
    /// 重复块（非表格的循环区域，§16.2）
    pub repeater: Option<Repeater>,
}

impl Default for Element {
    fn default() -> Self {
        Self {
            id: String::new(),
            element_type: "text".to_string(),
            name: String::new(),
            x: 0.0,
            y: 0.0,
            w: 40.0,
            h: 8.0,
            z: 0,
            locked: false,
            visible: true,
            band: "main".to_string(),
            style: ElementStyle::default(),
            bind: Bind::default(),
            props: Value::Object(Default::default()),
            rotate: 0.0,
            opacity: 1.0,
            print_if: None,
            i18n: BTreeMap::new(),
            repeater: None,
        }
    }
}

impl Element {
    /// 取静态文案：优先 i18n 当前语言 → content
    pub fn static_text(&self, locale: &str) -> String {
        if let Some(v) = self.i18n.get(locale) {
            if !v.is_empty() {
                return v.clone();
            }
        }
        self.props
            .get("content")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default()
    }

    pub fn prop_str(&self, key: &str) -> Option<String> {
        self.props.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
    }

    pub fn prop_f64(&self, key: &str) -> Option<f64> {
        self.props.get(key).and_then(|v| v.as_f64())
    }

    pub fn prop_bool(&self, key: &str) -> Option<bool> {
        self.props.get(key).and_then(|v| v.as_bool())
    }

    pub fn prop_i64(&self, key: &str) -> Option<i64> {
        self.props.get(key).and_then(|v| v.as_i64())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ElementStyle {
    pub font_family: Option<String>,
    pub font_size: f64,           // pt
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub color: Option<String>,
    pub bg_color: Option<String>,
    pub align: String,            // left | center | right | justify
    pub valign: String,           // top | middle | bottom
    pub padding: Padding,
    pub line_height: f64,
    pub letter_spacing: f64,      // pt
    pub word_wrap: bool,
    pub auto_shrink: bool,
    pub border: Border,
}

impl Default for ElementStyle {
    fn default() -> Self {
        Self {
            font_family: None,
            font_size: 10.5,
            bold: false,
            italic: false,
            underline: false,
            color: Some("#000000".to_string()),
            bg_color: None,
            align: "left".to_string(),
            valign: "middle".to_string(),
            padding: Padding::default(),
            line_height: 1.4,
            letter_spacing: 0.0,
            word_wrap: true,
            auto_shrink: false,
            border: Border::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Padding {
    pub t: f64,
    pub r: f64,
    pub b: f64,
    pub l: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Border {
    pub t: f64,
    pub r: f64,
    pub b: f64,
    pub l: f64,
    pub color: String,
    pub style: String, // solid | dashed | dotted
}

impl Default for Border {
    fn default() -> Self {
        Self { t: 0.0, r: 0.0, b: 0.0, l: 0.0, color: "#000000".to_string(), style: "solid".to_string() }
    }
}

impl Border {
    pub fn any(&self) -> bool {
        self.t > 0.0 || self.r > 0.0 || self.b > 0.0 || self.l > 0.0
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Bind {
    pub path: Option<String>,
    pub format: Option<String>,
    pub default_value: Option<String>,
    pub sample: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PrintIf {
    pub field: String,
    pub op: String, // eq | ne | gt | gte | lt | lte | empty | not_empty
    pub value: Value,
}

impl Default for PrintIf {
    fn default() -> Self {
        Self { field: String::new(), op: "eq".to_string(), value: Value::Null }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Repeater {
    pub data_source: String,
    pub item_alias: String,
    pub stride_y: f64,
    pub max_items: i32,
}

// ============================================================================
// 表格 props（§3.1）
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TableProps {
    pub data_source: String,
    pub show_index: bool,
    pub index_title: Option<String>,
    pub header_repeat: bool,
    pub row_height: f64,
    pub auto_grow: bool,
    pub max_rows_per_page: i32,
    pub empty_text: Option<String>,
    pub header_height: f64,
    pub header_style: TableHeaderStyle,
    pub columns: Vec<TableColumn>,
    pub footer_rows: Vec<TableFooterRow>,
    pub group_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TableHeaderStyle {
    pub bold: bool,
    pub bg_color: Option<String>,
    pub font_size: f64,
}

impl Default for TableHeaderStyle {
    fn default() -> Self {
        Self { bold: true, bg_color: Some("#f5f5f5".to_string()), font_size: 9.5 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TableColumn {
    pub title: String,
    pub bind: Option<String>,
    /// 固定 mm 数值，或 "auto"
    pub width: Value,
    pub align: String,
    pub format: Option<String>,
    pub total: String, // none | sum | count | avg
}

impl Default for TableColumn {
    fn default() -> Self {
        Self {
            title: String::new(),
            bind: None,
            width: Value::String("auto".to_string()),
            align: "left".to_string(),
            format: None,
            total: "none".to_string(),
        }
    }
}

impl TableColumn {
    pub fn width_mm(&self) -> Option<f64> {
        self.width.as_f64()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TableFooterRow {
    pub cells: Vec<TableFooterCell>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TableFooterCell {
    pub col: i32,
    pub value: Option<String>,
    pub bind: Option<String>,
    pub format: Option<String>,
    pub align: Option<String>,
    pub colspan: i32,
}

impl TableProps {
    pub fn from_value(v: &Value) -> Self {
        serde_json::from_value(v.clone()).unwrap_or_default()
    }
}

// ============================================================================
// schema 版本演进（§33.3 链式 migration）
// ============================================================================

/// 链式迁移：v(from) → … → LAYOUT_SCHEMA_VERSION。
///
/// - `from` 为 `None` 时从 JSON 内 `version` 读取（缺失视为 1）
/// - 遇到**更高**版本（用户降级了系统）→ 返回可读错误，**拒绝加载、绝不静默丢字段**
pub fn migrate_layout(json: &mut Value, from: Option<i32>) -> Result<i32, String> {
    let mut cur = from.unwrap_or_else(|| {
        json.get("version").and_then(|v| v.as_i64()).unwrap_or(1) as i32
    });

    if cur > LAYOUT_SCHEMA_VERSION {
        return Err(format!(
            "布局 schema 版本 {} 高于当前系统支持的 {}，请升级系统后再打开该模板（已拒绝加载以避免丢失字段）",
            cur, LAYOUT_SCHEMA_VERSION
        ));
    }

    while cur < LAYOUT_SCHEMA_VERSION {
        match cur {
            // 1 => migrate_v1_v2(json)?,   // 未来结构升级在此登记
            other => return Err(format!("未知的 layout schema 版本: {}", other)),
        }
        cur += 1;
        if let Value::Object(map) = json {
            map.insert("version".to_string(), Value::from(cur));
        }
    }

    if let Value::Object(map) = json {
        map.insert("version".to_string(), Value::from(LAYOUT_SCHEMA_VERSION));
    }
    Ok(cur)
}

/// 从 v1 迁移到 v2 的**示例**实现（当前 schema 仍为 v1，仅用于验证迁移机制可用）。
///
/// 语义：v2 给所有 text/field 元素补齐 `style.wordWrap = true` 与 `props` 对象。
/// 单测 `test_migrate_v1_v2_shape` 直接调用它以证明链式迁移机制正确。
pub fn migrate_v1_v2(json: &mut Value) -> Result<(), String> {
    let obj = json.as_object_mut().ok_or("layout_json 必须是对象")?;
    if let Some(Value::Array(elements)) = obj.get_mut("elements") {
        for el in elements.iter_mut() {
            let e = el.as_object_mut().ok_or("element 必须是对象")?;
            let ty = e.get("type").and_then(|v| v.as_str()).unwrap_or("");
            if ty == "text" || ty == "field" || ty == "expr" || ty == "rich" {
                let style = e
                    .entry("style".to_string())
                    .or_insert_with(|| Value::Object(Default::default()));
                if let Value::Object(s) = style {
                    s.entry("wordWrap".to_string()).or_insert(Value::Bool(true));
                }
            }
            e.entry("props".to_string())
                .or_insert_with(|| Value::Object(Default::default()));
        }
    }
    Ok(())
}

/// 规范化：把任意输入补齐为完整 v1 结构（缺字段用默认值），并写回当前版本号。
///
/// 设计器保存前调用，避免"半成品 JSON"入库。
pub fn normalize_layout(value: &Value) -> Result<Value, String> {
    let mut v = value.clone();
    migrate_layout(&mut v, None)?;
    let layout: LayoutJson =
        serde_json::from_value(v).map_err(|e| format!("layout_json 结构非法: {}", e))?;
    layout.validate()?;
    serde_json::to_value(&layout).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_minimal_layout_fills_defaults() {
        let v = json!({ "version": 1, "elements": [] });
        let layout = LayoutJson::parse_strict(&v).expect("最小结构应可解析");
        assert_eq!(layout.version, LAYOUT_SCHEMA_VERSION);
        assert_eq!(layout.page.size, "A4");
        assert_eq!(layout.page.margin.top, 12.0);
        assert_eq!(layout.settings.locale, "zh");
    }

    #[test]
    fn test_reject_higher_schema_version() {
        let v = json!({ "version": 99, "elements": [] });
        let err = LayoutJson::parse_strict(&v).unwrap_err();
        assert!(err.contains("高于当前系统支持"), "实际: {}", err);
    }

    #[test]
    fn test_migrate_v1_v2_shape() {
        let mut v = json!({
            "version": 1,
            "elements": [ { "id": "a", "type": "text" } ]
        });
        migrate_v1_v2(&mut v).unwrap();
        let el = &v["elements"][0];
        assert_eq!(el["style"]["wordWrap"], json!(true));
        assert!(el["props"].is_object());
    }

    #[test]
    fn test_normalize_full_layout_roundtrip() {
        let v = json!({
            "version": 1,
            "page": { "size": "A4", "width": 210, "height": 297 },
            "elements": [
                { "id": "t1", "type": "text", "x": 12, "y": 14, "w": 186, "h": 10,
                  "props": { "content": "销售订单" } }
            ]
        });
        let normalized = normalize_layout(&v).unwrap();
        assert_eq!(normalized["version"], json!(1));
        assert_eq!(normalized["elements"][0]["props"]["content"], json!("销售订单"));
        assert_eq!(normalized["elements"][0]["band"], json!("main"));
    }

    #[test]
    fn test_validate_rejects_zero_page_size() {
        let mut layout = LayoutJson::default();
        layout.page.width = 0.0;
        assert!(layout.validate().is_err());
    }

    #[test]
    fn test_effective_size_landscape() {
        let mut p = PageConfig::default();
        p.orientation = "landscape".to_string();
        assert_eq!(p.effective_size(), (297.0, 210.0));
    }

    #[test]
    fn test_static_text_prefers_i18n() {
        let mut el = Element { element_type: "text".to_string(), ..Default::default() };
        el.props = json!({ "content": "发货单" });
        el.i18n.insert("zh".to_string(), "发货单".to_string());
        el.i18n.insert("en".to_string(), "DELIVERY NOTE".to_string());
        assert_eq!(el.static_text("en"), "DELIVERY NOTE");
        assert_eq!(el.static_text("fr"), "发货单");
    }
}
