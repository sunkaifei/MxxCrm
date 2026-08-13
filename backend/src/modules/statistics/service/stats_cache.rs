//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::core::kit::CONTEXT;
use crate::modules::statistics::service::stats_range::{StatsRange, StatsScope};

/// 缓存 key：stats:{module}:{endpoint}:{scope_hash}:{params_hash}
/// scope_hash 保证不同数据权限用户的缓存互不串数据
pub fn stats_cache_key(
    endpoint: &str,
    scope: &StatsScope,
    range: &StatsRange,
    extra: &str,
) -> String {
    let scope_hash = match scope {
        None => "all".to_string(),
        Some(ids) => {
            // 对排序后的可见用户ID集合做稳定哈希（FNV-1a）：
            // - 不同权限集合必然产生不同 key，杜绝"取首元素"造成的跨用户缓存串数据（越权）
            // - 相同权限集合可安全共享缓存（如同部门管理者减少重复计算）
            // - 集合元素间以分隔符消歧，避免 [1,23] 与 [12,3] 碰撞
            let mut v = ids.clone();
            v.sort_unstable();
            format!("h{:x}", fnv1a_ids(&v))
        }
    };
    let range_str = format!(
        "{}-{}",
        range.start.map_or("na".to_string(), |d| d.format("%Y%m%d").to_string()),
        range.end.map_or("na".to_string(), |d| d.format("%Y%m%d").to_string()),
    );
    format!("stats:{}:{}:{}:{}-{}", "statistics", endpoint, scope_hash, range_str, extra)
}

/// FNV-1a 64 位稳定哈希：不依赖 std 默认哈希实现，跨版本、跨平台可复现
/// （缓存 key 仅用于进程内隔离，无需防碰撞攻击，FNV 足够且稳定）
fn fnv1a_ids(ids: &[i64]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = FNV_OFFSET;
    for id in ids {
        for b in id.to_string().as_bytes() {
            hash ^= *b as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        // 数字间分隔符：避免 [1,23] 与 [12,3] 产生相同哈希
        hash ^= 0xff;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// TTL：范围含当日 300s；纯历史/全部 1800s
pub fn stats_ttl(range: &StatsRange) -> Duration {
    if range.covers_today() {
        Duration::from_secs(300)
    } else {
        Duration::from_secs(1800)
    }
}

/// 缓存读取 + 回源计算 + 写入（缓存故障降级直查，不阻断业务；仅成功结果写缓存）
pub async fn get_or_set<T, F, Fut>(key: &str, ttl: Duration, loader: F) -> crate::core::errors::error::Result<T>
where
    T: Serialize + DeserializeOwned + Clone,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = crate::core::errors::error::Result<T>>,
{
    // 读缓存（失败视为 miss）
    if let Ok(cached) = CONTEXT.cache_service.get_string(key).await {
        if !cached.is_empty() {
            if let Ok(v) = serde_json::from_str::<T>(&cached) {
                log::debug!("[stats][cache] hit key={}", key);
                return Ok(v);
            }
        }
    }

    // 回源计算（失败直接返回错误，不缓存）
    let value = loader().await?;

    // 写缓存（失败仅 warn，不影响返回）
    if let Ok(json) = serde_json::to_string(&value) {
        if let Err(e) = CONTEXT
            .cache_service
            .set_string_ex(key, &json, Some(ttl))
            .await
        {
            log::warn!("[stats][cache] 写入失败降级直查: {}", e);
        }
    }
    Ok(value)
}

/// 清除全部统计缓存（手动重算成功后调用）
pub async fn invalidate_all_stats_cache() {
    match CONTEXT.cache_service.keys("stats:*").await {
        Ok(keys) => {
            for k in keys {
                let _ = CONTEXT.cache_service.del(&k).await;
            }
            log::info!("[stats][cache] 已清除全部统计缓存");
        }
        Err(e) => log::warn!("[stats][cache] 扫描缓存键失败: {}", e),
    }
}
