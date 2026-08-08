//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究。
//!

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::Mutex;
use tokio::sync::mpsc;

/// 全局 WebSocket 连接注册表
/// 一个用户可能多端登录，所以每个 user_id 对应多个发送端
#[derive(Clone, Default)]
pub struct ConnectionRegistry {
    /// user_id -> Vec<mpsc::UnboundedSender<String>>
    inner: Arc<Mutex<HashMap<i64, Vec<mpsc::UnboundedSender<String>>>>>,
}

impl ConnectionRegistry {
    /// 获取全局唯一实例
    pub fn global() -> Self {
        use parking_lot::Mutex as PLMutex;
        static INSTANCE: PLMutex<Option<ConnectionRegistry>> = PLMutex::new(None);
        let mut guard = INSTANCE.lock();
        if let Some(inst) = guard.as_ref() {
            return inst.clone();
        }
        let inst = ConnectionRegistry::default();
        *guard = Some(inst.clone());
        inst
    }

    /// 注册一个用户的 WebSocket 连接
    pub fn register(&self, user_id: i64, tx: mpsc::UnboundedSender<String>) {
        let mut map = self.inner.lock();
        map.entry(user_id).or_default().push(tx);
        log::debug!("[WebSocket] 注册用户 {} 的连接，当前在线用户数: {}", user_id, map.len());
    }

    /// 注销一个用户的 WebSocket 连接
    pub fn unregister(&self, user_id: i64) {
        let mut map = self.inner.lock();
        if let Some(addrs) = map.get_mut(&user_id) {
            // 移除所有已关闭的发送端
            addrs.retain(|tx| !tx.is_closed());
            if addrs.is_empty() {
                map.remove(&user_id);
            }
        }
        log::debug!("[WebSocket] 注销用户 {} 的连接", user_id);
    }

    /// 向指定用户推送消息（所有在线设备都会收到）
    pub fn send_to_user(&self, user_id: i64, message: String) {
        let map = self.inner.lock();
        if let Some(senders) = map.get(&user_id) {
            for tx in senders {
                let _ = tx.send(message.clone());
            }
            log::debug!("[WebSocket] 推送消息到用户 {}，连接数: {}", user_id, senders.len());
        }
    }

    /// 判断用户是否在线
    pub fn is_online(&self, user_id: i64) -> bool {
        let map = self.inner.lock();
        map.get(&user_id).map_or(false, |v| !v.is_empty())
    }

    /// 获取当前在线用户数
    pub fn online_count(&self) -> usize {
        let map = self.inner.lock();
        map.len()
    }

    /// 强制断开指定用户的所有 WebSocket 连接（踢下线时调用）
    ///
    /// 实现方式：向该用户的每个发送通道推送一条约定关闭指令 `__close__`，
    /// 由 `session.rs` 推送任务识别并调用 `session.close()` 主动关闭连接，
    /// 随后移除已关闭的发送端。
    pub fn kick(&self, user_id: i64) {
        let mut map = self.inner.lock();
        if let Some(senders) = map.get_mut(&user_id) {
            let count = senders.len();
            for tx in senders.iter() {
                let _ = tx.send(CLOSE_COMMAND.to_string());
            }
            // 清理已关闭的发送端
            senders.retain(|tx| !tx.is_closed());
            if senders.is_empty() {
                map.remove(&user_id);
            }
            log::info!("[WebSocket] 踢下线用户 {}，断开连接数: {}", user_id, count);
        }
    }
}

/// 推送任务识别的关闭指令（见 session.rs 推送分支）
pub const CLOSE_COMMAND: &str = "__close__";
