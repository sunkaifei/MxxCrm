<script lang="ts" setup>
// 交接单详情页（F5）：交接项确认、财务结算、离职进度时间线、审批记录
// 入口：用户列表「更多-交接单」（adminId）或直接传 recordId
// 角色矩阵见方案 3.6.4：assignee 确认/不适用、HR 代确认/转派/中止、财务结算
import { computed, h, ref, watch } from 'vue';

import {
  Alert,
  Button,
  DatePicker,
  Descriptions,
  DescriptionsItem,
  Drawer,
  Empty,
  Input,
  message,
  Modal,
  Select,
  Spin,
  Table,
  Tag,
  Timeline,
  TimelineItem,
} from 'ant-design-vue';
import type { ColumnsType } from 'ant-design-vue/es/table';

import { useAccessStore, useUserStore } from '@vben/stores';

import {
  abortResignApi,
  confirmResignItemApi,
  getAdminOptionsApi,
  getResignDetailApi,
  getResignListApi,
  settleResignApi,
  transferResignAssigneeApi,
} from '#/api';

const props = defineProps<{
  visible: boolean;
  recordId?: number;
  adminId?: number;
}>();

const emit = defineEmits<{
  'update:visible': [val: boolean];
  success: [];
}>();

const accessStore = useAccessStore();
const userStore = useUserStore();

const currentUserId = computed(() =>
  Number(userStore.userInfo?.userId ?? userStore.userInfo?.id ?? 0),
);
const isHrOverride = computed(() =>
  accessStore.hasAccessCode('system:resign:confirm'),
);
const canSettle = computed(() =>
  accessStore.hasAccessCode('system:resign:settle'),
);

// 最大化状态：默认 75% 宽度，最大化到 100%
const isMaximized = ref(false);
const drawerWidth = computed(() => (isMaximized.value ? '100%' : '75%'));

function toggleMaximize() {
  isMaximized.value = !isMaximized.value;
}
function handleClose() {
  emit('update:visible', false);
}

const drawerOpen = ref(false);
watch(
  () => props.visible,
  (val) => {
    drawerOpen.value = val;
  },
  { immediate: true },
);
watch(drawerOpen, (val) => {
  if (!val) {
    isMaximized.value = false;
    emit('update:visible', false);
  }
});

const loading = ref(false);
const detail = ref<any>(null);
const recordList = ref<any[]>([]);
const listLoading = ref(false);

// 交接项列
const itemColumns: ColumnsType = [
  { title: '交接项', dataIndex: 'itemName', width: 160 },
  { title: '确认人', dataIndex: 'assigneeName', width: 120 },
  {
    title: '状态',
    dataIndex: 'statusName',
    width: 90,
    customRender: ({ record }: any) => {
      const color =
        record.status === 1 ? 'success' : record.status === 2 ? 'default' : 'warning';
      return h(Tag, { color }, () => record.statusName);
    },
  },
  {
    title: '确认时间',
    dataIndex: 'confirmTime',
    width: 160,
    customRender: ({ text }: any) => text || '-',
  },
  {
    title: '操作',
    key: 'action',
    width: 200,
    customRender: ({ record }: any) => renderItemActions(record),
  },
];

// 审批实例列
const instanceColumns: ColumnsType = [
  { title: '实例ID', dataIndex: 'id', width: 90 },
  { title: '标题', dataIndex: 'businessTitle', minWidth: 140 },
  { title: '提交人', dataIndex: 'submitterName', width: 110 },
  {
    title: '状态',
    dataIndex: 'statusName',
    width: 100,
    customRender: ({ record }: any) => {
      const color =
        record.status === 3
          ? 'success'
          : record.status === 1 || record.status === 2
            ? 'processing'
            : 'warning';
      return h(Tag, { color }, () => record.statusName);
    },
  },
  {
    title: '提交时间',
    dataIndex: 'submittedAt',
    width: 160,
    customRender: ({ text }: any) => text || '-',
  },
];

// 当前用户对该项的操作权限
function canOperateItem(item: any) {
  if (detail.value?.status !== 1) return false;
  if (item.status !== 0) return false;
  return item.assigneeId === currentUserId.value || isHrOverride.value;
}

