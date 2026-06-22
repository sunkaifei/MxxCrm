-- 产品SKU变体表
CREATE TABLE IF NOT EXISTS mxx_product_sku (
    id              BIGSERIAL PRIMARY KEY,
    product_id      BIGINT NOT NULL REFERENCES mxx_product_main(id) ON DELETE CASCADE,
    sku_code        VARCHAR(64) NOT NULL,
    color           VARCHAR(32),
    size            VARCHAR(32),
    price           NUMERIC(15,2),
    cost_price      NUMERIC(15,2),
    stock           INTEGER DEFAULT 0,
    image_url       VARCHAR(255),
    is_active       BOOLEAN DEFAULT true,
    created_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- SKU编码唯一索引
CREATE UNIQUE INDEX IF NOT EXISTS idx_product_sku_code ON mxx_product_sku(sku_code);

-- SKU所属产品索引
CREATE INDEX IF NOT EXISTS idx_product_sku_product ON mxx_product_sku(product_id);
