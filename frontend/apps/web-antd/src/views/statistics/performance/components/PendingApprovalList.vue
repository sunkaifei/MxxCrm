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
import { $t } from '#/locales';

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
      pageSize: 200,
    });
    pendingList.value = res?.items || [];
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
    message.error(error?.message || $t('page.statistics.performancePlan.loadDetailFailed'));
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
    message.warning(
      $t('page.statistics.performancePlan.rejectReasonRequired'),
    );
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
      message.success($t('page.statistics.performancePlan.approveSuccess'));
    } else {
      await rejectPlanApi(params.planId, params.reason);
      message.success($t('page.statistics.performancePlan.rejected'));
    }
    approvalModalVisible.value = false;
    detailVisible.value = false;
    await loadPendingList();
    emit('refresh');
  } catch (error: any) {
    message.error(
      error?.message || $t('page.statistics.performancePlan.operationFailed'),
    );
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
  if (num >= 10_000)
    return `¥${(num / 10_000).toFixed(2)}${$t('page.statistics.performancePlan.wanUnit')}`;
  return `¥${num.toLocaleString()}`;
}

const tableColumns = computed(() => [
  {
    title: $t('page.statistics.performancePlan.employee'),
    dataIndex: 'employeeName',
    key: 'employeeName',
    width: 120,
  },
  {
    title: $t('page.statistics.performancePlan.year'),
    dataIndex: 'year',
    key: 'year',
    width: 80,
  },
  {
    title: $t('page.statistics.performancePlan.totalContractTarget'),
    dataIndex: 'totalContractTarget',
    key: 'totalContractTarget',
    align: 'right' as const,
    customRender: ({ text }: any) => formatCurrency(text),
  },
  {
    title: $t('page.statistics.performancePlan.totalPaymentTarget'),
    dataIndex: 'totalPaymentTarget',
    key: 'totalPaymentTarget',
    align: 'right' as const,
    customRender: ({ text }: any) => formatCurrency(text),
  },
  {
    title: $t('page.statistics.performancePlan.approvalLevel'),
    key: 'level',
    width: 120,
    customRender: ({ record }: any) => {
      const cur = record.approvalLevel || 0;
      const total = record.totalLevels || 0;
      return $t('page.statistics.performancePlan.levelProgress', {
        cur,
        total,
      });
    },
  },
  {
    title: $t('page.statistics.performancePlan.submitTime'),
    dataIndex: 'submitTime',
    key: 'submitTime',
    width: 170,
    customRender: ({ text }: any) => text || '-',
  },
  {
    title: $t('page.statistics.performancePlan.operate'),
    key: 'action',
    width: 160,
    fixed: 'right' as const,
  },
]);

// 审批节点状态映射
const nodeStatusMap: Record<number, { color: string; text: string }> = {
  0: { color: 'processing', text: $t('page.statistics.performancePlan.toBeApproved') },
  1: { color: 'success', text: $t('page.statistics.performancePlan.approved') },
  2: { color: 'error', text: $t('page.statistics.performancePlan.rejected') },
  3: { color: 'default', text: $t('page.statistics.performancePlan.skipped') },
};

// 审批动作映射
const actionMap: Record<number, { color: string; text: string }> = {
  1: { color: 'blue', text: $t('page.statistics.performancePlan.submitted') },
  2: { color: 'green', text: $t('page.statistics.performancePlan.approve') },
  3: { color: 'red', text: $t('page.statistics.performancePlan.reject') },
  4: { color: 'orange', text: $t('page.statistics.performancePlan.applyModify') },
  5: { color: 'purple', text: $t('page.statistics.performancePlan.withdraw') },
};
</script>

<template>
  <Drawer
    :open="visible"
    :title="$t('page.statistics.performancePlan.pendingApprovalTitle')"
    width="1100px"
    :body-style="{ padding: '0' }"
    @close="emit('update:visible', false)"
  >
    <Spin :spinning="loading">
      <!-- 待审批列表 -->
      <div class="p-4">
        <div v-if="pendingList.length === 0 && !loading" class="py-12">
          <Empty
            :description="$t('page.statistics.performancePlan.noPendingPlans')"
          />
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
                  {{ $t('page.statistics.performancePlan.viewDetail') }}
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
      :title="$t('page.statistics.performancePlan.planDetail')"
      width="720px"
      :body-style="{ padding: '16px' }"
    >
      <Spin :spinning="detailLoading">
        <template v-if="currentPlanDetail">
          <!-- 基本信息 -->
          <Card
            size="small"
            class="mb-4"
            :title="$t('page.statistics.performancePlan.basicInfo')"
          >
            <div class="grid grid-cols-2 gap-3 text-sm">
              <div>
                <span class="text-gray-500">{{
                  $t('page.statistics.performancePlan.employee')
                }}：</span>
                <span class="font-medium">{{
                  currentPlanDetail.employeeName
                }}</span>
              </div>
              <div>
                <span class="text-gray-500">{{
                  $t('page.statistics.performancePlan.year')
                }}：</span>
                <span class="font-medium">{{
                  $t('page.statistics.performancePlan.yearValue', {
                    n: currentPlanDetail.year,
                  })
                }}</span>
              </div>
              <div>
                <span class="text-gray-500">{{
                  $t('page.statistics.performancePlan.status')
                }}：</span>
                <Tag color="processing">
{{
                  $t('page.statistics.performancePlan.toBeApproved')
                }}
