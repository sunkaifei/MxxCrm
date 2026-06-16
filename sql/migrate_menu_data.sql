UPDATE mxx_system_menu SET type = 'FOLDER' WHERE type = 'M';
UPDATE mxx_system_menu SET type = 'MENU' WHERE type = 'C';
UPDATE mxx_system_menu SET type = 'BUTTON' WHERE type = 'F';

UPDATE mxx_system_menu SET name = 'page.dashboard.title' WHERE route_name = 'Dashboard';
UPDATE mxx_system_menu SET name = 'page.dashboard.analytics' WHERE route_name = 'Analytics';
UPDATE mxx_system_menu SET name = 'page.dashboard.workspace' WHERE route_name = 'Workspace';

UPDATE mxx_system_menu SET name = 'page.system.title' WHERE route_name = 'System';
UPDATE mxx_system_menu SET name = 'page.system.user.title' WHERE route_name = 'Admin';
UPDATE mxx_system_menu SET name = 'page.system.role.title' WHERE route_name = 'Role';
UPDATE mxx_system_menu SET name = 'page.system.menu.title' WHERE route_name = 'Menu';
UPDATE mxx_system_menu SET name = 'page.system.dept.title' WHERE route_name = 'Dept';
UPDATE mxx_system_menu SET name = 'page.system.post.title' WHERE route_name = 'Post';
UPDATE mxx_system_menu SET name = 'page.system.config.title' WHERE route_name = 'Config';
UPDATE mxx_system_menu SET name = 'page.system.dict.title' WHERE route_name = 'Dict';
UPDATE mxx_system_menu SET name = 'page.system.notice.title' WHERE route_name = 'Notice';
UPDATE mxx_system_menu SET name = 'page.system.log.title' WHERE route_name = 'Log';

UPDATE mxx_system_menu SET name = 'page.crm.title' WHERE route_name = 'Crm';
UPDATE mxx_system_menu SET name = 'page.crm.lead.title' WHERE route_name = 'Lead';
UPDATE mxx_system_menu SET name = 'page.crm.customer.title' WHERE route_name = 'Customer';
UPDATE mxx_system_menu SET name = 'page.crm.contact.title' WHERE route_name = 'Contact';
UPDATE mxx_system_menu SET name = 'page.crm.opportunity.title' WHERE route_name = 'Opportunity';
UPDATE mxx_system_menu SET name = 'page.crm.contract.title' WHERE route_name = 'Contract';
UPDATE mxx_system_menu SET name = 'page.crm.leadPool.title' WHERE route_name = 'LeadPool';
UPDATE mxx_system_menu SET name = 'page.crm.followup.title' WHERE route_name = 'Followup';

UPDATE mxx_system_menu SET name = 'page.sale.title' WHERE route_name = 'Sale';
UPDATE mxx_system_menu SET name = 'page.sale.order.title' WHERE route_name = 'Order';
UPDATE mxx_system_menu SET name = 'page.sale.orderItem.title' WHERE route_name = 'OrderItem';
UPDATE mxx_system_menu SET name = 'page.sale.payment.title' WHERE route_name = 'Payment';

UPDATE mxx_system_menu SET name = 'page.product.title' WHERE route_name = 'Product';
UPDATE mxx_system_menu SET name = 'page.product.list.title' WHERE route_name = 'ProductMain';
UPDATE mxx_system_menu SET name = 'page.product.category.title' WHERE route_name = 'ProductCategory';
UPDATE mxx_system_menu SET name = 'page.product.inventory.title' WHERE route_name = 'Inventory';
UPDATE mxx_system_menu SET name = 'page.product.warehouse.title' WHERE route_name = 'Warehouse';

UPDATE mxx_system_menu SET name = 'page.purchase.title' WHERE route_name = 'Purchase';
UPDATE mxx_system_menu SET name = 'page.purchase.supplier.title' WHERE route_name = 'Supplier';
UPDATE mxx_system_menu SET name = 'page.purchase.po.title' WHERE route_name = 'PurchaseOrder';
UPDATE mxx_system_menu SET name = 'page.purchase.item.title' WHERE route_name = 'PurchaseItem';

UPDATE mxx_system_menu SET name = 'page.attachment.title' WHERE route_name = 'Attachment';
UPDATE mxx_system_menu SET name = 'page.attachment.file.title' WHERE route_name = 'File';

UPDATE mxx_system_menu SET name = 'button.system.admin.create' WHERE perm = 'system:admin:create';
UPDATE mxx_system_menu SET name = 'button.system.admin.edit' WHERE perm = 'system:admin:edit';
UPDATE mxx_system_menu SET name = 'button.system.admin.delete' WHERE perm = 'system:admin:delete';
UPDATE mxx_system_menu SET name = 'button.system.admin.assignRole' WHERE perm = 'system:admin:assignRole';
UPDATE mxx_system_menu SET name = 'button.system.admin.assignDept' WHERE perm = 'system:admin:assignDept';

UPDATE mxx_system_menu SET name = 'button.system.role.create' WHERE perm = 'system:role:create';
UPDATE mxx_system_menu SET name = 'button.system.role.edit' WHERE perm = 'system:role:edit';
UPDATE mxx_system_menu SET name = 'button.system.role.delete' WHERE perm = 'system:role:delete';
UPDATE mxx_system_menu SET name = 'button.system.role.assignMenu' WHERE perm = 'system:role:assignMenu';

