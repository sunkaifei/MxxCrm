<script lang="ts" setup>
/**
 * 公海审批中心弹窗（v3.0 公海优化 §5.8/§6.4）
 * 申领审批（通过=领取到名下）+ 延期审批（通过=回收时点顺延）；拒绝必填理由
 */
import { computed, ref, watch } from 'vue';

import {
  Button,
  message,
  Modal,
  Popconfirm,
  Select,
  Table,
  TabPane,
  Tabs,
  Tag,
  Textarea,
} from 'ant-design-vue';

import {
  auditPoolApplyApi,
  auditRetainApi,
  getPoolApplyPageApi,
  getRetainPageApi,
} from '#/api/core/crm/lead-pool';

const props = defineProps<{
  defaultTab?: 'apply' | 'retain';
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'changed'): void;
  (e: 'update:visible', value: boolean): void;
}>();

const innerVisible = computed({
  get: () => props.visible,
  set: (val: boolean) => emit('update:visible', val),
});

const statusOptions = [
  { label: '待审批', value: 1 },
  { label: '已通过', value: 2 },
  { label: '已拒绝', value: 3 },
  { label: '全部', value: undefined },
];
const statusText: Record<number, string> = {
  1: '待审批',
  2: '已通过',
  3: '已拒绝',
};
const statusColor: Record<number, string> = {
  1: 'processing',
  2: 'success',
  3: 'error',
};

// ==================== 申领审批 ====================
const activeTab = ref<'apply' | 'retain'>('apply');

const applyLoading = ref(false);
const applyRows = ref<any[]>([]);
const applyPage = ref(1);
const applyTotal = ref(0);
const applyStatus = ref<number | undefined>(1);

const applyColumns = [
  { dataIndex: 'leadName', key: 'leadName', title: '线索名称', width: 160 },
  { dataIndex: 'poolName', key: 'poolName', title: '所属池', width: 120 },
  { dataIndex: 'userName', key: 'userName', title: '申请人', width: 100 },
  {
    dataIndex: 'reason',
    ellipsis: true,
    key: 'reason',
    title: '申请理由',
  },
  { dataIndex: 'createTime', key: 'createTime', title: '申请时间', width: 160 },
  { dataIndex: 'status', key: 'status', title: '状态', width: 90 },
  {
    dataIndex: 'auditByName',
    key: 'auditByName',
    title: '审批人',
    width: 100,
  },
  { dataIndex: 'action', key: 'action', title: '操作', width: 120 },
];

async function loadApply() {
  applyLoading.value = true;
  try {
    const data = await getPoolApplyPageApi({
      page: applyPage.value,
      pageSize: 10,
      status: applyStatus.value,
    });
    applyRows.value = data?.items || [];
    applyTotal.value = Number(data?.total || 0);
  } finally {
    applyLoading.value = false;
  }
}

const applyPagination = computed(() => ({
  current: applyPage.value,
  onChange: (p: number) => {
    applyPage.value = p;
    void loadApply();
  },
  pageSize: 10,
  showSizeChanger: false,
  total: applyTotal.value,
}));

// ==================== 延期审批 ====================
const retainLoading = ref(false);
const retainRows = ref<any[]>([]);
const retainPage = ref(1);
const retainTotal = ref(0);
const retainStatus = ref<number | undefined>(1);

const retainColumns = [
  { dataIndex: 'leadName', key: 'leadName', title: '线索名称', width: 160 },
  { dataIndex: 'userName', key: 'userName', title: '申请人', width: 100 },
  {
    dataIndex: 'extendDays',
    key: 'extendDays',
    title: '延长天数',
    width: 90,
    align: 'center' as const,
  },
  {
    dataIndex: 'reason',
    ellipsis: true,
    key: 'reason',
    title: '申请理由',
  },
  { dataIndex: 'createTime', key: 'createTime', title: '申请时间', width: 160 },
  { dataIndex: 'status', key: 'status', title: '状态', width: 90 },
  {
    dataIndex: 'auditByName',
    key: 'auditByName',
    title: '审批人',
    width: 100,
  },
  { dataIndex: 'action', key: 'action', title: '操作', width: 120 },
];

async function loadRetain() {
  retainLoading.value = true;
  try {
    const data = await getRetainPageApi({
      page: retainPage.value,
      pageSize: 10,
      status: retainStatus.value,
    });
    retainRows.value = data?.items || [];
    retainTotal.value = Number(data?.total || 0);
  } finally {
    retainLoading.value = false;
  }
}

const retainPagination = computed(() => ({
  current: retainPage.value,
  onChange: (p: number) => {
    retainPage.value = p;
    void loadRetain();
  },
  pageSize: 10,
  showSizeChanger: false,
  total: retainTotal.value,
}));

// ==================== 打开加载 ====================
watch(
  () => props.visible,
  (val) => {
    if (val) {
      activeTab.value = props.defaultTab || 'apply';
      applyPage.value = 1;
      retainPage.value = 1;
      applyStatus.value = 1;
      retainStatus.value = 1;
      void loadApply();
      void loadRetain();
    }
  },
);

function onApplyStatusChange() {
  applyPage.value = 1;
  void loadApply();
}

