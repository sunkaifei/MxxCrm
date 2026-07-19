<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';
import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, onMounted, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { useAccessStore, useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Popconfirm, Tabs, Tag, message } from 'ant-design-vue';
import { useRoute, useRouter } from 'vue-router';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deleteContractApi,
  getContractInfoApi,
  getContractListApi,
  submitContractApi,
} from '#/api';
import { $t } from '#/locales';
import ContractDrawer from './drawer.vue';
import ApprovalDrawer from './approval-drawer.vue';
import SalesProcessGuide from '../../sale/components/SalesProcessGuide.vue';
import CustomerDetailDrawer from '../components/CustomerDetailDrawer.vue';

const accessStore = useAccessStore();
const userStore = useUserStore();
const router = useRouter();
const route = useRoute();

// 全部合同 Tab 显示条件：超级管理员 / 系统管理员 / data_scope=全部数据
const canViewAll = computed(() => {
  const roles = userStore.userInfo?.roles ?? [];
  const dataScope = (userStore.userInfo as any)?.dataScope ?? (userStore.userInfo as any)?.data_scope;
  if (roles.includes('super_admin') || roles.includes('system_admin')) return true;
  return dataScope === 1;
});

// 下属合同 Tab 显示条件：超级管理员 / 系统管理员 / 数据权限含部门（2/3/4）
const canViewSubordinate = computed(() => {
  const roles = userStore.userInfo?.roles ?? [];
  const dataScope = (userStore.userInfo as any)?.dataScope ?? (userStore.userInfo as any)?.data_scope;
  if (roles.includes('super_admin') || roles.includes('system_admin')) return true;
  return dataScope === 2 || dataScope === 3 || dataScope === 4;
});

const allTabList = [
  { key: 'all', label: '全部合同' },
  { key: 'my', label: '我的合同' },
  { key: 'subordinate', label: '下属合同' },
];

const tabList = computed(() => {
  const keys: string[] = [];
  if (canViewAll.value) keys.push('all');
  keys.push('my');
  if (canViewSubordinate.value) keys.push('subordinate');
  return allTabList.filter(t => keys.includes(t.key));
});

const activeTab = ref('my');

function handleTabChange(key: string) {
  activeTab.value = key;
  gridApi.query();
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

// 当前登录用户ID
const currentUserId = computed(() => {
  const id = userStore.userInfo?.userId;
  if (!id) return undefined;
  const num = Number(id);
  return Number.isFinite(num) ? num : undefined;
});

// 审批弹窗状态
const approvalVisible = ref(false);
const approvalContractId = ref<number | null>(null);

// ========== 合同状态映射（系统自动驱动）==========
// approvalStatus: 0=草稿 1=待审批 2=审批中 3=已通过(执行中) 4=已驳回
const contractStatusMap: Record<number, { label: string; color: string; description: string }> = {
  0: { label: '草稿', color: 'default', description: '已创建，待提交审批' },
  1: { label: '待审批', color: 'processing', description: '已提交，等待审批人处理' },
  2: { label: '审批中', color: 'warning', description: '正在多级审批流转中' },
  3: { label: '执行中', color: 'success', description: '审批通过，合同生效执行' },
  4: { label: '已驳回', color: 'error', description: '审批被驳回，可修改后重新提交' },
};

const actionText: Record<number, string> = {
  1: '提交',
  2: '审批通过',
  3: '驳回',
};

/**
 * 判断合同是否可编辑
 * - 草稿(0) 或 已驳回(4)：可编辑
 * - 其他状态（已进入审批流程）：不可编辑，只读查看
 */
function canEdit(row: any): boolean {
  const status = row.approvalStatus;
  return status === 0 || status === 4;
}

/** 是否可以删除（同编辑权限） */
function canDelete(row: any): boolean {
  return canEdit(row);
}

/** 是否可以提交审批（草稿或已驳回） */
function canSubmit(row: any): boolean {
  return row.approvalStatus === 0 || row.approvalStatus === 4;
}

/** 是否可以查看审批进度 */
function canViewApproval(row: any): boolean {
  return (row.approvalStatus >= 1 && row.approvalStatus <= 3) && !!row.instanceId;
}

/** 审批是否已通过（可进入发货环节） */
function isApproved(row: any): boolean {
  return row.approvalStatus === 3;
}

/** 是否已发货（合同关联订单已发货，后端返回 shipStatus=1） */
function hasShipment(row: any): boolean {
  return row.shipStatus === 1;
}

// ========== 搜索表单 ==========
const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'contractNo',
      label: '合同编号',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'customerName',
      label: '客户',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'approvalStatus',
      label: '审批状态',
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: [
          { value: 0, label: '草稿' },
          { value: 1, label: '待审批' },
          { value: 2, label: '审批中' },
          { value: 3, label: '已通过' },
          { value: 4, label: '已驳回' },
        ],
      },
    },
  ],
};

