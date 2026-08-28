<script lang="ts" setup>
import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, ref, watch } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideFilePenLine, LucideSearch, LucideTrash2 } from '@vben/icons';
import { useAccessStore, useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  Card,
  Col,
  Drawer,
  Form,
  Input,
  message,
  Modal,
  Popconfirm,
  Row,
  Tabs,
  Tag,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteContactApi, getContactListApi } from '#/api';
import { useDataScopeTabs } from '#/composables/use-data-scope-tabs';
import { useSuperAdminGuard } from '#/composables/use-super-admin-guard';
import { $t } from '#/locales';

import CustomerDetailDrawer from '../components/CustomerDetailDrawer.vue';
import ContactDetail from './detail.vue';
import ContactDrawer from './drawer.vue';
import RecycleBin from '../components/RecycleBin.vue';

const accessStore = useAccessStore();
const userStore = useUserStore();

const roleLabelMap: Record<number, string> = {
  0: '决策人',
  1: '影响者',
  2: '使用者',
  3: '其他',
};
const roleColorMap: Record<number, string> = {
  0: 'red',
  1: 'orange',
  2: 'blue',
  3: 'default',
};

const detailVisible = ref(false);
const detailId = ref<null | number>(null);

function openDetail(row: any) {
  const id = row.id ?? row.id_;
  if (!id) {
    message.error('联系人ID不存在');
    return;
  }
  detailId.value = Number(id);
  detailVisible.value = true;
}
function closeDetail() {
  detailVisible.value = false;
  detailId.value = null;
}
function handleDetailEdit(contact: any) {
  closeDetail();
  openDrawer(false, contact);
}

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

// data_scope 决定可见的 Tab（超管/系统管理员按 dataScope=1 处理，与后端一致）
const { dataScope } = useDataScopeTabs();

const activeTab = ref('my');
const allTabList = [
  { key: 'all', label: '全部联系人' },
  { key: 'my', label: '我的联系人' },
  { key: 'subordinate', label: '下属联系人' },
];
const tabList = computed(() => {
  const scope = dataScope.value;
  let allowedKeys: string[];
  switch (scope) {
    case 1: {
      allowedKeys = ['all', 'my', 'subordinate'];
      break;
    }
    case 2:
    case 4: {
      allowedKeys = ['my', 'subordinate'];
      break;
    }
    default: {
      allowedKeys = ['my'];
      break;
    }
  }
  return allTabList.filter((t) => allowedKeys.includes(t.key));
});

watch(
  tabList,
  (newTabs) => {
    const keys = newTabs.map((t) => t.key);
    const firstKey = keys[0];
    if (!keys.includes(activeTab.value) && firstKey !== undefined) {
      activeTab.value = firstKey;
    }
  },
  { immediate: true },
);

// 是否为下属视图（下属视图下不能操作）
const isSubordinateView = computed(() => activeTab.value === 'subordinate');

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

const { isSuperAdmin } = useSuperAdminGuard();

function handleTabChange(key: number | string) {
  activeTab.value = String(key);
  if (key === 'recycle') return;
  gridApi.query();
}

