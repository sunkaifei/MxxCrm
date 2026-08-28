<script lang="ts" setup>
import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, ref, watch } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import {
  LucideMail,
  LucideMaximize2,
  LucideMinimize2,
  LucidePlus,
  LucideSearch,
  LucideUpload,
  LucideUsers,
} from '@vben/icons';
import { useAccessStore, useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  Card,
  Col,
  Drawer,
  Dropdown,
  Form,
  Input,
  message,
  Modal,
  Popconfirm,
  Row,
  Select,
  Tabs,
  Tag,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteCustomerApi, getCustomerListApi } from '#/api';
import { addCustomerToPoolApi } from '#/api/core/crm/customer-pool';
import { useDataScopeTabs } from '#/composables/use-data-scope-tabs';
import { useSuperAdminGuard } from '#/composables/use-super-admin-guard';
import { $t } from '#/locales';

import CustomerDetailDrawer from '../components/CustomerDetailDrawer.vue';
import ReasonFormModal from '../components/ReasonFormModal.vue';
import SendMailModal from '../components/SendMailModal.vue';
import TransferModal from '../components/TransferModal.vue';
import ContactDrawer from '../contact/drawer.vue';
import OpportunityDetail from '../opportunity/detail.vue';
import RecycleBin from '../components/RecycleBin.vue';
import CustomerFollowupDrawer from './followup-drawer.vue';

const accessStore = useAccessStore();
const userStore = useUserStore();

// data_scope 决定可见的 Tab
// 1=全部数据 → 全部Tab  2=自定义 → my+subordinate+todayFollow
// 3=本部门 → my+todayFollow  4=本部门及以下 → all+my+subordinate+todayFollow
// 5=仅本人 → my+todayFollow
// 超管（user_type=1）/系统管理员（data_scope=1）统一按 dataScope=1 处理，与后端一致
const { dataScope } = useDataScopeTabs();

const activeTab = ref('my');
// 下属视图下只能查看，不能进行写操作
const isSubordinateView = computed(() => activeTab.value === 'subordinate');
const allTabList = [
  { key: 'all', label: '全部客户' },
  { key: 'my', label: '我的客户' },
  { key: 'subordinate', label: '下属客户' },
  { key: 'todayFollow', label: '今日跟进客户' },
];
// 根据 data_scope 过滤可见的Tab
const tabList = computed(() => {
  const scope = dataScope.value;
  let allowedKeys: string[];
  switch (scope) {
    case 1: {
      allowedKeys = ['all', 'my', 'subordinate', 'todayFollow'];
      break;
    }
    case 2:
    case 4: {
      allowedKeys = ['my', 'subordinate', 'todayFollow'];
      break;
    }
    default: {
      allowedKeys = ['my', 'todayFollow'];
      break;
    }
  }
  return allTabList.filter((t) => allowedKeys.includes(t.key));
});
// 当Tab权限变化时，确保当前激活的Tab仍然可见
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

// 等级颜色映射 - 1:无级别 2:重点客户 3:优质客户 4:普通客户 5:其他
const levelColorMap: Record<string, string> = {
  1: 'default',
  2: 'red',
  3: 'orange',
  4: 'blue',
  5: 'green',
};
const levelLabelMap: Record<string, string> = {
  1: '无级别',
  2: '重点客户',
  3: '优质客户',
  4: '普通客户',
  5: '其他',
};

// 行业映射 - 后端存储数值
const industryLabelMap: Record<number, string> = {
  1: '零售',
  2: '批发',
  3: '制造',
  4: '贸易代理',
  5: '电商',
  6: '微商',
  7: '社交电商',
  8: '其他',
};

// 来源映射 - 后端存储数值
const sourceLabelMap: Record<number, string> = {
  1: '官网',
  2: '展会',
  3: '社交媒体',
  4: '客户转介',
  5: '陌生拜访',
  6: '海关数据',
  7: '邮件营销',
  8: '阿里国际站',
  9: 'Amazon',
  10: 'TikTok',
  11: '微信',
  12: '其他',
};