function renderItemActions(item: any) {
  if (!canOperateItem(item)) return null;
  const children = [
    h(
      Button,
      { type: 'link', size: 'small', onClick: () => openConfirmModal(item, false) },
      { default: () => '确认' },
    ),
    h(
      Button,
      { type: 'link', size: 'small', onClick: () => openConfirmModal(item, true) },
      { default: () => '不适用' },
    ),
  ];
  if (isHrOverride.value) {
    children.push(
      h(
        Button,
        { type: 'link', size: 'small', onClick: () => openTransferModal(item) },
        { default: () => '转派' },
      ),
    );
  }
  return children;
}

// ---- 确认/不适用 ----
const confirmModalVisible = ref(false);
const confirmItem = ref<any>(null);
const confirmIsNa = ref(false);
const confirmRemark = ref('');
const confirming = ref(false);

function openConfirmModal(item: any, isNa: boolean) {
  confirmItem.value = item;
  confirmIsNa.value = isNa;
  confirmRemark.value = '';
  confirmModalVisible.value = true;
}

async function handleConfirm() {
  const item = confirmItem.value;
  if (!item) return;
  confirming.value = true;
  try {
    await confirmResignItemApi(detail.value.id, {
      itemId: item.id,
      isNa: confirmIsNa.value,
      remark: confirmRemark.value?.trim() || undefined,
    });
    message.success('确认成功');
    confirmModalVisible.value = false;
    loadDetail(detail.value.id);
    emit('success');
  } catch (error: any) {
    message.error(error?.message || '确认失败');
  } finally {
    confirming.value = false;
  }
}

// ---- 转派 ----
const transferModalVisible = ref(false);
const transferItem = ref<any>(null);
const transferTarget = ref<number | undefined>(undefined);
const userOptions = ref<{ label: string; value: number }[]>([]);
const transferring = ref(false);

async function loadUserOptions() {
  try {
    const resp: any = await getAdminOptionsApi({ bizOnly: true });
    const list = resp?.data ?? resp ?? [];
    userOptions.value = (Array.isArray(list) ? list : []).map((u: any) => ({
      label: u.label,
      value: Number(u.value),
    }));
  } catch {
    userOptions.value = [];
  }
}

function openTransferModal(item: any) {
  transferItem.value = item;
  transferTarget.value = undefined;
  loadUserOptions();
  transferModalVisible.value = true;
}

async function handleTransfer() {
  if (!transferTarget.value) {
    message.warning('请选择新的确认人');
    return;
  }
  transferring.value = true;
  try {
    await transferResignAssigneeApi(detail.value.id, {
      itemId: transferItem.value.id,
      newAssigneeId: transferTarget.value,
    });
    message.success('转派成功');
    transferModalVisible.value = false;
    loadDetail(detail.value.id);
  } catch (error: any) {
    message.error(error?.message || '转派失败');
  } finally {
    transferring.value = false;
  }
}

// ---- 财务结算 ----
const settleVisible = ref(false);
const settleDate = ref<any>(null);
const settling = ref(false);

async function handleSettle() {
  settling.value = true;
  try {
    await settleResignApi(detail.value.id, {
      leaveDate: settleDate.value?.format?.('YYYY-MM-DD'),
    });
    message.success('结算确认完成，该员工已完全离职');
    settleVisible.value = false;
    loadDetail(detail.value.id);
    emit('success');
  } catch (error: any) {
    message.error(error?.message || '结算失败');
  } finally {
    settling.value = false;
  }
}

// ---- 中止 ----
const abortModalVisible = ref(false);
const abortReason = ref('');
const aborting = ref(false);

const canAbort = computed(() => {
  const d = detail.value;
  if (!d) return false;
  if (![1, 2].includes(d.status)) return false;
  return d.adminId === currentUserId.value || isHrOverride.value;
});

async function handleAbort() {
  if (!abortReason.value?.trim()) {
    message.warning('请填写中止原因');
    return;
  }
  aborting.value = true;
  try {
    await abortResignApi(detail.value.id, { reason: abortReason.value.trim() });
    message.success('离职流程已中止');
    abortModalVisible.value = false;
    loadDetail(detail.value.id);
    emit('success');
  } catch (error: any) {
    message.error(error?.message || '中止失败');
  } finally {
    aborting.value = false;
  }
}