UPDATE mxx_system_menu SET name = 'button.system.menu.create' WHERE perm = 'system:menu:create';
UPDATE mxx_system_menu SET name = 'button.system.menu.edit' WHERE perm = 'system:menu:edit';
UPDATE mxx_system_menu SET name = 'button.system.menu.delete' WHERE perm = 'system:menu:delete';

UPDATE mxx_system_menu SET name = 'button.crm.lead.create' WHERE perm = 'crm:lead:create';
UPDATE mxx_system_menu SET name = 'button.crm.lead.edit' WHERE perm = 'crm:lead:edit';
UPDATE mxx_system_menu SET name = 'button.crm.lead.delete' WHERE perm = 'crm:lead:delete';
UPDATE mxx_system_menu SET name = 'button.crm.lead.convert' WHERE perm = 'crm:lead:convert';
UPDATE mxx_system_menu SET name = 'button.crm.lead.assign' WHERE perm = 'crm:lead:assign';

UPDATE mxx_system_menu SET name = 'button.crm.customer.create' WHERE perm = 'crm:customer:create';
UPDATE mxx_system_menu SET name = 'button.crm.customer.edit' WHERE perm = 'crm:customer:edit';
UPDATE mxx_system_menu SET name = 'button.crm.customer.delete' WHERE perm = 'crm:customer:delete';
UPDATE mxx_system_menu SET name = 'button.crm.customer.assign' WHERE perm = 'crm:customer:assign';

UPDATE mxx_system_menu SET name = 'button.crm.opportunity.create' WHERE perm = 'crm:opportunity:create';
UPDATE mxx_system_menu SET name = 'button.crm.opportunity.edit' WHERE perm = 'crm:opportunity:edit';
UPDATE mxx_system_menu SET name = 'button.crm.opportunity.delete' WHERE perm = 'crm:opportunity:delete';

UPDATE mxx_system_menu SET name = 'button.crm.contract.create' WHERE perm = 'crm:contract:create';
UPDATE mxx_system_menu SET name = 'button.crm.contract.edit' WHERE perm = 'crm:contract:edit';
UPDATE mxx_system_menu SET name = 'button.crm.contract.delete' WHERE perm = 'crm:contract:delete';

UPDATE mxx_system_menu SET name = 'button.sale.order.create' WHERE perm = 'sale:order:create';
UPDATE mxx_system_menu SET name = 'button.sale.order.edit' WHERE perm = 'sale:order:edit';
UPDATE mxx_system_menu SET name = 'button.sale.order.delete' WHERE perm = 'sale:order:delete';
UPDATE mxx_system_menu SET name = 'button.sale.order.confirm' WHERE perm = 'sale:order:confirm';
UPDATE mxx_system_menu SET name = 'button.sale.order.ship' WHERE perm = 'sale:order:ship';
UPDATE mxx_system_menu SET name = 'button.sale.order.complete' WHERE perm = 'sale:order:complete';

UPDATE mxx_system_menu SET name = 'button.sale.payment.create' WHERE perm = 'sale:payment:create';
UPDATE mxx_system_menu SET name = 'button.sale.payment.edit' WHERE perm = 'sale:payment:edit';
UPDATE mxx_system_menu SET name = 'button.sale.payment.delete' WHERE perm = 'sale:payment:delete';

UPDATE mxx_system_menu SET name = 'button.product.create' WHERE perm = 'product:create';
UPDATE mxx_system_menu SET name = 'button.product.edit' WHERE perm = 'product:edit';
UPDATE mxx_system_menu SET name = 'button.product.delete' WHERE perm = 'product:delete';

UPDATE mxx_system_menu SET name = 'button.product.category.create' WHERE perm = 'product:category:create';
UPDATE mxx_system_menu SET name = 'button.product.category.edit' WHERE perm = 'product:category:edit';
UPDATE mxx_system_menu SET name = 'button.product.category.delete' WHERE perm = 'product:category:delete';

UPDATE mxx_system_menu SET name = 'button.product.warehouse.create' WHERE perm = 'product:warehouse:create';
UPDATE mxx_system_menu SET name = 'button.product.warehouse.edit' WHERE perm = 'product:warehouse:edit';
UPDATE mxx_system_menu SET name = 'button.product.warehouse.delete' WHERE perm = 'product:warehouse:delete';

UPDATE mxx_system_menu SET name = 'button.purchase.supplier.create' WHERE perm = 'purchase:supplier:create';
UPDATE mxx_system_menu SET name = 'button.purchase.supplier.edit' WHERE perm = 'purchase:supplier:edit';
UPDATE mxx_system_menu SET name = 'button.purchase.supplier.delete' WHERE perm = 'purchase:supplier:delete';

UPDATE mxx_system_menu SET name = 'button.purchase.po.create' WHERE perm = 'purchase:po:create';
UPDATE mxx_system_menu SET name = 'button.purchase.po.edit' WHERE perm = 'purchase:po:edit';
UPDATE mxx_system_menu SET name = 'button.purchase.po.delete' WHERE perm = 'purchase:po:delete';
UPDATE mxx_system_menu SET name = 'button.purchase.po.confirm' WHERE perm = 'purchase:po:confirm';
UPDATE mxx_system_menu SET name = 'button.purchase.po.receive' WHERE perm = 'purchase:po:receive';

UPDATE mxx_system_menu SET name = 'button.attachment.file.upload' WHERE perm = 'attachment:file:upload';
UPDATE mxx_system_menu SET name = 'button.attachment.file.delete' WHERE perm = 'attachment:file:delete';

SELECT 'Menu data migration completed' AS result;