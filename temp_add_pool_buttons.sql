-- 在公海客户菜单 (id=366) 下添加两个按钮
INSERT INTO mxx_system_menu (id, parent_id, tree_path, name, type, perm, status, sort) 
VALUES 
(380, 366, '/366', 'return_pool_btn', 'BUTTON', 'crm:customer:return-pool', 1, 1),
(381, 366, '/366', 'claim_btn', 'BUTTON', 'crm:customer:claim', 1, 2);
