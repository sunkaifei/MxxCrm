//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use sea_orm::*;

pub async fn init_message_tables(db: &DbConn) -> Result<(), DbErr> {
    let sql = r#"
        CREATE TABLE IF NOT EXISTS mxx_user_online (
            id BIGSERIAL PRIMARY KEY,
            user_id BIGINT NOT NULL DEFAULT 0,
            session_id VARCHAR(64) NOT NULL,
            device_type INTEGER,
            ip_address VARCHAR(255),
            user_agent VARCHAR(500),
            last_heartbeat TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            status INTEGER DEFAULT 1,
            create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_user_online_user_id ON mxx_user_online(user_id);
        CREATE INDEX IF NOT EXISTS idx_user_online_session_id ON mxx_user_online(session_id);
        CREATE INDEX IF NOT EXISTS idx_user_online_status ON mxx_user_online(status);

        CREATE TABLE IF NOT EXISTS mxx_system_notification (
            id BIGSERIAL PRIMARY KEY,
            title VARCHAR(200) NOT NULL,
            content TEXT,
            type INTEGER NOT NULL DEFAULT 1,
            biz_type VARCHAR(50),
            biz_id BIGINT,
            sender_id BIGINT,
            receiver_id BIGINT NOT NULL DEFAULT 0,
            is_read INTEGER DEFAULT 0,
            read_time TIMESTAMP,
            link_url VARCHAR(500),
            create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_notification_receiver ON mxx_system_notification(receiver_id);
        CREATE INDEX IF NOT EXISTS idx_notification_read ON mxx_system_notification(receiver_id, is_read);
        CREATE INDEX IF NOT EXISTS idx_notification_sender ON mxx_system_notification(sender_id);
        CREATE INDEX IF NOT EXISTS idx_notification_type ON mxx_system_notification(type);

        CREATE TABLE IF NOT EXISTS mxx_chat_session (
            id BIGSERIAL PRIMARY KEY,
            session_type INTEGER NOT NULL DEFAULT 1,
            session_name VARCHAR(200),
            avatar_url VARCHAR(500),
            last_message_id BIGINT,
            last_message_content TEXT,
            last_message_time TIMESTAMP,
            member_count INTEGER DEFAULT 0,
            create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_chat_session_type ON mxx_chat_session(session_type);
        CREATE INDEX IF NOT EXISTS idx_chat_session_last_time ON mxx_chat_session(last_message_time);

        CREATE TABLE IF NOT EXISTS mxx_chat_message (
            id BIGSERIAL PRIMARY KEY,
            session_id BIGINT NOT NULL,
            sender_id BIGINT NOT NULL,
            sender_nickname VARCHAR(100) NOT NULL,
            sender_avatar VARCHAR(500),
            content TEXT NOT NULL,
            message_type INTEGER DEFAULT 2,
            content_type INTEGER DEFAULT 1,
            file_url VARCHAR(500),
            file_name VARCHAR(200),
            file_size BIGINT DEFAULT 0,
            biz_type VARCHAR(50),
            biz_id BIGINT,
            read_status INTEGER DEFAULT 0,
            read_time TIMESTAMP,
            is_recalled INTEGER DEFAULT 0,
            send_time TIMESTAMP,
            create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_chat_message_session ON mxx_chat_message(session_id);
        CREATE INDEX IF NOT EXISTS idx_chat_message_sender ON mxx_chat_message(sender_id);
        CREATE INDEX IF NOT EXISTS idx_chat_message_send_time ON mxx_chat_message(send_time);
        CREATE INDEX IF NOT EXISTS idx_chat_message_create_time ON mxx_chat_message(create_time);

        CREATE TABLE IF NOT EXISTS mxx_chat_session_member (
            id BIGSERIAL PRIMARY KEY,
            session_id BIGINT NOT NULL,
            user_id BIGINT NOT NULL DEFAULT 0,
            user_type INTEGER DEFAULT 1,
            nickname VARCHAR(100),
            avatar VARCHAR(500),
            is_owner INTEGER DEFAULT 0,
            is_muted INTEGER DEFAULT 0,
            is_pinned INTEGER DEFAULT 0,
            unread_count INTEGER DEFAULT 0,
            last_read_message_id BIGINT DEFAULT 0,
            join_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            leave_time TIMESTAMP,
            create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            deleted INTEGER DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_session_member_user ON mxx_chat_session_member(user_id);
        CREATE INDEX IF NOT EXISTS idx_session_member_session ON mxx_chat_session_member(session_id);
        CREATE INDEX IF NOT EXISTS idx_session_member_sid_uid ON mxx_chat_session_member(session_id, user_id);

        CREATE TABLE IF NOT EXISTS mxx_user_notification_setting (
            id BIGSERIAL PRIMARY KEY,
            user_id BIGINT NOT NULL DEFAULT 0,
            user_type INTEGER DEFAULT 1,
            notify_type VARCHAR(50) NOT NULL,
            enabled INTEGER DEFAULT 1,
            sound_enabled INTEGER DEFAULT 1,
            vibration_enabled INTEGER DEFAULT 1,
            push_enabled INTEGER DEFAULT 1,
            quiet_start VARCHAR(10),
            quiet_end VARCHAR(10),
            create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            deleted INTEGER DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_notify_setting_user ON mxx_user_notification_setting(user_id);
        CREATE INDEX IF NOT EXISTS idx_notify_setting_type ON mxx_user_notification_setting(notify_type);
    "#;

    db.execute_unprepared(sql).await?;

    // 插入菜单数据
    init_menu_data(db).await?;

    Ok(())
}

async fn init_menu_data(db: &DbConn) -> Result<(), DbErr> {
    let menu_sql = r#"
        INSERT INTO mxx_system_menu (
            parent_id, name, path, component, route_name, redirect,
            type, perm, icon, sort, status,
            affix_tab, hide_children_in_menu, hide_in_breadcrumb, hide_in_menu, hide_in_tab, keep_alive,
            params, deleted, create_time
        )
        SELECT
            299, 'page.company.message.title', '/company/message', 'company/message/index', 'CompanyMessage', NULL,
            'MENU', 'company:message:list', 'lucide:message-square', 50, 1,
            0, 0, 0, 0, 0, 0,
            NULL, 0, NOW()
        WHERE NOT EXISTS (
            SELECT 1 FROM mxx_system_menu
            WHERE name = 'page.company.message.title' AND deleted = 0
        );
    "#;

    db.execute_unprepared(menu_sql).await?;

    Ok(())
}
