-- 文章-标签关联表
-- 命名规范：mxx_{module}_{entity1}_{entity2}_merge
CREATE TABLE IF NOT EXISTS mxx_articles_article_label_merge (
  id BIGSERIAL PRIMARY KEY,
  article_id BIGINT NOT NULL,
  label_id BIGINT NOT NULL,
  create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(article_id, label_id)
);

-- 文章ID索引：用于按文章查询其标签
CREATE INDEX IF NOT EXISTS idx_article_label_article ON mxx_articles_article_label_merge(article_id);

-- 标签ID索引：用于按标签查询其文章列表
CREATE INDEX IF NOT EXISTS idx_article_label_label ON mxx_articles_article_label_merge(label_id);