// ---- 数据加载 ----
async function loadDetail(id: number) {
  loading.value = true;
  try {
    const res: any = await getResignDetailApi(id);
    detail.value = res?.data?.data ?? res?.data ?? res ?? null;
  } catch (error: any) {
    detail.value = null;
    message.error(error?.message || '加载交接单失败');
  } finally {
    loading.value = false;
  }
}

// 从用户列表进入：查询该员工全部交接单（按 adminId 过滤），多条时显示选择列表
async function loadListByAdmin(adminId: number) {
  listLoading.value = true;
  recordList.value = [];
  try {
    const res: any = await getResignListApi({ page: 1, pageSize: 100 });
    const data = res?.data?.data ?? res?.data ?? res ?? null;
    const list = (Array.isArray(data) ? data : data?.items || data?.list || []) as any[];
    recordList.value = list.filter((r: any) => r.adminId === adminId);
    if (recordList.value.length === 1) {
      loadDetail(recordList.value[0].id);
    }
  } catch (error: any) {
    message.error(error?.message || '加载交接单列表失败');
  } finally {
    listLoading.value = false;
  }
}

watch(
  () => props.visible,
  (val) => {
    if (!val) return;
    detail.value = null;
    recordList.value = [];
    if (props.recordId) {
      loadDetail(props.recordId);
    } else if (props.adminId) {
      loadListByAdmin(props.adminId);
    }
  },
);

// 状态时间线
const timelineSteps = computed(() => {
  const d = detail.value;
  if (!d) return [];
  const steps = [
    { label: '离职申请', desc: d.createTime ? `提交于 ${d.createTime}` : '' },
    { label: '交接中', desc: '' },
    { label: '交接完成', desc: '' },
    { label: '结算完成', desc: d.actualLeaveDate ? `实际离职日 ${d.actualLeaveDate}` : '' },
    { label: '已离职', desc: '' },
  ];
  return steps;
});

const currentStepIndex = computed(() => {
  const map: Record<number, number> = { 1: 1, 2: 2, 3: 3, 4: 4, 5: 0 };
  return map[detail.value?.status] ?? 0;
});

// 离职类型文案
const RESIGN_TYPE_TEXT: Record<number, string> = {
  1: '主动辞职',
  2: '协商解除',
  3: '辞退',
};

// 交接进度
const confirmCount = computed(() => {
  const items = detail.value?.items || [];
  return items.filter((i: any) => i.status !== 0).length;
});
</script>

