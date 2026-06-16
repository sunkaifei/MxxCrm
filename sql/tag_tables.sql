CREATE TABLE IF NOT EXISTS mxx_system_tag_group (
    id BIGSERIAL PRIMARY KEY,
    group_name VARCHAR(64) NOT NULL,
    group_color VARCHAR(16) DEFAULT '#1890ff',
    sort_order INT DEFAULT 0,
    description VARCHAR(255),
    is_global BOOLEAN DEFAULT true,
    created_by BIGINT REFERENCES mxx_system_admin(id),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_by BIGINT REFERENCES mxx_system_admin(id),
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted INT DEFAULT 0
);

CREATE TABLE IF NOT EXISTS mxx_system_tag (
    id BIGSERIAL PRIMARY KEY,
    group_id BIGINT REFERENCES mxx_system_tag_group(id),
    tag_name VARCHAR(64) NOT NULL,
    tag_color VARCHAR(16) DEFAULT '#1890ff',
    description VARCHAR(255),
    is_global BOOLEAN DEFAULT true,
    created_by BIGINT REFERENCES mxx_system_admin(id),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_by BIGINT REFERENCES mxx_system_admin(id),
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted INT DEFAULT 0
);

CREATE TABLE IF NOT EXISTS mxx_system_tag_merge (
    id BIGSERIAL PRIMARY KEY,
    tag_id BIGINT NOT NULL REFERENCES mxx_system_tag(id),
    entity_type VARCHAR(32) NOT NULL,
    entity_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_tag_group_name ON mxx_system_tag_group(group_name, deleted);
CREATE INDEX IF NOT EXISTS idx_tag_group_created_by ON mxx_system_tag_group(created_by, deleted);
CREATE INDEX IF NOT EXISTS idx_tag_group_sort ON mxx_system_tag_group(sort_order, deleted);

CREATE INDEX IF NOT EXISTS idx_tag_group_id ON mxx_system_tag(group_id, deleted);
CREATE INDEX IF NOT EXISTS idx_tag_created_by ON mxx_system_tag(created_by, deleted);
CREATE UNIQUE INDEX IF NOT EXISTS idx_tag_merge_unique ON mxx_system_tag_merge(tag_id, entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_tag_merge_entity ON mxx_system_tag_merge(entity_type, entity_id);

INSERT INTO mxx_system_tag_group (group_name, group_color, sort_order, description, is_global) VALUES
('来源标签', '#1890ff', 1, '用于标记线索/客户的来源渠道', true),
('意向度标签', '#faad14', 2, '用于标记客户购买意向程度', true),
('产品偏好', '#52c41a', 3, '用于标记客户感兴趣的产品类别', true),
('购买偏好', '#722ed1', 4, '用于标记客户购买决策偏好', true),
('客户等级', '#f5222d', 5, '用于标记客户重要程度', true),
('其他标签', '#d9d9d9', 6, '其他自定义标签', true);

INSERT INTO mxx_system_tag (group_id, tag_name, tag_color, description, is_global) VALUES
(1, '展会', '#1890ff', '展会获取的线索', true),
(1, '阿里国际站', '#52c41a', '阿里国际站询盘', true),
(1, '老客户推荐', '#faad14', '老客户转介绍', true),
(1, '官网', '#722ed1', '官网表单提交', true),
(2, '高意向', '#f5222d', '对产品有强烈购买意愿', true),
(2, '中意向', '#faad14', '有一定购买意愿', true),
(2, '低意向', '#1890ff', '购买意愿较低', true),
(2, '暂无意向', '#d9d9d9', '暂时没有购买计划', true),
(5, 'VIP', '#f5222d', '重要客户', true),
(5, 'SVIP', '#722ed1', '核心客户', true),
(5, '普通', '#1890ff', '普通客户', true),
(5, '潜在', '#d9d9d9', '潜在客户', true),
(4, '价格敏感', '#faad14', '对价格比较敏感', true),
(4, '品质优先', '#52c41a', '重视产品品质', true),
(4, '交货期优先', '#1890ff', '重视交货时间', true),
(6, '需跟进', '#f5222d', '需要及时跟进', true),
(6, '已报价', '#52c41a', '已发送报价单', true),
(6, '待回款', '#faad14', '等待客户回款', true);
