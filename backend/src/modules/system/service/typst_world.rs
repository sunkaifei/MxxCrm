//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! Typst 编译世界（World 实现）。
//!
//! v1.0 只实现了字体加载，`World::file` 直接返回 `NotFound`，导致 `#image(...)` 编译失败。
//! 本版本按设计文档 §18 扩展为**虚拟文件系统**：
//! - 编译期把素材（Logo / 底图 / 条码图 / 签章）以 `Bytes` 注入内存，**不落盘**；
//! - 无磁盘 IO、无并发冲突、无清理负担、天然只读安全（§18.4）；
//! - 同时新增 `typst-svg` 导出能力，支撑设计器「精确预览 / 分页预览」（§1.2 / §34.6）。

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::OnceLock;
use typst::compile;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_layout::PagedDocument;
use typst_pdf::{PdfOptions, pdf};

/// 全局字体缓存（启动时加载一次，后续复用）
static GLOBAL_FONTS: OnceLock<RwLock<Vec<Font>>> = OnceLock::new();

/// 初始化全局字体（启动时调用一次）
///
/// 加载 typst-assets 内置西文字体和 backend/assets/fonts/ 目录下的中文字体。
/// 重复调用时只有第一次生效（OnceLock 语义）。
pub fn init_fonts() {
    let mut fonts = Vec::new();

    // 1. typst-assets 内置字体（西文）
    for font_data in typst_assets::fonts() {
        for font in Font::iter(Bytes::new(font_data)) {
            fonts.push(font);
        }
    }

    // 2. 加载中文字体（从 backend/assets/fonts/ 目录）
    let cjk_font_paths = [
        "assets/fonts/SourceHanSansSC-Regular.otf",
        "assets/fonts/SourceHanSansSC-Bold.otf",
        "assets/fonts/SourceHanSerifSC-Regular.otf",
    ];
    for path in cjk_font_paths {
        if std::path::Path::new(path).exists() {
            match std::fs::read(path) {
                Ok(data) => {
                    for font in Font::iter(Bytes::new(data)) {
                        fonts.push(font);
                    }
                    log::info!("加载字体: {}", path);
                }
                Err(e) => log::warn!("字体加载失败 {}: {}", path, e),
            }
        } else {
            log::warn!("字体文件不存在: {}", path);
        }
    }

    log::info!("字体加载完成，共 {} 个字体", fonts.len());
    let _ = GLOBAL_FONTS.set(RwLock::new(fonts));
}

/// 获取字体列表的克隆
fn get_fonts() -> Vec<Font> {
    GLOBAL_FONTS
        .get()
        .map(|lock| lock.read().clone())
        .unwrap_or_default()
}

/// 虚拟文件系统：编译期可读取的素材
///
/// key = 相对于项目根的路径（如 `assets/logo_12.png`）
pub type AssetFs = HashMap<String, Vec<u8>>;

/// typst 编译所需的 World 实现
pub struct TypstWorld {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    main: FileId,
    source: Source,
    /// 虚拟素材文件（§18.2）
    assets: AssetFs,
}

impl TypstWorld {
    /// 创建一个新的 TypstWorld（不带素材）
    pub fn new(source_text: &str) -> Self {
        Self::with_assets(source_text, AssetFs::new())
    }

    /// 创建带虚拟素材的 TypstWorld
    pub fn with_assets(source_text: &str, assets: AssetFs) -> Self {
        let fonts = get_fonts();
        let library = Library::builder().build();
        let book = FontBook::from_fonts(&fonts);
        let vpath = VirtualPath::new("main.typ").expect("main.typ 是合法路径");
        let main = FileId::new(RootedPath::new(VirtualRoot::Project, vpath));
        let source = Source::new(main, source_text.into());

        Self {
            library: LazyHash::new(library),
            book: LazyHash::new(book),
            fonts,
            main,
            source,
            assets,
        }
    }

    /// 归一化虚拟路径：统一分隔符并去掉根前缀
    fn normalize(path: &str) -> String {
        let p = path.replace('\\', "/");
        p.trim_start_matches('/').to_string()
    }
}

impl World for TypstWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main {
            Ok(self.source.clone())
        } else {
            Err(FileError::NotFound(std::path::PathBuf::new()))
        }
    }

    /// 虚拟文件系统读取（§18.2）——替代 v1.0 的 `NotFound`
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let raw = id.vpath().as_rootless_path().to_string_lossy().to_string();
        let key = Self::normalize(&raw);
        match self.assets.get(&key) {
            Some(bytes) => Ok(Bytes::new(bytes.clone())),
            None => Err(FileError::NotFound(std::path::PathBuf::from(key))),
        }
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<Duration>) -> Option<Datetime> {
        None
    }
}

