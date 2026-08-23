<script lang="ts" setup>
import type { UploadFile } from 'ant-design-vue';

import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { useAccessStore, useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import {
  Alert,
  Button,
  Dropdown,
  Menu,
  message,
  Modal,
  Tabs,
  Tag,
  Tooltip,
  Upload,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  deleteContractApi,
  executeContractApi,
  getContractInfoApi,
  getContractListApi,
  signContractApi,
  submitContractApi,
  uploadFileApi,
} from '#/api';
import { PageUsageGuide } from '#/components/PageUsageGuide';
import { useDataScopeTabs } from '#/composables/use-data-scope-tabs';
import { $t } from '#/locales';

import SalesProcessGuide from '../../sale/components/SalesProcessGuide.vue';
import CustomerDetailDrawer from '../components/CustomerDetailDrawer.vue';
import ApprovalDrawer from './approval-drawer.vue';
import ContractDrawer from './drawer.vue';

// 合同管理使用说明步骤数（与 i18n 中 page.crm.contract.guide.steps 数组对齐）
const guideStepCount = 5;

const accessStore = useAccessStore();
const userStore = useUserStore();
const router = useRouter();
const route = useRoute();

// 全部/下属合同 Tab 显示条件
const { canViewAll, canViewSubordinate } = useDataScopeTabs();

// 是否是超级管理员（data_scope=1 表示全部数据权限，即超管；user_type=1 表示超管用户类型）
const isSuperAdmin = computed(() => {
  const userInfo = userStore.userInfo as any;
  const dataScope = userInfo?.dataScope ?? userInfo?.data_scope;
  const userType = userInfo?.userType ?? userInfo?.user_type;
  return dataScope === 1 || userType === 1;
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
  return allTabList.filter((t) => keys.includes(t.key));
});

const activeTab = ref('my');

// 是否为下属视图（下属视图下只能查看，不能操作）
const isSubordinateView = computed(() => activeTab.value === 'subordinate');

function handleTabChange(key: number | string) {
  activeTab.value = key as string;
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
const approvalContractId = ref<null | number>(null);

// ========== 合同状态映射（系统自动驱动）==========
// approvalStatus: 0=草稿 1=待审批 2=审批中 3=已通过(执行中) 4=已驳回 5=已撤回 6=待修改
const contractStatusMap: Record<
  number,
  { color: string; description: string; label: string }
> = {
  0: { label: '草稿', color: 'default', description: '已创建，待提交审批' },
  1: {
    label: '待审批',
    color: 'processing',
    description: '已提交，等待审批人处理',
  },
  2: { label: '审批中', color: 'warning', description: '正在多级审批流转中' },
  3: {
    label: '执行中',
    color: 'success',
    description: '审批通过，合同生效执行',
  },
  4: {
    label: '已驳回',
    color: 'error',
    description: '审批被驳回，可修改后重新提交',
  },
  5: {
    label: '已撤回',
    color: 'warning',
    description: '审批已撤回，可修改后重新提交',
  },
  6: {
    label: '待修改',
    color: 'processing',
    description: '审批被退回，待修改后重新提交',
  },
};

// 合同状态枚举字符串 → 数字映射（后端 ContractStatus 可能以字符串形式返回）
const contractStatusNumMap: Record<string, number> = {
  draft: 1,
  signed: 2,
  executing: 3,
  completed: 4,
  terminated: 5,
};

/**
 * 合同综合阶段（根据 approval_status + status 计算）
 * status: 1=草稿 2=已签署 3=执行中 4=已完成 5=已终止
 */
function getContractPhase(row: any): string {
  const approvalStatus = row.approvalStatus;
  const rawStatus = row.status;
  const status =
    typeof rawStatus === 'number'
      ? rawStatus
      : (contractStatusNumMap[String(rawStatus).toLowerCase()] ??
        Number(rawStatus));

  if (approvalStatus === 0) return 'draft'; // 草稿
  if ([1, 2].includes(approvalStatus)) return 'reviewing'; // 审批中
  if (approvalStatus === 3 && status === 1) return 'pendingSign'; // 待签署（审批通过未签署）
  if (approvalStatus === 3 && status === 2) return 'signed'; // 已签署
  if (approvalStatus === 3 && status === 3) return 'executing'; // 执行中
  if (approvalStatus === 3 && status === 4) return 'completed'; // 已完成
  if ([4, 5, 6].includes(approvalStatus)) return 'rejected'; // 被驳回/撤回/退回
  if (status === 5) return 'terminated'; // 已终止
  return 'draft';
}

/**
 * 判断合同是否可编辑
 * - 草稿(0) / 已驳回(4) / 已撤回(5) / 待修改(6)：可编辑
 * - 其他状态（已进入审批流程）：不可编辑，只读查看
 */
function canEdit(row: any): boolean {
  const status = row.approvalStatus;
  return [0, 4, 5, 6].includes(status);
}

/** 是否可以删除（同编辑权限） */
function canDelete(row: any): boolean {
  return canEdit(row);
}

/** 是否可以提交审批（草稿/已驳回/已撤回/待修改） */
function canSubmit(row: any): boolean {
  return [0, 4, 5, 6].includes(row.approvalStatus);
}

/** 是否可以查看审批进度（有审批实例且状态>=1） */
function canViewApproval(row: any): boolean {
  return row.approvalStatus >= 1 && row.approvalStatus <= 6 && !!row.instanceId;
}

/** 是否可以下载合同（审批通过后的所有状态） */
function canDownload(row: any): boolean {
  return row.approvalStatus === 3 || [4, 5].includes(row.approvalStatus);
}

/** 是否已发货（合同关联订单已发货，后端返回 shipStatus=1） */
function hasShipment(row: any): boolean {
  return row.shipStatus === 1;
}

/** 操作列是否需要展示「更多」下拉菜单 */
function hasMoreActions(row: any): boolean {
  return (
    canEdit(row) ||
    canDelete(row) ||
    canDownload(row) ||
    getContractPhase(row) === 'pendingSign' ||
    (['completed', 'executing', 'signed'].includes(getContractPhase(row)) &&
      row.contractFile) ||
    getContractPhase(row) === 'executing'
  );
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
          { value: 5, label: '已撤回' },
          { value: 6, label: '待修改' },
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
  cellConfig: { isHover: true } as any,
  rowConfig: { height: 48 },
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
      title: '负责人',
      field: 'assignedToName',
      width: 100,
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
      width: 200,
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
  if (isSuperAdmin.value) {
    Modal.warning({
      title: '超级管理员不参与业务操作',
      content:
        '超级管理员仅负责系统管理，请使用业务账号（如销售总监、销售经理、业务员）创建和管理合同。',
      okText: '我知道了',
    });
    return;
  }
  drawerApi.setData({ create: true });
  drawerApi.open();
}

/** 打开抽屉：编辑（仅草稿/驳回） */
function handleEdit(row: any) {
  drawerApi.setData({ create: false, row });
  drawerApi.open();
}

/** 从审批详情跳转到编辑合同 */
function handleEditFromApproval(contractId: number) {
  // 从列表数据中找到对应行
  const row = gridApi.grid
    ?.getTableData()
    .fullData?.find((r: any) => r.id === contractId);
  if (row) {
    handleEdit(row);
  } else {
    // 如果列表中找不到（可能翻页了），用一个临时行
    handleEdit({ id: contractId });
  }
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
  // 检查合同负责人是否是超管
  if (row.assignedTo && isSuperAdmin.value) {
    message.warning(
      '当前合同负责人是超级管理员，请先将合同分配给业务人员再提交审批',
    );
    return;
  }
  try {
    await submitContractApi(row.id);
    window.$message?.success('提交成功，等待审批');
    // 刷新列表获取最新数据（含 instanceId）
    await gridApi.query();
    // 打开审批抽屉，让用户可以配置抄送/转办等
    const updatedRow = gridApi.grid
      ?.getTableData()
      .fullData?.find((r: any) => r.id === row.id);
    if (updatedRow?.instanceId) {
      approvalContractId.value = row.id;
      approvalVisible.value = true;
    }
  } catch (error: any) {
    message.error(error?.message || '提交失败');
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

/** 确认执行合同（已签署 → 执行中） */
function handleExecute(row: any) {
  Modal.confirm({
    title: '确认执行合同',
    content: '确认该合同已签署完成，开始执行？',
    onOk: async () => {
      try {
        await executeContractApi(row.id);
        message.success('合同已进入执行状态');
        gridApi.query();
      } catch (error: any) {
        message.error(error?.message || '操作失败');
      }
    },
  });
}

/** 下载合同 PDF */
function handleDownload(row: any) {
  window.open(`/api/system/contract/pdf/${row.id}`, '_blank');
}

/** 查看签署件 */
function handleViewSignedFile(row: any) {
  if (row.contractFile) {
    window.open(row.contractFile, '_blank');
  } else {
    message.warning('暂无签署件');
  }
}

/** 删除确认 */
function handleDeleteConfirm(row: any) {
  Modal.confirm({
    title: '确认删除',
    content: '确定要删除该合同吗？',
    okType: 'danger',
    onOk: () => handleDelete(row),
  });
}

// ========== 上传签署件弹窗 ==========
const signModalVisible = ref(false);
const signContractRowId = ref<null | number>(null);
const signFileList = ref<UploadFile[]>([]);
const signImageList = ref<UploadFile[]>([]);
const signSubmitting = ref(false);

function handleSign(row: any) {
  signContractRowId.value = row.id;
  signFileList.value = [];
  signImageList.value = [];
  signModalVisible.value = true;
}

async function signCustomRequest(options: any) {
  const { file, onSuccess, onError } = options;
  try {
    const result: any = await uploadFileApi(
      file,
      'contract',
      signContractRowId.value ?? undefined,
    );
    onSuccess?.(result, file);
  } catch (error: any) {
    console.error('上传失败:', error);
    onError?.(error);
    message.error(error?.message || '上传失败');
  }
}

async function handleSignSubmit() {
  const contractId = signContractRowId.value;
  if (!contractId) {
    return;
  }
  const fileUrl = signFileList.value
    .filter((f) => f.status === 'done')
    .map((f) => (f.response as any)?.url)
    .find(Boolean);
  const imageUrls = signImageList.value
    .filter((f) => f.status === 'done')
    .map((f) => (f.response as any)?.url)
    .filter(Boolean);

  if (!fileUrl && imageUrls.length === 0) {
    message.warning('请上传合同签署件或扫描件');
    return;
  }
  signSubmitting.value = true;
  try {
    await signContractApi(contractId, {
      contractFile: fileUrl || undefined,
      contractImages: imageUrls.join(',') || undefined,
    });
    message.success('签署件上传成功');
    signModalVisible.value = false;
    gridApi.query();
  } catch (error: any) {
    message.error(error?.message || '上传失败');
  } finally {
    signSubmitting.value = false;
  }
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
    <PageUsageGuide
      :title="$t('page.crm.contract.guide.title')"
      :brief="$t('page.crm.contract.guide.brief')"
      :expand-text="$t('page.crm.contract.guide.expand')"
      :collapse-text="$t('page.crm.contract.guide.collapse')"
    >
      <div v-for="i in guideStepCount" :key="i" class="page-guide-step-item">
        <div class="page-guide-step-index">{{ i }}</div>
        <div class="page-guide-step-content">
          <div class="page-guide-step-title">
            {{ $t(`page.crm.contract.guide.steps[${i - 1}].title`) }}
          </div>
          <div class="page-guide-step-desc">
            {{ $t(`page.crm.contract.guide.steps[${i - 1}].desc`) }}
          </div>
        </div>
      </div>
    </PageUsageGuide>
    <SalesProcessGuide current-step="contract" />
    <Alert
      v-if="isSuperAdmin"
      type="info"
      show-icon
      message="您当前是超级管理员，仅可查看数据。创建合同、提交审批等业务操作请使用业务账号登录。"
      style="margin-bottom: 12px"
    />
    <Grid :table-title="$t('page.crm.contract.title')">
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
            !isSubordinateView &&
            !isSuperAdmin &&
            accessStore.hasAccessCode('crm:contract:save')
          "
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
        <a
          v-if="row.customerId"
          class="text-blue-600 cursor-pointer hover:text-blue-800"
          @click="() => openCustomerDetail(row)"
          >{{ row.customerName || '-' }}</a
        >
        <span v-else>{{ row.customerName || '-' }}</span>
      </template>

      <!-- 金额格式化 -->
      <template #amountSlot="{ row }">
        <span class="font-medium">
          ¥{{
            Number(row.totalAmount || 0).toLocaleString('zh-CN', {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })
          }}
        </span>
      </template>

      <!-- 创建时间 -->
      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <!-- 审批状态标签 -->
      <template #approvalStatus="{ row }">
        <Tooltip
          v-if="
            row.approvalStatus === 4 ||
            row.approvalStatus === 5 ||
            row.approvalStatus === 6
          "
        >
          <Tag
            :color="contractStatusMap[row.approvalStatus]?.color || 'default'"
            :class="canViewApproval(row) ? 'cursor-pointer' : ''"
            @click="canViewApproval(row) ? openApproval(row) : null"
          >
            {{ contractStatusMap[row.approvalStatus]?.label || '未知' }}
          </Tag>
          <template #title>
            <div>{{ contractStatusMap[row.approvalStatus]?.description }}</div>
            <div v-if="canEdit(row)" style="margin-top: 4px; color: #d4b106">
              点击「编辑」修改后，再「提交审批」重新审核
            </div>
          </template>
        </Tooltip>
        <Tag
          v-else
          :color="contractStatusMap[row.approvalStatus]?.color || 'default'"
          :class="canViewApproval(row) ? 'cursor-pointer' : ''"
          @click="canViewApproval(row) ? openApproval(row) : null"
        >
          {{ contractStatusMap[row.approvalStatus]?.label || '未知' }}
        </Tag>
      </template>

      <!-- 操作列：主按钮 + 更多下拉菜单 -->
      <template #action="{ row }">
        <div class="flex items-center flex-nowrap gap-1">
          <!-- 主按钮（按生命周期优先级展示一个） -->
          <Button
            v-if="
              !isSubordinateView &&
              !isSuperAdmin &&
              accessStore.hasAccessCode('crm:contract:submit') &&
              canSubmit(row)
            "
            type="link"
            size="small"
            @click="() => handleSubmit(row)"
          >
            提交审批
          </Button>
          <Button
            v-else-if="
              !isSubordinateView &&
              accessStore.hasAccessCode('crm:contract:sign') &&
              getContractPhase(row) === 'pendingSign'
            "
            type="primary"
            size="small"
            @click="() => handleSign(row)"
          >
            上传签署件
          </Button>
          <Button
            v-else-if="
              !isSubordinateView &&
              accessStore.hasAccessCode('crm:contract:execute') &&
              getContractPhase(row) === 'signed'
            "
            type="primary"
            size="small"
            @click="() => handleExecute(row)"
          >
            确认执行
          </Button>
          <Button
            v-else-if="canViewApproval(row)"
            type="link"
            size="small"
            @click="() => openApproval(row)"
          >
            查看审批
          </Button>

          <!-- 更多下拉菜单 -->
          <Dropdown v-if="hasMoreActions(row)" :trigger="['click']">
            <Button type="link" size="small" @click.prevent> 更多 ▾ </Button>
            <template #overlay>
              <Menu>
                <!-- 编辑（草稿/驳回/撤回/退回） -->
                <Menu.Item
                  v-if="
                    !isSubordinateView &&
                    accessStore.hasAccessCode('crm:contract:update') &&
                    canEdit(row)
                  "
                  @click="() => handleEdit(row)"
                >
                  {{ row.approvalStatus === 6 ? '修改' : '编辑' }}
                </Menu.Item>
                <!-- 删除（草稿/驳回/撤回/退回） -->
                <Menu.Item
                  v-if="
                    !isSubordinateView &&
                    accessStore.hasAccessCode('crm:contract:delete') &&
                    canDelete(row)
                  "
                  danger
                  @click="() => handleDeleteConfirm(row)"
                >
                  删除
                </Menu.Item>
                <!-- 下载合同（审批通过后均可） -->
                <Menu.Item
                  v-if="canDownload(row)"
                  @click="() => handleDownload(row)"
                >
                  下载合同
                </Menu.Item>
                <!-- 上传签署件（待签署） -->
                <Menu.Item
                  v-if="
                    !isSubordinateView &&
                    accessStore.hasAccessCode('crm:contract:sign') &&
                    getContractPhase(row) === 'pendingSign'
                  "
                  @click="() => handleSign(row)"
                >
                  上传签署件
                </Menu.Item>
                <!-- 查看签署件（已签署及以后） -->
                <Menu.Item
                  v-if="
                    ['signed', 'executing', 'completed'].includes(
                      getContractPhase(row),
                    ) && row.contractFile
                  "
                  @click="() => handleViewSignedFile(row)"
                >
                  查看签署件
                </Menu.Item>
                <!-- 发货（执行中） -->
                <Menu.Item
                  v-if="
                    !isSubordinateView &&
                    getContractPhase(row) === 'executing' &&
                    !hasShipment(row)
                  "
                  @click="() => handleShip(row)"
                >
                  发货
                </Menu.Item>
                <!-- 查看发货（执行中已发货） -->
                <Menu.Item
                  v-if="
                    getContractPhase(row) === 'executing' && hasShipment(row)
                  "
                  @click="() => handleViewShipment(row)"
                >
                  查看发货
                </Menu.Item>
              </Menu>
            </template>
          </Dropdown>
        </div>
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
      @go-edit="handleEditFromApproval"
    />

    <!-- 客户详情抽屉 -->
    <CustomerDetailDrawer
      v-model:visible="customerDetailVisible"
      :id="customerDetailId"
    />

    <!-- 上传签署件弹窗 -->
    <Modal
      v-model:open="signModalVisible"
      title="上传签署件"
      :confirm-loading="signSubmitting"
      :mask-closable="false"
      ok-text="确认签署"
      cancel-text="取消"
      @ok="handleSignSubmit"
    >
      <div class="space-y-4 py-2">
        <div>
          <div class="mb-1 font-medium">合同文件（PDF/Word）</div>
          <Upload
            :file-list="signFileList"
            :custom-request="signCustomRequest"
            :max-count="1"
            accept=".pdf,.doc,.docx"
            list-type="text"
            @change="(info: any) => (signFileList = info.fileList)"
          >
            <Button>点击上传</Button>
          </Upload>
        </div>
        <div>
          <div class="mb-1 font-medium">合同扫描件（图片）</div>
          <Upload
            :file-list="signImageList"
            :custom-request="signCustomRequest"
            :max-count="9"
            multiple
            accept="image/*"
            list-type="picture-card"
            @change="(info: any) => (signImageList = info.fileList)"
          >
            <div class="text-gray-400">点击上传</div>
          </Upload>
        </div>
      </div>
    </Modal>
  </Page>
</template>
<style scoped></style>
