# Mxx-CRM 客户关系管理系统

> 基于 Rust Actix-Web + Vue3 构建的企业级客户关系管理系统

## 项目简介

Mxx-CRM 是一款现代化的客户关系管理系统，采用前后端分离架构，后端使用 Rust Actix-Web 框架，前端使用 Vue3 + TypeScript + Ant Design Vue。

**核心特点：**
- ✅ **前后端一体化**：前端页面已嵌入后端二进制文件，启动即可访问
- ✅ **高性能**：基于 Rust 语言，性能优异
- ✅ **安全可靠**：JWT 认证 + RBAC 权限控制
- ✅ **模块化设计**：清晰的模块划分，易于扩展

## 技术栈

### 后端技术
| 技术 | 版本 | 说明 |
|------|------|------|
| Rust | 1.77+ | 编程语言 |
| Actix-Web | 4.13+ | Web 框架 |
| SeaORM | 2.0+ | ORM 框架 |
| PostgreSQL | 15+ | 数据库 |
| Redis | 7+ | 缓存 |
| JWT | - | 身份认证 |
| rust-embed | 8.11+ | 静态文件嵌入 |

### 前端技术
| 技术 | 版本 | 说明 |
|------|------|------|
| Vue | 3.4+ | 前端框架 |
| TypeScript | 5.3+ | 类型语言 |
| Ant Design Vue | 4.1+ | UI 组件库 |
| Vite | 5.0+ | 构建工具 |

## 功能模块

### 1. 系统管理模块
- **用户管理**：管理员账号管理、角色分配、状态控制
- **角色管理**：角色创建、权限配置、数据范围设置
- **菜单管理**：动态菜单配置、权限标识管理
- **部门管理**：组织架构管理、部门层级设置
- **岗位管理**：岗位信息维护

### 2. CRM 客户模块
- **线索管理**：线索录入、跟进、转化、回收
- **客户管理**：客户信息管理、等级分类、标签管理
- **联系人管理**：客户联系人信息维护
- **商机管理**：商机阶段跟踪、预测分析
- **合同管理**：合同签订、执行跟踪、归档管理

### 3. 销售模块
- **订单管理**：订单创建、状态跟踪、发货管理
- **支付管理**：支付记录、对账管理
- **产品管理**：产品信息、分类管理、库存管理

### 4. 采购模块
- **供应商管理**：供应商信息维护、评级管理
- **采购订单**：采购申请、审批、入库管理

### 5. 系统配置
- **数据字典**：系统参数配置、枚举值管理
- **系统日志**：操作日志、登录日志记录
- **通知管理**：系统通知、消息推送

## 项目结构

```
MxxCrm/
├── backend/                    # 后端代码
│   ├── src/
│   │   ├── core/              # 核心模块（工具、中间件、配置）
│   │   ├── modules/           # 业务模块
│   │   │   ├── system/        # 系统管理
│   │   │   ├── crm/           # 客户管理
│   │   │   ├── sale/          # 销售管理
│   │   │   └── ...
│   │   ├── routes/            # 路由配置
│   │   ├── utils/             # 工具函数
│   │   ├── embed_frontend.rs  # 前端静态文件嵌入
│   │   └── main.rs            # 入口文件
│   ├── config/                # 配置文件
│   └── Cargo.toml             # Rust 依赖
├── frontend/                  # 前端代码
│   └── apps/
│       └── web-antd/          # Vue3 前端应用
│           ├── src/           # 源代码
│           ├── dist/          # 构建产物（已嵌入后端）
│           └── package.json   # Node 依赖
├── docs/                      # 文档
│   └── 02-数据库设计.md       # 数据库设计文档
├── sql/                       # SQL 脚本
│   ├── init.sql               # 数据库表结构初始化
│   ├── init_data.sql          # 默认数据初始化（角色、用户、菜单）
│   └── migrate_menu_data.sql  # 菜单数据迁移脚本
└── README.md                  # 项目说明
```

## 快速开始

### 环境要求
- Rust 1.77+
- Node.js 20+（仅前端开发时需要）
- PostgreSQL 15+
- Redis 7+

### 前置准备

**1. 构建前端（仅首次或前端代码变更时需要）**

```bash
cd frontend/apps/web-antd

# 安装依赖（首次）
npm install

# 构建前端（产物输出到 dist 目录）
npm run build
```

**2. 初始化数据库

