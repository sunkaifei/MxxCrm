CREATE TABLE IF NOT EXISTS mxx_system_pdf_download_log (
    id BIGSERIAL PRIMARY KEY,
    record_id BIGINT NOT NULL,
    doc_type VARCHAR(50),
    doc_id BIGINT,
    doc_no VARCHAR(200),
    file_name VARCHAR(500),
    operator_id BIGINT,
    operator_name VARCHAR(100),
    ip_address VARCHAR(100),
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_pdf_dl_record_id ON mxx_system_pdf_download_log(record_id);
CREATE INDEX IF NOT EXISTS idx_pdf_dl_create_time ON mxx_system_pdf_download_log(create_time);
CREATE INDEX IF NOT EXISTS idx_pdf_dl_doc_type ON mxx_system_pdf_download_log(doc_type);
