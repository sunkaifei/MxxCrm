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

/// 可视化 PDF 模板设计器建表迁移批次名（后续调整结构时递增）
const MIGRATION_PDF_DESIGNER_TABLES: &str = "pdf_designer_tables_v89";

/// 初始化可视化 PDF 模板设计器相关表与列（版本化迁移：已应用则跳过）。
///
/// 与 `sql/v89_pdf_designer.sql` 等价，保证「全新库直接启动即可用」，
/// 老库若已手工执行过 SQL，则由 `table_exists` 分支直接标记已迁移。
pub async fn init_pdf_designer_tables(db: &DbConn) -> Result<(), DbErr> {
    if db_migration::migration_applied(db, MIGRATION_PDF_DESIGNER_TABLES).await? {
        return Ok(());
    }
    // 老库兼容：素材表已存在（手工执行过 v89 SQL），直接标记已迁移
    if db_migration::table_exists(db, "mxx_system_pdf_asset").await? {
        db_migration::mark_migration_applied(db, MIGRATION_PDF_DESIGNER_TABLES).await?;
        return Ok(());
    }

    // 1) 扩展模板表（engine 分流：html = legacy；layout = 可视化设计器）
    let alter_sql = r#"
        ALTER TABLE mxx_system_pdf_template
            ADD COLUMN IF NOT EXISTS engine      varchar(16) NOT NULL DEFAULT 'html',
            ADD COLUMN IF NOT EXISTS layout_json jsonb,
            ADD COLUMN IF NOT EXISTS base_pdf_id bigint,
            ADD COLUMN IF NOT EXISTS width_mm    numeric(8, 2),
            ADD COLUMN IF NOT EXISTS height_mm   numeric(8, 2),
            ADD COLUMN IF NOT EXISTS version     integer NOT NULL DEFAULT 1,
            ADD COLUMN IF NOT EXISTS parent_id   bigint,
            ADD COLUMN IF NOT EXISTS preview_url varchar(512);

        CREATE INDEX IF NOT EXISTS idx_pdf_template_engine
            ON mxx_system_pdf_template (engine) WHERE deleted = 0;

        UPDATE mxx_system_pdf_template SET engine = 'html' WHERE engine IS NULL;

        ALTER TABLE mxx_system_pdf_record
            ADD COLUMN IF NOT EXISTS file_hash varchar(128);

        CREATE TABLE IF NOT EXISTS mxx_system_pdf_asset (
            id          bigserial PRIMARY KEY,
            name        varchar(128) NOT NULL,
            category    varchar(32)  NOT NULL,
            file_url    varchar(512) NOT NULL,
            file_path   varchar(512),
            file_size   bigint,
            md5         varchar(64),
            width_px    integer,
            height_px   integer,
            sort        integer      NOT NULL DEFAULT 0,
            status      integer      NOT NULL DEFAULT 1,
            create_by   bigint,
            create_time timestamp(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
            update_time timestamp(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
            deleted     integer      NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_pdf_asset_category
            ON mxx_system_pdf_asset (category) WHERE deleted = 0;
        CREATE INDEX IF NOT EXISTS idx_pdf_asset_md5
            ON mxx_system_pdf_asset (md5) WHERE deleted = 0;

        CREATE TABLE IF NOT EXISTS mxx_system_pdf_field_meta (
            id             bigserial PRIMARY KEY,
            doc_type       varchar(32)  NOT NULL,
            group_code     varchar(64)  NOT NULL,
            group_name     varchar(64)  NOT NULL,
            field_path     varchar(128) NOT NULL,
            field_name     varchar(128) NOT NULL,
            data_type      varchar(16)  NOT NULL DEFAULT 'string',
            sample         varchar(255),
            sample_long    varchar(512),
            sample_presets jsonb,
            nullable       boolean      NOT NULL DEFAULT true,
            sample_source  varchar(16)  NOT NULL DEFAULT 'meta',
            sort           integer      NOT NULL DEFAULT 0,
            status         integer      NOT NULL DEFAULT 1,
            deleted        integer      NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_pdf_field_doc
            ON mxx_system_pdf_field_meta (doc_type, group_code) WHERE deleted = 0;
        CREATE UNIQUE INDEX IF NOT EXISTS uk_pdf_field_path
            ON mxx_system_pdf_field_meta (doc_type, field_path) WHERE deleted = 0;

        CREATE TABLE IF NOT EXISTS mxx_system_pdf_template_version (
            id           bigserial PRIMARY KEY,
            template_id  bigint       NOT NULL,
            version      integer      NOT NULL,
            name         varchar(128),
            doc_type     varchar(32),
            engine       varchar(16),
            layout_json  jsonb,
            content      text,
            change_note  varchar(255),
            create_by    bigint,
            create_time  timestamp(6) NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_pdf_version_template
            ON mxx_system_pdf_template_version (template_id, version DESC);
    "#;
    db.execute_unprepared(alter_sql).await?;

    // 2) 菜单与权限 seed（幂等）
    //    ⚠️ mxx_system_menu_id_seq 常落后于 MAX(id)，先同步避免主键冲突 23505
    let menu_sql = r#"
        SELECT setval('mxx_system_menu_id_seq', (SELECT MAX(id) FROM mxx_system_menu));

        -- ⚠️ parent_id 必须是「系统管理」目录（67），不能挂在 900（PDF模板列表页）下：
        --    900 是带 component 的页面节点，一旦它有 children，前端会把它渲染成「子菜单父项」，
        --    自身页面从侧边栏消失，且子页面会被嵌进 900 组件的 router-view 里（该页面无 RouterView）→ 打不开。
        --    目录/页面容器请统一用 EmptyLayout，详见 249/370/275。
        INSERT INTO mxx_system_menu
            (name, path, component, route_name, perm, type, parent_id, hide_in_menu, sort, status)
        SELECT 'page.system.pdfDesigner.title',
               '/system/pdf-designer',
               'views/system/pdf-designer/index.vue',
               'SystemPdfDesigner',
               'system:pdf-designer:design',
               'MENU', 67, 0, 61, 1
        WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'system:pdf-designer:design');

        INSERT INTO mxx_system_menu
            (name, path, component, route_name, perm, type, parent_id, hide_in_menu, sort, status)
        SELECT 'page.system.pdfAsset.title',
               '/system/pdf-asset',
               'views/system/pdf-asset/index.vue',
               'SystemPdfAsset',
               'system:pdf-asset:list',
               'MENU', 67, 0, 62, 1
        WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu WHERE perm = 'system:pdf-asset:list');

        INSERT INTO mxx_system_menu
            (name, path, component, route_name, perm, type, parent_id, hide_in_menu, sort, status)
        SELECT v.n, NULL, NULL, NULL, v.p, 'BUTTON', p.id, 0, v.s, 1
        FROM (VALUES
            ('page.system.pdfDesigner.button.preview', 'system:pdf-designer:list',        11),
            ('page.system.pdfDesigner.button.save',    'system:pdf-designer:save',        12),
            ('page.system.pdfDesigner.button.publish', 'system:pdf-designer:set-default', 13),
            ('page.system.pdfDesigner.button.bundle',  'system:pdf-designer:bundle',      14),
            ('page.system.pdfAsset.button.create',     'system:pdf-asset:create',         21),
            ('page.system.pdfAsset.button.edit',       'system:pdf-asset:update',         22),
            ('page.system.pdfAsset.button.delete',     'system:pdf-asset:delete',         23)
        ) AS v(n, p, s)
        CROSS JOIN LATERAL (
            SELECT m.id FROM mxx_system_menu m
            WHERE m.perm = CASE
                    WHEN v.p LIKE 'system:pdf-designer:%' THEN 'system:pdf-designer:design'
                    ELSE 'system:pdf-asset:list'
                END
            ORDER BY m.id LIMIT 1
        ) p
        WHERE NOT EXISTS (SELECT 1 FROM mxx_system_menu m WHERE m.perm = v.p);

        INSERT INTO mxx_system_role_menu_merge (role_id, menu_id, status)
        SELECT r.role_id, m.id, 1
        FROM mxx_system_role_menu_merge r
        CROSS JOIN mxx_system_menu m
        WHERE r.menu_id = 900
          AND m.perm IN (
              'system:pdf-designer:design', 'system:pdf-designer:list',
              'system:pdf-designer:save', 'system:pdf-designer:set-default',
              'system:pdf-designer:bundle',
              'system:pdf-asset:list', 'system:pdf-asset:create',
              'system:pdf-asset:update', 'system:pdf-asset:delete')
          AND NOT EXISTS (
              SELECT 1 FROM mxx_system_role_menu_merge x
              WHERE x.role_id = r.role_id AND x.menu_id = m.id);

        INSERT INTO mxx_system_role_menu_merge (role_id, menu_id, status)
        SELECT 4, m.id, 1
        FROM mxx_system_menu m
        WHERE m.perm IN (
              'system:pdf-designer:design', 'system:pdf-designer:list',
              'system:pdf-designer:save', 'system:pdf-designer:set-default',
              'system:pdf-designer:bundle',
              'system:pdf-asset:list', 'system:pdf-asset:create',
              'system:pdf-asset:update', 'system:pdf-asset:delete')
          AND NOT EXISTS (
              SELECT 1 FROM mxx_system_role_menu_merge x
              WHERE x.role_id = 4 AND x.menu_id = m.id);
    "#;
    db.execute_unprepared(menu_sql).await?;

    db_migration::mark_migration_applied(db, MIGRATION_PDF_DESIGNER_TABLES).await?;
    Ok(())
}

/// PDF 设计器菜单结构修正迁移批次名
const MIGRATION_PDF_MENU_STRUCTURE: &str = "pdf_menu_structure_v90";

/// 修正 PDF 设计器 / 素材页的菜单挂载位置与可见性（v90）。
///
/// v89 把 `system:pdf-designer:design` 与 `system:pdf-asset:list` 两个页面菜单
/// 挂到了 `900`（PDF模板列表页）下，并让设计器页 `hide_in_menu=1`。这在 vben 的
/// `accessMode: backend` 下有两个问题：
/// ① `900` 自带 `component`，一旦它有 children，前端会把它当成「子菜单父项」渲染，
///    模板列表页从侧边栏消失；子页面还会被嵌进 `900` 组件的 `<RouterView>` 里，
///    而该页面没有 `<RouterView>`，导致素材页无法打开；
/// ② 设计器页作为隐藏页，用户无法从菜单进入。
///
/// 本迁移把两者上移为「系统管理」（67）下的一级菜单，并把设计器页恢复为可见，
/// 按钮权限同步挂到各自页面菜单下。幂等：仅在值不一致时更新。
pub async fn fix_pdf_menu_structure(db: &DbConn) -> Result<(), DbErr> {
    if db_migration::migration_applied(db, MIGRATION_PDF_MENU_STRUCTURE).await? {
        return Ok(());
    }

    let fix_sql = r#"
        -- 设计器页 / 素材页：从 900 上移到「系统管理」目录，设计器页恢复菜单可见
        UPDATE mxx_system_menu
           SET parent_id = 67, hide_in_menu = 0, sort = 61
         WHERE perm = 'system:pdf-designer:design'
           AND (parent_id <> 67 OR hide_in_menu <> 0 OR sort <> 61);

        UPDATE mxx_system_menu
           SET parent_id = 67, sort = 62
         WHERE perm = 'system:pdf-asset:list'
           AND (parent_id <> 67 OR sort <> 62);

        -- 按钮权限挂到各自页面菜单下，保证角色授权树层级正确
        UPDATE mxx_system_menu
           SET parent_id = (SELECT m.id FROM mxx_system_menu m
                             WHERE m.perm = 'system:pdf-designer:design'
                             ORDER BY m.id LIMIT 1)
         WHERE perm IN ('system:pdf-designer:list', 'system:pdf-designer:save',
                        'system:pdf-designer:set-default', 'system:pdf-designer:bundle')
           AND parent_id <> (SELECT m.id FROM mxx_system_menu m
                              WHERE m.perm = 'system:pdf-designer:design'
                              ORDER BY m.id LIMIT 1);

        UPDATE mxx_system_menu
           SET parent_id = (SELECT m.id FROM mxx_system_menu m
                             WHERE m.perm = 'system:pdf-asset:list'
                             ORDER BY m.id LIMIT 1)
         WHERE perm IN ('system:pdf-asset:create', 'system:pdf-asset:update',
                        'system:pdf-asset:delete')
           AND parent_id <> (SELECT m.id FROM mxx_system_menu m
                              WHERE m.perm = 'system:pdf-asset:list'
                              ORDER BY m.id LIMIT 1);
    "#;
    db.execute_unprepared(fix_sql).await?;

    db_migration::mark_migration_applied(db, MIGRATION_PDF_MENU_STRUCTURE).await?;
    Ok(())
}
