-- ============================================================
-- Mxx-CRM 数据库字段级问题修复脚本
-- 基于 schema_comparison_report.md 第五节
-- ============================================================

BEGIN;

-- ============================================================
-- 5.1 字段命名不统一：重命名 updated_time → update_time
-- ============================================================

-- 重命名 updated_time → update_time（表中仅有 updated_time 的情况）
ALTER TABLE mxx_crm_followup RENAME COLUMN updated_time TO update_time;
ALTER TABLE mxx_crm_lead RENAME COLUMN updated_time TO update_time;
ALTER TABLE mxx_sale_payment RENAME COLUMN updated_time TO update_time;
ALTER TABLE mxx_system_area RENAME COLUMN updated_time TO update_time;
ALTER TABLE mxx_system_tag RENAME COLUMN updated_time TO update_time;
ALTER TABLE mxx_system_tag_group RENAME COLUMN updated_time TO update_time;
ALTER TABLE mxx_sale_order_item RENAME COLUMN updated_time TO update_time;

-- 重命名 created_at → create_time
ALTER TABLE mxx_system_tag_merge RENAME COLUMN created_at TO create_time;

-- 删除冗余的 updated_time 字段（这些表同时存在 updated_time 和 update_time）
ALTER TABLE mxx_inventory_stock DROP COLUMN updated_time;
ALTER TABLE mxx_inventory_warehouse DROP COLUMN updated_time;
ALTER TABLE mxx_sale_order DROP COLUMN updated_time;
ALTER TABLE mxx_purchase_po DROP COLUMN updated_time;
ALTER TABLE mxx_purchase_supplier DROP COLUMN updated_time;

-- ============================================================
-- 5.2 TIMESTAMPTZ → TIMESTAMP（项目规范要求使用 TIMESTAMP）
-- 转换为 Asia/Shanghai 时区的 wall-clock 时间
-- ============================================================

-- mxx_product
ALTER TABLE mxx_product ALTER COLUMN create_time TYPE TIMESTAMP WITHOUT TIME ZONE USING create_time AT TIME ZONE 'Asia/Shanghai';
ALTER TABLE mxx_product ALTER COLUMN update_time TYPE TIMESTAMP WITHOUT TIME ZONE USING update_time AT TIME ZONE 'Asia/Shanghai';

-- mxx_product_category
ALTER TABLE mxx_product_category ALTER COLUMN create_time TYPE TIMESTAMP WITHOUT TIME ZONE USING create_time AT TIME ZONE 'Asia/Shanghai';
ALTER TABLE mxx_product_category ALTER COLUMN update_time TYPE TIMESTAMP WITHOUT TIME ZONE USING update_time AT TIME ZONE 'Asia/Shanghai';

-- mxx_product_sku
ALTER TABLE mxx_product_sku ALTER COLUMN create_time TYPE TIMESTAMP WITHOUT TIME ZONE USING create_time AT TIME ZONE 'Asia/Shanghai';
ALTER TABLE mxx_product_sku ALTER COLUMN update_time TYPE TIMESTAMP WITHOUT TIME ZONE USING update_time AT TIME ZONE 'Asia/Shanghai';

-- mxx_system_area（update_time 已由 updated_time 重命名而来）
ALTER TABLE mxx_system_area ALTER COLUMN create_time TYPE TIMESTAMP WITHOUT TIME ZONE USING create_time AT TIME ZONE 'Asia/Shanghai';
ALTER TABLE mxx_system_area ALTER COLUMN update_time TYPE TIMESTAMP WITHOUT TIME ZONE USING update_time AT TIME ZONE 'Asia/Shanghai';

-- mxx_system_tag
ALTER TABLE mxx_system_tag ALTER COLUMN create_time TYPE TIMESTAMP WITHOUT TIME ZONE USING create_time AT TIME ZONE 'Asia/Shanghai';
ALTER TABLE mxx_system_tag ALTER COLUMN update_time TYPE TIMESTAMP WITHOUT TIME ZONE USING update_time AT TIME ZONE 'Asia/Shanghai';

-- mxx_system_tag_group
ALTER TABLE mxx_system_tag_group ALTER COLUMN create_time TYPE TIMESTAMP WITHOUT TIME ZONE USING create_time AT TIME ZONE 'Asia/Shanghai';
ALTER TABLE mxx_system_tag_group ALTER COLUMN update_time TYPE TIMESTAMP WITHOUT TIME ZONE USING update_time AT TIME ZONE 'Asia/Shanghai';

-- mxx_system_tag_merge（create_time 已由 created_at 重命名而来）
ALTER TABLE mxx_system_tag_merge ALTER COLUMN create_time TYPE TIMESTAMP WITHOUT TIME ZONE USING create_time AT TIME ZONE 'Asia/Shanghai';

