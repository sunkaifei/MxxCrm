UPDATE mxx_system_admin 
SET password = '$2b$12$8kVReYyYJFw2NE9Sv5jQwuV0KiTZ9GP4FnsPLoxdKFgEtV2J0Myn6'
WHERE user_name = 'admin';

UPDATE mxx_system_admin 
SET password = '$2b$12$8kVReYyYJFw2NE9Sv5jQwuV0KiTZ9GP4FnsPLoxdKFgEtV2J0Myn6'
WHERE user_name = 'system';

UPDATE mxx_system_admin 
SET password = '$2b$12$8kVReYyYJFw2NE9Sv5jQwuV0KiTZ9GP4FnsPLoxdKFgEtV2J0Myn6'
WHERE user_name = 'sales';

UPDATE mxx_system_admin 
SET password = '$2b$12$8kVReYyYJFw2NE9Sv5jQwuV0KiTZ9GP4FnsPLoxdKFgEtV2J0Myn6'
WHERE user_name = 'manager';

UPDATE mxx_system_admin 
SET password = '$2b$12$8kVReYyYJFw2NE9Sv5jQwuV0KiTZ9GP4FnsPLoxdKFgEtV2J0Myn6'
WHERE user_name = 'rep';

UPDATE mxx_system_admin 
SET password = '$2b$12$8kVReYyYJFw2NE9Sv5jQwuV0KiTZ9GP4FnsPLoxdKFgEtV2J0Myn6'
WHERE user_name = 'purchase';

SELECT '密码重置完成，所有用户密码已更新为 admin123' AS result;