// 详情/新建抽屉统一使用 CustomerDetailDrawer
const detailVisible = ref(false);
const detailId = ref<null | number>(null);
const detailCustomerType = ref<number | undefined>(undefined);

// 打开详情/编辑（带 id）
function openDetail(row: any) {
  const id = row.id ?? row.id_;
  if (!id) {
    message.error('客户ID不存在');
    return;
  }
  detailId.value = Number(id);
  detailCustomerType.value = undefined;
  detailVisible.value = true;
}
// 新建客户（传 customerType）
function openCreate(customerType: number) {
  detailId.value = null;
  detailCustomerType.value = customerType;
  detailVisible.value = true;
}
// 新建成功回调：关闭抽屉 + 刷新列表
function handleDetailCreated() {
  detailVisible.value = false;
  detailId.value = null;
  detailCustomerType.value = undefined;
  gridApi.query();
}

// 搜索表单
const searchForm = ref({
  companyName: '',
  mobile: '',
  phone: '',
  dealStatus: undefined as number | undefined,
  level: undefined as number | undefined,
  industry: undefined as number | undefined,
  source: undefined as number | undefined,
  customerType: undefined as number | undefined,
  wechat: '',
  qq: '',
});

const { isSuperAdmin } = useSuperAdminGuard();

function handleTabChange(key: number | string) {
  activeTab.value = key as string;
  if (key === 'recycle') return;
  gridApi.query();
}

function handleSearch() {
  gridApi.query();
}

function handleReset() {
  searchForm.value = {
    companyName: '',
    mobile: '',
    phone: '',
    dealStatus: undefined,
    level: undefined,
    industry: undefined,
    source: undefined,
    customerType: undefined,
    wechat: '',
    qq: '',
  };
  gridApi.query();
}

// 客户类型显示：名称按类型返回（个人用 personName，企业用 companyName）
function getCustomerDisplayName(row: any): string {
  if (Number(row.customerType) === 2) {
    return row.personName || row.companyName || '-';
  }
  return row.companyName || row.personName || '-';
}
function getCustomerTypeColor(type: any): string {
  return Number(type) === 2 ? 'green' : 'blue';
}
function getCustomerTypeLabel(type: any): string {
  return Number(type) === 2 ? '个人' : '企业';
}

const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, export: true, refresh: true, zoom: true },
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  rowConfig: { height: 'auto' as any },
  stripe: true,
  checkboxConfig: { checkField: 'checked', trigger: 'row' },

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }) => {
        const values = searchForm.value;
        const result = await getCustomerListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          keywords: values.companyName || undefined,
          customerType: values.customerType || undefined,
          level: values.level,
          industry: values.industry || undefined,
          source: values.source || undefined,
          mobile: values.mobile || undefined,
          phone: values.phone || undefined,
          wechat: values.wechat || undefined,
          qq: values.qq || undefined,
          dealStatus: values.dealStatus,
          listType: activeTab.value,
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
    {
      title: '编号',
      field: 'customerNo',
      width: 150,
      headerAlign: 'center',
      align: 'center',
      slots: { default: 'customerNo' },
    },
    {
      title: '类型',
      field: 'customerType',
      width: 70,
      align: 'center',
      slots: { default: 'customerType' },
    },
    {
      title: '客户名称',
      field: 'companyName',
      minWidth: 200,
      headerAlign: 'center',
      align: 'left',
      slots: { default: 'customerName' },
    },
    {
      title: '等级',
      field: 'level',
      width: 80,
      slots: { default: 'level' },
    },
    {
      title: '行业',
      field: 'industry',
      width: 90,
      formatter: ({ cellValue }: any) =>
        industryLabelMap[cellValue] || cellValue || '-',
    },
    { title: '国家', field: 'country', width: 80 },
    {
      title: '来源',
      field: 'source',
      width: 100,
      formatter: ({ cellValue }: any) =>
        sourceLabelMap[cellValue] || cellValue || '-',
    },
    {
      title: '联系人',
      field: 'contactCount',
      width: 70,
      align: 'center',
      formatter: ({ cellValue }: any) => cellValue ?? '-',
    },
    {
      title: '商机数',
      field: 'opportunityCount',
      width: 70,
      align: 'center',
      formatter: ({ cellValue }: any) => cellValue ?? '-',
    },
    {
      title: '负责人',
      field: 'assigneeName',
      width: 90,
      formatter: ({ cellValue }: any) => cellValue || '-',
    },
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
      width: 180,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions });

