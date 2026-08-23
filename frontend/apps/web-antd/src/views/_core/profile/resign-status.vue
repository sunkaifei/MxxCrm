<script lang="ts" setup>
// 个人中心「离职申请」（F7）：员工本人发起离职申请、查看本人交接单与离职审批进度
// 身份一律取自 JWT（后端 /profile/resign/apply + /profile/resign/my），员工无需权限码
import { computed, h, onMounted, ref } from 'vue';

import {
  Alert,
  Button,
  DatePicker,
  Drawer,
  Empty,
  Form,
  FormItem,
  Input,
  message,
  Modal,
  RadioButton,
  RadioGroup,
  Select,
  Spin,
  Table,
  Tag,
} from 'ant-design-vue';
import type { ColumnsType } from 'ant-design-vue/es/table';

import { useUserStore } from '@vben/stores';

import {
  abortResignApi,
  getAdminOptionsApi,
  getApprovalFlowPreviewApi,
  getMyResignApi,
  submitMyResignApplyApi,
} from '#/api';

import ResignDetailDrawer from '../../system/user/resign-detail.vue';

const userStore = useUserStore();

const loading = ref(false);
const data = ref<any>(null);
const detailVisible = ref(false);
const detailRecordId = ref<number | undefined>(undefined);

// 交接单状态映射（与后端 record_status_name 一致）
const recordStatusMap: Record<number, { color: string; label: string }> = {
  1: { label: '交接中', color: 'processing' },
  2: { label: '交接完成', color: 'warning' },
  3: { label: '结算完成', color: 'blue' },
  4: { label: '已离职', color: 'success' },
  5: { label: '已中止', color: 'default' },
};

const RESIGN_TYPE_TEXT: Record<number, string> = {
  1: '主动辞职',
  2: '协商解除',
  3: '辞退',
};

// 实例状态映射
const statusMap: Record<number, { color: string; label: string }> = {
  1: { label: '待审批', color: 'processing' },
  2: { label: '审批中', color: 'warning' },
  3: { label: '已通过', color: 'success' },
  4: { label: '已驳回', color: 'error' },
  5: { label: '已撤回', color: 'default' },
  6: { label: '待修改', color: 'orange' },
};

// 交接单列
const recordColumns: ColumnsType = [
  { title: 'ID', dataIndex: 'id', width: 80 },
  {
    title: '离职类型',
    dataIndex: 'resignType',
    width: 100,
    customRender: ({ text }: any) => RESIGN_TYPE_TEXT[text] || '-',
  },
  {
    title: '期望离职日',
    dataIndex: 'resignDate',
    width: 120,
    customRender: ({ text }: any) => text || '-',
  },
  {
    title: '状态',
    dataIndex: 'statusName',
    width: 100,
    customRender: ({ record }: any) => {
      const m = recordStatusMap[record.status] || { label: record.statusName || '-', color: 'default' };
      return h(Tag, { color: m.color }, () => m.label);
    },
  },
  {
    title: '创建时间',
    dataIndex: 'createTime',
    width: 160,
    customRender: ({ text }: any) => text || '-',
  },
  {
    title: '操作',
    key: 'action',
    width: 150,
    customRender: ({ record }: any) => {
      // 完全离职前（status∈{1,2}）发起人可自助中止（方案 3.5.5）
      const canAbort = [1, 2].includes(record.status);
      return h('div', { class: 'flex items-center' }, () => [
        h(
          Button,
          { type: 'link', size: 'small', onClick: () => openDetail(record.id) },
          { default: () => '查看交接单' },
        ),
        canAbort
          ? h(
              Button,
              {
                type: 'link',
                size: 'small',
                danger: true,
                onClick: () => openAbort(record),
              },
              { default: () => '中止离职' },
            )
          : null,
      ]);
    },
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
      const m = statusMap[record.status] || { label: record.statusName || '-', color: 'default' };
      return h(Tag, { color: m.color }, () => m.label);
    },
  },
  {
    title: '提交时间',
    dataIndex: 'submittedAt',
    width: 160,
    customRender: ({ text }: any) => text || '-',
  },
];

