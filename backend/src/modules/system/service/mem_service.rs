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
use std::time::Instant;

use futures_util::future::BoxFuture;
use moka::sync::Cache as MokaCache;

use crate::core::errors::error::Result;
use crate::modules::system::service::cache_service::ICacheService;

/// Memory Cache Service（moka 内核版）
///
/// 统计性能优化（文档 A3）：原 SyncHashMap + 手写 TTL 懒回收实现存在两个短板：
/// 1. 每次读写全表扫描过期键（O(n)）
/// 2. 无容量上限，理论上可无限增长
/// 升级为 moka（Rust 版 Caffeine/Ehcache3 同类）：
/// - 读写 O(1)、TinyLFU 淘汰、max_capacity 硬上限
/// - key 级 TTL 通过 value 内嵌过期时间戳实现（读取时惰性判断，moka 默认 TTL 兜底清理）
/// 对外 ICacheService trait 保持不变，所有调用方零改动。
#[derive(Debug)]
pub struct MemService {
    /// (value, expire_at)：None = 永不过期（由 moka 默认 TTL 兜底）
    pub cache: MokaCache<String, (String, Option<Instant>)>,
}

impl Default for MemService {
    fn default() -> Self {
        Self {
            cache: MokaCache::builder()
                // 统计+其他缓存总条数上限（防内存无限增长）
                .max_capacity(50_000)
                // 默认 TTL 上限保护（兜底清理已逻辑过期但未被读取的条目）
                .time_to_live(Duration::from_secs(3600))
                .build(),
        }
    }
}

impl MemService {
    fn get_alive(&self, k: &str) -> Option<String> {
        match self.cache.get(k) {
            Some((v, Some(exp))) if Instant::now() >= exp => {
                // 已逻辑过期，顺手清除
                self.cache.remove(k);
                None
            }
            Some((v, _)) => Some(v),
            None => None,
        }
    }
}

impl ICacheService for MemService {
    fn set_string(&self, k: &str, v: &str) -> BoxFuture<'_, Result<String>> {
        self.cache.insert(k.to_string(), (v.to_string(), None));
        let v = v.to_string();
        Box::pin(async move { Ok(v) })
    }

    fn get_string(&self, k: &str) -> BoxFuture<'_, Result<String>> {
        let v = self.get_alive(k).unwrap_or_default();
        Box::pin(async move { Ok(v) })
    }

    fn set_string_ex(&self, k: &str, v: &str, t: Option<Duration>) -> BoxFuture<'_, Result<String>> {
        let expire_at = t.map(|d| Instant::now() + d);
        self.cache.insert(k.to_string(), (v.to_string(), expire_at));
        let v = v.to_string();
        Box::pin(async move { Ok(v) })
    }

    fn ttl(&self, k: &str) -> BoxFuture<'_, Result<i64>> {
        // 近似语义：-2 不存在；-1 永不过期；>0 剩余秒数
        let v = match self.cache.get(k) {
            Some((_, Some(exp))) => {
                let now = Instant::now();
                if now >= exp {
                    -2
                } else {
                    (exp - now).as_secs() as i64
                }
            }
            Some((_, None)) => -1,
            None => -2,
        };
        Box::pin(async move { Ok(v) })
    }

    fn del(&self, k: &str) -> BoxFuture<'_, Result<i64>> {
        if self.cache.remove(k).is_some() {
            Box::pin(async { Ok(1) })
        } else {
            // key 不存在时静默返回 0：缓存批量清理场景（stats:* 前缀）不应因 miss 报错中断
            Box::pin(async { Ok(0) })
        }
    }

    fn keys(&self, pattern: &str) -> BoxFuture<'_, Result<Vec<String>>> {
        // 支持 glob 风格前缀匹配（如 "stats:*"）。最小化实现：仅处理尾部 "*" 通配。
        let prefix = pattern.strip_suffix('*').unwrap_or(pattern);
        let result: Vec<String> = self
            .cache
            .iter()
            .filter_map(|(k, _)| {
                if pattern == "*" || k.starts_with(prefix) {
                    Some(k.to_string())
                } else {
                    None
                }
            })
            .collect();
        Box::pin(async move { Ok(result) })
    }
}
