<script lang="ts" setup>
// 提交审核确认抽屉（F2）：HR/管理员代提交入职审批前的确认页
// 展示员工信息摘要 + 审批流程预览；重新提交时展示上次驳回/撤回/退回意见
import { computed, ref, watch } from 'vue';

import {
  Button,
  Descriptions,
  DescriptionsItem,
  Drawer,
  message,
  Spin,
  Tag,
} from 'ant-design-vue';

import {
  getApprovalDetailApi,
  getApprovalFlowPreviewApi,
  submitApprovalApi,
} from '#/api';

const props = defineProps<{
  row: any;
  visible: boolean;
}>();

const emit = defineEmits<{
  success: [];
  'update:visible': [val: boolean];
}>();

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

const flowLoading = ref(false);
const flowPreview = ref<any>(null);
const submitting = ref(false);

// 流程预览节点（审批类型节点按 nodeOrder 排序）
const flowNodes = computed(() => {
  const nodes: any[] = flowPreview.value?.nodes || [];
  return nodes
    .filter((n: any) => n.nodeType === 2)
    .toSorted((a: any, b: any) => a.nodeOrder - b.nodeOrder);
});

// 是否为重新提交（最近实例 ∈ {4 驳回,5 撤回,6 退回修改}）
const isResubmit = computed(
  () =>
    props.row?.auditStatus === 0 &&
    [4, 5, 6].includes(props.row?.approvalStatus),
);

// 上次驳回/撤回/退回意见（从最新实例日志取 action=2/7 的 comment）
const lastOpinion = ref('');
const lastInstanceStatus = ref<number | undefined>(undefined);

const statusTextMap: Record<number, string> = {
  4: '驳回',
  5: '撤回',
  6: '退回修改',
};

// approverType 展示文本（与审批引擎 approver_type 定义一致）
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
  lastOpinion.value = '';
  lastInstanceStatus.value = undefined;
  try {
    const res: any = await getApprovalFlowPreviewApi('user_approval');
    flowPreview.value = res?.data?.data ?? res?.data ?? res ?? null;
  } catch {
    flowPreview.value = null;
  } finally {
    flowLoading.value = false;
  }
}

async function loadLastOpinion() {
  const instanceId = props.row?.approvalInstanceId;
  if (!instanceId) return;
  try {
    const res: any = await getApprovalDetailApi(instanceId);
    const inst = res?.data?.data ?? res?.data ?? res ?? null;
    if (!inst) return;
    lastInstanceStatus.value = inst.status;
    const logs: any[] = inst.logs || [];
    const rejectLog = [...logs]
      .reverse()
      .find((log: any) => [2, 7].includes(log.action) && log.comment);
    lastOpinion.value = rejectLog?.comment || '';
  } catch {
    // 无权限或加载失败时静默忽略，不影响提交
  }
}

watch(
  () => props.visible,
  (val) => {
    if (val) {
      loadFlowPreview();
      if (isResubmit.value) {
        loadLastOpinion();
      }
    }
  },
);

async function handleSubmit() {
  const row = props.row;
  if (!row?.id) return;
  submitting.value = true;
  try {
    await submitApprovalApi({
      flowCode: 'user_approval',
      businessType: 'user',
      businessId: row.id,
      businessTitle: row.nickName || row.userName || `用户#${row.id}`,
    });
    message.success('已提交审核，等待审批人处理');
    handleClose();
    emit('success');
  } catch (error: any) {
    message.error(error?.message || '提交失败');
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <Drawer
    v-model:open="drawerOpen"
    :title="isResubmit ? '重新提交入职审批' : '提交入职审批'"
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
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </Button>
      </div>
    </template>

    <div class="flex h-full flex-col">
      <div class="flex-1 overflow-y-auto px-6 py-4 space-y-5">
        <!-- 员工信息摘要 -->
        <section>
          <h4 class="mb-3 text-base font-semibold" style="color: hsl(var(--foreground))">
            员工信息
          </h4>
          <Descriptions
            :column="2"
            size="small"
            :bordered="false"
            class="submit-audit-desc"
          >
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
          </Descriptions>
        </section>

        <!-- 上次驳回/撤回/退回意见（重新提交场景） -->
        <section v-if="isResubmit">
          <h4 class="mb-3 text-base font-semibold" style="color: hsl(var(--foreground))">
            上次审批意见
          </h4>
          <div
            class="rounded-lg border px-4 py-3"
            style="border-color: hsl(var(--destructive) / 40%); background: hsl(var(--destructive) / 8%)"
          >
            <div class="mb-1 text-xs font-medium" style="color: hsl(var(--destructive))">
              {{
                lastInstanceStatus !== undefined
                  ? `审批已被${statusTextMap[lastInstanceStatus] || '退回'}`
                  : '审批未通过'
              }}
            </div>
            <div class="text-sm" style="color: hsl(var(--foreground) / 85%)">
              {{ lastOpinion || '（无意见记录）' }}
            </div>
          </div>
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
                <!-- 发起人节点 -->
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
                    {{ row?.nickName || row?.userName || '-' }}
                  </div>
                </div>
                <span class="text-lg" style="color: hsl(var(--muted-foreground))">→</span>

                <!-- 审批节点 -->
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

        <!-- 审批记录（历史实例摘要） -->
        <section v-if="isResubmit && lastOpinion">
          <h4 class="mb-3 text-base font-semibold" style="color: hsl(var(--foreground))">
            提示
          </h4>
          <div class="text-sm" style="color: hsl(var(--muted-foreground))">
            重新提交将创建新的审批实例，历史记录保留可查。提交后不可修改，请确认档案信息已完善。
          </div>
        </section>
      </div>

      <!-- 底部操作区 -->
      <div
        class="border-t px-6 py-4 flex items-center justify-end gap-3"
        style="border-color: hsl(var(--border))"
      >
        <Button @click="handleClose">取消</Button>
        <Button type="primary" :loading="submitting" @click="handleSubmit">
          确认提交
        </Button>
      </div>
    </div>
  </Drawer>
</template>

<style scoped>
.submit-audit-desc :deep(.ant-descriptions-item-label) {
  width: 90px;
  color: hsl(var(--muted-foreground));
}

.submit-audit-desc :deep(.ant-descriptions-item-content) {
  color: hsl(var(--foreground));
}
</style>
