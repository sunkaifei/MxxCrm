//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 文字点选验证码服务（C-2）：生成 / 校验 / ticket / 三态存储分发
//!
//! 存储三态（方案 §4.1）：
//! - Cache 态（默认）：走 cache_service，随 cache_type 落内存或 Redis
//! - Db 态：独立表 mxx_system_captcha（多实例无 Redis / 审计留存）
//!
//! 一次性语义：点选会话"读到即消费"（Cache：get 后 del，del 计数=0 视为已被并发消费）；
//! Db 态用 `DELETE ... RETURNING data` 原子消费。

use base64::{engine, Engine as _};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbConn, EntityTrait, Statement};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::kit::captcha_click;
use crate::core::kit::CONTEXT;
use crate::modules::system::entity::captcha;
use crate::modules::system::service::config_service;

const CLICK_TYPE: &str = "click_verify";
const TICKET_TYPE: &str = "click_ticket";

// ==================== 配置读取（缓存优先，miss 回查 DB）====================

async fn config_value(key: &str, default: &str) -> String {
    match CONTEXT.cache_service.get_string(&format!("config:{}", key)).await {
        Ok(v) if !v.is_empty() => v,
        _ => {
            let db_val = config_service::find_value_by_key_from_db(key)
                .await
                .unwrap_or_else(|| default.to_string());
            let _ = CONTEXT.cache_service.set_string(&format!("config:{}", key), &db_val).await;
            db_val
        }
    }
}

/// 兼容保留：默认页面（登录）形态
pub async fn get_captcha_mode() -> String {
    get_page_captcha_mode("login").await
}

/// 验证码设置：按页面独立读取形态（login / register）
/// 优先级：总开关 login_captcha_enabled=0 → disabled；否则取页面配置
/// （image / click / none），非法值回退 click（登录）/ image（注册）
pub async fn get_page_captcha_mode(page: &str) -> String {
    let enabled = config_value("login_captcha_enabled", "1").await;
    if enabled != "1" {
        return "disabled".to_string();
    }
    let default = if page == "register" { "image" } else { "click" };
    let mode = config_value(&format!("{}_captcha_type", page), default).await;
    if mode != "image" && mode != "click" && mode != "none" {
        return default.to_string();
    }
    mode
}

async fn get_store() -> String {
    config_value("captcha_store", "cache").await
}

async fn get_ttl() -> u64 {
    config_value("captcha_ttl", "180").await.parse().unwrap_or(180)
}

async fn get_ticket_ttl() -> u64 {
    config_value("captcha_ticket_ttl", "120").await.parse().unwrap_or(120)
}

async fn get_tolerance() -> f64 {
    config_value("captcha_click_tolerance", "15").await.parse().unwrap_or(15.0)
}

async fn get_font_paths() -> Vec<String> {
    let configured = config_value("captcha_font_path", "").await;
    let mut paths = Vec::new();
    if !configured.is_empty() {
        paths.push(configured);
    }
    paths.extend(captcha_click::DEFAULT_FONT_PATHS.iter().map(|s| s.to_string()));
    paths
}

