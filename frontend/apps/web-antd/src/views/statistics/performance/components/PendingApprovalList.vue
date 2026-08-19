<script lang="ts" setup>
import { computed, ref, watch } from 'vue';

import { IconifyIcon } from '@vben/icons';

import {
  Button,
  Card,
  Drawer,
  Empty,
  Input,
  message,
  Modal,
  Spin,
  Table,
  Tag,
  Timeline,
  TimelineItem,
} from 'ant-design-vue';

import {
  approvePlanApi,
  getPlanDetailApi,
  getPlanListApi,
  rejectPlanApi,
} from '#/api/core/statistics';

const props = defineProps<{
  visible: boolean;
  year: number;
}>();

const emit = defineEmits<{
  (e: 'update:visible', val: boolean): void;
  (e: 'refresh'): void;
}>();

const loading = ref(false);
const pendingList = ref<any[]>([]);

// 审批操作相关
const detailVisible = ref(false);
const detailLoading = ref(false);
const currentPlanDetail = ref<any>(null);
const currentPlanId = ref<null | number>(null);

// 审批弹窗
const approvalModalVisible = ref(false);
const approvalAction = ref<'approve' | 'reject'>('approve');
const approvalComment = ref('');
const submitting = ref(false);

// ===== 加载待审批列表 =====
async function loadPendingList() {
  loading.value = true;
  try {
    const res: any = await getPlanListApi({
      year: props.year,
      pendingMyApproval: true,
    });
    pendingList.value = Array.isArray(res) ? res : res?.data || [];
  } catch (error: any) {
    console.error('加载待审批列表失败', error);
    pendingList.value = [];
  } finally {
    loading.value = false;
  }
}

// ===== 查看计划详情 =====
async function viewDetail(planId: number) {
  currentPlanId.value = planId;
  detailVisible.value = true;
  detailLoading.value = true;
  try {
    const res: any = await getPlanDetailApi(planId);
    currentPlanDetail.value = res?.data || res;
  } catch (error: any) {
    message.error(error?.message || '加载详情失败');
  } finally {
    detailLoading.value = false;
  }
}

// ===== 打开审批弹窗 =====
function openApprovalModal(action: 'approve' | 'reject') {
  approvalAction.value = action;
  approvalComment.value = '';
  approvalModalVisible.value = true;
}

// ===== 提交审批 =====
async function submitApproval() {
  if (!currentPlanId.value) return;

  if (approvalAction.value === 'reject' && !approvalComment.value.trim()) {
    message.warning('驳回时必须填写原因');
    return;
  }

  submitting.value = true;
  try {
    const params = {
      planId: currentPlanId.value,
      reason: approvalComment.value.trim() || undefined,
    };
    if (approvalAction.value === 'approve') {
      await approvePlanApi(params.planId, params.reason);
      message.success('已通过审批');
    } else {
      await rejectPlanApi(params.planId, params.reason);
      message.success('已驳回');
    }
    approvalModalVisible.value = false;
    detailVisible.value = false;
    await loadPendingList();
    emit('refresh');
  } catch (error: any) {
    message.error(error?.message || '操作失败');
  } finally {
    submitting.value = false;
  }
}

// ===== 监听 visible =====
watch(
  () => props.visible,
  (val) => {
    if (val) loadPendingList();
  },
);

watch(
  () => props.year,
  () => {
    if (props.visible) loadPendingList();
  },
);

// ===== 格式化 =====
function formatCurrency(val: any): string {
  const num = Number(val || 0);
  if (num >= 10_000) return `¥${(num / 10_000).toFixed(2)}万`;
  return `¥${num.toLocaleString()}`;
}

