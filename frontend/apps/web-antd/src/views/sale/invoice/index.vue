<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, createVNode, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  Dropdown,
  Input,
  Menu,
  MenuItem,
  Modal,
  Tabs,
  Tag,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deleteInvoiceApi,
  downloadFileApi,
  getAttachmentsByEntityApi,
  getInvoiceListApi,
  voidInvoiceApi,
} from '#/api';
import { useDataScopeTabs } from '#/composables/use-data-scope-tabs';
import { $t } from '#/locales';

import SalesProcessGuide from '../components/SalesProcessGuide.vue';
import InvoiceApprovalDrawer from './approval-drawer.vue';
import InvoiceDetailDrawer from './detail-drawer.vue';
import InvoiceDrawer from './drawer.vue';
import InvoiceSubmitDrawer from './submit-drawer.vue';

const accessStore = useAccessStore();

const { canViewAll, canViewSubordinate } = useDataScopeTabs();

const allTabList = [
  { key: 'all', label: '全部发票' },
  { key: 'my', label: '我的发票' },
  { key: 'subordinate', label: '下属发票' },
];

const tabList = computed(() => {
  const keys: string[] = [];
  if (canViewAll.value) keys.push('all');
  keys.push('my');
  if (canViewSubordinate.value) keys.push('subordinate');
  return allTabList.filter((t) => keys.includes(t.key));
});

const activeTab = ref('my');

// 是否为下属视图（下属视图下只能查看，不能操作）
const isSubordinateView = computed(() => activeTab.value === 'subordinate');

function handleTabChange(key: number | string) {
  activeTab.value = key as string;
  gridApi.query();
}

const typeOptions = [
  { label: '增值税专用发票', value: 1 },
  { label: '增值税普通发票', value: 2 },
  { label: '形式发票(PI)', value: 3 },
  { label: '商业发票(CI)', value: 4 },
];

// 发票业务状态（1=草稿、2=待开票、3=已开票、4=已作废、5=已红冲）
const statusOptions = [
  { label: '草稿', value: 1 },
  { label: '待开票', value: 2 },
  { label: '已开票', value: 3 },
  { label: '已作废', value: 4 },
  { label: '已红冲', value: 5 },
];

// 发票状态列：派生展示（status + approvalStatus 联合计算，不落库，参考规则 1.0）
// 优先级：终态（已作废/已红冲）> 审核中 > 已驳回 > 已通过（待开票/已开票）> 草稿
function getInvoiceStatus(row: any): { label: string; color: string } {
  const st = Number(row.status);
  const ap = row.approvalStatus ?? 0;
  if (st === 4) return { label: '已作废', color: 'red' };
  if (st === 5) return { label: '已红冲', color: 'magenta' };
  if (ap === 1 || ap === 2) return { label: '审核中', color: 'processing' };
  if (ap === 4) return { label: '已驳回', color: 'error' };
  if (ap === 3) {
    if (st === 3) return { label: '已开票', color: 'green' };
    return { label: '待开票', color: 'blue' };
  }
  return { label: '草稿', color: 'default' };
}

// 可提交审批 / 可编辑删除 的审批状态（与订单一致：草稿或已驳回）
function isEditableApproval(row: any) {
  return (
    row.approvalStatus === 0 ||
    row.approvalStatus === 4 ||
    row.approvalStatus === null ||
    row.approvalStatus === undefined
  );
}

const typeColorMap: Record<number, string> = {
  1: 'blue',
  2: 'cyan',
  3: 'orange',
  4: 'purple',
};

const typeLabelMap: Record<number, string> = {
  1: '增值税专用发票',
  2: '增值税普通发票',
  3: '形式发票(PI)',
  4: '商业发票(CI)',
};

