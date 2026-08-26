<script lang="ts" setup>
// 审核抽屉（F3）：当前节点审批人（needAudit）从用户列表打开
// Tab1 员工信息 / Tab2 流程预览（节点高亮 + 审批人）/ Tab3 审核记录（Timeline）
// 操作区：仅当前节点审批人显示「通过」「驳回」（驳回理由必填）
import { computed, ref, watch } from 'vue';

import { formatDateTime } from '@vben/utils';
import { useUserStore } from '@vben/stores';

import {
  Button,
  Descriptions,
  DescriptionsItem,
  Drawer,
  Input,
  message,
  Spin,
  TabPane,
  Tabs,
  Tag,
  Timeline,
} from 'ant-design-vue';

import { getApprovalDetailApi, processApprovalApi } from '#/api';
import { sortApprovalNodes } from '#/api/core/system/approval';

const props = defineProps<{
  row: any;
  instanceId: number | null;
  visible: boolean;
}>();

const emit = defineEmits<{
  success: [];
  'update:visible': [val: boolean];
}>();

const userStore = useUserStore();

// 最大化状态：默认 75% 宽度，最大化到 100%
const isMaximized = ref(false);
const drawerWidth = computed(() => (isMaximized.value ? '100%' : '75%'));

function toggleMaximize() {
  isMaximized.value = !isMaximized.value;
}
function handleClose() {
  emit('update:visible', false);
}

// 本地控制 Drawer 开关：父组件打开时同步打开
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
const comment = ref('');
const actionLoading = ref(false);
const activeTab = ref('info');

const instance = computed(() => detail.value);

// 实例状态映射
const statusMap: Record<number, { color: string; label: string }> = {
  1: { label: '待审批', color: 'processing' },
  2: { label: '审批中', color: 'warning' },
  3: { label: '已通过', color: 'success' },
  4: { label: '已驳回', color: 'error' },
  5: { label: '已撤回', color: 'default' },
  6: { label: '待修改', color: 'orange' },
};

// 日志动作
const logActionText: Record<number, string> = {
  1: '通过',
  2: '驳回',
  3: '转办',
  4: '委派',
  5: '加签',
  6: '退回',
  7: '撤回',
  8: '抄送',
};

const logActionColor: Record<number, string> = {
  1: 'green',
  2: 'red',
  3: 'blue',
  4: 'blue',
  5: 'orange',
  6: 'orange',
  7: 'gray',
  8: 'cyan',
};

// 审批模式映射
const approveModeMap: Record<number, string> = {
  1: '或签',
  2: '会签',
  3: '依次审批',
};

// 节点状态：0未到达 1审批中 2已通过 3已驳回 4已完成
const nodeStatusMap: Record<number, { label: string; textClass: string; borderClass: string; bgClass: string }> = {
  0: { label: '未到达', textClass: 'node-muted', borderClass: 'node-border', bgClass: 'node-bg' },
  1: { label: '审批中', textClass: 'node-active-text', borderClass: 'node-active-border', bgClass: 'node-active-bg' },
  2: { label: '已通过', textClass: 'node-ok-text', borderClass: 'node-ok-border', bgClass: 'node-ok-bg' },
  3: { label: '已驳回', textClass: 'node-err-text', borderClass: 'node-err-border', bgClass: 'node-err-bg' },
  4: { label: '已完成', textClass: 'node-ok-text', borderClass: 'node-ok-border', bgClass: 'node-ok-bg' },
};

// 是否当前节点审批人（后端 needAudit 已保证，抽屉内二次校验候选池）
const currentUserId = computed(() => userStore.userInfo?.userId);
const canApprove = computed(() => {
  if (!instance.value) return false;
  if (instance.value.status !== 1 && instance.value.status !== 2) return false;
  const uidNum = Number(currentUserId.value);
  if (!uidNum) return false;
  const candidates: number[] = instance.value.candidateApprovers || [];
  if (candidates.length > 0) return candidates.includes(uidNum);
  return Number(instance.value.currentApproverId) === uidNum;
});

// 是否发起人
const isSubmitter = computed(() => {
  if (!instance.value) return false;
  return Number(instance.value.submitterId) === Number(currentUserId.value);
});

