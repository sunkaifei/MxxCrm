SET client_encoding = 'UTF8';

-- Add image column to product category table
ALTER TABLE mxx_product_category ADD COLUMN IF NOT EXISTS image VARCHAR(512) DEFAULT NULL;
