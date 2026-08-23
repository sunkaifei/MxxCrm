<script lang="ts" setup>
// 个人中心「我的交接任务」（F7 补充）：交接确认人（assignee）视角
// 查看本人被指派的交接项并完成确认（确认/不适用），身份一律取自 JWT，无需权限码
import { computed, h, onMounted, ref } from 'vue';

import {
  Alert,
  Button,
  Empty,
  Input,
  message,
  Modal,
  Spin,
  Table,
  Tag,
} from 'ant-design-vue';
import type { ColumnsType } from 'ant-design-vue/es/table';

import {
  confirmResignItemApi,
  getMyTransferItemsApi,
} from '#/api';

import ResignDetailDrawer from '../../system/user/resign-detail.vue';

const loading = ref(false);
const data = ref<any[]>([]);

const detailVisible = ref(false);
const detailRecordId = ref<number | undefined>(undefined);

// 交接项状态映射（与后端 item_status_name 一致）
const itemStatusMap: Record<number, { color: string; label: string }> = {
  0: { label: '待确认', color: 'warning' },
  1: { label: '已确认', color: 'success' },
  2: { label: '不适用', color: 'default' },
};

// 交接单状态映射（与后端 record_status_name 一致）
const recordStatusMap: Record<number, { color: string; label: string }> = {
  1: { label: '交接中', color: 'processing' },
  2: { label: '交接完成', color: 'warning' },
  3: { label: '结算完成', color: 'blue' },
  4: { label: '已离职', color: 'success' },
  5: { label: '已中止', color: 'default' },
};

// 待确认数量（仅主表交接中时计入）
const pendingCount = computed(
  () =>
    data.value.filter(
      (it: any) => it.status === 0 && it.recordStatus === 1,
    ).length,
);

const columns: ColumnsType = [
  {
    title: '被离职员工',
    dataIndex: 'resignUserName',
    width: 120,
    customRender: ({ text }: any) => text || '-',
  },
  {
    title: '交接项',
    dataIndex: 'itemName',
    width: 110,
  },
  {
    title: '所属交接单',
    dataIndex: 'recordStatusName',
    width: 110,
    customRender: ({ record }: any) => {
      const m =
        recordStatusMap[record.recordStatus] || {
          label: record.recordStatusName || '-',
          color: 'default',
        };
      return h(Tag, { color: m.color }, () => m.label);
    },
  },
  {
    title: '项状态',
    dataIndex: 'statusName',
    width: 100,
    customRender: ({ record }: any) => {
      const m =
        itemStatusMap[record.status] || {
          label: record.statusName || '-',
          color: 'default',
        };
      return h(Tag, { color: m.color }, () => m.label);
    },
  },
  {
    title: '确认备注/时间',
    key: 'confirmInfo',
    minWidth: 180,
    customRender: ({ record }: any) =>
      record.confirmTime
        ? `${record.confirmTime}${record.confirmRemark ? `（${record.confirmRemark}）` : ''}`
        : '-',
  },
  {
    title: '操作',
    key: 'action',
    width: 200,
    customRender: ({ record }: any) => {
      const canOperate = record.status === 0 && record.recordStatus === 1;
      const buttons = [
        h(
          Button,
          {
            type: 'link',
            size: 'small',
            disabled: !canOperate,
            onClick: () => openConfirm(record, false),
          },
          { default: () => '确认完成' },
        ),
        h(
          Button,
          {
            type: 'link',
            size: 'small',
            disabled: !canOperate,
            onClick: () => openConfirm(record, true),
          },
          { default: () => '不适用' },
        ),
        h(
          Button,
          {
            type: 'link',
            size: 'small',
            onClick: () => openDetail(record.recordId),
          },
          { default: () => '查看交接单' },
        ),
      ];
      return h('div', { class: 'flex items-center' }, () => buttons);
    },
  },
];