const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, export: true, refresh: true, zoom: true },
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  rowConfig: { height: 'auto' } as any,
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
        // 无数据 150px，有数据按内容自适应
        const items = (result as any)?.items ?? [];
        const gridEl = gridApi.grid?.$el as HTMLElement | undefined;
        if (gridEl) {
          if (items.length === 0) {
            gridEl.style.setProperty('height', '150px', 'important');
          } else {
            gridEl.style.removeProperty('height');
          }
        }
        // 等DOM渲染完成后同步固定列行高并居中内容
        const syncFixedColumn = (retry = 0) => {
          const $el = gridApi.grid?.$el as HTMLElement | undefined;
          if (!$el) return;
          const mainBody = $el.querySelector('.vxe-table--body-wrapper tbody');
          const fixedRightBody = $el.querySelector(
            '.vxe-table--fixed-right-wrapper tbody',
          );
          if (!mainBody || !fixedRightBody) {
            if (retry < 3) setTimeout(() => syncFixedColumn(retry + 1), 200);
            return;
          }
          const rows1 = mainBody.querySelectorAll('tr.vxe-body--row');
          const rows2 = fixedRightBody.querySelectorAll('tr.vxe-body--row');
          const len = Math.min(rows1.length, rows2.length);
          if (len === 0) return;
          for (let i = 0; i < len; i++) {
            const h = (rows1[i] as HTMLElement).offsetHeight;
            if (h === 0) continue;
            (rows2[i] as HTMLElement).style.height = `${h}px`;
            const tds = (rows2[i] as HTMLElement).querySelectorAll('td');
            tds.forEach((td: Element) => {
              const cell = td.querySelector('.vxe-cell');
              if (cell) {
                (cell as HTMLElement).style.display = 'flex';
                (cell as HTMLElement).style.alignItems = 'center';
                (cell as HTMLElement).style.justifyContent = 'center';
                (cell as HTMLElement).style.height = `${h}px`;
              }
            });
          }
        };
        requestAnimationFrame(() => {
          syncFixedColumn();
          setTimeout(() => syncFixedColumn(), 200);
          setTimeout(() => syncFixedColumn(), 500);
        });
        return result;
      },
    },
  },

  columns: [
    { type: 'checkbox', width: 50 },
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: '姓名', field: 'name', width: 120, slots: { default: 'name' } },
    {
      title: '当前公司',
      field: 'companyName',
      minWidth: 160,
      slots: { default: 'companyName' },
    },
    { title: '职位', field: 'title', width: 120 },
    {
      title: '角色',
      field: 'roleType',
      width: 90,
      slots: { default: 'roleType' },
    },
    {
      title: '首要',
      field: 'isPrimary',
      width: 60,
      align: 'center',
      formatter: ({ cellValue }: any) => (cellValue ? '★' : '-'),
    },
    {
      // 归属人（人脉资产归属）：我的人脉高亮标记；0/空 = 公共池
      title: '归属人',
      field: 'ownerName',
      width: 110,
      formatter: ({ cellValue, row }: any) =>
        row?.createdBy === userStore.userInfo?.userId
          ? `★ ${cellValue || '我'}`
          : cellValue || '公共池',
    },
    { title: '邮箱', field: 'email', width: 180 },
    { title: '手机', field: 'mobile', width: 130 },
    {
      title: $t('ui.table.createTime'),
      field: 'createTime',
      slots: { default: 'createdAt' },
      width: 160,
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 100,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions });

