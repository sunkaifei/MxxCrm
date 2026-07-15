-- 客户表 industry 字段从枚举类型改为 INT4
-- 将 mxx_crm_customer 表中的 industry 列从 mxx_industry_type 枚举类型修改为 INTEGER

-- 1. 创建临时列
ALTER TABLE mxx_crm_customer
  ADD COLUMN IF NOT EXISTS industry_int INTEGER;

-- 2. 将已有的枚举值转换为对应的整数编码
UPDATE mxx_crm_customer
SET industry_int = CASE
  WHEN industry::text = 'retail' THEN 1
  WHEN industry::text = 'wholesale' THEN 2
  WHEN industry::text = 'manufacturer' THEN 3
  WHEN industry::text = 'trade_agent' THEN 4
  WHEN industry::text = 'ecommerce' THEN 5
  WHEN industry::text = 'wechat_business' THEN 6
  WHEN industry::text = 'social' THEN 7
  WHEN industry::text = 'other' THEN 8
  ELSE NULL
END;

-- 3. 删除旧列，重命名新列
ALTER TABLE mxx_crm_customer DROP COLUMN industry;
ALTER TABLE mxx_crm_customer RENAME COLUMN industry_int TO industry;

-- 4. 为 industry 列创建索引
CREATE INDEX IF NOT EXISTS idx_mxx_crm_customer_industry ON mxx_crm_customer(industry);