/// 编译 typst 源码为 PDF
///
/// 返回 PDF 字节数组，编译失败时返回错误消息字符串（多条以 `; ` 分隔）。
pub fn compile_to_pdf(source: &str) -> Result<Vec<u8>, String> {
    compile_to_pdf_with_assets(source, AssetFs::new())
}

/// 带虚拟素材编译为 PDF（含 Logo / 条码 / 底图）
pub fn compile_to_pdf_with_assets(source: &str, assets: AssetFs) -> Result<Vec<u8>, String> {
    let document = compile_document(source, assets)?;
    pdf(&document, &PdfOptions::default()).map_err(|errors| join_errors(errors))
}

/// 编译为分页文档（供 PDF / SVG / 页数统计共用）
pub fn compile_document(source: &str, assets: AssetFs) -> Result<PagedDocument, String> {
    let world = TypstWorld::with_assets(source, assets);
    let warned = compile::<PagedDocument>(&world);

    for warning in &warned.warnings {
        log::warn!("typst 警告: {}", warning.message);
    }

    warned.output.map_err(|errors| join_errors(errors))
}

/// 编译为每页一个 SVG 字符串（设计器精确预览 / 分页预览，§1.2 / §34.6）
pub fn compile_to_svg_pages(source: &str, assets: AssetFs) -> Result<Vec<String>, String> {
    let document = compile_document(source, assets)?;
    let opts = typst_svg::SvgOptions::default();
    Ok(document.pages().iter().map(|p| typst_svg::svg(p, &opts)).collect())
}

/// 单页尺寸（pt）：[ (width, height, page_index) ]
pub fn page_sizes(source: &str, assets: AssetFs) -> Result<Vec<(f64, f64)>, String> {
    let document = compile_document(source, assets)?;
    Ok(document
        .pages()
        .iter()
        .map(|p| {
            let s = p.frame.size();
            (s.x.to_pt(), s.y.to_pt())
        })
        .collect())
}

fn join_errors(errors: impl IntoIterator<Item = typst::diag::SourceDiagnostic>) -> String {
    let msgs: Vec<String> = errors
        .into_iter()
        .map(|e| {
            let mut m = e.message.to_string();
            if let Some(h) = e.hints.first() {
                m.push_str(&format!("（提示：{}）", h.v))
            }
            m
        })
        .collect();
    msgs.join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assets_with_png() -> AssetFs {
        // 1x1 红色 PNG
        const PNG: &[u8] = &[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
            0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
            0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8,
            0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D, 0xB0, 0x00, 0x00, 0x00,
            0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ];
        let mut fs = AssetFs::new();
        fs.insert("assets/logo_12.png".to_string(), PNG.to_vec());
        fs
    }

    #[test]
    fn test_world_reads_virtual_asset() {
        // §18.6：含 Logo 的最小模板编译成功
        let src = "#set page(paper: \"a4\")\n#place(dx: 10mm, dy: 10mm, image(\"/assets/logo_12.png\", width: 20mm))\n";
        let out = compile_to_pdf_with_assets(src, assets_with_png());
        assert!(out.is_ok(), "含素材编译失败: {:?}", out.err());
        assert!(!out.unwrap().is_empty());
    }

    #[test]
    fn test_missing_asset_returns_readable_error() {
        let src = "#set page(paper: \"a4\")\n#place(dx: 10mm, dy: 10mm, image(\"/assets/none.png\", width: 20mm))\n";
        let err = compile_to_pdf_with_assets(src, assets_with_png()).unwrap_err();
        assert!(!err.is_empty(), "缺失素材应返回可读错误而非 panic");
    }

    #[test]
    fn test_svg_export_page_count() {
        let src = "#set page(paper: \"a4\")\n第一页\n#pagebreak()\n第二页\n";
        let pages = compile_to_svg_pages(src, AssetFs::new()).unwrap();
        assert_eq!(pages.len(), 2);
        assert!(pages[0].contains("svg"));
    }

    #[test]
    fn test_page_sizes_a4() {
        let src = "#set page(paper: \"a4\")\n内容\n";
        let sizes = page_sizes(src, AssetFs::new()).unwrap();
        assert_eq!(sizes.len(), 1);
        // A4 = 595.28 x 841.89 pt，允许 1pt 误差
        assert!((sizes[0].0 - 595.28).abs() < 1.0, "宽 {}pt", sizes[0].0);
        assert!((sizes[0].1 - 841.89).abs() < 1.0, "高 {}pt", sizes[0].1);
    }
}
