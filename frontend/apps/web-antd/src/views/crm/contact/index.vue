<script lang="ts" setup>
import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, ref, watch } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2, LucideSearch } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Card, Col, Drawer, Form, Input, Popconfirm, Row, Select, Tabs, Tag, Modal, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteContactApi, getContactListApi } from '#/api';
import { $t } from '#/locales';
import CustomerDetailDrawer from '../components/CustomerDetailDrawer.vue';
import ContactDrawer from './drawer.vue';
import ContactDetail from './detail.vue';

const accessStore = useAccessStore();
const userStore = useUserStore();

const roleLabelMap: Record<number, string> = {
  0: '决策人', 1: '影响者', 2: '使用者', 3: '其他',
};
const roleColorMap: Record<number, string> = {
  0: 'red', 1: 'orange', 2: 'blue', 3: 'default',
};

const detailVisible = ref(false);
const detailId = ref<number | null>(null);

function openDetail(row: any) {
  const id = row.id ?? row.id_;
  if (!id) { message.error('联系人ID不存在'); return; }
  detailId.value = Number(id);
  detailVisible.value = true;
}
function closeDetail() { detailVisible.value = false; detailId.value = null; }
function handleDetailEdit(contact: any) { closeDetail(); openDrawer(false, contact); }

// 客户详情抽屉
const customerDetailVisible = ref(false);
const customerDetailId = ref<number | string | undefined>(undefined);

function openCustomerDetail(row: any) {
  const id = row.customerId ?? row.customer_id;
  if (!id) {
    message.error('客户ID不存在');
    return;
  }
  customerDetailId.value = Number(id);
  customerDetailVisible.value = true;
}

function handleViewCustomer(customerId: number) {
  if (!customerId) {
    message.error('客户ID不存在');
    return;
  }
  customerDetailId.value = customerId;
  customerDetailVisible.value = true;
}

const dataScope = computed(() => {
  const scope = (userStore.userInfo as any)?.dataScope ?? (userStore.userInfo as any)?.data_scope;
  const roles = userStore.userInfo?.roles ?? [];
  if (roles.includes('super_admin') || roles.includes('system_admin')) return 1;
  return typeof scope === 'number' ? scope : 5;
});

const activeTab = ref('my');
const allTabList = [
  { key: 'my', label: '我的联系人' },
  { key: 'subordinate', label: '下属联系人' },
];
const tabList = computed(() => {
  const scope = dataScope.value;
  let allowedKeys: string[];
  switch (scope) {
    case 1:
    case 2:
    case 4:
      allowedKeys = ['my', 'subordinate'];
      break;
    case 3:
    case 5:
    default:
      allowedKeys = ['my'];
      break;
  }
  return allTabList.filter(t => allowedKeys.includes(t.key));
});

watch(tabList, (newTabs) => {
  const keys = newTabs.map(t => t.key);
  if (!keys.includes(activeTab.value) && keys.length > 0) {
    activeTab.value = keys[0];
  }
}, { immediate: true });

const searchForm = ref({
  customerName: '',
  name: '',
  mobile: '',
  phone: '',
  wechat: '',
  email: '',
});

function handleSearch() {
  gridApi.query();
}

function handleReset() {
  searchForm.value = {
    customerName: '',
    name: '',
    mobile: '',
    phone: '',
    wechat: '',
    email: '',
  };
  gridApi.query();
}

function handleTabChange(key: string) {
  activeTab.value = key;
  gridApi.query();
}

const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, export: true, refresh: true, zoom: true },
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { height: 40 },
  stripe: true,
  checkboxConfig: { checkField: 'checked', trigger: 'row' },

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }) => {
        const result = await getContactListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          listType: activeTab.value,
          ...searchForm.value,
        });
        const items = (result as any)?.items ?? [];
        const gridEl = gridApi.grid?.$el as HTMLElement | undefined;
        if (gridEl) {
          gridEl.style.height = items.length === 0 ? '280px' : '';
        }
        return result;
      },
    },
  },

  columns: [
    { type: 'checkbox', width: 50 },
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: '姓名', field: 'name', width: 120, slots: { default: 'name' } },
    { title: '当前公司', field: 'companyName', minWidth: 160, slots: { default: 'companyName' } },
    { title: '职位', field: 'title', width: 120 },
    {
      title: '角色', field: 'roleType', width: 90, slots: { default: 'roleType' },
    },
    {
      title: '首要', field: 'isPrimary', width: 60, align: 'center',
      formatter: ({ cellValue }: any) => cellValue ? '★' : '-',
    },
    { title: '邮箱', field: 'email', width: 180 },
    { title: '手机', field: 'mobile', width: 130 },
    {
      title: $t('ui.table.createTime'), field: 'createTime', slots: { default: 'createdAt' }, width: 160,
    },
    {
      title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 100,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions });

const [FormDrawer, drawerApi] = useVbenDrawer({
  connectedComponent: ContactDrawer,
  onClosed() { if (drawerApi.getData()?.needRefresh) gridApi.query(); },
});

function openDrawer(create: boolean, row?: any) { drawerApi.setData({ create, row }); drawerApi.open(); }
function handleCreate() { openDrawer(true); }
function handleEdit(row: any) { openDrawer(false, row); }

async function handleDelete(row: any) {
  row.pending = true;
  try { await deleteContactApi([row.id]); message.success($t('ui.notification.delete_success')); }
  finally { row.pending = false; gridApi.query(); }
}