// 流程预览节点（按连线拓扑排序，与设计器画布/引擎执行顺序一致）
const flowNodes = computed(() => {
  const inst = instance.value || {};
  return sortApprovalNodes(inst.flowNodes, inst.flowEdges).filter(
    (n: any) => n.nodeType === 2,
  );
});

// 发起人节点（首节点自动通过）
const hasSubmitterNode = computed(() => instance.value?.submitterName);

async function loadDetail() {
  if (!props.instanceId) return;
  loading.value = true;
  try {
    const res: any = await getApprovalDetailApi(props.instanceId);
    detail.value = res?.data?.data ?? res?.data ?? res ?? null;
  } catch (error: any) {
    message.error(error?.message || '加载审批详情失败');
  } finally {
    loading.value = false;
  }
}

// 审批通过
async function handleApprove() {
  if (!props.instanceId) return;
  actionLoading.value = true;
  try {
    await processApprovalApi({
      action: 1,
      approverId: currentUserId.value,
      approverName: userStore.userInfo?.realName || userStore.userInfo?.username,
      comment: comment.value || undefined,
      instanceId: props.instanceId,
    });
    message.success('已审批通过，用户将自动启用');
    handleClose();
    emit('success');
  } catch (error: any) {
    message.error(error?.message || '审批失败');
  } finally {
    actionLoading.value = false;
  }
}

// 驳回（理由必填，后端同样兜底校验）
async function handleReject() {
  if (!props.instanceId) return;
  if (!comment.value.trim()) {
    message.warning('驳回时必须填写驳回原因');
    return;
  }
  actionLoading.value = true;
  try {
    await processApprovalApi({
      action: 2,
      approverId: currentUserId.value,
      approverName: userStore.userInfo?.realName || userStore.userInfo?.username,
      comment: comment.value.trim(),
      instanceId: props.instanceId,
    });
    message.success('已驳回，HR 可修改档案后重新提交');
    handleClose();
    emit('success');
  } catch (error: any) {
    message.error(error?.message || '驳回失败');
  } finally {
    actionLoading.value = false;
  }
}

watch(
  () => props.visible,
  (val) => {
    if (val) {
      comment.value = '';
      activeTab.value = 'info';
      loadDetail();
    }
  },
);
</script>

