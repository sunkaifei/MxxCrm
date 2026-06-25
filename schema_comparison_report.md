# 数据库与本地程序（Entity）对比核查报告

**核查日期**: 2026-06-24
**数据库**: mxxcrm_data @ 115.190.210.106:5432
**对比基准**: 以本地程序（backend/src/modules/*/entity/*.rs）为主

---

## 一、总体情况

| 项目 | 数量 |
|------|------|
| 数据库现有表 | 57 张 |
| 本地程序 Entity 定义的表 | 约 100+ 张 |
| 数据库缺失的表 | 约 50 张 |
| 数据库有但无 Entity 的表 | 6 张 |
| 表名不匹配 | 2 张 |

---

## 二、数据库缺失的表（Entity 有定义，数据库无对应表）

### 2.1 finance 模块（全部 5 张缺失）

| 表名 | Entity 文件路径 |
|------|----------------|
| mxx_member_fee | backend/src/modules/finance/entity/member_fee.rs |
| mxx_member_product | backend/src/modules/finance/entity/member_product.rs |
| mxx_payment_record | backend/src/modules/finance/entity/payment_record.rs |
| mxx_refund_record | backend/src/modules/finance/entity/refund_record.rs |
| mxx_finance_statistics | backend/src/modules/finance/entity/finance_statistics.rs |

### 2.2 shop 模块（全部 17 张缺失）

| 表名 | Entity 文件路径 |
|------|----------------|
| mxx_shop | backend/src/modules/shop/entity/shop.rs |
| mxx_shop_cart | backend/src/modules/shop/entity/shop_cart.rs |
| mxx_shop_category | backend/src/modules/shop/entity/shop_category.rs |
| mxx_shop_delivery | backend/src/modules/shop/entity/shop_delivery.rs |
| mxx_shop_order | backend/src/modules/shop/entity/shop_order.rs |
| mxx_shop_order_item | backend/src/modules/shop/entity/shop_order_item.rs |
| mxx_shop_promotion | backend/src/modules/shop/entity/shop_promotion.rs |
| mxx_shop_promotion_spu | backend/src/modules/shop/entity/shop_promotion_spu.rs |
| mxx_shop_refund | backend/src/modules/shop/entity/shop_refund.rs |
| mxx_shop_review | backend/src/modules/shop/entity/shop_review.rs |
| mxx_shop_settlement | backend/src/modules/shop/entity/shop_settlement.rs |
| mxx_shop_sku | backend/src/modules/shop/entity/shop_sku.rs |
| mxx_shop_spec | backend/src/modules/shop/entity/shop_spec.rs |
| mxx_shop_spec_value | backend/src/modules/shop/entity/shop_spec_value.rs |
| mxx_shop_spu | backend/src/modules/shop/entity/shop_spu.rs |
| mxx_shop_supplier_apply | backend/src/modules/shop/entity/shop_supplier_apply.rs |
| mxx_shop_user_merge | backend/src/modules/shop/entity/shop_user_merge.rs |

### 2.3 articles 模块（全部 4 张缺失）

| 表名 | Entity 文件路径 |
|------|----------------|
| mxx_article | backend/src/modules/articles/entity/article.rs |
| mxx_article_category | backend/src/modules/articles/entity/category.rs |
| mxx_article_comment | backend/src/modules/articles/entity/comment.rs |
| mxx_label | backend/src/modules/articles/entity/label.rs |

### 2.5 message 模块（全部 3 张缺失）

| 表名 | Entity 文件路径 |
|------|----------------|
| mxx_chat_message | backend/src/modules/message/entity/chat_message.rs |
| mxx_chat_session | backend/src/modules/message/entity/chat_session.rs |
| mxx_chat_session_participant | backend/src/modules/message/entity/chat_session_participant.rs |

### 2.6 website 模块（8/9 缺失，仅 mxx_system_admin_template_merge 存在）

| 表名 | Entity 文件路径 |
|------|----------------|
| mxx_navigation | backend/src/modules/website/entity/navigation.rs |
| mxx_template | backend/src/modules/website/entity/template.rs |
| mxx_template_category | backend/src/modules/website/entity/template_category.rs |
| mxx_template_data | backend/src/modules/website/entity/template_data.rs |
| mxx_template_user_data | backend/src/modules/website/entity/template_user_data.rs |
| mxx_website | backend/src/modules/website/entity/website.rs |
| mxx_website_links | backend/src/modules/website/entity/website_links.rs |
| mxx_website_template_merge | backend/src/modules/website/entity/website_template_merge.rs |

### 2.7 statistics 模块（3/6 缺失）

| 表名 | Entity 文件路径 |
|------|----------------|
| mxx_statistics_performance_plan | backend/src/modules/statistics/entity/performance_plan.rs |
| mxx_statistics_plan_approval_log | backend/src/modules/statistics/entity/plan_approval_log.rs |
| mxx_statistics_plan_monthly_target | backend/src/modules/statistics/entity/plan_monthly_target.rs |

### 2.8 sale 模块（2 张缺失）

| 表名 | Entity 文件路径 |
|------|----------------|
| mxx_sale_invoice | backend/src/modules/sale/entity/invoice.rs |
| mxx_sale_quotation | backend/src/modules/sale/entity/quotation.rs |

### 2.9 crm 模块（1 张缺失）

| 表名 | Entity 文件路径 |
|------|----------------|
| mxx_crm_customer_contact_merge | backend/src/modules/crm/entity/customer_contact_merge.rs |

### 2.10 system 模块（1 张缺失）

| 表名 | Entity 文件路径 | 说明 |
|------|----------------|------|
| mxx_ip_address | backend/src/modules/system/entity/ip_address.rs | Entity 表名为 mxx_ip_address，SQL 文件中为 mxx_system_ip_address，数据库中均不存在 |

---

## 三、数据库有但无 Entity 的表（6 张）

| 表名 | 可能用途 |
|------|---------|
| mxx_attachment_file | 附件文件表（upload 模块无对应 entity） |
| mxx_inventory_transaction | 库存流水表（inventory 模块无对应 entity） |
| mxx_seq_generator | 序列号生成器（无对应模块） |
| mxx_system_email | 系统邮件配置（system 模块无对应 entity） |
| mxx_system_email_templet | 系统邮件模板（system 模块无对应 entity） |
| mxx_system_role_dept_merge | 角色部门关联表（system 模块无对应 entity） |

---

## 四、表名不匹配

| Entity 表名 | 数据库实际表名 | Entity 文件路径 | 说明 |
|------------|---------------|----------------|------|
| mxx_notice | mxx_system_notice | backend/src/modules/system/entity/notice.rs | Entity 缺少 system 前缀 |
| mxx_ip_address | （不存在） | backend/src/modules/system/entity/ip_address.rs | SQL 文件中为 mxx_system_ip_address，数据库中也不存在 |

---

## 五、Entity 中的特殊映射关系

### 5.1 列名与字段名不一致（通过 column_name 显式指定）

| 表名 | 数据库列名 | Rust 字段名 | Entity 文件 |
|------|----------|-----------|------------|
| mxx_crm_contract | name | title | contract.rs |
| mxx_crm_opportunity | name | title | opportunity.rs |
| mxx_purchase_supplier | name | company_name | supplier.rs |
| mxx_purchase_supplier | contact_person | contact_name | supplier.rs |
| mxx_purchase_supplier | phone | contact_phone | supplier.rs |
| mxx_purchase_supplier | email | contact_email | supplier.rs |
| mxx_system_menu | type | r#type | menu.rs |

### 5.2 使用 #[sea_orm(ignore)] 的字段（不映射到数据库）

| 表名 | 忽略的字段 | Entity 文件 |
|------|----------|------------|
| mxx_crm_opportunity | opportunity_no, contact_id, lead_id, source, tags, custom_fields | opportunity.rs |
| mxx_purchase_supplier | region, currency, credit_limit, credit_days, bank_name, bank_account, tax_id, tags, description, custom_fields | supplier.rs |

---

## 六、数据库现有表完整清单（57 张）

1. mxx_attachment
2. mxx_attachment_category
3. mxx_attachment_file
4. mxx_crm_contact
5. mxx_crm_contract
6. mxx_crm_customer
7. mxx_crm_followup
8. mxx_crm_lead
9. mxx_crm_lead_pool
10. mxx_crm_opportunity
11. mxx_inventory_stock
12. mxx_inventory_transaction
13. mxx_inventory_warehouse
14. mxx_product
15. mxx_product_category
16. mxx_product_sku
17. mxx_product_spec
18. mxx_product_spec_value
19. mxx_purchase_item
20. mxx_purchase_po
21. mxx_purchase_supplier
22. mxx_sale_order
23. mxx_sale_order_item
24. mxx_sale_payment
25. mxx_seq_generator
26. mxx_sku_template
27. mxx_sku_template_spec
28. mxx_sku_template_spec_value
29. mxx_sms_verification
30. mxx_statistics_access_record
31. mxx_statistics_performance_target
32. mxx_statistics_source
33. mxx_system_admin
34. mxx_system_admin_dept_merge
35. mxx_system_admin_notice_merge
36. mxx_system_admin_post_merge
37. mxx_system_admin_role_merge
38. mxx_system_admin_template_merge
39. mxx_system_area
40. mxx_system_config
41. mxx_system_dept
42. mxx_system_dept_menu_merge
43. mxx_system_dict
44. mxx_system_dict_data
45. mxx_system_email
46. mxx_system_email_templet
47. mxx_system_log
48. mxx_system_menu
49. mxx_system_notice
50. mxx_system_post
51. mxx_system_region
52. mxx_system_role
53. mxx_system_role_dept_merge
54. mxx_system_role_menu_merge
55. mxx_system_tag
56. mxx_system_tag_group
57. mxx_system_tag_merge

---

## 七、建议处理优先级

### P0 - 影响核心功能（必须修复）
1. 创建 finance 模块全部 5 张表（会员付费功能不可用）
2. 修复 mxx_notice → mxx_system_notice 表名不匹配

### P1 - 影响重要功能（尽快修复）
1. 创建 shop 模块全部 17 张表（商城功能不可用）
2. 创建 message 模块全部 3 张表（聊天功能不可用）
3. 创建 website 模块 8 张缺失表（网站功能不可用）
4. 创建 articles 模块全部 4 张表（文章功能不可用）
5. 创建 sale 模块 mxx_sale_invoice, mxx_sale_quotation 表

### P2 - 影响辅助功能（计划修复）
1. 创建 statistics 模块 3 张缺失表
2. 创建 mxx_crm_customer_contact_merge 表
3. 创建 mxx_ip_address 表

### P3 - 数据清理（可选）
1. 为数据库中无 Entity 的 6 张表创建对应 Entity 文件
