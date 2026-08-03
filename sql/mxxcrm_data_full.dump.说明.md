# 数据库备份说明

## 备份文件

- **文件名**: `mxxcrm_data_full.dump`
- **格式**: PostgreSQL custom(压缩二进制,`-Fc`)
- **来源**: 生产库 `115.190.210.106:5432/mxxcrm_data`
- **备份时间**: 2026-08-03 02:10:50
- **文件大小**: 1.78 MB

## 恢复命令(Linux)

```bash
PGPASSWORD="目标库密码" pg_restore -h 目标主机 -p 5432 -U postgres -d 目标库名 -c /path/to/mxxcrm_data_full.dump
```

**参数说明:**
- `-h` 目标主机地址
- `-p` 端口(默认 5432)
- `-U` 数据库用户名
- `-d` 目标数据库名
- `-c` 先 drop 已有对象再创建(会清空目标库现有数据,谨慎使用)

**前置要求:**
- 已安装 postgresql-client(`pg_restore` 版本需 ≥ 16.0)
- 目标数据库需已存在

**示例:**
```bash
# 新建空库后恢复
createdb -h 127.0.0.1 -U postgres mxxcrm_data_restore
PGPASSWORD="xxx" pg_restore -h 127.0.0.1 -U postgres -d mxxcrm_data_restore /path/to/mxxcrm_data_full.dump

# 覆盖恢复(危险!会清空目标库现有数据)
PGPASSWORD="xxx" pg_restore -h 127.0.0.1 -U postgres -d mxxcrm_data -c /path/to/mxxcrm_data_full.dump
```