async function handleBatchDelete() {
  const records = gridApi.grid?.getCheckboxRecords();
  if (!records?.length) { message.warning('请先选择要删除的联系人'); return; }
  Modal.confirm({
    title: '批量删除',
    content: `确定批量删除 ${records.length} 个联系人？`,
    onOk: async () => {
      try {
        await Promise.all(records.map((r: any) => deleteContactApi(r.id)));
        message.success(`已删除 ${records.length} 个联系人`);
        gridApi.query();
      } catch { /* ignore */ }
    },
  });
}
</script>

<template>
  <Page>
    <Card :bordered="false" class="mb-[15px]">
      <Tabs v-model:activeKey="activeTab" @change="handleTabChange" class="mb-4">
        <Tabs.TabPane v-for="tab in tabList" :key="tab.key" :tab="tab.label" />
      </Tabs>

      <Form :model="searchForm" layout="inline" :label-col="{ style: { width: '80px' } }" class="contact-search-form">
        <Row :gutter="[16, 12]" style="width: 100%">
          <Col :xs="24" :sm="24" :md="12">
            <Form.Item label="客户" name="customerName">
              <Input v-model:value="searchForm.customerName" placeholder="请输入客户" allow-clear style="width: 100%" />
            </Form.Item>
          </Col>
          <Col :xs="24" :sm="24" :md="12">
            <Form.Item label="姓名" name="name">
              <Input v-model:value="searchForm.name" placeholder="请输入姓名" allow-clear style="width: 100%" />
            </Form.Item>
          </Col>
          <Col :xs="24" :sm="24" :md="12">
            <Form.Item label="手机" name="mobile">
              <Input v-model:value="searchForm.mobile" placeholder="请输入手机" allow-clear style="width: 100%" />
            </Form.Item>
          </Col>
          <Col :xs="24" :sm="24" :md="12">
            <Form.Item label="电话" name="phone">
              <Input v-model:value="searchForm.phone" placeholder="请输入电话" allow-clear style="width: 100%" />
            </Form.Item>
          </Col>
          <Col :xs="24" :sm="24" :md="12">
            <Form.Item label="微信" name="wechat">
              <Input v-model:value="searchForm.wechat" placeholder="请输入微信号" allow-clear style="width: 100%" />
            </Form.Item>
          </Col>
          <Col :xs="24" :sm="24" :md="12">
            <Form.Item label="邮箱" name="email">
              <Input v-model:value="searchForm.email" placeholder="请输入电子邮箱" allow-clear style="width: 100%" />
            </Form.Item>
          </Col>
        </Row>

        <div class="flex flex-wrap items-center gap-2 mt-3">
          <Button type="primary" :icon="h(LucideSearch)" @click="handleSearch">搜索</Button>
          <Button @click="handleReset">刷新</Button>
          <Button v-if="accessStore.hasAccessCode('crm:contact:create')" type="primary" @click="handleCreate">
            {{ $t('page.crm.contact.button.create') }}
          </Button>
          <Button @click="handleBatchDelete" danger ghost>批量删除</Button>
        </div>
      </Form>
    </Card>

    <Grid :table-title="$t('page.crm.contact.title')" style="margin-top: 15px">
      <template #createdAt="{ row }">{{ formatDateTime(row.createTime) }}</template>

      <template #roleType="{ row }">
        <Tag :color="roleColorMap[row.roleType] || 'default'">{{ roleLabelMap[row.roleType] || '-' }}</Tag>
      </template>

      <template #name="{ row }">
        <a class="cursor-pointer text-blue-600 hover:text-blue-800" @click="() => openDetail(row)">{{ row.name }}</a>
      </template>

      <template #companyName="{ row }">
        <a v-if="row.customerId || row.customer_id" class="cursor-pointer text-blue-600 hover:text-blue-800" @click="() => openCustomerDetail(row)">{{ row.companyName || '-' }}</a>
        <span v-else class="text-gray-500">{{ row.companyName || '-' }}</span>
      </template>

      <template #action="{ row }">
        <Button v-if="accessStore.hasAccessCode('crm:contact:update')" type="link" :icon="h(LucideFilePenLine)" @click="() => handleEdit(row)" />
        <Popconfirm :title="$t('ui.text.do_you_want_delete', { moduleName: $t('page.crm.contact.title') })" :ok-text="$t('ui.button.ok')" :cancel-text="$t('ui.button.cancel')" @confirm="handleDelete(row)">
          <Button v-if="accessStore.hasAccessCode('crm:contact:delete')" type="link" danger :icon="h(LucideTrash2)" />
        </Popconfirm>
      </template>
    </Grid>
    <FormDrawer />

    <Drawer v-model:open="detailVisible" :width="1000" placement="right" :destroy-on-close="true" :mask-closable="true" :closable="true" title="联系人详情" :body-style="{ padding: 0, maxHeight: 'calc(100vh - 110px)', overflow: 'auto' }" @close="closeDetail">
      <ContactDetail v-if="detailId" :id="detailId" @edit="handleDetailEdit" @unbind="gridApi.query()" @view-customer="handleViewCustomer" />
    </Drawer>

    <CustomerDetailDrawer v-model:visible="customerDetailVisible" :id="customerDetailId" />
  </Page>
</template>

<style scoped>
.contact-search-form :deep(.ant-form-item) {
  margin-bottom: 0;
  width: 100%;
}
.contact-search-form :deep(.ant-form-item-control) {
  flex: 1;
}
</style>