const [FormDrawer, drawerApi] = useVbenDrawer({
  connectedComponent: ContactDrawer,
  onClosed() {
    if (drawerApi.getData()?.needRefresh) gridApi.query();
  },
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({ create, row });
  drawerApi.open();
}
function handleCreate() {
  openDrawer(true);
}
function handleEdit(row: any) {
  openDrawer(false, row);
}

// 删除显隐：本人创建 + 24h 内（后端删除守卫为准，前端仅预判）
const CRM_DELETE_WINDOW_MS = 24 * 60 * 60 * 1000;
function canDeleteContact(row: any): boolean {
  if (row.createdBy !== userStore.userInfo?.userId) return false;
  if (!row.createTime) return false;
  return Date.now() - new Date(row.createTime).getTime() <= CRM_DELETE_WINDOW_MS;
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteContactApi([row.id]);
    message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleBatchDelete() {
  const records = gridApi.grid?.getCheckboxRecords();
  if (!records?.length) {
    message.warning('请先选择要删除的联系人');
    return;
  }
  Modal.confirm({
    title: '批量删除',
    content: `确定批量删除 ${records.length} 个联系人？`,
    onOk: async () => {
      try {
        const ids = records.map((r: any) => r.id);
        await deleteContactApi(ids);
        message.success(`已删除 ${records.length} 个联系人`);
        gridApi.query();
      } catch {
        /* ignore */
      }
    },
  });
}
</script>

<template>
  <Page>
    <Card :bordered="false" class="contact-filter-card mb-4">
      <Tabs
        v-model:active-key="activeTab"
        @change="handleTabChange"
        class="mb-4"
      >
        <Tabs.TabPane v-for="tab in tabList" :key="tab.key" :tab="tab.label" />
        <Tabs.TabPane v-if="isSuperAdmin" key="recycle" tab="回收站" />
      </Tabs>

      <RecycleBin v-show="activeTab === 'recycle'" :module="'contact'" />

      <Form
        v-show="activeTab !== 'recycle'"
        :model="searchForm"
        layout="inline"
        :label-col="{ style: { width: '80px' } }"
        class="contact-search-form"
      >
        <Row :gutter="[16, 12]" style="width: 100%">
          <Col :xs="24" :sm="24" :md="12">
            <Form.Item label="客户" name="customerName">
              <Input
                v-model:value="searchForm.customerName"
                placeholder="请输入客户"
                allow-clear
                style="width: 100%"
              />
            </Form.Item>
          </Col>
          <Col :xs="24" :sm="24" :md="12">
            <Form.Item label="姓名" name="name">
              <Input
                v-model:value="searchForm.name"
                placeholder="请输入姓名"
                allow-clear
                style="width: 100%"
              />
            </Form.Item>
          </Col>
          <Col :xs="24" :sm="24" :md="12">
            <Form.Item label="手机" name="mobile">
              <Input
                v-model:value="searchForm.mobile"
                placeholder="请输入手机"
                allow-clear
                style="width: 100%"
              />
            </Form.Item>
          </Col>
          <Col :xs="24" :sm="24" :md="12">
            <Form.Item label="电话" name="phone">
              <Input
                v-model:value="searchForm.phone"
                placeholder="请输入电话"
                allow-clear
                style="width: 100%"
              />
            </Form.Item>
          </Col>
          <Col :xs="24" :sm="24" :md="12">
            <Form.Item label="微信" name="wechat">
              <Input
                v-model:value="searchForm.wechat"
                placeholder="请输入微信号"
                allow-clear
                style="width: 100%"
              />
            </Form.Item>
          </Col>
          <Col :xs="24" :sm="24" :md="12">
            <Form.Item label="邮箱" name="email">
              <Input
                v-model:value="searchForm.email"
                placeholder="请输入电子邮箱"
                allow-clear
                style="width: 100%"
              />
            </Form.Item>
          </Col>
        </Row>

        <div class="flex flex-wrap items-center gap-2 mt-3">
          <Button type="primary" :icon="h(LucideSearch)" @click="handleSearch">
            搜索
          </Button>
          <Button @click="handleReset">刷新</Button>
          <Button
            v-if="
              !isSubordinateView &&
              accessStore.hasAccessCode('crm:contact:save')
            "
            type="primary"
            @click="handleCreate"
          >
            {{ $t('page.crm.contact.button.create') }}
          </Button>
          <Button
            v-if="!isSubordinateView"
            @click="handleBatchDelete"
            danger
            ghost
          >
            批量删除
          </Button>
        </div>
      </Form>
    </Card>

    <Grid
      v-show="activeTab !== 'recycle'"
      :table-title="$t('page.crm.contact.title')"
      class="contact-grid-card"
    >
      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #roleType="{ row }">
        <Tag :color="roleColorMap[row.roleType] || 'default'">
          {{ roleLabelMap[row.roleType] || '-' }}
        </Tag>
      </template>

      <template #name="{ row }">
        <a
          class="cursor-pointer text-blue-600 hover:text-blue-800"
          @click="() => openDetail(row)"
          >{{ row.name }}</a
        >
      </template>

      <template #companyName="{ row }">
        <a
          v-if="row.customerId || row.customer_id"
          class="cursor-pointer text-blue-600 hover:text-blue-800"
          @click="() => openCustomerDetail(row)"
          >{{ row.companyName || '-' }}</a
        >
        <span v-else class="text-gray-500">{{ row.companyName || '-' }}</span>
      </template>

      <template #action="{ row }">
        <Button
          v-if="
            !isSubordinateView &&
            accessStore.hasAccessCode('crm:contact:update')
          "
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />
        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.crm.contact.title'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="handleDelete(row)"
        >
          <Button
            v-if="
              !isSubordinateView &&
              accessStore.hasAccessCode('crm:contact:delete') &&
              canDeleteContact(row)
            "
            type="link"
            danger
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </Grid>
    <FormDrawer />

    <Drawer
      v-model:open="detailVisible"
      :width="1000"
      placement="right"
      :destroy-on-close="true"
      :mask-closable="true"
      :closable="true"
      title="联系人详情"
      :body-style="{
        padding: 0,
        maxHeight: 'calc(100vh - 110px)',
        overflow: 'auto',
      }"
      @close="closeDetail"
    >
      <ContactDetail
        v-if="detailId"
        :id="detailId"
        @edit="handleDetailEdit"
        @unbind="gridApi.query()"
        @view-customer="handleViewCustomer"
      />
    </Drawer>

    <CustomerDetailDrawer
      v-model:visible="customerDetailVisible"
      :id="customerDetailId"
    />
  </Page>
</template>

<style scoped>
/* 筛选卡片与表格卡片间距（scoped 固化，不依赖 Tailwind 工具类） */
.contact-filter-card {
  margin-bottom: 16px;
}

.contact-grid-card {
  margin-top: 16px;
}

.contact-search-form :deep(.ant-form-item) {
  width: 100%;
  margin-bottom: 0;
}

.contact-search-form :deep(.ant-form-item-control) {
  flex: 1;
}
</style>