-- mxx_sale_payment（update_time 已由 updated_time 重命名而来）
ALTER TABLE mxx_sale_payment ALTER COLUMN create_time TYPE TIMESTAMP WITHOUT TIME ZONE USING create_time AT TIME ZONE 'Asia/Shanghai';
ALTER TABLE mxx_sale_payment ALTER COLUMN update_time TYPE TIMESTAMP WITHOUT TIME ZONE USING update_time AT TIME ZONE 'Asia/Shanghai';

-- mxx_inventory_transaction（created_at 保留原名，仅改类型）
ALTER TABLE mxx_inventory_transaction ALTER COLUMN created_at TYPE TIMESTAMP WITHOUT TIME ZONE USING created_at AT TIME ZONE 'Asia/Shanghai';

-- mxx_attachment_file（created_at 保留原名，仅改类型）
ALTER TABLE mxx_attachment_file ALTER COLUMN created_at TYPE TIMESTAMP WITHOUT TIME ZONE USING created_at AT TIME ZONE 'Asia/Shanghai';

-- mxx_seq_generator（updated_at 保留原名，仅改类型）
ALTER TABLE mxx_seq_generator ALTER COLUMN updated_at TYPE TIMESTAMP WITHOUT TIME ZONE USING updated_at AT TIME ZONE 'Asia/Shanghai';

-- ============================================================
-- 5.3 字段名不匹配
-- ============================================================

-- 修复拼写错误：sorce_url_type → source_url_type
ALTER TABLE mxx_statistics_access_record RENAME COLUMN sorce_url_type TO source_url_type;

-- 注：mxx_system_post 的 is_del→deleted 仅需改 entity（数据库已是 deleted）
-- 注：mxx_system_tag_merge 的 created_at→create_time 已在 5.1 处理

-- ============================================================
-- 5.4 字段类型不匹配
-- ============================================================

-- mxx_system_admin_notice_merge.is_read: bigint → integer
ALTER TABLE mxx_system_admin_notice_merge ALTER COLUMN is_read TYPE INTEGER USING is_read::INTEGER;

-- mxx_system_admin_post_merge.admin_id: integer → bigint
ALTER TABLE mxx_system_admin_post_merge ALTER COLUMN admin_id TYPE BIGINT USING admin_id::BIGINT;

-- mxx_system_admin_post_merge.post_id: integer → bigint
ALTER TABLE mxx_system_admin_post_merge ALTER COLUMN post_id TYPE BIGINT USING post_id::BIGINT;

-- mxx_statistics_access_record.login_user_id: integer → bigint
ALTER TABLE mxx_statistics_access_record ALTER COLUMN login_user_id TYPE BIGINT USING login_user_id::BIGINT;

-- mxx_statistics_performance_target.id: integer → bigint（主键）
ALTER TABLE mxx_statistics_performance_target ALTER COLUMN id TYPE BIGINT USING id::BIGINT;

-- ============================================================
-- 5.6 数据库补充缺失字段
-- ============================================================

-- mxx_sale_order_item（DB 缺少 entity 中已有的字段）
-- 注：sku, discount, total_amount, notes, deleted 已存在
-- 注：update_time 已由 updated_time 重命名而来
ALTER TABLE mxx_sale_order_item ADD COLUMN IF NOT EXISTS product_code VARCHAR(64);
ALTER TABLE mxx_sale_order_item ADD COLUMN IF NOT EXISTS spec VARCHAR(255);
ALTER TABLE mxx_sale_order_item ADD COLUMN IF NOT EXISTS unit VARCHAR(32);
ALTER TABLE mxx_sale_order_item ADD COLUMN IF NOT EXISTS amount NUMERIC(18,2);
ALTER TABLE mxx_sale_order_item ADD COLUMN IF NOT EXISTS tax_amount NUMERIC(18,2);
ALTER TABLE mxx_sale_order_item ADD COLUMN IF NOT EXISTS remark VARCHAR(255);

-- mxx_sale_payment（DB 缺少 entity 中已有的字段）
-- 注：bank_account, transaction_id, notes 已存在
-- 注：update_time 已由 updated_time 重命名而来
ALTER TABLE mxx_sale_payment ADD COLUMN IF NOT EXISTS contract_id BIGINT;
ALTER TABLE mxx_sale_payment ADD COLUMN IF NOT EXISTS status VARCHAR(32);
ALTER TABLE mxx_sale_payment ADD COLUMN IF NOT EXISTS transaction_no VARCHAR(64);
ALTER TABLE mxx_sale_payment ADD COLUMN IF NOT EXISTS remark VARCHAR(255);

COMMIT;