// 商机详情抽屉（使用 OpportunityDetail 组件）
const oppDetailVisible = ref(false);
const oppDetailCustomerId = ref<number | string | undefined>(undefined);
const oppDetailCustomerName = ref<string>('');

const [ContactDrawerInstance, contactDrawerApi] = useVbenDrawer({
  connectedComponent: ContactDrawer,
  onClosed() {
    if (contactDrawerApi.getData()?.needRefresh) gridApi.query();
  },
});

// 跟进抽屉
function handleFollowup(row: any) {
  followupVisible.value = true;
  followupCustomerId.value = undefined;
  followupNeedRefresh.value = false;
  // nextTick 确保组件重建后 props.id 能正确触发 fetch
  import('vue').then(({ nextTick }) =>
    nextTick(() => {
      followupCustomerId.value = row.id;
    }),
  );
}
const followupVisible = ref(false);
const followupCustomerId = ref<number | undefined>(undefined);
const followupFullscreen = ref(false);
const followupNeedRefresh = ref(false);
function handleFollowupClose() {
  followupVisible.value = false;
  followupCustomerId.value = undefined;
  if (followupNeedRefresh.value) gridApi.query();
}
function toggleFollowupFullscreen() {
  followupFullscreen.value = !followupFullscreen.value;
}