// 有进行中交接单（status∈{1,2,3}）时不可再发起
const hasActiveRecord = computed(() =>
  (data.value?.records || []).some((r: any) => [1, 2, 3].includes(r.status)),
);

async function loadData() {
  loading.value = true;
  try {
    const res: any = await getMyResignApi();
    data.value = res?.data?.data ?? res?.data ?? res ?? null;
  } catch (error: any) {
    data.value = null;
  } finally {
    loading.value = false;
  }
}

function openDetail(id: number) {
  detailRecordId.value = id;
  detailVisible.value = true;
}

// ==================== 本人中止离职流程 ====================
const abortVisible = ref(false);
const abortSubmitting = ref(false);
const abortTarget = ref<any>(null);
const abortReason = ref('');

function openAbort(record: any) {
  abortTarget.value = record;
  abortReason.value = '';
  abortVisible.value = true;
}

async function handleAbort() {
  const reason = abortReason.value?.trim();
  if (!reason) {
    message.warning('请填写中止原因');
    return;
  }
  abortSubmitting.value = true;
  try {
    await abortResignApi(abortTarget.value.id, { reason });
    message.success('离职流程已中止');
    abortVisible.value = false;
    loadData();
  } catch (error: any) {
    message.error(error?.message || '操作失败');
  } finally {
    abortSubmitting.value = false;
  }
}

// ==================== 本人发起离职申请抽屉 ====================
const applyVisible = ref(false);
const isMaximized = ref(false);
const drawerWidth = computed(() => (isMaximized.value ? '100%' : '75%'));
const formRef = ref();
const submitting = ref(false);
const flowLoading = ref(false);
const flowPreview = ref<any>(null);
const userOptions = ref<{ label: string; value: number }[]>([]);
const confirmVisible = ref(false);

const formData = ref({
  resignType: 1,
  resignDate: null as any,
  reason: '',
  transferToAdminId: undefined as number | undefined,
});

const flowNodes = computed(() => {
  const nodes: any[] = flowPreview.value?.nodes || [];
  return nodes
    .filter((n: any) => n.nodeType === 2)
    .toSorted((a: any, b: any) => a.nodeOrder - b.nodeOrder);
});

function approverTypeText(node: any) {
  const map: Record<number, string> = {
    1: '指定成员',
    2: '指定角色',
    3: '部门主管',
    4: '发起人自己',
    5: '指定岗位',
    6: '直属上级',
    7: '部门主管链',
  };
  return map[node.approverType] || '审批人';
}

async function loadFlowPreview() {
  flowLoading.value = true;
  try {
    const res: any = await getApprovalFlowPreviewApi('resign_approval');
    flowPreview.value = res?.data?.data ?? res?.data ?? res ?? null;
  } catch {
    flowPreview.value = null;
  } finally {
    flowLoading.value = false;
  }
}

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

function openApply() {
  formData.value = {
    resignType: 1,
    resignDate: null,
    reason: '',
    transferToAdminId: undefined,
  };
  confirmVisible.value = false;
  applyVisible.value = true;
  loadFlowPreview();
  loadUserOptions();
}

function toggleMaximize() {
  isMaximized.value = !isMaximized.value;
}

async function handleSubmit() {
  try {
    await formRef.value.validate();
  } catch {
    return;
  }
  submitting.value = true;
  try {
    await submitMyResignApplyApi({
      resignType: formData.value.resignType,
      resignDate: formData.value.resignDate?.format?.('YYYY-MM-DD'),
      reason: formData.value.reason?.trim() || undefined,
      transferToAdminId: formData.value.transferToAdminId,
    });
    message.success('离职申请已提交，等待审批人处理');
    applyVisible.value = false;
    loadData();
  } catch (error: any) {
    message.error(error?.message || '提交失败');
  } finally {
    submitting.value = false;
  }
}

const selfName = computed(() => {
  const u: any = userStore.userInfo || {};
  return u.nickName || u.realName || u.username || '-';
});

onMounted(loadData);

defineExpose({ reload: loadData });
</script>

