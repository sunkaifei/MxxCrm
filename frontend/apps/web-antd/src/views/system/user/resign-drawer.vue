<script lang="ts" setup>
// 离职申请抽屉（F4）：HR/管理员代发起离职审批
// 表单（类型/日期/原因/交接人）+ 流程预览（resign_approval）+ 历史记录
import { computed, ref, watch } from 'vue';

import {
  Button,
  DatePicker,
  Descriptions,
  DescriptionsItem,
  Drawer,
  Form,
  FormItem,
  Input,
  message,
  RadioGroup,
  RadioButton,
  Select,
  Spin,
  Tag,
} from 'ant-design-vue';

import {
  getAdminOptionsApi,
  getApprovalFlowPreviewApi,
  submitResignApplyApi,
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

const formRef = ref();
const submitting = ref(false);
const flowLoading = ref(false);
const flowPreview = ref<any>(null);
const userOptions = ref<{ label: string; value: number }[]>([]);

const formData = ref({
  resignType: 1, // 1主动辞职 2协商解除 3辞退
  resignDate: null as any, // dayjs 对象，提交时格式化
  reason: '',
  transferToAdminId: undefined as number | undefined,
});

// 流程预览节点（审批类型节点按 nodeOrder 排序）
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

watch(
  () => props.visible,
  (val) => {
    if (val) {
      formRef.value?.resetFields();
      formData.value = {
        resignType: 1,
        resignDate: null,
        reason: '',
        transferToAdminId: undefined,
      };
      loadFlowPreview();
      loadUserOptions();
    }
  },
);

async function handleSubmit() {
  const row = props.row;
  if (!row?.id) return;
  try {
    await formRef.value.validate();
  } catch {
    return;
  }
  submitting.value = true;
  try {
    await submitResignApplyApi({
      adminId: row.id,
      resignType: formData.value.resignType,
      resignDate: formData.value.resignDate?.format?.('YYYY-MM-DD'),
      reason: formData.value.reason?.trim() || undefined,
      transferToAdminId: formData.value.transferToAdminId,
    });
    message.success('离职申请已提交，等待审批人处理');
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
          <Descriptions :column="2" size="small" :bordered="false" class="resign-desc">
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
            <DescriptionsItem label="入职时间">
              {{ row?.hireDate || '-' }}
            </DescriptionsItem>
          </Descriptions>
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
            <FormItem name="transferToAdminId" label="交接人（接手员工）">
              <Select
                v-model:value="formData.transferToAdminId"
                :options="userOptions"
                placeholder="请选择交接人"
                show-search
                option-filter-prop="label"
                allow-clear
              />
            </FormItem>
            <FormItem name="reason" label="离职原因">
              <Input.TextArea
                v-model:value="formData.reason"
                :rows="3"
                :maxlength="500"
                show-count
                placeholder="请填写离职原因（审批人可见，交接确认人不可见）"
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
                  <div class="text-sm font-medium" style="color: hsl(var(--primary))">
                    发起人
                  </div>
                  <div class="mt-1 text-xs" style="color: hsl(var(--muted-foreground))">
                    {{ row?.nickName || row?.userName || '-' }}
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
        <Button @click="handleClose">取消</Button>
        <Button type="primary" :loading="submitting" @click="handleSubmit">
          提交审批
        </Button>
      </div>
    </div>
  </Drawer>
</template>

<style scoped>
.resign-desc :deep(.ant-descriptions-item-label) {
  width: 90px;
  color: hsl(var(--muted-foreground));
}

.resign-desc :deep(.ant-descriptions-item-content) {
  color: hsl(var(--foreground));
}
</style>