// 新建企业/个人客户
function handleCreate() {
  openCreate(1);
}
function handleCreateEnterprise() {
  openCreate(1);
}
function handleCreatePersonal() {
  openCreate(2);
}
// 修改客户：直接走详情抽屉（详情 Tab 内表单可直接编辑）
function handleEdit(row: any) {
  openDetail(row);
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteCustomerApi([row.id]);
    message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

// ===== 退回公海（原因类型必选，选"其他"需补充说明）=====
const poolReasonVisible = ref(false);
const poolReasonSubmitting = ref(false);
const poolReasonRow = ref<null | { id: number | string }>(null);

function openPoolReason(row: any) {
  poolReasonRow.value = row;
  poolReasonVisible.value = true;
}

async function onPoolReasonConfirm({
  reason,
  reasonType,
}: {
  reason: string;
  reasonType?: number;
}) {
  const row = poolReasonRow.value;
  if (!row) return;
  if (reasonType === undefined) return;
  poolReasonSubmitting.value = true;
  try {
    await addCustomerToPoolApi({ id: Number(row.id), reason, reasonType });
    message.success('已退回公海');
    poolReasonVisible.value = false;
    gridApi.query();
  } catch {
    message.error('退回公海失败');
  } finally {
    poolReasonSubmitting.value = false;
  }
}

// 删除按钮显隐（自建 + 本人 + 未超删除窗口；后端删除校验为准）
// 客户删除窗口 1 小时，与后端 delete_guard_service 常量对齐
const CUSTOMER_DELETE_WINDOW_MS = 60 * 60 * 1000;

function canDeleteCustomer(row: any): boolean {
  if (row.fromPool !== 0) return false;
  if (row.createdBy !== userStore.userInfo?.userId) return false;
  if (!row.createTime) return false;
  return (
    Date.now() - new Date(row.createTime).getTime() <=
    CUSTOMER_DELETE_WINDOW_MS
  );
}

async function handleBatchDelete() {
  const records = gridApi.grid?.getCheckboxRecords();
  if (!records?.length) {
    message.warning('请先选择要删除的客户');
    return;
  }
  Modal.confirm({
    title: '批量删除',
    content: `确定批量删除 ${records.length} 个客户？`,
    onOk: async () => {
      try {
        const ids = records.map((r: any) => r.id);
        await deleteCustomerApi(ids);
        message.success(`已删除 ${records.length} 个客户`);
        gridApi.query();
      } catch {
        /* ignore */
      }
    },
  });
}

// 客户转移
const transferVisible = ref(false);
const transferCustomerIds = ref<number[]>([]);

// 发邮件
const sendMailVisible = ref(false);
const sendMailCustomer = ref<any>(null);

function handleSendMail() {
  const records = gridApi.grid?.getCheckboxRecords();
  if (!records?.length) {
    message.warning('请先选择客户');
    return;
  }
  sendMailCustomer.value = records[0];
  sendMailVisible.value = true;
}

function handleBatchTransfer() {
  const records = gridApi.grid?.getCheckboxRecords();
  if (!records?.length) {
    message.warning('请先选择要转移的客户');
    return;
  }
  transferCustomerIds.value = records.map((r: any) => Number(r.id));
  transferVisible.value = true;
}

function handleTransfer(row: any) {
  const id = Number(row.id ?? row.id_);
  if (!id) {
    message.error('客户ID不存在');
    return;
  }
  transferCustomerIds.value = [id];
  transferVisible.value = true;
}

function onTransferSuccess(_data: {
  affectedTotal: number;
  transferredCount: number;
}) {
  transferVisible.value = false;
  transferCustomerIds.value = [];
  gridApi.query();
}

// 商机抽屉：从客户列表点击"添加商机"打开 OpportunityDetail，预填所属企业
function handleAddOpportunity(row: any) {
  oppDetailCustomerId.value = row.id;
  oppDetailCustomerName.value = row.companyName || '';
  oppDetailVisible.value = true;
}
function handleOppDetailClose() {
  oppDetailVisible.value = false;
  oppDetailCustomerId.value = undefined;
  oppDetailCustomerName.value = '';
  gridApi.query();
}
function handleOppDetailCreated() {
  gridApi.query();
}

// 联系人抽屉
function handleAddContact(row: any) {
  contactDrawerApi.setData({
    create: true,
    row: { customerId: row.id, companyName: row.companyName },
  });
  contactDrawerApi.open();
}
</script>

<template>
  <Page>
    <Card :bordered="false" class="customer-filter-card mb-4">
      <Tabs
        v-model:active-key="activeTab"
        @change="handleTabChange"
        class="mb-4"
      >
        <Tabs.TabPane v-for="tab in tabList" :key="tab.key" :tab="tab.label" />
        <Tabs.TabPane v-if="isSuperAdmin" key="recycle" tab="回收站" />
      </Tabs>

      <RecycleBin v-show="activeTab === 'recycle'" :module="'customer'" />

      <Form
        v-show="activeTab !== 'recycle'"
        :model="searchForm"
        layout="inline"
        :label-col="{ style: { width: '90px' } }"
        class="customer-search-form"
      >
        <div class="customer-search-form-wrapper">
          <Row :gutter="[16, 12]" style="width: 100%">
            <Col :xs="24" :sm="24" :md="12">
              <Form.Item label="客户名称" name="companyName">
                <Input
                  v-model:value="searchForm.companyName"
                  placeholder="请输入客户名称/姓名"
                  allow-clear
                  style="width: 100%"
                />
              </Form.Item>
            </Col>
            <Col :xs="24" :sm="24" :md="12">
              <Form.Item label="客户类型" name="customerType">
                <Select
                  v-model:value="searchForm.customerType"
                  placeholder="请选择客户类型"
                  allow-clear
                  style="width: 100%"
                >
                  <Select.Option :value="1">企业客户</Select.Option>
                  <Select.Option :value="2">个人客户</Select.Option>
                </Select>
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
              <Form.Item label="成交状态" name="dealStatus">
                <Select
                  v-model:value="searchForm.dealStatus"
                  placeholder="请选择成交状态"
                  allow-clear
                  style="width: 100%"
                >
                  <Select.Option :value="1">未成交</Select.Option>
                  <Select.Option :value="2">已成交</Select.Option>
                </Select>
              </Form.Item>
            </Col>
            <Col :xs="24" :sm="24" :md="12">
              <Form.Item label="客户级别" name="level">
                <Select
                  v-model:value="searchForm.level"
                  placeholder="请选择客户级别"
                  allow-clear
                  style="width: 100%"
                >
                  <Select.Option :value="1">无级别</Select.Option>
                  <Select.Option :value="2">重点客户</Select.Option>
                  <Select.Option :value="3">优质客户</Select.Option>
                  <Select.Option :value="4">普通客户</Select.Option>
                  <Select.Option :value="5">其他</Select.Option>
                </Select>
              </Form.Item>
            </Col>
            <Col :xs="24" :sm="24" :md="12">
              <Form.Item label="客户行业" name="industry">
                <Select
                  v-model:value="searchForm.industry"
                  placeholder="请选择客户行业"
                  allow-clear
                  style="width: 100%"
                >
                  <Select.Option :value="1">零售</Select.Option>
                  <Select.Option :value="2">批发</Select.Option>
                  <Select.Option :value="3">制造</Select.Option>
                  <Select.Option :value="4">贸易代理</Select.Option>
                  <Select.Option :value="5">电商</Select.Option>
                  <Select.Option :value="6">微商</Select.Option>
                  <Select.Option :value="7">社交电商</Select.Option>
                  <Select.Option :value="8">其他</Select.Option>
                </Select>
              </Form.Item>
            </Col>
            <Col :xs="24" :sm="24" :md="12">
              <Form.Item label="客户来源" name="source">
                <Select
                  v-model:value="searchForm.source"
                  placeholder="请选择客户来源"
                  allow-clear
                  style="width: 100%"
                >
                  <Select.Option :value="1">官网</Select.Option>
                  <Select.Option :value="2">展会</Select.Option>
                  <Select.Option :value="3">社交媒体</Select.Option>
                  <Select.Option :value="4">客户转介</Select.Option>
                  <Select.Option :value="5">陌生拜访</Select.Option>
                  <Select.Option :value="6">海关数据</Select.Option>
                  <Select.Option :value="7">邮件营销</Select.Option>
                  <Select.Option :value="8">阿里国际站</Select.Option>
                  <Select.Option :value="9">Amazon</Select.Option>
                  <Select.Option :value="10">TikTok</Select.Option>
                  <Select.Option :value="11">微信</Select.Option>
                  <Select.Option :value="12">其他</Select.Option>
                </Select>
              </Form.Item>
            </Col>
            <Col :xs="24" :sm="24" :md="12">
              <Form.Item label="微信" name="wechat">
                <Input
                  v-model:value="searchForm.wechat"
                  placeholder="请输入微信"
                  allow-clear
                  style="width: 100%"
                />
              </Form.Item>
            </Col>
            <Col :xs="24" :sm="24" :md="12">
              <Form.Item label="QQ" name="qq">
                <Input
                  v-model:value="searchForm.qq"
                  placeholder="请输入QQ"
                  allow-clear
                  style="width: 100%"
                />
              </Form.Item>
            </Col>
          </Row>
        </div>

        <div class="flex flex-wrap items-center gap-2 mt-3">
          <Button type="default" :icon="h(LucideSearch)" @click="handleSearch">
            搜索
          </Button>
          <Button type="default" @click="handleReset">刷新</Button>
          <Button
            v-if="
              !isSubordinateView &&
              accessStore.hasAccessCode('crm:customer:save')
            "
            type="primary"
            :icon="h(LucidePlus)"
            @click="handleCreate"
          >
            新增
          </Button>
          <Button :icon="h(LucideUpload)">导入</Button>
          <Button
            v-if="
              !isSubordinateView &&
              accessStore.hasAccessCode('crm:customer:transfer')
            "
            :icon="h(LucideUsers)"
            @click="handleBatchTransfer"
          >
            批量转移客户
          </Button>
          <Button>资料回收</Button>
          <Button>发短信</Button>
          <Button :icon="h(LucideMail)" @click="handleSendMail">发邮件</Button>
        </div>
      </Form>
    </Card>

    <Grid
      v-show="activeTab !== 'recycle'"
      :table-title="$t('page.crm.customer.title')"
      class="customer-grid-card"
    >
      <template #toolbar-tools>
        <Dropdown
          v-if="
            !isSubordinateView && accessStore.hasAccessCode('crm:customer:save')
          "
          :trigger="['click']"
        >
          <Button type="primary" class="mr-2">
            {{ $t('page.crm.customer.button.create') }} ▾
          </Button>
          <template #overlay>
            <div class="customer-more-menu">
              <div class="more-menu-item" @click="handleCreateEnterprise">
                <span>新建企业客户</span>
              </div>
              <div class="more-menu-item" @click="handleCreatePersonal">
                <span>新建个人客户</span>
              </div>
            </div>
          </template>
        </Dropdown>
        <Button
          v-if="!isSubordinateView"
          @click="handleBatchDelete"
          class="mr-2"
          danger
          ghost
        >
          批量删除
        </Button>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #customerNo="{ row }">
        <a
          class="cursor-pointer text-blue-600 hover:text-blue-800"
          @click="() => openDetail(row)"
          >{{ row.customerNo || '-' }}</a
        >
      </template>

      <template #customerType="{ row }">
        <Tag :color="getCustomerTypeColor(row.customerType)">
          {{ getCustomerTypeLabel(row.customerType) }}
        </Tag>
      </template>

      <template #customerName="{ row }">
        <div>
          <a
            class="cursor-pointer text-blue-600 hover:text-blue-800"
            @click="() => openDetail(row)"
          >
            {{ getCustomerDisplayName(row) }}
          </a>
          <div
            v-if="row.tags && row.tags.length > 0"
            class="mt-1 flex flex-wrap gap-1"
          >
            <Tag
              v-for="tag in row.tags"
              :key="tag.id"
              :color="tag.tagColor || 'blue'"
              class="!mr-0 !mb-1"
              style="font-size: 12px; line-height: 18px"
            >
              {{ tag.tagName }}
            </Tag>
          </div>
        </div>
      </template>
      <template #level="{ row }">
        <Tag :color="levelColorMap[row.level] || 'default'">
          {{ levelLabelMap[row.level] || row.level || '-' }}
        </Tag>
      </template>

      <template #action="{ row }">
        <span class="action-btns">
          <a
            v-if="
              !isSubordinateView &&
              accessStore.hasAccessCode('crm:customer:followup')
            "
            class="action-btn"
            @click="() => handleFollowup(row)"
            >跟进</a
          >
          <a
            v-if="
              !isSubordinateView &&
              accessStore.hasAccessCode('crm:customer:return-pool')
            "
            class="action-btn"
            @click="() => openPoolReason(row)"
            >公海</a
          >
          <Dropdown :trigger="['click']">
            <a class="action-btn more-btn">更多 ▾</a>
            <template #overlay>
              <div class="customer-more-menu">
                <div
                  v-if="!isSubordinateView"
                  class="more-menu-item"
                  @click="() => handleAddOpportunity(row)"
                >
                  <span>添加商机</span>
                </div>
                <div
                  v-if="!isSubordinateView"
                  class="more-menu-item"
                  @click="() => handleAddContact(row)"
                >
                  <span>添加联系人</span>
                </div>
                <div
                  v-if="
                    !isSubordinateView &&
                    accessStore.hasAccessCode('crm:customer:transfer')
                  "
                  class="more-menu-item"
                  @click="() => handleTransfer(row)"
                >
                  <span>转移</span>
                </div>
                <div
                  v-if="
                    !isSubordinateView &&
                    accessStore.hasAccessCode('crm:customer:update')
                  "
                  class="more-menu-item"
                  @click="() => handleEdit(row)"
                >
                  <span>修改</span>
                </div>
                <Popconfirm
                  v-if="
                    !isSubordinateView &&
                    accessStore.hasAccessCode('crm:customer:delete') &&
                    canDeleteCustomer(row)
                  "
                  :title="
                    $t('ui.text.do_you_want_delete', {
                      moduleName: $t('page.crm.customer.title'),
                    })
                  "
                  :ok-text="$t('ui.button.ok')"
                  :cancel-text="$t('ui.button.cancel')"
                  @confirm="handleDelete(row)"
                >
                  <div class="more-menu-item danger">
                    <span>删除</span>
                  </div>
                </Popconfirm>
              </div>
            </template>
          </Dropdown>
        </span>
      </template>
    </Grid>
    <Drawer
      v-model:open="followupVisible"
      :width="followupFullscreen ? '100vw' : '75%'"
      placement="right"
      :destroy-on-close="true"
      :mask-closable="true"
      :closable="true"
      title="客户跟进"
      :footer="null"
      @close="handleFollowupClose"
    >
      <template #extra>
        <Button
          type="text"
          :icon="h(followupFullscreen ? LucideMinimize2 : LucideMaximize2)"
          @click="toggleFollowupFullscreen"
        />
      </template>
      <CustomerFollowupDrawer
        v-if="followupCustomerId"
        :id="followupCustomerId"
        @refresh="followupNeedRefresh = true"
      />
    </Drawer>

    <CustomerDetailDrawer
      v-model:visible="detailVisible"
      :id="detailId ?? undefined"
      :customer-type="detailCustomerType"
      @created="handleDetailCreated"
    />

    <Drawer
      v-model:open="oppDetailVisible"
      :width="1200"
      placement="right"
      :destroy-on-close="true"
      :mask-closable="true"
      :closable="true"
      title="新建商机"
      :body-style="{
        padding: 0,
        maxHeight: 'calc(100vh - 110px)',
        overflow: 'auto',
      }"
      @close="handleOppDetailClose"
    >
      <OpportunityDetail
        :customer-id="oppDetailCustomerId"
        :customer-name="oppDetailCustomerName"
        @created="handleOppDetailCreated"
      />
    </Drawer>

    <ContactDrawerInstance />

    <TransferModal
      v-model:visible="transferVisible"
      :customer-ids="transferCustomerIds"
      @success="onTransferSuccess"
    />

    <SendMailModal
      v-model:visible="sendMailVisible"
      :customer-id="sendMailCustomer?.id"
      :customer-name="sendMailCustomer?.companyName"
    />

    <ReasonFormModal
      v-model:visible="poolReasonVisible"
      title="退回公海"
      mode="pool"
      ok-text="确认退回"
      :submitting="poolReasonSubmitting"
      @confirm="onPoolReasonConfirm"
    />
  </Page>
