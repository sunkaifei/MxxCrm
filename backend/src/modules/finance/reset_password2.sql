UPDATE mxx_system_admin SET password = '$2a$10$ddekNYPqokhs.N77STQjeOLE6vmJKOo95G.1yJc4CHPCVC9vZI/O2' WHERE id IN (3, 11);
SELECT id, user_name, substring(password, 1, 4) AS prefix, length(password) AS pwd_len FROM mxx_system_admin WHERE id IN (3, 11);
