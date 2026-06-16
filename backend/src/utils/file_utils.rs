//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!
use crate::core::errors::error::Result;
use std::path::Path;
use std::{fs, io::{Read, Write}};
use walkdir::WalkDir;

// 引入 std::os::unix::fs::PermissionsExt
#[cfg(target_os = "linux")]
use std::os::unix::fs::PermissionsExt;


/// 文件
pub struct File {}

impl File {
    pub fn new() -> Self {
        File {}
    }

    /// 判断是否存在
    pub fn exists(&self, f: &str) -> bool {
        if Path::new(f).exists() {
            return true;
        }

        false
    }

    /// 删除文件
    pub fn remove(&self, f: &str) -> Result<()> {
        fs::remove_file(f)?;

        Ok(())
    }

    /// 创建文件
    pub fn create(&self, f: &str) -> Result<fs::File> {
        let file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(f)?;

        Ok(file)
    }

    /// 读取
    pub fn read(&self, f: &str) -> Result<String> {
        let mut file = fs::File::open(f)?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let contents = String::from_utf8_lossy(&buffer).to_string();

        Ok(contents)
    }

    /// 写入信息
    pub fn write(&self, f: &str, content: String) -> Result<()> {
        let path = Path::new(f);

        let mut file = fs::File::create(path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }

    /// 创建文件夹
    pub fn mkdir(&self, d: &str) -> Result<()> {
        fs::create_dir_all(d)?;

        Ok(())
    }

    /// 删除文件夹
    pub fn rmdir(&self, d: &str) -> Result<()> {
        fs::remove_dir(d)?;

        Ok(())
    }
}

/// 复制文件夹及文件下所有文件夹和文件
/// * src: 源目录
/// * dst: 目标目录
/// * permissions_dir: 目录权限
pub fn copy_dir_all(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
    permissions_dir: Option<impl AsRef<Path>>,
) -> Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    // 获取指定目录的权限（仅在 Linux 系统上）
    let permissions = if cfg!(target_os = "linux") {
        if let Some(permissions_dir) = permissions_dir {
            Some(fs::metadata(permissions_dir.as_ref())?.permissions())
        } else {
            None
        }
    } else {
        None
    };

    if !dst.exists() {
        fs::create_dir_all(dst)?;

        // 如果系统是 Linux 并且获取到了权限，应用到目标目录
        if cfg!(target_os = "linux") {
            if let Some(permissions) = &permissions {
                fs::set_permissions(dst, permissions.clone())?;
            }
        }
    }

    for entry in WalkDir::new(src).follow_links(false) {
        let entry = entry?;
        let src_path = entry.path();
        let relative_path = src_path.strip_prefix(src)?;
        let dst_path = dst.join(relative_path);

        if entry.file_type().is_dir() {
            if !dst_path.exists() {
                fs::create_dir_all(&dst_path)?;

                // 如果系统是 Linux 并且获取到了权限，应用到子目录
                if cfg!(target_os = "linux") {
                    if let Some(permissions) = &permissions {
                        fs::set_permissions(&dst_path, permissions.clone())?;
                    }
                }
            }
        } else {
            fs::copy(src_path, &dst_path)?;

            // 如果系统是 Linux 并且获取到了权限，应用到文件
            if cfg!(target_os = "linux") {
                if let Some(permissions) = &permissions {
                    fs::set_permissions(&dst_path, permissions.clone())?;
                }
            }
        }
    }

    Ok(())
}