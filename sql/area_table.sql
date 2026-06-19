CREATE TABLE IF NOT EXISTS mxx_system_area (
    id          BIGSERIAL PRIMARY KEY,
    parent_id   BIGINT DEFAULT 0,
    name        VARCHAR(128) NOT NULL,
    name_en     VARCHAR(128),
    code        VARCHAR(64),
    level       INT DEFAULT 1,
    sort        INT DEFAULT 0,
    country_code VARCHAR(10),
    latitude    DECIMAL(10,6),
    longitude   DECIMAL(10,6),
    created_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_area_parent_id ON mxx_system_area(parent_id);
CREATE INDEX IF NOT EXISTS idx_area_level ON mxx_system_area(level);
CREATE INDEX IF NOT EXISTS idx_area_country_code ON mxx_system_area(country_code);

COMMENT ON TABLE mxx_system_area IS '行政区划表';
COMMENT ON COLUMN mxx_system_area.id IS '主键';
COMMENT ON COLUMN mxx_system_area.parent_id IS '父级ID';
COMMENT ON COLUMN mxx_system_area.name IS '名称';
COMMENT ON COLUMN mxx_system_area.name_en IS '英文名称';
COMMENT ON COLUMN mxx_system_area.code IS '编码';
COMMENT ON COLUMN mxx_system_area.level IS '层级:1国家/2省州/3市/4区县';
COMMENT ON COLUMN mxx_system_area.sort IS '排序';
COMMENT ON COLUMN mxx_system_area.country_code IS '国家代码(ISO 3166-1)';
COMMENT ON COLUMN mxx_system_area.latitude IS '纬度';
COMMENT ON COLUMN mxx_system_area.longitude IS '经度';
