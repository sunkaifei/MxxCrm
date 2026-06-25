-- 创建合同审批日志表
CREATE TABLE IF NOT EXISTS mxx_crm_contract_approval_log (
    id              BIGSERIAL       PRIMARY KEY,
    contract_id     BIGINT          NOT NULL,
    action          INTEGER         NOT NULL,
    operator_id     BIGINT          NOT NULL,
    operator_name   VARCHAR(64),
    reason          VARCHAR(500),
    previous_status INTEGER,
    new_status      INTEGER,
    current_stage   INTEGER,
    next_stage      INTEGER,
    create_time     TIMESTAMP       DEFAULT CURRENT_TIMESTAMP,
    deleted         INTEGER         DEFAULT 0
);

COMMENT ON TABLE  mxx_crm_contract_approval_log IS '合同审批日志表';
COMMENT ON COLUMN mxx_crm_contract_approval_log.id              IS '主键ID';
COMMENT ON COLUMN mxx_crm_contract_approval_log.contract_id     IS '合同ID';
COMMENT ON COLUMN mxx_crm_contract_approval_log.action           IS '操作类型（1=提交，2=审批通过，3=驳回）';
COMMENT ON COLUMN mxx_crm_contract_approval_log.operator_id     IS '操作人ID';
COMMENT ON COLUMN mxx_crm_contract_approval_log.operator_name   IS '操作人姓名';
COMMENT ON COLUMN mxx_crm_contract_approval_log.reason           IS '审批意见/驳回原因';
COMMENT ON COLUMN mxx_crm_contract_approval_log.previous_status  IS '操作前审批状态';
COMMENT ON COLUMN mxx_crm_contract_approval_log.new_status       IS '操作后审批状态';
COMMENT ON COLUMN mxx_crm_contract_approval_log.current_stage   IS '当前审批阶段';
COMMENT ON COLUMN mxx_crm_contract_approval_log.next_stage       IS '下一审批阶段';
COMMENT ON COLUMN mxx_crm_contract_approval_log.create_time     IS '创建时间';
COMMENT ON COLUMN mxx_crm_contract_approval_log.deleted           IS '软删除标识（0-未删除，1-已删除）';

CREATE INDEX IF NOT EXISTS idx_contract_approval_log_contract_id ON mxx_crm_contract_approval_log (contract_id);
CREATE INDEX IF NOT EXISTS idx_contract_approval_log_create_time ON mxx_crm_contract_approval_log (create_time);