<template>
  <Drawer
    v-model:open="drawerOpen"
    :title="`交接单详情${detail ? `（${detail.statusName}）` : ''}`"
    placement="right"
    :width="drawerWidth"
    :closable="false"
    :header-style="{ borderBottom: '1px solid hsl(var(--border))', padding: '16px 24px' }"
  >
    <template #extra>
      <div class="flex items-center gap-1">
        <Button type="text" size="small" @click="toggleMaximize">
          {{ isMaximized ? '⤓ 还原' : '⤢' }}
        </Button>
        <Button type="text" size="small" @click="handleClose">
          <svg
            class="w-4 h-4"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            stroke-width="2"
          >
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </Button>
      </div>
    </template>

    <div class="flex h-full flex-col">
      <div class="flex-1 overflow-y-auto px-6 py-4 space-y-5">
        <Spin :spinning="loading" wrapper-class-name="min-h-[200px]">
          <!-- 多交接单选择列表 -->
          <template v-if="recordList.length > 1">
            <section>
              <h4 class="mb-3 text-base font-semibold" style="color: hsl(var(--foreground))">
                该员工共有 {{ recordList.length }} 条交接单记录，请选择查看
              </h4>
              <div class="space-y-2">
                <div
                  v-for="r in recordList"
                  :key="r.id"
                  class="flex items-center justify-between rounded-lg border px-4 py-3 cursor-pointer"
                  style="border-color: hsl(var(--border))"
                  @click="loadDetail(r.id)"
                >
                  <div>
                    <span class="text-sm font-medium" style="color: hsl(var(--foreground))">
                      #{{ r.id }} · {{ r.statusName }}
                    </span>
                    <span class="ml-2 text-xs" style="color: hsl(var(--muted-foreground))">
                      {{ r.createTime }}
                    </span>
                  </div>
                  <span class="text-sm" style="color: hsl(var(--primary))">查看 →</span>
                </div>
              </div>
            </section>
          </template>

          <!-- 交接单详情 -->
          <template v-else-if="detail">
            <!-- 主信息 -->
            <section>
              <div class="flex items-center justify-between">
                <h4 class="mb-3 text-base font-semibold" style="color: hsl(var(--foreground))">
                  基本信息
                </h4>
                <div class="flex items-center gap-2">
                  <Tag color="processing">{{ detail.statusName }}</Tag>
                  <Tag>已确认 {{ confirmCount }}/{{ (detail.items || []).length }}</Tag>
                </div>
              </div>
              <Descriptions :column="2" size="small" :bordered="false" class="resign-detail-desc">
                <DescriptionsItem label="员工">
                  {{ detail.adminInfo?.nickName || detail.adminInfo?.userName || '-' }}
                </DescriptionsItem>
                <DescriptionsItem label="离职类型">
                  {{ RESIGN_TYPE_TEXT[detail.resignType] || '-' }}
                </DescriptionsItem>
                <DescriptionsItem label="期望离职日">
                  {{ detail.resignDate || '-' }}
                </DescriptionsItem>
                <DescriptionsItem label="实际离职日">
                  {{ detail.actualLeaveDate || '-' }}
                </DescriptionsItem>
                <DescriptionsItem label="交接人">
                  {{ detail.transferToName || '-' }}
                </DescriptionsItem>
                <DescriptionsItem label="创建时间">
                  {{ detail.createTime || '-' }}
                </DescriptionsItem>
              </Descriptions>
              <div
                v-if="detail.reason"
                class="mt-3 rounded-lg px-4 py-3"
                style="background: hsl(var(--muted) / 50%)"
              >
                <div class="mb-1 text-xs font-medium" style="color: hsl(var(--muted-foreground))">
                  离职原因
                </div>
                <div class="text-sm" style="color: hsl(var(--foreground) / 85%)">
                  {{ detail.reason }}
                </div>
              </div>
            </section>

            <!-- 交接项确认 -->
            <section>
              <h4 class="mb-3 text-base font-semibold" style="color: hsl(var(--foreground))">
                交接项确认
              </h4>
              <Alert
                v-if="canAbort"
                type="warning"
                show-icon
                class="mb-3"
                message="全部交接项确认后自动进入「交接完成」；如需中止流程请点击下方「中止离职」"
              />
              <Table
                :columns="itemColumns"
                :data-source="detail.items || []"
                :pagination="false"
                size="small"
                row-key="id"
              />
            </section>

            <!-- 财务结算（财务角色、status=2 时可操作） -->
            <section v-if="canSettle">
              <h4 class="mb-3 text-base font-semibold" style="color: hsl(var(--foreground))">
                财务结算
              </h4>
              <div
                class="rounded-lg border px-4 py-4"
                style="border-color: hsl(var(--border))"
              >
                <div class="text-sm mb-3" style="color: hsl(var(--muted-foreground))">
                  {{ detail.status === 2 ? '交接已完成，可进行财务结算；结算确认后该员工账号将停用并完全离职。' : detail.status === 3 ? '已结算完成，等待最终确认。' : '当前状态不可结算。' }}
                </div>
                <div v-if="detail.status === 2" class="flex items-center gap-3">
                  <DatePicker
                    v-model:value="settleDate"
                    style="width: 220px"
                    placeholder="实际离职日期（可留空）"
                  />
                  <Button type="primary" @click="settleVisible = true">
                    确认结算
                  </Button>
                </div>
              </div>
            </section>

            <!-- 离职进度时间线 -->
            <section>
              <h4 class="mb-3 text-base font-semibold" style="color: hsl(var(--foreground))">
                离职进度
              </h4>
              <div class="rounded-lg px-5 py-5" style="background: hsl(var(--muted) / 50%)">
                <Timeline>
                  <TimelineItem
                    v-for="(step, idx) in timelineSteps"
                    :key="step.label"
                    :color="idx <= currentStepIndex ? 'blue' : 'gray'"
                  >
                    <div
                      class="text-sm font-medium"
                      :style="{ color: idx <= currentStepIndex ? 'hsl(var(--foreground))' : 'hsl(var(--muted-foreground))' }"
                    >
                      {{ step.label }}
                    </div>
                    <div v-if="step.desc" class="text-xs mt-0.5" style="color: hsl(var(--muted-foreground))">
                      {{ step.desc }}
                    </div>
                  </TimelineItem>
                </Timeline>
              </div>
            </section>

            <!-- 审批记录 -->
            <section>
              <h4 class="mb-3 text-base font-semibold" style="color: hsl(var(--foreground))">
                审批记录
              </h4>
              <Table
                :columns="instanceColumns"
                :data-source="detail.instances || []"
                :pagination="false"
                size="small"
                row-key="id"
              />
            </section>
          </template>

          <Empty v-else-if="!loading && !listLoading" description="未找到交接单记录" />
        </Spin>
      </div>

      <!-- 底部操作区 -->
      <div
        v-if="detail"
        class="border-t px-6 py-4 flex items-center justify-end gap-3"
        style="border-color: hsl(var(--border))"
      >
        <Button danger v-if="canAbort" @click="abortModalVisible = true">
          中止离职
        </Button>
        <Button @click="handleClose">关闭</Button>
      </div>
    </div>

    <!-- 确认/不适用 弹窗 -->
    <Modal
      v-model:open="confirmModalVisible"
      :title="confirmIsNa ? '标记为不适用' : '确认交接项'"
      :ok-text="confirmIsNa ? '确认不适用' : '确认完成'"
      :cancel-text="'取消'"
      :confirm-loading="confirming"
      @ok="handleConfirm"
    >
      <div class="space-y-3">
        <div class="text-sm" style="color: hsl(var(--foreground))">
          交接项：<b>{{ confirmItem?.itemName }}</b>
        </div>
        <div
          v-if="confirmIsNa"
          class="text-sm"
          style="color: hsl(var(--muted-foreground))"
        >
          标记「不适用」后该项将跳过确认，且不可撤销。
        </div>
        <Input.TextArea
          v-model:value="confirmRemark"
          :rows="3"
          :maxlength="200"
          placeholder="备注（可选）"
        />
      </div>
    </Modal>

    <!-- 转派 弹窗 -->
    <Modal
      v-model:open="transferModalVisible"
      title="转派交接确认人"
      :ok-text="'确认转派'"
      :cancel-text="'取消'"
      :confirm-loading="transferring"
      @ok="handleTransfer"
    >
      <div class="space-y-3">
        <div class="text-sm" style="color: hsl(var(--foreground))">
          交接项：<b>{{ transferItem?.itemName }}</b>（当前确认人：{{ transferItem?.assigneeName }}）
        </div>
        <Select
          v-model:value="transferTarget"
          :options="userOptions"
          style="width: 100%"
          placeholder="选择新的确认人"
          show-search
          option-filter-prop="label"
        />
      </div>
    </Modal>

    <!-- 结算确认 弹窗 -->
    <Modal
      v-model:open="settleVisible"
      title="确认财务结算"
      :ok-text="'确认结算'"
      :cancel-text="'取消'"
      :confirm-loading="settling"
      @ok="handleSettle"
    >
      <div class="text-sm space-y-2" style="color: hsl(var(--muted-foreground))">
        <div>结算确认后，该员工账号将停用并标记为「已离职」，此操作不可撤销。</div>
        <div v-if="detail">实际离职日期：{{ settleDate?.format?.('YYYY-MM-DD') || detail.actualLeaveDate || detail.resignDate || '（未指定）' }}</div>
      </div>
    </Modal>

    <!-- 中止 弹窗 -->
    <Modal
      v-model:open="abortModalVisible"
      title="中止离职流程"
      :ok-text="'确认中止'"
      :cancel-text="'取消'"
      :confirm-loading="aborting"
      @ok="handleAbort"
    >
      <div class="space-y-3">
        <div class="text-sm" style="color: hsl(var(--muted-foreground))">
          中止后未确认的交接项将批量关闭，交接单状态变为「已中止」，可重新发起离职申请。
        </div>
        <Input.TextArea
          v-model:value="abortReason"
          :rows="3"
          :maxlength="500"
          show-count
          placeholder="请填写中止原因（必填）"
        />
      </div>
    </Modal>
  </Drawer>
</template>

<style scoped>
.resign-detail-desc :deep(.ant-descriptions-item-label) {
  width: 100px;
  color: hsl(var(--muted-foreground));
}

.resign-detail-desc :deep(.ant-descriptions-item-content) {
  color: hsl(var(--foreground));
}
</style>
