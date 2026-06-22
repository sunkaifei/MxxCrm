SET client_encoding = 'UTF8';

-- SKU template main table
CREATE TABLE IF NOT EXISTS mxx_sku_template (
    id              BIGSERIAL PRIMARY KEY,
    template_name   VARCHAR(128) NOT NULL,
    product_type    VARCHAR(32) DEFAULT NULL,
    description     TEXT DEFAULT NULL,
    created_by      BIGINT DEFAULT NULL,
    create_time     TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_by      BIGINT DEFAULT NULL,
    update_time     TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted         INT DEFAULT 0
);

-- Template spec definition table
CREATE TABLE IF NOT EXISTS mxx_sku_template_spec (
    id              BIGSERIAL PRIMARY KEY,
    template_id     BIGINT NOT NULL REFERENCES mxx_sku_template(id) ON DELETE CASCADE,
    name            VARCHAR(64) NOT NULL,
    sort_order      INT DEFAULT 0,
    create_time     TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time     TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_template_spec_template ON mxx_sku_template_spec(template_id);

-- Template spec value table
CREATE TABLE IF NOT EXISTS mxx_sku_template_spec_value (
    id              BIGSERIAL PRIMARY KEY,
    spec_id         BIGINT NOT NULL REFERENCES mxx_sku_template_spec(id) ON DELETE CASCADE,
    value           VARCHAR(128) NOT NULL,
    sort_order      INT DEFAULT 0,
    create_time     TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time     TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_template_spec_value_spec ON mxx_sku_template_spec_value(spec_id);

-- Add template_id column to product main table
ALTER TABLE mxx_product_main ADD COLUMN IF NOT EXISTS template_id BIGINT DEFAULT NULL;
CREATE INDEX IF NOT EXISTS idx_product_main_template ON mxx_product_main(template_id);
