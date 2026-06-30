//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
//! 文件安全校验模块
//! 提供多层文件校验：扩展名白名单、文件大小、扩展名与MIME匹配、文件头魔数、图片内容、SHA-256 哈希
//!

use std::collections::HashMap;
use std::sync::OnceLock;

use sha2::{Digest, Sha256};

use crate::core::errors::error::{Error, Result};

/// 图片最大尺寸：10MB
pub const MAX_IMAGE_SIZE: i64 = 10 * 1024 * 1024;
/// 文档最大尺寸：50MB
pub const MAX_DOC_SIZE: i64 = 50 * 1024 * 1024;

/// 业务类型配置：允许的扩展名白名单 + 是否公开访问
pub struct EntityTypeConfig {
    /// 允许的扩展名白名单（小写）
    pub allowed_exts: &'static [&'static str],
    /// 是否公开访问
    pub is_public: bool,
}

/// product: 商品图片（公开）
const PRODUCT_EXTS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg"];
/// avatar: 头像（公开）
const AVATAR_EXTS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "webp"];
/// contract: 合同
const CONTRACT_EXTS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "webp", "pdf", "doc", "docx"];
/// invoice: 发票
const INVOICE_EXTS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "webp", "pdf"];
/// quotation: 报价单
const QUOTATION_EXTS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "webp", "pdf", "doc", "docx", "xls", "xlsx"];
/// payment: 付款凭证
const PAYMENT_EXTS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "webp", "pdf"];
/// common: 通用附件
const COMMON_EXTS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "webp", "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt"];

/// 图片类扩展名集合（用于判断走图片大小限制还是文档大小限制）
const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg"];

/// 全局 entity_type 配置表（一次性初始化）
static ENTITY_TYPE_MAP: OnceLock<HashMap<&'static str, EntityTypeConfig>> = OnceLock::new();

/// 获取 entity_type 配置表
fn entity_type_map() -> &'static HashMap<&'static str, EntityTypeConfig> {
    ENTITY_TYPE_MAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("product", EntityTypeConfig { allowed_exts: PRODUCT_EXTS, is_public: true });
        m.insert("avatar", EntityTypeConfig { allowed_exts: AVATAR_EXTS, is_public: true });
        m.insert("contract", EntityTypeConfig { allowed_exts: CONTRACT_EXTS, is_public: false });
        m.insert("invoice", EntityTypeConfig { allowed_exts: INVOICE_EXTS, is_public: false });
        m.insert("quotation", EntityTypeConfig { allowed_exts: QUOTATION_EXTS, is_public: false });
        m.insert("payment", EntityTypeConfig { allowed_exts: PAYMENT_EXTS, is_public: false });
        m.insert("common", EntityTypeConfig { allowed_exts: COMMON_EXTS, is_public: false });
        m
    })
}

/// 根据 entity_type 获取配置
pub fn get_entity_type_config(entity_type: &str) -> Result<&'static EntityTypeConfig> {
    entity_type_map()
        .get(entity_type)
        .ok_or_else(|| Error::from(format!("不支持的业务类型: {}", entity_type)))
}

/// 判断扩展名是否为图片
pub fn is_image_ext(ext: &str) -> bool {
    IMAGE_EXTS.contains(&ext)
}