<template>
  <Drawer
    v-model:open="drawerOpen"
    title="员工审核"
    placement="right"
    :width="drawerWidth"
    :closable="false"
    :body-style="{
      padding: 0,
      display: 'flex',
      flexDirection: 'column',
      height: '100%',
    }"
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
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </Button>
      </div>
    </template>

    <Spin :spinning="loading" class="audit-spin">
      <div v-if="detail" class="flex h-full flex-col">
        <!-- 顶部：标题 + 状态 -->
        <div class="px-6 pt-4 pb-0">
          <div class="flex items-start justify-between">
            <div class="flex items-center gap-3">
              <h2 class="text-xl font-bold m-0" style="color: hsl(var(--foreground))">
                {{ row?.nickName || row?.userName || '员工' }} 入职审批
              </h2>
              <Tag :color="statusMap[detail.status]?.color || 'default'">
                {{ statusMap[detail.status]?.label || '未知' }}
              </Tag>
            </div>
          </div>
          <div class="mt-2 text-sm" style="color: hsl(var(--muted-foreground))">
            提交人：{{ detail.submitterName || '-' }} ·
            {{ detail.submittedAt ? formatDateTime(detail.submittedAt) : '-' }} 提交
          </div>
        </div>

        <!-- Tab 导航 -->
        <div class="px-6 mt-3" style="border-bottom: 1px solid hsl(var(--border))">
          <Tabs v-model:active-key="activeTab" class="audit-tabs">
            <TabPane key="info" tab="员工信息" />
            <TabPane key="flow" tab="流程预览" />
            <TabPane key="record" tab="审核记录" />
          </Tabs>
        </div>

        <!-- Tab 内容区 -->
        <div class="flex-1 overflow-y-auto px-6 py-4">
          <!-- ====== 员工信息 Tab ====== -->
          <div v-if="activeTab === 'info'">
            <Descriptions :column="2" size="small" class="audit-desc">
              <DescriptionsItem label="姓名">
                {{ row?.nickName || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="用户名">
                {{ row?.userName || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="部门">
                {{ row?.deptName || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="岗位">
                {{ row?.postName || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="角色">
                {{ row?.roleName || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="手机号">
                {{ row?.mobile || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="邮箱">
                {{ row?.email || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="入职时间">
                {{ row?.hireDate || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="直属上级">
                {{ row?.directManagerName || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="参与工资核算">
                {{ row?.salaryEnabled === 1 ? '参与' : '不参与' }}
              </DescriptionsItem>
            </Descriptions>
            <div class="mt-4 text-xs" style="color: hsl(var(--muted-foreground))">
              档案敏感字段（证件号/银行卡号等）已按权限脱敏，审核内容以档案系统为准。
            </div>
          </div>

          <!-- ====== 流程预览 Tab ====== -->
          <div v-if="activeTab === 'flow'">
            <div class="rounded-lg px-5 py-6 overflow-x-auto" style="background: hsl(var(--muted) / 50%)">
              <div class="flex items-center justify-center gap-2 flex-wrap">
                <!-- 发起人节点 -->
                <div
                  v-if="hasSubmitterNode"
                  class="rounded-lg border px-4 py-3 text-center min-w-[100px]"
                  style="border-color: hsl(var(--primary) / 50%); background: hsl(var(--primary) / 10%)"
                >
                  <div class="flex items-center justify-center gap-1.5">
                    <span class="text-sm font-medium" style="color: hsl(var(--primary))">发起人</span>
                  </div>
                  <div class="mt-1 text-xs" style="color: hsl(var(--muted-foreground))">
                    {{ detail.submitterName || '-' }}
                  </div>
                </div>
                <span v-if="hasSubmitterNode && flowNodes.length > 0" class="text-lg" style="color: hsl(var(--muted-foreground))">→</span>

                <!-- 审批节点（实例进度高亮） -->
                <template v-for="(node, idx) in flowNodes" :key="node.nodeKey">
                  <div class="flex items-center gap-2">
                    <div
                      class="rounded-lg border px-4 py-3 text-center min-w-[120px] transition-all"
                      :class="nodeStatusMap[node.nodeStatus]?.bgClass || 'node-bg'"
                      :style="
                        node.nodeStatus === 1
                          ? { borderColor: 'hsl(var(--primary))', borderWidth: '2px' }
                          : {}
                      "
                    >
                      <div class="flex items-center justify-center gap-1.5">
                        <span
                          class="text-sm font-medium"
                          :class="nodeStatusMap[node.nodeStatus]?.textClass || 'node-muted'"
                        >
                          {{ node.nodeName }}
                        </span>
                        <Tag
                          v-if="node.approveMode && approveModeMap[node.approveMode]"
                          :color="node.approveMode === 2 ? 'purple' : node.approveMode === 3 ? 'orange' : 'blue'"
                        >
                          {{ approveModeMap[node.approveMode] }}
                        </Tag>
                      </div>
                      <div class="mt-1 text-xs" style="color: hsl(var(--muted-foreground))">
                        {{ node.approverName || '—' }}
                      </div>
                      <div v-if="node.nodeStatus === 1" class="mt-0.5 text-xs" style="color: hsl(var(--primary))">
                        待审批
                      </div>
                    </div>
                    <span
                      v-if="idx < flowNodes.length - 1"
                      class="text-lg"
                      style="color: hsl(var(--muted-foreground))"
                    >→</span>
                  </div>
                </template>

                <!-- 结束节点 -->
                <template v-if="flowNodes.length > 0">
                  <span class="text-lg" style="color: hsl(var(--muted-foreground))">→</span>
                  <div
                    class="rounded-lg border px-4 py-3 text-center min-w-[100px]"
                    :class="
                      detail.status === 3
                        ? 'node-ok-border node-ok-bg'
                        : detail.status === 4
                          ? 'node-err-border node-err-bg'
                          : 'node-border node-bg'
                    "
                  >
                    <span
                      class="text-sm font-medium"
                      :class="
                        detail.status === 3
                          ? 'node-ok-text'
                          : detail.status === 4
                            ? 'node-err-text'
                            : 'node-muted'
                      "
                    >
                      {{ detail.status === 3 ? '审批通过' : detail.status === 4 ? '已驳回' : '结束' }}
                    </span>
                  </div>
                </template>
              </div>
            </div>
          </div>

          <!-- ====== 审核记录 Tab ====== -->
          <div v-if="activeTab === 'record'">
            <div v-if="detail.logs && detail.logs.length > 0">
              <Timeline>
                <Timeline.Item
                  v-for="log in detail.logs"
                  :key="log.id"
                  :color="logActionColor[log.action] || 'blue'"
                >
                  <div class="font-medium" style="color: hsl(var(--foreground))">
                    {{ log.nodeName || logActionText[log.action] || '审批' }}
                  </div>
                  <div class="text-sm mt-0.5" style="color: hsl(var(--muted-foreground))">
                    {{ log.approverName || log.operatorName }} ·
                    {{ logActionText[log.action] || '--' }} ·
                    {{ formatDateTime(log.createTime || log.create_at) }}
                  </div>
                  <div
                    v-if="log.comment || log.reason"
                    class="text-sm mt-1 rounded px-2 py-1"
                    style="color: hsl(var(--foreground) / 85%); background: hsl(var(--muted) / 50%)"
                  >
                    {{ log.comment || log.reason }}
                  </div>
                </Timeline.Item>
              </Timeline>
            </div>
            <div v-else class="text-sm py-6 text-center" style="color: hsl(var(--muted-foreground))">
              暂无审核记录
            </div>
          </div>
        </div>

        <!-- ====== 底部操作区：仅当前节点审批人 ====== -->
        <div
          v-if="canApprove"
          class="border-t px-6 py-4"
          style="border-color: hsl(var(--border))"
        >
          <Input.TextArea
            v-model:value="comment"
            :rows="2"
            placeholder="请输入审批意见（驳回时必填）"
          />
          <div class="mt-3 flex items-center justify-end gap-3">
            <Button danger :loading="actionLoading" @click="handleReject">驳回</Button>
            <Button type="primary" :loading="actionLoading" @click="handleApprove">通过</Button>
          </div>
        </div>
        <div
          v-else
          class="border-t px-6 py-3 flex justify-between items-center"
          style="border-color: hsl(var(--border))"
        >
          <span class="text-sm" style="color: hsl(var(--muted-foreground))">
            <template v-if="isSubmitter && [4, 5, 6].includes(detail.status)">
              审批{{ statusMap[detail.status]?.label }}，请修改档案后重新提交
            </template>
            <template v-else-if="[3, 4, 5].includes(detail.status)">
              审批已{{ statusMap[detail.status]?.label }}
            </template>
            <template v-else>暂无审批操作权限</template>
          </span>
          <Button @click="handleClose">关闭</Button>
        </div>
      </div>
    </Spin>
  </Drawer>
</template>

<style scoped>
.audit-spin {
  display: flex;
  height: 100%;
}

.audit-spin :deep(.ant-spin-container) {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.audit-tabs :deep(.ant-tabs-nav) {
  margin-bottom: 0;
}

.audit-desc :deep(.ant-descriptions-item-label) {
  width: 110px;
  color: hsl(var(--muted-foreground));
}

.audit-desc :deep(.ant-descriptions-item-content) {
  color: hsl(var(--foreground));
}

/* 节点状态样式：CSS 变量驱动，明暗模式自适应 */
.node-bg {
  background: hsl(var(--card));
}

.node-border {
  border-color: hsl(var(--border));
}

.node-muted {
  color: hsl(var(--muted-foreground));
}

.node-active-bg {
  background: hsl(var(--primary) / 10%);
}

.node-active-border {
  border-color: hsl(var(--primary) / 40%);
}

.node-active-text {
  color: hsl(var(--primary));
}

.node-ok-bg {
  background: hsl(var(--success) / 12%);
}

.node-ok-border {
  border-color: hsl(var(--success) / 40%);
}

.node-ok-text {
  color: hsl(var(--success));
}

.node-err-bg {
  background: hsl(var(--destructive) / 10%);
}

.node-err-border {
  border-color: hsl(var(--destructive) / 40%);
}

.node-err-text {
  color: hsl(var(--destructive));
}
</style>
