<script lang="ts" setup>
/**
 * 调拨单详情抽屉（右侧）：
 * - 单据信息 + 明细清单
 * - 审批信息（审批实例流转记录，仓管 → CEO 两级）
 * - 审批操作：审批中且当前用户为节点候选人时显示 通过/驳回
 * - 草稿态提供 提交审批 / 编辑 / 删除 入口（由父页处理跳转）
 */
import { ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';


import {
  Button,
  Descriptions,
  DescriptionsItem,
  message,
  Popconfirm,
  Table,
  Tag,
  Textarea,
} from 'ant-design-vue';

import { getApprovalDetailApi } from '#/api';
import {
  getTransferInfoApi,
  submitTransferApprovalApi,
  transferApproveApi,
  transferRejectApi,
} from '#/api/core/product/transfer';
import { useAccessStore } from '@vben/stores';

const emit = defineEmits<{
  needRefresh: [];
  edit: [id: number];
  remove: [id: number];
}>();

const transferId = ref<number>(0);
const loading = ref(false);
const main = ref<any>({});
const items = ref<any[]>([]);
const approval = ref<any>(null);
const approvalLoading = ref(false);
const actionLoading = ref(false);
const rejectComment = ref('');
const showRejectInput = ref(false);

const accessStore = useAccessStore();

function canAudit(code: string) {
  return accessStore.hasAccessCode(code);
}

const STATUS_MAP: Record<number, { label: string; color: string }> = {
  0: { label: '草稿', color: 'default' },
  1: { label: '审批中', color: 'processing' },
  2: { label: '待出库', color: 'warning' },
  3: { label: '待入库', color: 'cyan' },
  4: { label: '已完成', color: 'success' },
  5: { label: '已取消', color: 'default' },
  6: { label: '已驳回', color: 'error' },
};

function statusTag(status?: number) {
  return STATUS_MAP[status ?? 0] ?? { label: String(status ?? ''), color: 'default' };
}

async function load() {
  if (!transferId.value) return;
  loading.value = true;
  try {
    const res: any = await getTransferInfoApi(transferId.value);
    const rawMain = res?.main ?? {};
    // 后端 get_detail 直出实体 snake_case 字段，此处归一为 camelCase 供模板使用
    main.value = {
      ...rawMain,
      transferNo: rawMain.transferNo ?? rawMain.transfer_no,
      fromWarehouseName: rawMain.fromWarehouseName ?? rawMain.from_warehouse_name,
      toWarehouseName: rawMain.toWarehouseName ?? rawMain.to_warehouse_name,
      createdByName: rawMain.createdByName ?? rawMain.created_by_name,
      createTime: rawMain.createTime ?? rawMain.create_time,
      totalQuantity: rawMain.totalQuantity ?? rawMain.total_quantity,
      remark: rawMain.remark,
      status: rawMain.status,
      instanceId: rawMain.instanceId ?? rawMain.instance_id,
    };
    items.value = res?.items ?? [];
    approval.value = null;
    const instanceId = Number(main.value?.instanceId || 0);
    if (instanceId > 0) {
      approvalLoading.value = true;
      try {
        approval.value = await getApprovalDetailApi(instanceId);
      } catch {
        approval.value = null;
      } finally {
        approvalLoading.value = false;
      }
    }
  } finally {
    loading.value = false;
  }
}

async function onSubmitApproval() {
  actionLoading.value = true;
  try {
    await submitTransferApprovalApi(transferId.value);
    message.success('已提交审批（仓管 → CEO 两级审批）');
    emit('needRefresh');
    await load();
  } catch (e: any) {
    message.error(e?.msg || e?.message || '提交失败');
  } finally {
    actionLoading.value = false;
  }
}

async function onApprove() {
  actionLoading.value = true;
  try {
    await transferApproveApi(transferId.value);
    message.success('审批通过');
    emit('needRefresh');
    await load();
  } catch (e: any) {
    message.error(e?.msg || e?.message || '操作失败');
  } finally {
    actionLoading.value = false;
  }
}

async function onReject() {
  if (!rejectComment.value.trim()) {
    message.error('请填写驳回原因');
    return;
  }
  actionLoading.value = true;
  try {
    await transferRejectApi(transferId.value, rejectComment.value.trim());
    message.success('已驳回，单据退回草稿');
    rejectComment.value = '';
    showRejectInput.value = false;
    emit('needRefresh');
    await load();
  } catch (e: any) {
    message.error(e?.msg || e?.message || '操作失败');
  } finally {
    actionLoading.value = false;
  }
}

const itemColumns = [
  { title: '产品名称', field: 'productName' },
  { title: '规格', field: 'productSku' },
  { title: '数量', field: 'quantity' },
  { title: '备注', field: 'remark' },
];

const [Drawer, drawerApi] = useVbenDrawer({
  footer: false,
  class: 'w-[560px]',
  onOpenChange(isOpen) {
    if (isOpen) {
      const data = drawerApi.getData<{ id: number }>();
      transferId.value = Number(data?.id || 0);
      load();
    }
  },
});

defineExpose({ drawerApi });
</script>

<template>
  <Drawer :title="`调拨单详情 ${main.transferNo || ''}`">
    <div v-loading="loading" class="space-y-4">
      <Descriptions :column="2" size="small" bordered>
        <DescriptionsItem label="调拨单号" :span="2">
          {{ main.transferNo || '—' }}
        </DescriptionsItem>
        <DescriptionsItem label="状态">
          <Tag :color="statusTag(main.status).color">
            {{ statusTag(main.status).label }}
          </Tag>
        </DescriptionsItem>
        <DescriptionsItem label="总数量">
          {{ main.totalQuantity ?? '—' }}
        </DescriptionsItem>
        <DescriptionsItem label="源仓库">
          {{ main.fromWarehouseName || '—' }}
        </DescriptionsItem>
        <DescriptionsItem label="目标仓库">
          {{ main.toWarehouseName || '—' }}
        </DescriptionsItem>
        <DescriptionsItem label="创建人">
          {{ main.createdByName || '—' }}
        </DescriptionsItem>
        <DescriptionsItem label="创建时间">
          {{ main.createTime || '—' }}
        </DescriptionsItem>
        <DescriptionsItem label="备注" :span="2">
          {{ main.remark || '—' }}
        </DescriptionsItem>
      </Descriptions>

      <div>
        <p class="mb-2 font-medium">调拨明细</p>
        <Table
          :data-source="items"
          :columns="itemColumns"
          size="small"
          :pagination="false"
          row-key="id"
        />
      </div>

      <div v-if="approval">
        <p class="mb-2 font-medium">审批信息（仓管 → CEO 两级）</p>
        <Descriptions :column="1" size="small" bordered>
          <DescriptionsItem label="当前状态">
            {{ approval.statusName || approval.status }}
          </DescriptionsItem>
          <DescriptionsItem label="当前审批人">
            {{ (approval.candidateApproverNames || []).join('、') || '—' }}
          </DescriptionsItem>
        </Descriptions>
        <div v-if="(approval.logs || []).length" class="mt-2 space-y-1">
          <div v-for="(log, i) in approval.logs" :key="i" class="text-xs">
            <span class="text-gray-500">{{ log.createTime }}</span>
            <span class="ml-2">{{ log.operName }}</span>
            <span class="ml-2">{{ log.remark || log.actionName || '处理' }}</span>
          </div>
        </div>
      </div>

      <!-- 审批操作：审批中 + 有 audit 权限（引擎侧校验候选人） -->
      <div v-if="main.status === 1 && canAudit('product:transfer:audit')" class="border-t pt-3">
        <p class="mb-2 font-medium">审批操作</p>
        <Textarea
          v-model:value="rejectComment"
          :rows="2"
          placeholder="审批意见（选填）"
          class="mb-2"
        />
        <div class="flex gap-2">
          <Button
            type="primary"
            :loading="actionLoading"
            @click="onApprove"
          >
            通过
          </Button>
          <Button danger @click="showRejectInput = !showRejectInput">
            驳回
          </Button>
        </div>
        <div v-if="showRejectInput" class="mt-2 flex justify-end gap-2">
          <Popconfirm title="确认驳回该调拨单？" @confirm="onReject">
            <Button danger :loading="actionLoading">
              <template #icon><LucideX /></template>
              确认驳回
            </Button>
          </Popconfirm>
        </div>
      </div>

      <!-- 草稿/驳回：提交审批入口 -->
      <div
        v-if="(main.status === 0 || main.status === 6) && canAudit('product:transfer:create')"
        class="border-t pt-3"
      >
        <Button
          type="primary"
          :loading="actionLoading"
          @click="onSubmitApproval"
        >
          <template #icon><LucideCheck /></template>
          提交审批
        </Button>
      </div>
    </div>
  </Drawer>
</template>