const tableColumns = computed(() => [
  {
    title: '员工',
    dataIndex: 'employeeName',
    key: 'employeeName',
    width: 120,
  },
  {
    title: '年份',
    dataIndex: 'year',
    key: 'year',
    width: 80,
  },
  {
    title: '合同目标总额',
    dataIndex: 'totalContractTarget',
    key: 'totalContractTarget',
    align: 'right' as const,
    customRender: ({ text }: any) => formatCurrency(text),
  },
  {
    title: '回款目标总额',
    dataIndex: 'totalPaymentTarget',
    key: 'totalPaymentTarget',
    align: 'right' as const,
    customRender: ({ text }: any) => formatCurrency(text),
  },
  {
    title: '审批层级',
    key: 'level',
    width: 120,
    customRender: ({ record }: any) => {
      const cur = record.approvalLevel || 0;
      const total = record.totalLevels || 0;
      return `第${cur}级 / 共${total}级`;
    },
  },
  {
    title: '提交时间',
    dataIndex: 'submitTime',
    key: 'submitTime',
    width: 170,
    customRender: ({ text }: any) => text || '-',
  },
  {
    title: '操作',
    key: 'action',
    width: 160,
    fixed: 'right' as const,
  },
]);

// 审批节点状态映射
const nodeStatusMap: Record<number, { color: string; text: string }> = {
  0: { color: 'processing', text: '待审批' },
  1: { color: 'success', text: '已通过' },
  2: { color: 'error', text: '已驳回' },
  3: { color: 'default', text: '已跳过' },
};

// 审批动作映射
const actionMap: Record<number, { color: string; text: string }> = {
  1: { color: 'blue', text: '提交' },
  2: { color: 'green', text: '通过' },
  3: { color: 'red', text: '驳回' },
  4: { color: 'orange', text: '修改申请' },
};
</script>