async function loadData() {
  loading.value = true;
  try {
    const res: any = await getMyTransferItemsApi();
    const list = res?.data?.data ?? res?.data ?? res;
    data.value = Array.isArray(list) ? list : [];
  } catch {
    data.value = [];
  } finally {
    loading.value = false;
  }
}

function openDetail(recordId: number) {
  detailRecordId.value = recordId;
  detailVisible.value = true;
}

// ==================== 确认弹窗 ====================
const confirmVisible = ref(false);
const confirmSubmitting = ref(false);
const confirmTarget = ref<any>(null);
const confirmIsNa = ref(false);
const confirmRemark = ref('');

function openConfirm(record: any, isNa: boolean) {
  confirmTarget.value = record;
  confirmIsNa.value = isNa;
  confirmRemark.value = '';
  confirmVisible.value = true;
}

async function handleConfirm() {
  const target = confirmTarget.value;
  if (!target) return;
  confirmSubmitting.value = true;
  try {
    await confirmResignItemApi(target.recordId, {
      itemId: target.itemId,
      isNa: confirmIsNa.value,
      remark: confirmRemark.value?.trim() || undefined,
    });
    message.success(confirmIsNa.value ? '已标记为不适用' : '交接项已确认');
    confirmVisible.value = false;
    loadData();
  } catch (error: any) {
    message.error(error?.message || '操作失败');
  } finally {
    confirmSubmitting.value = false;
  }
}

onMounted(loadData);

defineExpose({ reload: loadData });
</script>

<template>
  <div class="space-y-4">
    <Spin :spinning="loading">
      <div class="space-y-4">
        <!-- 说明 -->
        <Alert type="info" show-icon>
          <template #message>
            <span>我的交接任务</span>
          </template>
          <template #description>
            <div class="text-sm">
              同事离职时您被指派为交接确认人，请在交接完成后逐项确认；确认操作不可撤回（留痕）。无合适的交接内容时可选「不适用」。
            </div>
          </template>
        </Alert>

        <!-- 待办任务表 -->
        <div>
          <h4 class="mb-3 text-base font-semibold" style="color: hsl(var(--foreground))">
            交接项（待确认 {{ pendingCount }} 项）
          </h4>
          <Table
            :columns="columns"
            :data-source="data"
            :pagination="false"
            size="small"
            row-key="itemId"
          />
          <Empty
            v-if="!loading && data.length === 0"
            description="暂无被指派的交接任务"
          />
        </div>
      </div>
    </Spin>

    <!-- 确认弹窗 -->
    <Modal
      v-model:open="confirmVisible"
      :title="confirmIsNa ? '标记为不适用' : '确认交接完成'"
      :ok-text="confirmIsNa ? '确认不适用' : '确认完成'"
      :cancel-text="'取消'"
      :confirm-loading="confirmSubmitting"
      @ok="handleConfirm"
    >
      <div class="space-y-3">
        <div class="text-sm" style="color: hsl(var(--foreground) / 85%)">
          交接项：<b>{{ confirmTarget?.itemName }}</b>（被离职员工：{{ confirmTarget?.resignUserName }}）
        </div>
        <div class="text-sm" style="color: hsl(var(--muted-foreground))">
          {{ confirmIsNa
            ? '标记后该项视为无需交接确认，流程将继续推进。'
            : '确认后该项状态为「已确认」，不可撤回。' }}
        </div>
        <div>
          <div class="mb-1 text-sm" style="color: hsl(var(--foreground) / 85%)">
            备注（可选）
          </div>
          <Input.TextArea
            v-model:value="confirmRemark"
            :rows="3"
            :maxlength="200"
            placeholder="可填写交接说明（如客户已全部转移）"
          />
        </div>
      </div>
    </Modal>

    <ResignDetailDrawer
      v-model:visible="detailVisible"
      :record-id="detailRecordId"
      @success="loadData"
    />
  </div>
</template>
