<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2, LucideEye } from '@vben/icons';
import { useAccessStore, useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Popconfirm, Modal, Timeline, Tag, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteContractApi, getContractListApi, getContractInfoApi, submitContractApi } from '#/api';
import { $t } from '#/locales';
import ApprovalDrawer from './approval-drawer.vue';
import ContractDrawer from './drawer.vue';
import SalesProcessGuide from '../../sale/components/SalesProcessGuide.vue';

const accessStore = useAccessStore();
const userStore = useUserStore();

// 当前登录用户ID（用于判断是否为当前审批人）
const currentUserId = computed(() => {
  const id = userStore.userInfo?.userId;
  if (!id) return undefined;
  const num = Number(id);
  return Number.isFinite(num) ? num : undefined;
});

const detailVisible = ref(false);
const detailData = ref<any>(null);

// 审批弹窗状态
const approvalVisible = ref(false);
const approvalContractId = ref<number | null>(null);

const approvalStatusList: Record<number, { label: string; color: string }> = {
  0: { label: '草稿', color: 'default' },
  1: { label: '待审批', color: 'processing' },
  2: { label: '审批中', color: 'warning' },
  3: { label: '已通过', color: 'success' },
  4: { label: '已驳回', color: 'error' },
};

const actionText: Record<number, string> = {
  1: '提交',
  2: '审批通过',
  3: '驳回',
};

