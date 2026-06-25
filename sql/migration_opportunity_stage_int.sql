-- 商机阶段枚举转为 INTEGER
-- 映射关系：0=资格审查 1=需求分析 2=方案报价 3=商务谈判 4=已成交 5=已输单

-- 1. 移除 stage 列的默认值
ALTER TABLE mxx_crm_opportunity ALTER COLUMN stage DROP DEFAULT;

-- 2. 将枚举值转为整数
ALTER TABLE mxx_crm_opportunity
  ALTER COLUMN stage TYPE INTEGER
  USING (
    CASE stage::text
      WHEN 'qualification' THEN 0
      WHEN 'needs_analysis' THEN 1
      WHEN 'proposal' THEN 2
      WHEN 'negotiation' THEN 3
      WHEN 'won' THEN 4
      WHEN 'lost' THEN 5
      ELSE 0
    END
  );

-- 3. 设置新的默认值（0 = 资格审查）
ALTER TABLE mxx_crm_opportunity ALTER COLUMN stage SET DEFAULT 0;

-- 4. 删除枚举类型
DROP TYPE IF EXISTS mxx_opportunity_stage CASCADE;
