//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::db_migration;
use sea_orm::*;

/// 离职交接模块建表迁移批次名（后续调整结构时递增）
const MIGRATION_HR_RESIGN_TABLES: &str = "hr_resign_tables_v1";

/// 初始化离职交接模块表（版本化迁移：已应用则跳过）
pub async fn init_hr_resign_tables(db: &DbConn) -> Result<(), DbErr> {
    if db_migration::migration_applied(db, MIGRATION_HR_RESIGN_TABLES).await? {
        return Ok(());
    }
    // 老库兼容：交接单主表已存在（手工建过），直接标记已迁移
    if db_migration::table_exists(db, "mxx_hr_resign_record").await? {
        db_migration::mark_migration_applied(db, MIGRATION_HR_RESIGN_TABLES).await?;
        return Ok(());
    }

    let sql = r#"
        CREATE TABLE IF NOT EXISTS mxx_hr_resign_record (
            id                 BIGSERIAL PRIMARY KEY,
            admin_id           BIGINT NOT NULL,
            transfer_to_admin_id BIGINT,
            resign_type        INTEGER NOT NULL DEFAULT 1,
            resign_date        DATE,
            actual_leave_date  DATE,
            reason             TEXT,
            status             INTEGER NOT NULL DEFAULT 1,
            create_by          VARCHAR(64),
            create_time        TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            update_by          VARCHAR(64),
            update_time        TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_resign_record_admin ON mxx_hr_resign_record(admin_id);
        CREATE INDEX IF NOT EXISTS idx_resign_record_status ON mxx_hr_resign_record(status);

        CREATE TABLE IF NOT EXISTS mxx_hr_resign_transfer_item (
            id             BIGSERIAL PRIMARY KEY,
            record_id      BIGINT NOT NULL,
            item_key       VARCHAR(64),
            item_name      VARCHAR(64),
            assignee_id    BIGINT,
            status         INTEGER NOT NULL DEFAULT 0,
            confirm_remark TEXT,
            confirm_time   TIMESTAMP,
            create_time    TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            update_time    TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_resign_item_record ON mxx_hr_resign_transfer_item(record_id);
        CREATE INDEX IF NOT EXISTS idx_resign_item_assignee ON mxx_hr_resign_transfer_item(assignee_id);

        CREATE TABLE IF NOT EXISTS mxx_hr_resign_item_template (
            id               BIGSERIAL PRIMARY KEY,
            item_key         VARCHAR(64) NOT NULL,
            item_name        VARCHAR(64) NOT NULL,
            assignee_rule    INTEGER NOT NULL DEFAULT 1,
            assignee_role_id BIGINT,
            enabled          INTEGER NOT NULL DEFAULT 1,
            sort             INTEGER NOT NULL DEFAULT 0,
            create_time      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            update_time      TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_resign_tpl_key ON mxx_hr_resign_item_template(item_key);

        -- 预置 4 项（工作/客户/账号/资产），仅当模板表为空时插入
        INSERT INTO mxx_hr_resign_item_template (item_key, item_name, assignee_rule, assignee_role_id, enabled, sort)
        SELECT * FROM (VALUES
            ('work',     '工作交接', 1, NULL, 1, 1),
            ('customer', '客户交接', 1, NULL, 1, 2),
            ('account',  '账号权限', 2, NULL, 1, 3),
            ('asset',    '资产归还', 3, 5, 1, 4)
        ) AS v(item_key, item_name, assignee_rule, assignee_role_id, enabled, sort)
        WHERE NOT EXISTS (SELECT 1 FROM mxx_hr_resign_item_template);

        -- resign_approval 流程模板（部门负责人 -> 人事专员 -> 人事经理）
        INSERT INTO mxx_system_approval_flow
          (flow_code, flow_name, business_type, description, enabled, is_system, create_by, create_time, update_time)
        SELECT 'resign_approval', '离职审批', 'resign', '员工离职申请审批：部门负责人 -> 人事专员 -> 人事经理', 1, 1, 'system', now(), now()
        WHERE NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow WHERE flow_code = 'resign_approval');

        INSERT INTO mxx_system_approval_flow_node
          (flow_id, node_key, node_type, node_order, node_name, approver_type, approver_id, approve_mode, is_final, create_time)
        SELECT f.id, v.node_key, v.node_type, v.node_order, v.node_name, v.approver_type, v.approver_id, 1, v.is_final, now()
        FROM mxx_system_approval_flow f
        CROSS JOIN (VALUES
            ('start',        1, 1, '开始',     NULL::INT, NULL::BIGINT, 0),
            ('dept_manager', 2, 2, '部门负责人审批', 7, NULL, 0),
            ('hr_specialist',2, 3, '人事专员审批', 2, 14, 0),
            ('hr_manager',   2, 4, '人事经理审批', 2, 15, 0),
            ('end',          4, 5, '结束',     NULL::INT, NULL::BIGINT, 1)
        ) AS v(node_key, node_type, node_order, node_name, approver_type, approver_id, is_final)
        WHERE f.flow_code = 'resign_approval'
          AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_node n WHERE n.flow_id = f.id AND n.node_key = v.node_key);

        INSERT INTO mxx_system_approval_flow_edge
          (flow_id, source_node_key, target_node_key, condition_expr, label, create_time)
        SELECT f.id, v.source_node_key, v.target_node_key, v.condition_expr, v.label, now()
        FROM mxx_system_approval_flow f
        CROSS JOIN (VALUES
            ('start',        'dept_manager',  NULL::TEXT, '提交'),
            ('dept_manager', 'hr_specialist', NULL,       '通过'),
            ('hr_specialist','hr_manager',    NULL,       '通过'),
            ('hr_manager',   'end',           NULL,       '通过')
        ) AS v(source_node_key, target_node_key, condition_expr, label)
        WHERE f.flow_code = 'resign_approval'
          AND NOT EXISTS (SELECT 1 FROM mxx_system_approval_flow_edge e
                          WHERE e.flow_id = f.id AND e.source_node_key = v.source_node_key AND e.target_node_key = v.target_node_key);
    "#;
    db.execute_unprepared(sql).await?;
    db_migration::mark_migration_applied(db, MIGRATION_HR_RESIGN_TABLES).await?;
    Ok(())
}