<template>
  <div class="space-y-4">
    <Spin :spinning="loading">
      <div class="space-y-4">
        <!-- 发起入口 -->
        <Alert type="info" show-icon>
          <template #message>
            <div class="flex items-center justify-between gap-4">
              <span>我的离职申请</span>
              <Button
                type="primary"
                size="small"
                :disabled="hasActiveRecord"
                @click="openApply"
              >
                发起离职申请
              </Button>
            </div>
          </template>
          <template #description>
            <div v-if="hasActiveRecord" class="text-sm">
              存在进行中的离职流程（交接中/交接完成/结算完成），需待其结束或中止后方可重新发起。
            </div>
            <div v-else class="text-sm">
              提交后进入审批流程：离职申请审批 → 工作交接 → 财务结算 → 完全离职；历次记录保留可查。
            </div>
          </template>
        </Alert>

        <!-- 我的交接单 -->
        <div>
          <h4 class="mb-3 text-base font-semibold" style="color: hsl(var(--foreground))">
            我的交接单
          </h4>
          <Table
            :columns="recordColumns"
            :data-source="data?.records || []"
            :pagination="false"
            size="small"
            row-key="id"
          />
          <Empty v-if="data && (data.records || []).length === 0" description="暂无交接单记录" />
        </div>

        <!-- 离职审批记录 -->
        <div>
          <h4 class="mb-3 text-base font-semibold" style="color: hsl(var(--foreground))">
            离职审批记录
          </h4>
          <Table
            :columns="instanceColumns"
            :data-source="data?.instances || []"
            :pagination="false"
            size="small"
            row-key="id"
          />
          <Empty v-if="data && (data.instances || []).length === 0" description="暂无离职审批记录" />
        </div>
      </div>
    </Spin>

    <!-- 本人发起离职申请 -->
    <Drawer
      v-model:open="applyVisible"
      title="发起离职申请"
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
          <Button type="text" size="small" @click="applyVisible = false">
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </Button>
        </div>
      </template>

      <div class="flex h-full flex-col">
        <div class="flex-1 overflow-y-auto px-6 py-4 space-y-5">
          <!-- 申请人 -->
          <section>
            <h4 class="mb-3 text-base font-semibold" style="color: hsl(var(--foreground))">
              申请人
            </h4>
            <div class="text-sm" style="color: hsl(var(--foreground) / 85%)">
              {{ selfName }}
            </div>
          </section>

          <!-- 离职信息表单 -->
          <section>
            <h4 class="mb-3 text-base font-semibold" style="color: hsl(var(--foreground))">
              离职信息
            </h4>
            <Form ref="formRef" :model="formData" layout="vertical">
              <FormItem
                name="resignType"
                label="离职类型"
                :rules="[{ required: true, message: '请选择离职类型' }]"
              >
                <RadioGroup v-model:value="formData.resignType" button-style="solid">
                  <RadioButton :value="1">主动辞职</RadioButton>
                  <RadioButton :value="2">协商解除</RadioButton>
                  <RadioButton :value="3">辞退</RadioButton>
                </RadioGroup>
              </FormItem>
              <FormItem name="resignDate" label="期望离职日期">
                <DatePicker
                  v-model:value="formData.resignDate"
                  style="width: 100%"
                  placeholder="请选择期望离职日期"
                />
              </FormItem>
              <FormItem name="transferToAdminId" label="交接人">
                <Select
                  v-model:value="formData.transferToAdminId"
                  :options="userOptions"
                  style="width: 100%"
                  placeholder="请选择工作交接人（可留空）"
                  show-search
                  option-filter-prop="label"
                  allow-clear
                />
              </FormItem>
              <FormItem name="reason" label="离职原因">
                <Input.TextArea
                  v-model:value="formData.reason"
                  :rows="4"
                  :maxlength="500"
                  show-count
                  placeholder="请填写离职原因（仅审批链与人事可见）"
                />
              </FormItem>
            </Form>
          </section>

          <!-- 审批流程预览 -->
          <section>
            <h4 class="mb-3 text-base font-semibold" style="color: hsl(var(--foreground))">
              审批流程预览
            </h4>
            <Spin :spinning="flowLoading">
              <div
                v-if="flowNodes.length > 0"
                class="rounded-lg px-5 py-6 overflow-x-auto"
                style="background: hsl(var(--muted) / 50%)"
              >
                <div class="flex items-center justify-center gap-2 flex-wrap">
                  <div
                    class="rounded-lg border px-4 py-3 text-center min-w-[100px]"
                    style="border-color: hsl(var(--primary) / 50%); background: hsl(var(--primary) / 10%)"
                  >
                    <div class="flex items-center justify-center gap-1.5">
                      <span class="text-sm font-medium" style="color: hsl(var(--primary))">
                        发起人
                      </span>
                    </div>
                    <div class="mt-1 text-xs" style="color: hsl(var(--muted-foreground))">
                      {{ selfName }}
                    </div>
                  </div>
                  <span class="text-lg" style="color: hsl(var(--muted-foreground))">→</span>

                  <template v-for="(node, idx) in flowNodes" :key="node.nodeKey">
                    <div class="flex items-center gap-2">
                      <div
                        class="rounded-lg border px-4 py-3 text-center min-w-[120px]"
                        style="border-color: hsl(var(--border)); background: hsl(var(--card))"
                      >
                        <div class="flex items-center justify-center gap-1.5">
                          <span class="text-sm font-medium" style="color: hsl(var(--foreground))">
                            {{ node.nodeName }}
                          </span>
                          <Tag v-if="node.approveMode === 2" color="purple">会签</Tag>
                          <Tag v-else-if="node.approveMode === 3" color="orange">依次</Tag>
                        </div>
                        <div class="mt-1 text-xs" style="color: hsl(var(--muted-foreground))">
                          {{ node.approverTypeDesc || approverTypeText(node) }}
                        </div>
                      </div>
                      <span
                        v-if="idx < flowNodes.length - 1"
                        class="text-lg"
                        style="color: hsl(var(--muted-foreground))"
                      >→</span>
                    </div>
                  </template>
                </div>
              </div>
              <div v-else class="text-sm py-6 text-center" style="color: hsl(var(--muted-foreground))">
                流程模板未配置或已停用，请联系管理员
              </div>
            </Spin>
          </section>
        </div>

        <!-- 底部操作区 -->
        <div
          class="border-t px-6 py-4 flex items-center justify-end gap-3"
          style="border-color: hsl(var(--border))"
        >
          <Button @click="applyVisible = false">取消</Button>
          <Button type="primary" @click="confirmVisible = true">提交申请</Button>
        </div>
      </div>
    </Drawer>

    <!-- 提交确认弹窗 -->
    <Modal
      v-model:open="confirmVisible"
      title="确认提交离职申请"
      :ok-text="'确认提交'"
      :cancel-text="'取消'"
      :confirm-loading="submitting"
      @ok="handleSubmit"
    >
      <div class="text-sm space-y-2" style="color: hsl(var(--muted-foreground))">
        <div>提交后进入离职审批流程，账号状态不变；审批通过后生成交接单进入交接阶段。</div>
        <div>提交后不可自行撤回修改，请确认信息无误。</div>
      </div>
    </Modal>

    <!-- 本人中止离职弹窗（理由必填） -->
    <Modal
      v-model:open="abortVisible"
      title="中止离职流程"
      :ok-text="'确认中止'"
      :cancel-text="'取消'"
      :confirm-loading="abortSubmitting"
      @ok="handleAbort"
    >
      <div class="space-y-3">
        <div class="text-sm" style="color: hsl(var(--muted-foreground))">
          中止后该离职流程结束，您将回到在职状态，可重新发起新的离职申请；未确认的交接项将批量标记为「不适用」，历史记录保留。
        </div>
        <div>
          <div class="mb-1 text-sm" style="color: hsl(var(--foreground) / 85%)">
            中止原因（必填）
          </div>
          <Input.TextArea
            v-model:value="abortReason"
            :rows="3"
            :maxlength="200"
            placeholder="请填写中止原因"
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