</Tag>
              </div>
              <div>
                <span class="text-gray-500">{{
                  $t('page.statistics.performancePlan.version')
                }}：</span>
                <span class="font-medium"
                  >v{{ currentPlanDetail.version || 1 }}</span
                >
              </div>
              <div>
                <span class="text-gray-500">{{
                  $t('page.statistics.performancePlan.approvalLevel')
                }}：</span>
                <span class="font-medium">
                  {{
                    $t('page.statistics.performancePlan.levelProgress', {
                      cur: currentPlanDetail.approvalLevel,
                      total: currentPlanDetail.totalLevels,
                    })
                  }}
                </span>
              </div>
              <div>
                <span class="text-gray-500">{{ $t('page.statistics.performancePlan.submitTime') }}：</span>
                <span>{{ currentPlanDetail.submitTime || '-' }}</span>
              </div>
            </div>
          </Card>

          <!-- 月度目标 -->
          <Card size="small" class="mb-4" :title="$t('page.statistics.performancePlan.monthlyTargetDetail')">
            <Table
              :data-source="currentPlanDetail.monthlyTargets || []"
              :pagination="false"
              size="small"
              row-key="month"
            >
              <Table.Column
                :title="$t('page.statistics.performancePlan.month')"
                data-index="month"
                :width="80"
              >
                <template #default="{ text }">
{{
                  $t('page.statistics.performancePlan.monthValue', { n: text })
                }}
</template>
              </Table.Column>
              <Table.Column
                :title="$t('page.statistics.performancePlan.contractTargetAmount')"
                data-index="contractTargetAmount"
                align="right"
              >
                <template #default="{ text }">
                  {{ formatCurrency(text) }}
                </template>
              </Table.Column>
              <Table.Column
                :title="$t('page.statistics.performancePlan.paymentTargetAmount')"
                data-index="paymentTargetAmount"
                align="right"
              >
                <template #default="{ text }">
                  {{ formatCurrency(text) }}
                </template>
              </Table.Column>
              <Table.Column
                :title="$t('page.statistics.performancePlan.contractCount')"
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
            :title="$t('page.statistics.performancePlan.approvalChain')"
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
                  <span class="font-medium">{{
                    $t('page.statistics.performancePlan.levelValue', {
                      n: node.level,
                    })
                  }}</span>
                  <span>{{ node.approverName }}</span>
                  <Tag
                    v-if="nodeStatusMap[node.status as number]"
                    :color="nodeStatusMap[node.status as number]?.color"
                  >
                    {{ nodeStatusMap[node.status as number]?.text }}
                  </Tag>
                </div>
                <div v-if="node.comment" class="text-xs text-gray-500 mt-1">
                  {{ $t('page.statistics.performancePlan.comment') }}：{{
                    node.comment
                  }}
                </div>
              </TimelineItem>
            </Timeline>
          </Card>

          <!-- 审批记录 -->
          <Card
            v-if="currentPlanDetail.approvalLogs?.length"
            size="small"
            class="mb-4"
            :title="$t('page.statistics.performancePlan.approvalRecords')"
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
            <Button @click="detailVisible = false">
{{
              $t('page.statistics.performancePlan.close')
            }}
</Button>
            <Button danger @click="openApprovalModal('reject')">
              <IconifyIcon icon="lucide:x-circle" class="mr-1" />
              {{ $t('page.statistics.performancePlan.reject') }}
            </Button>
            <Button type="primary" @click="openApprovalModal('approve')">
              <IconifyIcon icon="lucide:check-circle" class="mr-1" />
              {{ $t('page.statistics.performancePlan.approve') }}
            </Button>
          </div>
        </template>
      </Spin>
    </Drawer>

    <!-- 审批弹窗 -->
    <Modal
      v-model:open="approvalModalVisible"
      :title="
        approvalAction === 'approve'
          ? $t('page.statistics.performancePlan.approvePlanTitle')
          : $t('page.statistics.performancePlan.rejectPlanTitle')
      "
      :confirm-loading="submitting"
      :ok-text="
        approvalAction === 'approve'
          ? $t('page.statistics.performancePlan.confirmApprove')
          : $t('page.statistics.performancePlan.confirmReject')
      "
      :ok-type="approvalAction === 'approve' ? 'primary' : 'danger'"
      @ok="submitApproval"
    >
      <div class="py-4">
        <div v-if="approvalAction === 'reject'" class="mb-2 text-red-500">
          <IconifyIcon icon="lucide:alert-triangle" class="mr-1" />
          {{ $t('page.statistics.performancePlan.rejectReasonTip') }}
        </div>
        <div v-else class="mb-2 text-gray-500">
          {{ $t('page.statistics.performancePlan.approveCommentTip') }}
        </div>
        <Input.TextArea
          v-model:value="approvalComment"
          :rows="4"
          :placeholder="
            approvalAction === 'approve'
              ? $t('page.statistics.performancePlan.approvePlaceholder')
              : $t('page.statistics.performancePlan.rejectPlaceholder')
          "
          :maxlength="500"
          show-count
        />
      </div>
    </Modal>
  </Drawer>
</template>
