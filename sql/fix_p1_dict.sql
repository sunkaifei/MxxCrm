-- P1: 补充 data_scope 字典值
DO $$
DECLARE
    dict_type_id BIGINT;
BEGIN
    -- 获取或创建字典类型
    SELECT id INTO dict_type_id FROM mxx_system_dict WHERE dict_code = 'data_scope';
    
    IF dict_type_id IS NULL THEN
        INSERT INTO mxx_system_dict (dict_name, dict_code, sort, status, create_time, update_time)
        VALUES ('数据范围', 'data_scope', 8, 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        RETURNING id INTO dict_type_id;
    END IF;

    -- 插入或忽略 仅本人数据权限
    INSERT INTO mxx_system_dict_data (dict_label, dict_value, dict_code, dict_sort, status, is_default, create_time, update_time)
    SELECT '仅本人数据权限', '5', 'data_scope', 4, 0, 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
    WHERE NOT EXISTS (SELECT 1 FROM mxx_system_dict_data WHERE dict_code = 'data_scope' AND dict_value = '5');
END $$;
