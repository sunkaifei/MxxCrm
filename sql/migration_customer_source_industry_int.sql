-- 客户表 source / industry 枚举字段转为 INTEGER
-- 注意：不删除 mxx_lead_source / mxx_industry_type 枚举类型，因为 lead / opportunity 表仍在使用

-- source 映射：1=官网 2=展会 3=社交媒体 4=客户转介 5=陌生拜访 6=海关数据 7=邮件营销 8=阿里国际站 9=Amazon 10=TikTok 11=微信 12=其他
-- industry 映射：1=零售 2=批发 3=制造 4=贸易代理 5=电商 6=微商 7=社交电商 8=其他

-- 1. source 字段：枚举 -> INTEGER
ALTER TABLE mxx_crm_customer ALTER COLUMN source DROP DEFAULT;

ALTER TABLE mxx_crm_customer
  ALTER COLUMN source TYPE INTEGER
  USING (
    CASE source::text
      WHEN 'website'       THEN 1
      WHEN 'exhibition'   THEN 2
      WHEN 'social'       THEN 3
      WHEN 'referral'     THEN 4
      WHEN 'cold_call'    THEN 5
      WHEN 'customs'      THEN 6
      WHEN 'email'        THEN 7
      WHEN 'alibaba'      THEN 8
      WHEN 'amazon'       THEN 9
      WHEN 'tiktok'       THEN 10
      WHEN 'wechat'       THEN 11
      WHEN 'other'        THEN 12
      ELSE NULL
    END
  );

-- 2. industry 字段：枚举 -> INTEGER
ALTER TABLE mxx_crm_customer ALTER COLUMN industry DROP DEFAULT;

ALTER TABLE mxx_crm_customer
  ALTER COLUMN industry TYPE INTEGER
  USING (
    CASE industry::text
      WHEN 'retail'          THEN 1
      WHEN 'wholesale'       THEN 2
      WHEN 'manufacturer'    THEN 3
      WHEN 'trade_agent'     THEN 4
      WHEN 'ecommerce'       THEN 5
      WHEN 'wechat_business' THEN 6
      WHEN 'social'          THEN 7
      WHEN 'other'           THEN 8
      ELSE NULL
    END
  );
