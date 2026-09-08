//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use std::sync::OnceLock;
use parking_lot::RwLock;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst::compile;
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

/// typst 编译所需的 World 实现
pub struct TypstWorld {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    main: FileId,
    source: Source,
}

impl TypstWorld {
    /// 创建一个新的 TypstWorld，传入 typst 源码文本
    pub fn new(source_text: &str) -> Self {
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
        }
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

    fn file(&self, _id: FileId) -> FileResult<Bytes> {
        Err(FileError::NotFound(std::path::PathBuf::new()))
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
    let world = TypstWorld::new(source);
    let warned = compile::<PagedDocument>(&world);

    match warned.output {
        Ok(document) => {
            // 打印警告
            for warning in &warned.warnings {
                log::warn!("typst 警告: {}", warning.message);
            }

            // 导出 PDF
            pdf(&document, &PdfOptions::default())
                .map_err(|errors| {
                    let msgs: Vec<String> = errors
                        .into_iter()
                        .map(|e| e.message.to_string())
                        .collect();
                    msgs.join("; ")
                })
        }
        Err(errors) => {
            let msgs: Vec<String> = errors
                .into_iter()
                .map(|e| e.message.to_string())
                .collect();
            Err(msgs.join("; "))
        }
    }
}