// ========== 表格配置 ==========
const gridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    export: true,
    refresh: true,
    zoom: true,
  },
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true },
  rowConfig: { height: 'auto' },
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getContractListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          listType: activeTab.value,
          ...formValues,
        });
      },
    },
  },

  columns: [
    {
      title: $t('ui.table.seq'),
      type: 'seq',
      width: 70,
    },
    {
      title: '合同编号',
      field: 'contractNo',
      slots: { default: 'contractNoSlot' },
      width: 160,
    },
    {
      title: '合同标题',
      field: 'title',
      slots: { default: 'titleSlot' },
      minWidth: 180,
    },
    {
      title: '客户',
      field: 'customerName',
      minWidth: 140,
      slots: { default: 'customerName' },
    },
    {
      title: '合同金额',
      field: 'totalAmount',
      minWidth: 120,
      align: 'right',
      slots: { default: 'amountSlot' },
    },
    {
      title: '审批状态',
      field: 'approvalStatus',
      width: 100,
      slots: { default: 'approvalStatus' },
    },
    {
      title: '开始日期',
      field: 'startDate',
      width: 110,
    },
    {
      title: '结束日期',
      field: 'endDate',
      width: 110,
    },
    {
      title: $t('ui.table.createTime'),
      field: 'createTime',
      width: 160,
      slots: { default: 'createdAt' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 260,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

// ========== Drawer（复用同一个组件，通过 create 区分新建/编辑/查看）==========
const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: ContractDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data && data.needRefresh) {
      gridApi.query();
    }
  },
});

/** 打开抽屉：新建 */
function handleCreate() {
  drawerApi.setData({ create: true });
  drawerApi.open();
}

/** 打开抽屉：编辑（仅草稿/驳回） */
function handleEdit(row: any) {
  drawerApi.setData({ create: false, row });
  drawerApi.open();
}