function onRetainStatusChange() {
  retainPage.value = 1;
  void loadRetain();
}

// ==================== 审批动作 ====================
async function approveApply(row: any) {
  await auditPoolApplyApi(row.id, true);
  message.success('已通过，线索已领取到申请人名下');
  await loadApply();
  emit('changed');
}

async function approveRetain(row: any) {
  await auditRetainApi(row.id, true);
  message.success('已通过，线索回收时点已顺延');
  await loadRetain();
  emit('changed');
}

const rejectVisible = ref(false);
const rejectSaving = ref(false);
const rejectRemark = ref('');
const rejectTarget = ref<null | { id: string; type: 'apply' | 'retain' }>(
  null,
);

function openReject(row: any, type: 'apply' | 'retain') {
  rejectTarget.value = { id: String(row.id), type };
  rejectRemark.value = '';
  rejectVisible.value = true;
}

async function confirmReject() {
  if (!rejectTarget.value) return;
  const remark = rejectRemark.value.trim();
  if (!remark) {
    message.warning('请填写拒绝理由');
    return;
  }
  rejectSaving.value = true;
  try {
    if (rejectTarget.value.type === 'apply') {
      await auditPoolApplyApi(rejectTarget.value.id, false, remark);
    } else {
      await auditRetainApi(rejectTarget.value.id, false, remark);
    }
    message.success('已拒绝');
    rejectVisible.value = false;
    if (rejectTarget.value.type === 'apply') {
      await loadApply();
    } else {
      await loadRetain();
    }
    emit('changed');
  } finally {
    rejectSaving.value = false;
  }
}
</script>

<template>
  <Modal
    v-model:open="innerVisible"
    :footer="null"
    :width="920"
    destroy-on-close
    title="公海审批"
  >
    <Tabs v-model:active-key="activeTab">
      <TabPane key="apply">
        <template #tab>申领审批</template>
        <div class="mb-3 flex items-center">
          <span class="text-sm text-gray-500 mr-2">状态：</span>
          <Select
            v-model:value="applyStatus"
            :options="statusOptions"
            class="w-32"
            @change="onApplyStatusChange"
          />
        </div>
        <Table
          :columns="applyColumns"
          :data-source="applyRows"
          :loading="applyLoading"
          :pagination="applyPagination"
          row-key="id"
          size="small"
        >
          <template #bodyCell="{ column, record }">
            <template v-if="column.key === 'status'">
              <Tag :color="statusColor[record.status] || 'default'">
                {{ statusText[record.status] || '-' }}
              </Tag>
            </template>
            <template v-else-if="column.key === 'action'">
              <template v-if="record.status === 1">
                <Popconfirm
                  title="确认通过？通过后线索将直接领取到申请人名下"
                  @confirm="approveApply(record)"
                >
                  <Button type="link" size="small" class="!p-0">通过</Button>
                </Popconfirm>
                <Button
                  type="link"
                  size="small"
                  class="!p-0 ml-2 text-red-600"
                  @click="openReject(record, 'apply')"
                >
                  拒绝
                </Button>
              </template>
              <span v-else class="text-gray-400">已处理</span>
            </template>
          </template>
        </Table>
      </TabPane>

      <TabPane key="retain">
        <template #tab>延期审批</template>
        <div class="mb-3 flex items-center">
          <span class="text-sm text-gray-500 mr-2">状态：</span>
          <Select
            v-model:value="retainStatus"
            :options="statusOptions"
            class="w-32"
            @change="onRetainStatusChange"
          />
        </div>
        <Table
          :columns="retainColumns"
          :data-source="retainRows"
          :loading="retainLoading"
          :pagination="retainPagination"
          row-key="id"
          size="small"
        >
          <template #bodyCell="{ column, record }">
            <template v-if="column.key === 'extendDays'">
              +{{ record.extendDays ?? 0 }} 天
            </template>
            <template v-else-if="column.key === 'status'">
              <Tag :color="statusColor[record.status] || 'default'">
                {{ statusText[record.status] || '-' }}
              </Tag>
            </template>
            <template v-else-if="column.key === 'action'">
              <template v-if="record.status === 1">
                <Popconfirm
                  title="确认通过？通过后该线索回收时点将顺延"
                  @confirm="approveRetain(record)"
                >
                  <Button type="link" size="small" class="!p-0">通过</Button>
                </Popconfirm>
                <Button
                  type="link"
                  size="small"
                  class="!p-0 ml-2 text-red-600"
                  @click="openReject(record, 'retain')"
                >
                  拒绝
                </Button>
              </template>
              <span v-else class="text-gray-400">已处理</span>
            </template>
          </template>
        </Table>
      </TabPane>
    </Tabs>

    <!-- 拒绝理由 -->
    <Modal
      v-model:open="rejectVisible"
      :confirm-loading="rejectSaving"
      ok-text="确认拒绝"
      title="拒绝理由"
      @ok="confirmReject"
    >
      <Textarea
        v-model:value="rejectRemark"
        :maxlength="200"
        :rows="3"
        placeholder="请填写拒绝理由（必填，将通知申请人）"
      />
    </Modal>
  </Modal>
</template>