// ==================== 存储数据结构 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickPoint {
    pub ch: char,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickCaptchaSession {
    pub points: Vec<ClickPoint>,
    pub tries: u32,
    pub created_at: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClickCaptchaVO {
    pub captcha_key: String,
    pub image_base64: String,
    pub hint_chars: Vec<String>,
    pub ttl: u64,
    pub image_width: u32,
    pub image_height: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClickVerifyVO {
    pub captcha_ticket: String,
    pub expires_in: u64,
}

#[derive(Debug, Deserialize)]
pub struct ClickPointInput {
    pub x: f64,
    pub y: f64,
}

// ==================== 存储分发（Cache / Db）====================

async fn store_put(db: &DbConn, key: &str, typ: &str, data: &serde_json::Value, ttl: u64) -> Result<(), String> {
    if get_store().await == "db" {
        let now = chrono::Local::now().naive_local();
        let row = captcha::ActiveModel {
            captcha_key: Set(Some(key.to_string())),
            captcha_type: Set(Some(typ.to_string())),
            data: Set(Some(data.clone())),
            tries: Set(Some(0)),
            expire_time: Set(Some(now + chrono::Duration::seconds(ttl as i64))),
            create_time: Set(Some(now)),
            ..Default::default()
        };
        captcha::Entity::insert(row).exec(db).await.map_err(|e| e.to_string())?;
        return Ok(());
    }
    CONTEXT
        .cache_service
        .set_string_ex(&format!("captcha:{}:{}", typ, key), &data.to_string(), Some(std::time::Duration::from_secs(ttl)))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 原子消费：读取并删除。返回 None = 不存在或已过期/已被消费。
/// Db 态用 DELETE..RETURNING 原子语义；Cache 态 del 计数兜底并发。
async fn store_take(db: &DbConn, key: &str, typ: &str) -> Option<(serde_json::Value, i32)> {
    if get_store().await == "db" {
        let stmt = Statement::from_sql_and_values(
            db.get_database_backend(),
            "DELETE FROM mxx_system_captcha WHERE captcha_key = $1 AND captcha_type = $2 AND expire_time > NOW() RETURNING data, tries",
            [key.into(), typ.into()],
        );
        let res = db.query_one_raw(stmt).await.ok()??;
        let data: serde_json::Value = res.try_get("", "data").ok()?;
        let tries: i32 = res.try_get("", "tries").unwrap_or(0);
        Some((data, tries))
    } else {
        let full = format!("captcha:{}:{}", typ, key);
        let raw = CONTEXT.cache_service.get_string(&full).await.ok()?;
        let deleted = CONTEXT.cache_service.del(&full).await.unwrap_or(0);
        if deleted == 0 {
            return None;
        }
        let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
        let tries = v.get("tries").and_then(|t| t.as_u64()).unwrap_or(0) as i32;
        Some((v, tries))
    }
}

/// 清理过期（Db 态；Cache 态靠 TTL 自动过期，无需处理）
pub async fn clean_expired(db: &DatabaseConnection) -> Result<u64, String> {
    if get_store().await != "db" {
        return Ok(0);
    }
    db.execute_unprepared("DELETE FROM mxx_system_captcha WHERE expire_time <= NOW()")
        .await
        .map(|r| r.rows_affected())
        .map_err(|e| e.to_string())
}

use sea_orm::Set;

// ==================== 生成 ====================

pub async fn generate_click_captcha(db: &DbConn) -> Result<ClickCaptchaVO, String> {
    let ttl = get_ttl().await;
    let paths = get_font_paths().await;
    captcha_click::load_font(&paths).map_err(|e| format!("点选验证码字体加载失败: {}（请检查 captcha_font_path 配置）", e))?;

    // 点选升级：图上绘制 5 个字（3 目标 + 2 干扰），提示与校验仅针对 3 个目标
    const DISTRACTOR_COUNT: usize = 2;
    let targets = captcha_click::pick_chars(captcha_click::CHAR_COUNT);
    let distractors = captcha_click::pick_distractors(&targets, DISTRACTOR_COUNT);
    let all_chars: Vec<char> = targets.iter().chain(distractors.iter()).copied().collect();
    // 布点：全部 5 字（最小间距 70 适配 5 点密度）；绘制顺序打乱使目标/干扰位置随机
    let all_points = captcha_click::place_points(captcha_click::IMAGE_W, captcha_click::IMAGE_H, all_chars.len(), 70.0);
    let mut draw_order: Vec<usize> = (0..all_chars.len()).collect();
    {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        draw_order.shuffle(&mut rng);
    }
    let draw_chars: Vec<char> = draw_order.iter().map(|&i| all_chars[i]).collect();
    let draw_points: Vec<(f64, f64)> = draw_order.iter().map(|&i| all_points[i]).collect();
    let img = captcha_click::render_click_image(&draw_chars, &draw_points, targets.len());
    // 目标点（前 3 个 all_chars 是目标）供校验留存
    let target_points: Vec<(f64, f64)> = (0..targets.len()).map(|i| all_points[i]).collect();

    let png = {
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png)
            .map_err(|e| format!("验证码图片编码失败: {}", e))?;
        buf.into_inner()
    };

    let key = Uuid::new_v4().to_string();
    let session = ClickCaptchaSession {
        points: targets
            .iter()
            .zip(target_points.iter())
            .map(|(c, (x, y))| ClickPoint { ch: *c, x: *x, y: *y })
            .collect(),
        tries: 0,
        created_at: chrono::Local::now().timestamp(),
    };
    let data = serde_json::to_value(&session).map_err(|e| e.to_string())?;
    store_put(db, &key, CLICK_TYPE, &data, ttl).await?;

    Ok(ClickCaptchaVO {
        captcha_key: key,
        image_base64: engine::general_purpose::STANDARD.encode(png),
        hint_chars: targets.iter().map(|c| c.to_string()).collect(),
        ttl,
        image_width: captcha_click::IMAGE_W,
        image_height: captcha_click::IMAGE_H,
    })
}

// ==================== 校验 + ticket ====================

/// 点选校验（一次性消费）→ 通过签发登录 ticket
pub async fn verify_click(
    db: &DbConn,
    captcha_key: &str,
    clicks: &[ClickPointInput],
    display_width: Option<f64>,
    display_height: Option<f64>,
) -> Result<ClickVerifyVO, String> {
    let Some((data, _)) = store_take(db, captcha_key, CLICK_TYPE).await else {
        return Err("验证码已过期，请刷新后重试".to_string());
    };
    let session: ClickCaptchaSession = serde_json::from_value(data).map_err(|e| e.to_string())?;

    // 前端按显示尺寸点击 → 等比换算回原图坐标
    let (dw, dh) = (display_width.unwrap_or(0.0), display_height.unwrap_or(0.0));
    let (sx, sy) = if dw > 0.0 && dh > 0.0 {
        (captcha_click::IMAGE_W as f64 / dw, captcha_click::IMAGE_H as f64 / dh)
    } else {
        (1.0, 1.0)
    };
    let tolerance = get_tolerance().await;
    let points: Vec<(f64, f64)> = session.points.iter().map(|p| (p.x, p.y)).collect();
    let clicks: Vec<(f64, f64)> = clicks.iter().map(|c| (c.x * sx, c.y * sy)).collect();

    captcha_click::verify_clicks(&points, &clicks, tolerance)?;

    // 通过 → 签发一次性登录 ticket
    let ticket = Uuid::new_v4().to_string();
    let ttl = get_ticket_ttl().await;
    let payload = serde_json::json!({ "created_at": chrono::Local::now().timestamp() });
    store_put(db, &ticket, TICKET_TYPE, &payload, ttl).await?;

    Ok(ClickVerifyVO { captcha_ticket: ticket, expires_in: ttl })
}

/// 登录二次校验：消费 ticket（一次性）
pub async fn consume_click_ticket(db: &DbConn, ticket: &str) -> Result<(), String> {
    if ticket.trim().is_empty() {
        return Err("验证码不能为空".to_string());
    }
    let Some(_) = store_take(db, ticket.trim(), TICKET_TYPE).await else {
        return Err("验证码已过期或无效，请重新完成验证".to_string());
    };
    Ok(())
}
