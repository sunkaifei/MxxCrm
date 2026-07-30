//! API 冲突检测脚本
//!
//! 用途：扫描前后端 API 定义，检测：
//!   1. 前端 `export const xxxApi` 同名冲突（通过 `export *` 汇总到 core/index.ts 的文件）
//!   2. 后端路由 scope 路径重复（多个 controller 注册同一 scope）
//!
//! 运行方式：`cargo run --bin check_api_conflicts`
//!
//! 退出码：
//!   0 = 无冲突
//!   1 = 发现冲突
//!
//! 详见 docs/api-registry.md

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> std::io::Result<()> {
    let exit_code = run_checks();
    std::process::exit(exit_code);
}

fn run_checks() -> i32 {
    let project_root = find_project_root().unwrap_or_else(|| PathBuf::from("."));
    let mut has_conflict = false;

    println!("=== Mxx-CRM API 冲突检测 ===\n");

    // ========== 检查 1：前端 API 同名冲突 ==========
    println!("[1/2] 检查前端 API 同名冲突...");
    let frontend_api_dir = project_root.join("frontend/apps/web-antd/src/api/core");
    let mut api_map: HashMap<String, Vec<String>> = HashMap::new();

    if frontend_api_dir.exists() {
        scan_frontend_api_dir(&frontend_api_dir, &PathBuf::new(), &mut api_map);
    }

    let mut fe_conflicts: Vec<String> = Vec::new();
    for (api_name, files) in &api_map {
        if files.len() > 1 {
            // 别名导出（同一文件内）不算冲突
            let unique_files: Vec<&String> = files.iter().collect::<std::collections::HashSet<_>>().into_iter().collect();
            if unique_files.len() > 1 {
                has_conflict = true;
                fe_conflicts.push(format!(
                    "  ❌ `{}` 在 {} 个文件中导出：\n{}",
                    api_name,
                    unique_files.len(),
                    unique_files.iter().map(|f| format!("     - {}", f)).collect::<Vec<_>>().join("\n")
                ));
            }
        }
    }

    if fe_conflicts.is_empty() {
        println!("  ✅ 前端 API 命名无冲突（共扫描 {} 个 API）\n", api_map.len());
    } else {
        println!("  发现 {} 个冲突：", fe_conflicts.len());
        for c in &fe_conflicts {
            println!("{}", c);
        }
        println!();
    }

    // ========== 检查 2：后端 scope 路径冲突 ==========
    println!("[2/2] 检查后端 scope 路径冲突...");
    let backend_src = project_root.join("backend/src");
    let mut scope_map: HashMap<String, Vec<String>> = HashMap::new();

    if backend_src.exists() {
        scan_backend_scopes(&backend_src, &mut scope_map);
    }

    let mut be_conflicts: Vec<String> = Vec::new();
    for (scope_path, controllers) in &scope_map {
        // 同一 controller 内的嵌套子 scope 不算冲突
        let unique_controllers: Vec<&String> = controllers.iter().collect::<std::collections::HashSet<_>>().into_iter().collect();
        if unique_controllers.len() > 1 {
            has_conflict = true;
            be_conflicts.push(format!(
                "  ❌ scope `{}` 在 {} 个 controller 中注册：\n{}",
                scope_path,
                unique_controllers.len(),
                unique_controllers.iter().map(|c| format!("     - {}", c)).collect::<Vec<_>>().join("\n")
            ));
        }
    }

    if be_conflicts.is_empty() {
        println!("  ✅ 后端 scope 路径无冲突（共扫描 {} 个 scope）\n", scope_map.len());
    } else {
        println!("  发现 {} 个冲突：", be_conflicts.len());
        for c in &be_conflicts {
            println!("{}", c);
        }
        println!();
    }

    // ========== 汇总 ==========
    println!("=== 检测汇总 ===");
    if has_conflict {
        println!("❌ 发现冲突，请修复后再提交代码");
        println!("   详见 docs/api-registry.md 命名规范");
        1
    } else {
        println!("✅ 所有检查通过");
        0
    }
}