```bash
# 创建数据库
createdb mxx_crm

# 执行初始化脚本
psql -d mxx_crm -f sql/init.sql

# 执行初始化数据脚本（包含菜单、角色、用户等）
psql -d mxx_crm -f sql/init_data.sql
```

### 3. 数据迁移（已有数据时）

如果数据库中已有旧数据（菜单类型为 M/C/F，名称为硬编码中文），需要执行迁移脚本：

```bash
# 迁移菜单类型和国际化键
psql -d mxx_crm -f sql/migrate_menu_data.sql
```

#### SQL脚本说明

| 脚本文件 | 用途 |
|----------|------|
| `sql/init.sql` | 数据库表结构初始化 |
| `sql/init_data.sql` | 默认数据初始化（角色、用户、菜单、示例数据） |
| `sql/migrate_menu_data.sql` | 菜单类型和国际化迁移（M/C/F → FOLDER/MENU/BUTTON，名称→国际化键） |

### 启动服务（前后端一体）

```bash
cd backend

# 设置环境变量
export DATABASE_URL=postgres://username:password@localhost:5432/mxx_crm
export REDIS_URL=redis://localhost:6379/0

# 编译并运行（前端已嵌入二进制）
cargo run --release
```

### 访问地址

| 类型 | 地址 |
|------|------|
| 前端页面 | http://localhost:8088 |
| API 接口 | http://localhost:8088/api |

### 默认账号

| 用户名 | 密码 | 角色 | 部门 | 权限说明 |
|--------|------|------|------|----------|
| admin | admin123 | 超级管理员 | 总经办 | 拥有系统所有权限 |
| system | admin123 | 系统管理员 | 技术部 | 系统配置管理 |
| sales_manager | admin123 | 销售经理 | 销售部 | 销售管理、CRM客户管理 |
| sales_staff | admin123 | 销售人员 | 销售一组 | 销售管理、CRM客户管理（数据范围受限） |
| purchase_staff | admin123 | 采购专员 | 采购部 | 采购管理 |
| crm_manager | admin123 | CRM管理员 | 销售部 | CRM客户管理 |
| product_staff | admin123 | 产品专员 | 技术部 | 产品管理、库存管理 |
| notice_staff | admin123 | 通知专员 | 总经办 | 通知管理 |

## 权限控制

系统采用基于角色的访问控制（RBAC）机制：

1. **用户 → 角色 → 菜单 → 权限标识**
2. **数据范围控制**：全部/自定义/本部门/本部门及以下
3. **控制器注解**：`#[permission("system:admin:list")]`

### 菜单类型定义

| 类型 | 说明 | 前端常量 |
|------|------|----------|
| FOLDER | 目录菜单（不可点击） | `MenuType.FOLDER` |
| MENU | 功能菜单（可点击跳转） | `MenuType.MENU` |
| BUTTON | 按钮权限（用于权限控制） | `MenuType.BUTTON` |

### 菜单国际化

系统支持菜单名称国际化，数据库中存储国际化键，前端通过 `$t()` 函数动态加载多语言文本：

- **国际化键格式**：`page.{module}.{submodule}.title`（菜单）、`button.{module}.{action}`（按钮）
- **示例**：`page.dashboard.title` → 中文"概览" / 英文"Dashboard"
- **国际化文件位置**：`frontend/apps/web-antd/src/locales/langs/`
  - `zh-CN/page.json` - 中文配置
  - `en-US/page.json` - 英文配置

## 部署说明

### 生产构建

```bash
cd backend

# 构建独立可执行文件（包含前端）
cargo build --release

# 产物位置
# target/release/mxx-saas
```

### 运行

```bash
# 设置数据库连接
export DATABASE_URL=postgres://username:password@localhost:5432/mxx_crm
export REDIS_URL=redis://localhost:6379/0

# 直接运行可执行文件
./target/release/mxx-saas
```

**注意**：启动后无需额外启动前端服务，前端页面已嵌入在二进制文件中，直接访问端口即可。

## 开发说明

### 前端开发

```bash
cd frontend/apps/web-antd

# 开发模式（热更新）
npm run dev

# 此时前端会代理 API 请求到后端
# 访问：http://localhost:5173
```

### 后端开发

```bash
cd backend

# 开发模式（自动重启）
cargo watch -x run
```

## 许可证

Copyright (c) 2024-2999 北京心月狐科技有限公司

## 联系方式

- 官网：https://www.mxxshop.com
- 邮箱：contact@mxxshop.com