const currencySymbolMap: Record<number, string> = {
  1: '¥',
  2: '$',
  3: '€',
  4: '£',
  5: '¥',
  6: 'HK$',
};

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'keywords',
      label: '关键词',
      componentProps: { placeholder: '发票号/客户/标题', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'invoiceType',
      label: '类型',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: typeOptions,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: '状态',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: statusOptions,
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, export: true, refresh: true, zoom: true },
  height: 'auto',
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true } as any,
  stripe: true,
  checkboxConfig: { checkMethod: () => true },
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const params: any = {
          page: page.currentPage,
          pageSize: page.pageSize,
          listType: activeTab.value,
        };
        if (formValues.keywords) params.keywords = formValues.keywords;
        if (formValues.invoiceType) params.invoiceType = formValues.invoiceType;
        if (formValues.status) params.status = formValues.status;
        return await getInvoiceListApi(params);
      },
    },
  },
  columns: [
    { type: 'checkbox', width: 50 },
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    {
      title: '发票号',
      field: 'invoiceNo',
      width: 160,
      slots: { default: 'invoiceNo' },
    },
    {
      title: '发票标题',
      field: 'title',
      minWidth: 150,
      slots: { default: 'title' },
    },
    {
      title: '类型',
      field: 'invoiceType',
      width: 140,
      slots: { default: 'invoiceType' },
    },
    { title: '客户名称', field: 'customerName', minWidth: 120 },
    {
      title: '金额',
      field: 'amount',
      width: 130,
      slots: { default: 'amount' },
    },
    {
      title: '发票状态',
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
    { title: '开票日期', field: 'invoiceDate', width: 110 },
    { title: '到期日', field: 'dueDate', width: 110 },
    {
      title: '创建时间',
      field: 'createTime',
      width: 160,
      slots: { default: 'createTime' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 190,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [FormDrawer, drawerApi] = useVbenDrawer({
  connectedComponent: InvoiceDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data?.needRefresh) gridApi.query();
  },
});

// 发票详情抽屉：点击发票号/标题打开
const [DetailDrawer, detailDrawerApi] = useVbenDrawer({
  connectedComponent: InvoiceDetailDrawer,
});

function handleViewDetail(row: any) {
  detailDrawerApi.setData({ row, isSubordinate: isSubordinateView.value });
  detailDrawerApi.open();
}

// ===== 审批流（invoice_approval：部门主管 → 财务审核） =====
// 提交审核页（确认后提交，不直接提交）
const submitVisible = ref(false);
const submitInvoiceId = ref<null | number>(null);

function handleOpenSubmit(row: any) {
  submitInvoiceId.value = row.id;
  submitVisible.value = true;
}

// 审批抽屉（审核中/审批详情，props 驱动）
const approvalVisible = ref(false);
const approvalInvoiceId = ref<null | number>(null);

function handleViewApproval(row: any) {
  approvalInvoiceId.value = row.id;
  approvalVisible.value = true;
}

function handleApprovalRefresh() {
  gridApi.query();
}

// 下载发票：下载财务审核后上传的税控发票文件（entity_type=invoice 附件）
async function handleDownloadInvoice(row: any) {
  try {
    const res: any = await getAttachmentsByEntityApi('invoice', row.id);
    const list = Array.isArray(res) ? res : (res?.items ?? []);
    if (!list || list.length === 0) {
      window.$message.warning('财务尚未上传发票文件');
      return;
    }
    // 取最新上传的一份
    const latest = list[0];
    const blob: any = await downloadFileApi(latest.id, 'download');
    const blobData = blob instanceof Blob ? blob : new Blob([blob]);
    const url = window.URL.createObjectURL(blobData);
    const link = document.createElement('a');
    link.href = url;
    link.download =
      latest.name || latest.fileName || `${row.invoiceNo || row.id}.pdf`;
    document.body.append(link);
    link.click();
    link.remove();
    window.URL.revokeObjectURL(url);
  } catch (error: any) {
    window.$message.error(error?.message || '下载失败');
  }
}

// 删除（Dropdown 内无法嵌套 Popconfirm，用 Modal.confirm）
function handleMenuDelete(row: any) {
  Modal.confirm({
    title: '删除发票',
    content: createVNode('div', null, [
      `确定删除发票「${row.title || row.invoiceNo || row.id}」吗？`,
    ]),
    okText: '删除',
    okType: 'danger',
    cancelText: '取消',
    onOk: async () => {
      await deleteInvoiceApi([row.id]);
      window.$message.success($t('ui.notification.delete_success'));
      gridApi.query();
    },
  });
}

// 作废/红冲（仅已开票 status=3；业务动作需理由，参考规则 1.2）
function handleVoid(row: any, action: 1 | 2) {
  const label = action === 1 ? '作废' : '红冲';
  const reason = ref('');
  Modal.confirm({
    title: `${label}发票`,
    content: createVNode('div', null, [
      createVNode('p', null, `确定${label}发票「${row.title || row.invoiceNo || row.id}」吗？该操作不可撤销。`),
      createVNode(Input, {
        placeholder: `请填写${label}理由`,
        onInput: (v: string) => (reason.value = v),
      }),
    ]),
    okText: `确认${label}`,
    okType: 'danger',
    cancelText: '取消',
    onOk: async () => {
      if (!reason.value?.trim()) {
        window.$message.warning(`${label}时必须填写理由`);
        return Promise.reject();
      }
      await voidInvoiceApi(row.id, action, reason.value.trim());
      window.$message.success(`${label}成功`);
      gridApi.query();
    },
  });
}

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

async function handleBatchDelete() {
  const records = gridApi.grid.getCheckboxRecords();
  if (records.length === 0) {
    window.$message.warning('请选择要删除的发票');
    return;
  }
  const ids = records.map((r: any) => r.id);
  await deleteInvoiceApi(ids);
  window.$message.success($t('ui.notification.delete_success'));
  gridApi.query();
}
</script>

<template>
  <Page auto-content-height>
    <SalesProcessGuide current-step="invoice" />
    <Grid :table-title="$t('page.sale.invoice.title')">
      <template #form-header>
        <Tabs
          v-model:active-key="activeTab"
          class="mb-3"
          @change="handleTabChange"
        >
          <Tabs.TabPane
            v-for="tab in tabList"
            :key="tab.key"
            :tab="tab.label"
          />
        </Tabs>
      </template>
      <template #toolbar-tools>
        <Button
          v-if="
            !isSubordinateView && accessStore.hasAccessCode('sale:invoice:save')
          "
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          新建发票
        </Button>
        <Button
          v-if="
            !isSubordinateView &&
            accessStore.hasAccessCode('sale:invoice:delete')
          "
          class="mr-2"
          @click="handleBatchDelete"
        >
          批量删除
        </Button>
      </template>

      <template #invoiceNo="{ row }">
        <a
          class="invoice-link"
          title="查看发票详情"
          @click="handleViewDetail(row)"
        >
          {{ row.invoiceNo || '-' }}
        </a>
      </template>

      <template #title="{ row }">
        <a
          class="invoice-link"
          title="查看发票详情"
          @click="handleViewDetail(row)"
        >
          {{ row.title || '-' }}
        </a>
      </template>

      <template #invoiceType="{ row }">
        <Tag :color="typeColorMap[row.invoiceType]">
          {{ typeLabelMap[row.invoiceType] || row.invoiceType }}
        </Tag>
      </template>

      <template #amount="{ row }">
        {{ currencySymbolMap[row.currency] || '¥' }}
        {{ row.amount?.toLocaleString?.() ?? row.amount }}
      </template>

      <template #status="{ row }">
        <Tag :color="getInvoiceStatus(row).color">
          {{ getInvoiceStatus(row).label }}
        </Tag>
      </template>

      <template #createTime="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <!-- 审核中：点击打开审批进度页（审批详情/流程图/流转记录） -->
        <a
          v-if="row.approvalStatus === 1 || row.approvalStatus === 2"
          class="invoice-link invoice-link--approving"
          title="查看审核进度"
          @click="handleViewApproval(row)"
        >
          审核中
        </a>
        <!-- 审核完成：点击打开审批详情 -->
        <a
          v-else-if="row.approvalStatus === 3"
          class="invoice-link invoice-link--done"
          title="查看审批详情"
          @click="handleViewApproval(row)"
        >
          审核完成
        </a>
        <!-- 提交审核：打开提交审核页（确认后才提交） -->
        <a
          v-else-if="
            !isSubordinateView &&
            accessStore.hasAccessCode('sale:invoice:update') &&
            isEditableApproval(row)
          "
          class="invoice-link"
          title="打开提交审核页"
          @click="handleOpenSubmit(row)"
        >
          提交审核
        </a>
        <Dropdown>
          <a class="invoice-link" @click.prevent> 更多 ▾ </a>
          <template #overlay>
            <Menu>
              <MenuItem key="download" @click="handleDownloadInvoice(row)">
                下载发票
              </MenuItem>
              <MenuItem
                v-if="
                  !isSubordinateView &&
                  accessStore.hasAccessCode('sale:invoice:update') &&
                  isEditableApproval(row)
                "
                key="edit"
                @click="handleEdit(row)"
              >
                修改
              </MenuItem>
              <MenuItem
                v-if="
                  !isSubordinateView &&
                  accessStore.hasAccessCode('sale:invoice:delete') &&
                  isEditableApproval(row)
                "
                key="delete"
                danger
                @click="handleMenuDelete(row)"
              >
                删除
              </MenuItem>
              <Menu.Divider v-if="row.status === 3" />
              <MenuItem
                v-if="
                  row.status === 3 &&
                  !isSubordinateView &&
                  accessStore.hasAccessCode('sale:invoice:update')
                "
                key="void"
                danger
                @click="handleVoid(row, 1)"
              >
                作废
              </MenuItem>
              <MenuItem
                v-if="
                  row.status === 3 &&
                  !isSubordinateView &&
                  accessStore.hasAccessCode('sale:invoice:update')
                "
                key="red"
                danger
                @click="handleVoid(row, 2)"
              >
                红冲
              </MenuItem>
            </Menu>
          </template>
        </Dropdown>
      </template>
    </Grid>
    <FormDrawer />
    <DetailDrawer />
    <!-- 提交审核页（提交前确认） -->
    <InvoiceSubmitDrawer
      v-model:visible="submitVisible"
      :invoice-id="submitInvoiceId"
      @success="handleApprovalRefresh"
    />
    <!-- 审批进度页（审核中/审批详情） -->
    <InvoiceApprovalDrawer
      v-model:visible="approvalVisible"
      :invoice-id="approvalInvoiceId"
      @success="handleApprovalRefresh"
    />
  </Page>
</template>

<style scoped>
/* 发票号/标题列的详情入口链接 */
.invoice-link {
  color: hsl(var(--primary));
  cursor: pointer;
}

.invoice-link:hover {
  text-decoration: underline;
}

/* 审核中：进行中状态样式 */
.invoice-link--approving {
  font-weight: 600;
}

/* 审核完成：通过状态样式 */
.invoice-link--done {
  color: hsl(152 60% 32%);
  font-weight: 600;
}
</style>