// 合同是否可编辑（草稿或已驳回状态才允许编辑）
function canEdit(row: any) {
  return row.approvalStatus === 0 || row.approvalStatus === 4;
}

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
      fieldName: 'status',
      label: '状态',
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    export: true,
    refresh: true,
    zoom: true,
  },
  height: 'auto',
  exportConfig: {},
  pagerConfig: {},
  cellConfig: {
    isHover: true,
  },
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getContractListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
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
    },
    {
      title: '客户',
      field: 'customerName',
    },
    {
      title: '合同金额',
      field: 'amount',
    },
    {
      title: '状态',
      field: 'status',
    },
    {
      title: '审批状态',
      field: 'approvalStatus',
      slots: { default: 'approvalStatus' },
    },
    {
      title: '开始日期',
      field: 'startDate',
    },
    {
      title: '结束日期',
      field: 'endDate',
    },
    {
      title: $t('ui.table.createTime'),
      field: 'createTime',
      slots: { default: 'createdAt' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 280,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: ContractDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data && data.needRefresh) {
      gridApi.query();
    }
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

// 打开审批进度查看弹窗
function openApproval(row: any) {
  if (!row.instanceId) {
    message.warning('该合同尚未提交审批');
    return;
  }
  approvalContractId.value = row.id;
  approvalVisible.value = true;
}

// 审批弹窗操作成功后刷新列表
function handleApprovalSuccess() {
  gridApi.query();
}

const openDetail = async (row: any) => {
  try {
    const res = await getContractInfoApi(row.id);
    detailData.value = res.data?.data;
    detailVisible.value = true;
  } catch (e: any) {
    message.error(e?.message || '加载详情失败');
  }
};

// 是否显示"提交审批"按钮（草稿或已驳回状态）
function canSubmit(row: any) {
  return row.approvalStatus === 0 || row.approvalStatus === 4;
}

// 是否显示"查看审批"按钮（待审批/审批中/已通过/已驳回）
function canViewApproval(row: any) {
  return row.approvalStatus >= 1 && row.instanceId;
}

// 是否显示"审批通过"标记（已通过）- 改用查看审批替代
function isApproved(row: any) {
  return false;
}
</script>

<template>
  <Page auto-content-height>
    <SalesProcessGuide current-step="contract" />
    <Grid :table-title="$t('page.crm.contract.title')">
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

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #approvalStatus="{ row }">
        <Tag
          :color="approvalStatusList[row.approvalStatus]?.color || 'default'"
          :class="canViewApproval(row) ? 'cursor-pointer' : ''"
          @click="canViewApproval(row) ? openApproval(row) : null"
        >
          {{ approvalStatusList[row.approvalStatus]?.label || '未知' }}
        </Tag>
      </template>

      <template #action="{ row }">
        <Button
          v-if="accessStore.hasAccessCode('crm:contract:info')"
          type="link"
          :icon="h(LucideEye)"
          @click="() => openDetail(row)"
        />

        <!-- 审批操作（动态展示） -->
        <!-- 1. 草稿或已驳回：显示"提交审批" -->
        <Button
          v-if="accessStore.hasAccessCode('crm:contract:submit') && canSubmit(row)"
          type="link"
          @click="() => handleSubmit(row)"
        >
          提交审批
        </Button>

        <!-- 2. 审批中：显示"查看审批" -->
        <Button
          v-if="canViewApproval(row)"
          type="link"
          @click="() => openApproval(row)"
        >
          查看审批
        </Button>

        <!-- 3. 已通过：显示"审批通过"标记 -->
        <span
          v-else-if="isApproved(row)"
          class="px-2 text-xs text-green-600"
        >
          审批通过
        </span>

        <Button
          v-if="accessStore.hasAccessCode('crm:contract:edit') && canEdit(row)"
          type="link"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />

        <Popconfirm
          :title="
            $t('ui.text.do_you_want_delete', {
              moduleName: $t('page.crm.contract.title'),
            })
          "
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="handleDelete(row)"
        >
          <Button
            v-if="accessStore.hasAccessCode('crm:contract:delete') && canEdit(row)"
            type="link"
            danger
            :icon="h(LucideTrash2)"
          />
        </Popconfirm>
      </template>
    </Grid>
    <Drawer />

    <Modal
      v-model:visible="detailVisible"
      :title="detailData?.title || '合同详情'"
      width="800px"
      :footer="null"
      destroy-on-close
    >
      <div v-if="detailData" class="space-y-6">
        <div class="flex justify-between items-center">
          <span class="text-gray-500">合同编号：</span>
          <span>{{ detailData.contractNo }}</span>
        </div>
        <div class="flex justify-between items-center">
          <span class="text-gray-500">客户：</span>
          <span>{{ detailData.customerName }}</span>
        </div>
        <div class="flex justify-between items-center">
          <span class="text-gray-500">合同金额：</span>
          <span>{{ detailData.amount }}</span>
        </div>
        <div class="flex justify-between items-center">
          <span class="text-gray-500">审批状态：</span>
          <Tag :color="approvalStatusList[detailData.approvalStatus]?.color || 'default'">
            {{ approvalStatusList[detailData.approvalStatus]?.label || '未知' }}
          </Tag>
        </div>
        <div class="flex justify-between items-center">
          <span class="text-gray-500">当前审批阶段：</span>
          <span>第{{ detailData.currentApprovalStage || 0 }}级审批</span>
        </div>

        <div v-if="detailData.approvalLogs && detailData.approvalLogs.length > 0">
          <h4 class="text-lg font-semibold mb-4">审批记录</h4>
          <Timeline>
            <Timeline.Item
              v-for="log in detailData.approvalLogs"
              :key="log.id"
              :color="log.action === 2 ? 'green' : log.action === 3 ? 'red' : 'blue'"
            >
              <div class="font-medium">{{ actionText[log.action] }}</div>
              <div class="text-gray-500 text-sm">
                {{ log.operatorName }} · {{ formatDateTime(log.createTime) }}
              </div>
              <div v-if="log.reason" class="text-gray-600 text-sm mt-1">
                {{ log.reason }}
              </div>
            </Timeline.Item>
          </Timeline>
        </div>
        <div v-else class="text-gray-400">暂无审批记录</div>
      </div>
    </Modal>

    <!-- 审批进度查看抽屉 -->
    <ApprovalDrawer
      v-model:visible="approvalVisible"
      :contract-id="approvalContractId"
      :current-user-id="currentUserId"
      @success="handleApprovalSuccess"
    />
  </Page>
</template>