</template>

<style scoped>
/* 筛选卡片与表格卡片间距（scoped 固化，不依赖 Tailwind 工具类） */
.customer-filter-card {
  margin-bottom: 16px;
}

.customer-grid-card {
  margin-top: 16px;
}

.customer-search-form :deep(.ant-form-item) {
  margin-bottom: 0;
}

.customer-search-form :deep(.ant-form-item-control) {
  flex: 1;
}

.customer-search-form-wrapper {
  width: 100%;
}

@media (min-width: 768px) {
  .customer-search-form-wrapper {
    width: 75%;
  }
}

.action-btns {
  display: inline-flex;
  gap: 15px;
  align-items: center;
  font-size: 13px;
}

.action-btn {
  line-height: 1;
  color: #1677ff;
  text-decoration: none;
  cursor: pointer;
}

.action-btn:hover {
  color: #4096ff;
}

.more-btn {
  white-space: nowrap;
}

.customer-more-menu {
  min-width: 130px;
  padding: 4px 0;
  background: #fff;
  border: 1px solid #e8e8e8;
  border-radius: 6px;
  box-shadow: 0 3px 10px rgb(0 0 0 / 8%);
}

.more-menu-item {
  padding: 8px 14px;
  font-size: 13px;
  color: #333;
  cursor: pointer;
  transition: background 0.2s;
}

.more-menu-item:hover {
  background: #f5f5f5;
}

.more-menu-item.danger {
  color: #ff4d4f;
}

.more-menu-item.danger:hover {
  background: #fff1f0;
}

/* 固定列内容垂直居中 */
:deep(.vxe-table--fixed-right-wrapper .vxe-body--column .vxe-cell) {
  display: flex !important;
  align-items: center !important;
  justify-content: center !important;
  height: 100% !important;
}
</style>