/** 打开抽屉：查看详情（只读模式） */
function handleView(row: any) {
  drawerApi.setData({ create: false, row, readonly: true });
  drawerApi.open();
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteContractApi([row.id]);
    message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleSubmit(row: any) {
  try {
    await submitContractApi(row.id);
    message.success('提交成功，等待审批');
    gridApi.query();
  } catch (e: any) {
    message.error(e?.message || '提交失败');
  }
}

function openApproval(row: any) {
  if (!row.instanceId) {
    message.warning('该合同尚未提交审批');
    return;
  }
  approvalContractId.value = row.id;
  approvalVisible.value = true;
}

function handleApprovalSuccess() {
  gridApi.query();
}

/** 发货：进入发货页面（按合同过滤） */
function handleShip(row: any) {
  router.push(`/sale/shipment?contractId=${row.id}`);
}

/** 查看发货：进入发货详细页面（按合同过滤并自动打开详情） */
function handleViewShipment(row: any) {
  router.push(`/sale/shipment?contractId=${row.id}&autoView=1`);
}

// 从订单创建合同后自动打开编辑抽屉
onMounted(async () => {
  const editContractId = route.query.editContractId;
  if (editContractId) {
    try {
      const info: any = await getContractInfoApi(Number(editContractId));
      const row = info || {};
      drawerApi.setData({ create: false, row, fromOrder: true });
      drawerApi.open();
      // 清除query参数，避免刷新后重复打开
      router.replace({ query: {} });
    } catch {
      message.warning('加载合同信息失败');
    }
  }
});
</script>

<template>
  <Page>
    <SalesProcessGuide current-step="contract" />
    <Grid :table-title="$t('page.crm.contract.title')">
      <template #form-header>
        <Tabs v-model:activeKey="activeTab" class="mb-3" @change="handleTabChange">
          <Tabs.TabPane v-for="tab in tabList" :key="tab.key" :tab="tab.label" />
        </Tabs>
      </template>
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('crm:contract:create')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.crm.contract.button.create') }}
        </Button>
      </template>

      <!-- 合同编号：点击打开详情 -->
      <template #contractNoSlot="{ row }">
        <span
          class="text-blue-600 hover:text-blue-800 cursor-pointer font-medium"
          @click="handleView(row)"
        >
          {{ row.contractNo || '-' }}
        </span>
      </template>

      <!-- 合同标题：点击打开详情 -->
      <template #titleSlot="{ row }">
        <span
          class="hover:text-blue-600 cursor-pointer"
          @click="handleView(row)"
        >
          {{ row.title || '-' }}
        </span>
      </template>

      <!-- 客户名称：点击打开客户详情 -->
      <template #customerName="{ row }">
        <a v-if="row.customerId" class="text-blue-600 cursor-pointer hover:text-blue-800" @click="() => openCustomerDetail(row)">{{ row.customerName || '-' }}</a>
        <span v-else>{{ row.customerName || '-' }}</span>
      </template>

      <!-- 金额格式化 -->
      <template #amountSlot="{ row }">
        <span class="font-medium">
          ¥{{ Number(row.totalAmount || 0).toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 }) }}
        </span>
      </template>

      <!-- 创建时间 -->
      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <!-- 审批状态标签 -->
      <template #approvalStatus="{ row }">
        <Tag
          :color="contractStatusMap[row.approvalStatus]?.color || 'default'"
          :class="canViewApproval(row) ? 'cursor-pointer' : ''"
          @click="canViewApproval(row) ? openApproval(row) : null"
        >
          {{ contractStatusMap[row.approvalStatus]?.label || '未知' }}
        </Tag>
      </template>

      <!-- 操作列 -->
      <template #action="{ row }">
        <!-- 1. 提交审批按钮（草稿或已驳回） -->
        <Button
          v-if="accessStore.hasAccessCode('crm:contract:submit') && canSubmit(row)"
          type="link"
          @click="() => handleSubmit(row)"
        >
          提交审批
        </Button>

        <!-- 2. 查看审批按钮（审批中） -->
        <Button
          v-if="canViewApproval(row)"
          type="link"
          @click="() => openApproval(row)"
        >
          查看审批
        </Button>

        <!-- 3. 已通过 → 发货 / 查看发货 -->
        <template v-if="isApproved(row)">
          <Button
            v-if="!hasShipment(row)"
            type="link"
            @click="() => handleShip(row)"
          >
            发货
          </Button>
          <Button
            v-else
            type="link"
            @click="() => handleViewShipment(row)"
          >
            查看发货
          </Button>
        </template>

        <!-- 4. 编辑按钮（仅草稿/驳回状态显示，文字） -->
        <Button
          v-if="accessStore.hasAccessCode('crm:contract:update') && canEdit(row)"
          type="link"
          @click="() => handleEdit(row)"
        >
          编辑
        </Button>

        <!-- 5. 删除按钮（仅草稿/驳回状态显示，文字） -->
        <Popconfirm
          :title="$t('ui.text.do_you_want_delete', { moduleName: $t('page.crm.contract.title') })"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('crm:contract:delete') && canDelete(row)"
            type="link"
            danger
          >
            删除
          </Button>
        </Popconfirm>
      </template>
    </Grid>

    <!-- 合同抽屉（新建/编辑/查看共用） -->
    <Drawer />

    <!-- 审批进度查看抽屉 -->
    <ApprovalDrawer
      v-model:visible="approvalVisible"
      :contract-id="approvalContractId"
      :current-user-id="currentUserId"
      @success="handleApprovalSuccess"
    />

    <!-- 客户详情抽屉 -->
    <CustomerDetailDrawer v-model:visible="customerDetailVisible" :id="customerDetailId" />
  </Page>
</template>
<style scoped>

</style>