<template>
  <Drawer
    :open="visible"
    title="待我审批的销售计划"
    width="1100px"
    :body-style="{ padding: '0' }"
    @close="emit('update:visible', false)"
  >
    <Spin :spinning="loading">
      <!-- 待审批列表 -->
      <div class="p-4">
        <div v-if="pendingList.length === 0 && !loading" class="py-12">
          <Empty description="暂无待审批的销售计划" />
        </div>

        <Table
          v-else
          :columns="tableColumns"
          :data-source="pendingList"
          row-key="id"
          :pagination="false"
          size="middle"
          :scroll="{ x: 900 }"
        >
          <template #bodyCell="{ column, record }">
            <template v-if="column.key === 'action'">
              <div class="flex gap-2">
                <Button type="link" size="small" @click="viewDetail(record.id)">
                  <IconifyIcon icon="lucide:eye" class="mr-1" />
                  查看详情
                </Button>
              </div>
            </template>
          </template>
        </Table>
      </div>
    </Spin>

    <!-- 计划详情抽屉 -->
    <Drawer
      v-model:open="detailVisible"
      title="计划详情"
      width="720px"
      :body-style="{ padding: '16px' }"
    >
      <Spin :spinning="detailLoading">
        <template v-if="currentPlanDetail">
          <!-- 基本信息 -->
          <Card size="small" class="mb-4" title="基本信息">
            <div class="grid grid-cols-2 gap-3 text-sm">
              <div>
                <span class="text-gray-500">员工：</span>
                <span class="font-medium">{{
                  currentPlanDetail.employeeName
                }}</span>
              </div>
              <div>
                <span class="text-gray-500">年份：</span>
                <span class="font-medium">{{ currentPlanDetail.year }} 年</span>
              </div>
              <div>
                <span class="text-gray-500">状态：</span>
                <Tag color="processing">待审批</Tag>
              </div>
              <div>
                <span class="text-gray-500">版本：</span>
                <span class="font-medium"
                  >v{{ currentPlanDetail.version || 1 }}</span
                >
              </div>
              <div>
                <span class="text-gray-500">审批层级：</span>
                <span class="font-medium">
                  第{{ currentPlanDetail.approvalLevel }}级 / 共{{
                    currentPlanDetail.totalLevels
                  }}级
                </span>
              </div>
              <div>
                <span class="text-gray-500">提交时间：</span>
                <span>{{ currentPlanDetail.submitTime || '-' }}</span>
              </div>
            </div>
          </Card>

          <!-- 月度目标 -->
          <Card size="small" class="mb-4" title="月度目标明细">
            <Table
              :data-source="currentPlanDetail.monthlyTargets || []"
              :pagination="false"
              size="small"
              row-key="month"
            >
              <Table.Column title="月份" data-index="month" :width="80">
                <template #default="{ text }">{{ text }}月</template>
              </Table.Column>
              <Table.Column
                title="合同目标金额"
                data-index="contractTargetAmount"
                align="right"
              >
                <template #default="{ text }">
                  {{ formatCurrency(text) }}
                </template>
              </Table.Column>
              <Table.Column
                title="回款目标金额"
                data-index="paymentTargetAmount"
                align="right"
              >
                <template #default="{ text }">
                  {{ formatCurrency(text) }}
                </template>
              </Table.Column>
              <Table.Column
                title="合同数量"
                data-index="contractTargetCount"
                align="right"
              />
            </Table>
          </Card>

          <!-- 审批节点链 -->
          <Card
            v-if="currentPlanDetail.approvalNodes?.length"
            size="small"
            class="mb-4"
            title="审批链"
          >
            <Timeline>
              <TimelineItem
                v-for="node in currentPlanDetail.approvalNodes"
                :key="node.id"
                :color="
                  node.status === 1
                    ? 'green'
                    : node.status === 2
                      ? 'red'
                      : 'blue'
                "
              >
                <div class="flex items-center gap-2">
                  <span class="font-medium">第{{ node.level }}级</span>
                  <span>{{ node.approverName }}</span>
                  <Tag
                    v-if="nodeStatusMap[node.status as number]"
                    :color="nodeStatusMap[node.status as number]?.color"
                  >
                    {{ nodeStatusMap[node.status as number]?.text }}
                  </Tag>
                </div>
                <div v-if="node.comment" class="text-xs text-gray-500 mt-1">
                  意见：{{ node.comment }}
                </div>
              </TimelineItem>
            </Timeline>
          </Card>

          <!-- 审批记录 -->
          <Card
            v-if="currentPlanDetail.approvalLogs?.length"
            size="small"
            class="mb-4"
            title="审批记录"
          >
            <Timeline>
              <TimelineItem
                v-for="log in currentPlanDetail.approvalLogs"
                :key="log.id"
              >
                <div class="flex items-center gap-2">
                  <Tag
                    v-if="actionMap[log.action as number]"
                    :color="actionMap[log.action as number]?.color"
                  >
                    {{ actionMap[log.action as number]?.text }}
                  </Tag>
                  <span class="font-medium">{{ log.operatorName }}</span>
                  <span class="text-xs text-gray-400">{{
                    log.createTime
                  }}</span>
                </div>
                <div v-if="log.reason" class="text-xs text-gray-500 mt-1">
                  {{ log.reason }}
                </div>
              </TimelineItem>
            </Timeline>
          </Card>

          <!-- 审批操作 -->
          <div class="flex justify-end gap-2 pt-4 border-t">
            <Button @click="detailVisible = false">关闭</Button>
            <Button danger @click="openApprovalModal('reject')">
              <IconifyIcon icon="lucide:x-circle" class="mr-1" />
              驳回
            </Button>
            <Button type="primary" @click="openApprovalModal('approve')">
              <IconifyIcon icon="lucide:check-circle" class="mr-1" />
              通过
            </Button>
          </div>
        </template>
      </Spin>
    </Drawer>

    <!-- 审批弹窗 -->
    <Modal
      v-model:open="approvalModalVisible"
      :title="approvalAction === 'approve' ? '审批通过' : '驳回计划'"
      :confirm-loading="submitting"
      :ok-text="approvalAction === 'approve' ? '确认通过' : '确认驳回'"
      :ok-type="approvalAction === 'approve' ? 'primary' : 'danger'"
      @ok="submitApproval"
    >
      <div class="py-4">
        <div v-if="approvalAction === 'reject'" class="mb-2 text-red-500">
          <IconifyIcon icon="lucide:alert-triangle" class="mr-1" />
          驳回时必须填写原因，提交人将收到通知
        </div>
        <div v-else class="mb-2 text-gray-500">
          审批意见为可选，填写后将记录在审批日志中
        </div>
        <Input.TextArea
          v-model:value="approvalComment"
          :rows="4"
          :placeholder="
            approvalAction === 'approve'
              ? '请输入审批意见（可选）'
              : '请输入驳回原因（必填）'
          "
          :maxlength="500"
          show-count
        />
      </div>
    </Modal>
  </Drawer>
</template>
