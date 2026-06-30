-- Attachment table rebuild script
-- Drop and recreate mxx_attachment table (new design, not compatible with old data)

-- 1. Drop old table
DROP TABLE IF EXISTS public.mxx_attachment CASCADE;

-- 2. Create new attachment table
CREATE TABLE public.mxx_attachment (
    id              BIGSERIAL    NOT NULL PRIMARY KEY,
    -- File basic info
    name            VARCHAR(255),              -- stored filename (UUID generated)
    original_name   VARCHAR(255),              -- original filename when uploaded
    path            VARCHAR(500),              -- relative storage path on server
    upload_url      VARCHAR(500),              -- access URL
    ext             VARCHAR(50),               -- file extension (e.g. jpg/pdf/docx)
    size            BIGINT,                    -- file size in bytes
    mime_type       VARCHAR(128),              -- MIME type (e.g. image/jpeg)
    file_hash       VARCHAR(64),               -- SHA-256 hash (deduplication + integrity)
    md5             VARCHAR(255),              -- MD5 (compatibility)
    -- Business relation
    entity_type     VARCHAR(64),               -- entity type: product/contract/invoice/quotation/avatar/payment/common
    entity_id       BIGINT,                    -- related business ID
    uploaded_by     BIGINT,                    -- uploader user ID
    -- Category and storage
    type_id         BIGINT,                    -- category ID (optional)
    storage_type    INTEGER     DEFAULT 1,     -- 1-local 2-qiniu 3-aliyun 4-tencent
    is_public       BOOLEAN     DEFAULT FALSE, -- public access flag
    -- Status management
    status          INTEGER     DEFAULT 0,     -- 0-normal 1-disabled
    count_quote     BIGINT      DEFAULT 0,     -- reference count
    create_time     TIMESTAMP,
    -- Soft delete
    deleted         INTEGER     DEFAULT 0      -- 0-not deleted 1-deleted
);

-- 3. Create indexes
CREATE INDEX idx_attachment_entity      ON public.mxx_attachment (entity_type, entity_id);
CREATE INDEX idx_attachment_uploaded_by ON public.mxx_attachment (uploaded_by);
CREATE INDEX idx_attachment_file_hash   ON public.mxx_attachment (file_hash);
CREATE INDEX idx_attachment_md5         ON public.mxx_attachment (md5);
