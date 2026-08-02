-- 为员工添加银行卡信息（用于银行代发文件导出测试）
-- 更新 admin 表的 bank_card_no、bank_name、bank_account_name 字段

UPDATE mxx_system_admin SET
    bank_card_no = '6222021234567890001',
    bank_name = '工商银行',
    bank_account_name = '超级管理员'
WHERE id = 3 AND (bank_card_no IS NULL OR bank_card_no = '');

UPDATE mxx_system_admin SET
    bank_card_no = '6222021234567890002',
    bank_name = '工商银行',
    bank_account_name = '销售经理'
WHERE id = 6 AND (bank_card_no IS NULL OR bank_card_no = '');

UPDATE mxx_system_admin SET
    bank_card_no = '6222021234567890003',
    bank_name = '工商银行',
    bank_account_name = '业务员'
WHERE id = 7 AND (bank_card_no IS NULL OR bank_card_no = '');

-- 同时为 mobile 字段补充（工行格式需要手机号）
UPDATE mxx_system_admin SET mobile = '13800000001' WHERE id = 3 AND (mobile IS NULL OR mobile = '');
UPDATE mxx_system_admin SET mobile = '13800000002' WHERE id = 6 AND (mobile IS NULL OR mobile = '');
UPDATE mxx_system_admin SET mobile = '13800000003' WHERE id = 7 AND (mobile IS NULL OR mobile = '');

SELECT id, user_name, nick_name, bank_card_no, bank_name, bank_account_name, mobile
FROM mxx_system_admin
WHERE id IN (3, 6, 7);