/// 获取文件扩展名（小写），不含 `.`
pub fn get_extension(file_name: &str) -> String {
    std::path::Path::new(file_name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase()
}

/// 计算 SHA-256 哈希（返回十六进制小写字符串）
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let bytes = hasher.finalize();
    // 使用项目内已有的 hex crate
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes.iter() {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// 校验通过后返回的文件元信息
pub struct ValidatedFile {
    /// 原始文件名
    pub original_name: String,
    /// 小写扩展名（不含 `.`）
    pub ext: String,
    /// MIME 类型（基于扩展名推断）
    pub mime_type: String,
    /// 文件大小（字节）
    pub size: i64,
    /// SHA-256 哈希
    pub file_hash: String,
    /// 是否公开访问
    pub is_public: bool,
}

/// 文件头魔数校验：返回扩展名是否与文件头匹配
/// 支持：JPEG/PNG/GIF/PDF/DOCX-XLSX(ZIP)/DOC-XLS(OLE)/BMP/WebP
fn verify_magic_number(ext: &str, head: &[u8]) -> bool {
    // 至少需要 4 字节才能判断
    if head.len() < 4 {
        return false;
    }
    match ext {
        "jpg" | "jpeg" => head[0..3] == [0xFF, 0xD8, 0xFF],
        "png" => head[0..4] == [0x89, 0x50, 0x4E, 0x47],
        "gif" => head[0..4] == [0x47, 0x49, 0x46, 0x38],
        "pdf" => head[0..4] == [0x25, 0x50, 0x44, 0x46],
        "docx" | "xlsx" | "pptx" => head[0..4] == [0x50, 0x4B, 0x03, 0x04],
        "doc" | "xls" | "ppt" => head[0..4] == [0xD0, 0xCF, 0x11, 0xE0],
        "bmp" => head[0..2] == [0x42, 0x4D],
        "webp" => head[0..4] == [0x52, 0x49, 0x46, 0x46],
        // svg/txt 是文本格式，无固定魔数，跳过校验
        "svg" | "txt" => true,
        _ => true,
    }
}

/// 根据扩展名推断 MIME 类型
fn mime_type_by_ext(ext: &str) -> String {
    match ext {
        "jpg" | "jpeg" => "image/jpeg".to_string(),
        "png" => "image/png".to_string(),
        "gif" => "image/gif".to_string(),
        "bmp" => "image/bmp".to_string(),
        "webp" => "image/webp".to_string(),
        "svg" => "image/svg+xml".to_string(),
        "pdf" => "application/pdf".to_string(),
        "doc" => "application/msword".to_string(),
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
        "xls" => "application/vnd.ms-excel".to_string(),
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
        "ppt" => "application/vnd.ms-powerpoint".to_string(),
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation".to_string(),
        "txt" => "text/plain".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

/// 校验图片内容：用 image crate 解码验证
/// 仅对图片类扩展名执行；其他类型跳过
fn verify_image_content(ext: &str, data: &[u8]) -> Result<()> {
    // svg 是文本格式，image crate 不支持，跳过
    if ext == "svg" {
        return Ok(());
    }
    if !is_image_ext(ext) {
        return Ok(());
    }
    // image 0.25 API：ImageReader::new(Cursor).with_guessed_format()?.decode()?
    let cursor = std::io::Cursor::new(data);
    match image::ImageReader::new(cursor).with_guessed_format() {
        Ok(reader) => match reader.decode() {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::from(format!("图片内容校验失败: {}", e))),
        },
        Err(e) => Err(Error::from(format!("图片格式识别失败: {}", e))),
    }
}

/// 综合校验上传文件
///
/// 校验顺序：
/// 1. 文件名非空、扩展名非空
/// 2. entity_type 在白名单内、扩展名在该 entity_type 允许列表中
/// 3. 文件大小限制（图片 10MB / 文档 50MB）
/// 4. 文件头魔数与扩展名匹配
/// 5. 图片内容用 image crate 解码校验
/// 6. 计算 SHA-256
pub fn validate_upload(
    entity_type: &str,
    original_name: &str,
    file_data: &[u8],
) -> Result<ValidatedFile> {
    // 1. 文件名校验
    if original_name.is_empty() {
        return Err(Error::from("文件名不能为空"));
    }
    let ext = get_extension(original_name);
    if ext.is_empty() {
        return Err(Error::from("文件扩展名不能为空"));
    }

    // 2. entity_type + 扩展名白名单
    let cfg = get_entity_type_config(entity_type)?;
    if !cfg.allowed_exts.contains(&ext.as_str()) {
        return Err(Error::from(format!(
            "业务类型 [{}] 不允许上传 .{} 文件",
            entity_type, ext
        )));
    }

    // 3. 文件大小校验
    let size = file_data.len() as i64;
    let max_size = if is_image_ext(&ext) { MAX_IMAGE_SIZE } else { MAX_DOC_SIZE };
    if size > max_size {
        return Err(Error::from(format!(
            "文件大小超出限制：图片最大 {}MB，文档最大 {}MB",
            MAX_IMAGE_SIZE / (1024 * 1024),
            MAX_DOC_SIZE / (1024 * 1024)
        )));
    }
    if size == 0 {
        return Err(Error::from("文件内容为空"));
    }

    // 4. 文件头魔数校验
    if !verify_magic_number(&ext, file_data) {
        return Err(Error::from("文件头魔数与扩展名不匹配，疑似伪造文件"));
    }

    // 5. 图片内容校验
    verify_image_content(&ext, file_data)?;

    // 6. SHA-256 哈希
    let file_hash = sha256_hex(file_data);

    // 先计算 mime_type，避免在结构体字面量中因 ext 已 move 而报错
    let mime_type = mime_type_by_ext(&ext);

    Ok(ValidatedFile {
        original_name: original_name.to_string(),
        ext,
        mime_type,
        size,
        file_hash,
        is_public: cfg.is_public,
    })
}