/// 递归扫描前端 API 目录，提取所有 `export const xxxApi` 声明
fn scan_frontend_api_dir(dir: &Path, rel_prefix: &Path, api_map: &mut HashMap<String, Vec<String>>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            // 跳过 node_modules
            if name == "node_modules" || name.starts_with('.') {
                continue;
            }
            let new_rel = rel_prefix.join(&name);
            scan_frontend_api_dir(&path, &new_rel, api_map);
        } else if path.extension().and_then(|e| e.to_str()) == Some("ts") {
            // 跳过 index.ts 聚合文件
            if name == "index.ts" {
                continue;
            }
            let rel_path = format!("api/core/{}/{}", rel_prefix.display(), name);
            scan_ts_file_for_api_exports(&path, &rel_path, api_map);
        }
    }
}

/// 扫描单个 .ts 文件，提取 `export const xxxApi` 和 `export async function xxxApi` 声明
fn scan_ts_file_for_api_exports(
    file_path: &Path,
    rel_path: &str,
    api_map: &mut HashMap<String, Vec<String>>,
) {
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    for line in content.lines() {
        let trimmed = line.trim();

        // 匹配 `export const xxxApi =` 或 `export const xxxApi = {`
        if let Some(name) = extract_export_name(trimmed, "export const ", "Api") {
            api_map.entry(name).or_default().push(rel_path.to_string());
            continue;
        }

        // 匹配 `export async function xxxApi` 或 `export function xxxApi`
        if let Some(name) = extract_function_export_name(trimmed) {
            if name.ends_with("Api") {
                api_map.entry(name).or_default().push(rel_path.to_string());
            }
        }
    }
}

/// 从 `export const XXX = ...` 行提取变量名
fn extract_export_name(line: &str, prefix: &str, suffix: &str) -> Option<String> {
    let rest = line.strip_prefix(prefix)?;
    // 取到 `=` 或 `:` 之前的部分
    let end = rest.find(|c: char| c == '=' || c == ':')?;
    let name = rest[..end].trim();
    if name.ends_with(suffix) && name.is_identifier_like() {
        Some(name.to_string())
    } else {
        None
    }
}

/// 从 `export async function XXX` 或 `export function XXX` 行提取函数名
fn extract_function_export_name(line: &str) -> Option<String> {
    let rest = line.strip_prefix("export ")?;
    let rest = rest.strip_prefix("async ").unwrap_or(rest);
    let rest = rest.strip_prefix("function ")?;
    let end = rest.find(|c: char| c == '(' || c.is_whitespace())?;
    let name = rest[..end].trim();
    if name.is_identifier_like() {
        Some(name.to_string())
    } else {
        None
    }
}

/// 递归扫描后端 src 目录，提取 `web::scope("/xxx")` 声明
fn scan_backend_scopes(dir: &Path, scope_map: &mut HashMap<String, Vec<String>>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            scan_backend_scopes(&path, scope_map);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            let rel_path = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            scan_rs_file_for_scopes(&path, &rel_path, scope_map);
        }
    }
}

/// 扫描 .rs 文件，提取 `web::scope("/xxx")` 声明
fn scan_rs_file_for_scopes(
    file_path: &Path,
    rel_path: &str,
    scope_map: &mut HashMap<String, Vec<String>>,
) {
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    for line in content.lines() {
        let trimmed = line.trim();
        // 匹配 web::scope("/xxx")
        if let Some(scope_path) = extract_scope_path(trimmed) {
            // 只记录有实际路径的 scope（跳过空字符串 scope）
            if !scope_path.is_empty() {
                scope_map.entry(scope_path).or_default().push(rel_path.to_string());
            }
        }
    }
}

/// 从 `web::scope("/xxx")` 行提取 scope 路径
fn extract_scope_path(line: &str) -> Option<String> {
    let idx = line.find("web::scope(")?;
    let rest = &line[idx + "web::scope(".len()..];
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// 查找项目根目录（包含 backend 和 frontend 目录的目录）
fn find_project_root() -> Option<PathBuf> {
    let current = std::env::current_dir().ok()?;
    let mut dir = current.as_path();
    loop {
        if dir.join("backend").exists() && dir.join("frontend").exists() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
}

/// 简单的标识符校验 trait
trait IdentifierLike {
    fn is_identifier_like(&self) -> bool;
}

impl IdentifierLike for str {
    fn is_identifier_like(&self) -> bool {
        !self.is_empty()
            && self.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$')
            && !self.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
    }
}
