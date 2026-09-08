//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use redis::aio::ConnectionManager;
use redis::{cmd, RedisResult, FromRedisValue, ToRedisArgs};

/// let mut redis_pool = create_redis_pool().await;
pub async fn create_redis_pool(redis_url: String) -> RedisResult<ConnectionManager> {
    let redis = redis::Client::open(redis_url)?;

    Ok(redis.get_connection_manager().await?)
}

pub async fn keys(redis: &mut ConnectionManager, key_pattern: &str) -> RedisResult<Vec<String>> {
    Ok(cmd("KEYS")
        .arg(key_pattern)
        .query_async::<_>(redis)
        .await?)
}

pub async fn delete(redis: &mut ConnectionManager, keys_to_delete: Vec<String>) -> RedisResult<i64> {
    if keys_to_delete.is_empty() {
        return Ok(0);
    }
    
    let mut del = cmd("DEL");
    for key in keys_to_delete {
        del.arg(key);
    }

    Ok(del.query_async::<_>(redis).await?)
}

#[warn(dependency_on_unit_never_type_fallback)]
pub async fn set<T: ToRedisArgs>(
    redis: &mut ConnectionManager,
    key: &str,
    value: T,
    cache_duration: u32,
) -> RedisResult<()> {
    cmd("SET")
        .arg(key)
        .arg(value)
        .query_async::<()>(redis)
        .await?;
    cmd("EXPIRE")
        .arg(key)
        .arg(cache_duration)
        .query_async::<()>(redis)
        .await?;
    Ok(())
}

pub async fn get<T: FromRedisValue>(redis: &mut ConnectionManager, cache_key: &str) -> RedisResult<Option<T>> {
    Ok(cmd("GET")
        .arg(cache_key)
        .query_async::<_>(redis)
        .await?)
}
