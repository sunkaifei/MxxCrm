<script setup lang="ts">
import { ref, reactive, onMounted, h } from 'vue';
import {
  Button, Table, Tag, Modal, message, Select, InputNumber, Input,
  Space, Descriptions, Timeline, Form,
} from 'ant-design-vue';
import type { TableColumnsType } from 'ant-design-vue/es/table';
import {
  getPlanListApi, createPlanApi, submitPlanApi,
  approvePlanApi, rejectPlanApi, modifyPlanApi,
  getPlanDetailApi, getPlanModifyDetailApi,
} from '#/api/core/statistics';

// ---- Status Config ----
const STATUS_MAP: Record<number, { text: string; color: string }> = {
  0: { text: '草稿', color: 'default' },
  1: { text: '待审批', color: 'orange' },
  2: { text: '已通过', color: 'green' },
  3: { text: '已驳回', color: 'red' },
};

// ---- Reactive State ----
const loading = ref(false);
const plans = ref<any[]>([]);
const currentYear = new Date().getFullYear();

const filters = reactive({
  year: currentYear,
  status: undefined as number | undefined,
});

// Create Modal
const createVisible = ref(false);
const createLoading = ref(false);
const createForm = reactive({
  year: currentYear + 1,
  monthlyTargets: Array.from({ length: 12 }, (_, i) => ({
    month: i + 1,
    contractTargetAmount: 0,
    paymentTargetAmount: 0,
    contractTargetCount: 0,
  })),
});

// Detail Modal
const detailVisible = ref(false);
const detailData = ref<any>(null);

// Approve/Reject Modal
const reviewVisible = ref(false);
const reviewAction = ref<'approve' | 'reject'>('approve');
const reviewForm = reactive({ planId: 0, reason: '' });
const reviewLoading = ref(false);

// Modify Modal
const modifyVisible = ref(false);
const modifyLoading = ref(false);
const modifyForm = reactive({
  planId: 0,
  reason: '',
  monthlyTargets: Array.from({ length: 12 }, (_, i) => ({
    month: i + 1,
    contractTargetAmount: 0,
    paymentTargetAmount: 0,
    contractTargetCount: 0,
  })),
});

// ---- Methods ----
const loadData = async () => {
  loading.value = true;
  try {
    const res = await getPlanListApi({
      year: filters.year,
      status: filters.status,
    });
    plans.value = res.data?.data || [];
  } catch (e: any) {
    console.error('加载计划列表失败', e);
  } finally {
    loading.value = false;
  }
};

const openCreateModal = () => {
  createForm.year = currentYear + 1;
  createForm.monthlyTargets = Array.from({ length: 12 }, (_, i) => ({
    month: i + 1,
    contractTargetAmount: 0,
    paymentTargetAmount: 0,
    contractTargetCount: 0,
  }));
  createVisible.value = true;
};

const handleCreate = async () => {
  createLoading.value = true;
  try {
    await createPlanApi({
      year: createForm.year,
      monthlyTargets: createForm.monthlyTargets,
    });
    message.success('创建成功');
    createVisible.value = false;
    loadData();
  } catch (e: any) {
    message.error(e?.message || '创建失败');
  } finally {
    createLoading.value = false;
  }
};

const handleSubmit = async (planId: number) => {
  Modal.confirm({
    title: '确认提交',
    content: '提交后将进入审批流程，且不可直接修改。确定提交吗？',
    onOk: async () => {
      try {
        await submitPlanApi(planId);
        message.success('提交成功，等待审批');
        loadData();
      } catch (e: any) {
        message.error(e?.message || '提交失败');
      }
    },
  });
};

const openDetail = async (planId: number) => {
  try {
    const res = await getPlanDetailApi(planId);
    detailData.value = res.data?.data;
    detailVisible.value = true;
  } catch (e: any) {
    message.error(e?.message || '加载详情失败');
  }
};

const openReview = (planId: number, action: 'approve' | 'reject') => {
  reviewAction.value = action;
  reviewForm.planId = planId;
  reviewForm.reason = '';
  reviewVisible.value = true;
};

const handleReview = async () => {
  reviewLoading.value = true;
  try {
    if (reviewAction.value === 'approve') {
      await approvePlanApi(reviewForm.planId, reviewForm.reason || undefined);
      message.success('已审批通过');
    } else {
      await rejectPlanApi(reviewForm.planId, reviewForm.reason || undefined);
      message.success('已驳回');
    }
    reviewVisible.value = false;
    loadData();
  } catch (e: any) {
    message.error(e?.message || '操作失败');
  } finally {
    reviewLoading.value = false;
  }
};

const openModify = async (planId: number) => {
  try {
    const res = await getPlanModifyDetailApi(planId);
    const detail = res.data?.data;
    if (detail?.monthlyTargets) {
      modifyForm.monthlyTargets = detail.monthlyTargets;
    }
    modifyForm.planId = planId;
    modifyForm.reason = '';
    modifyVisible.value = true;
  } catch (e: any) {
    message.error(e?.message || '加载修改数据失败');
  }
};

const handleModify = async () => {
  if (!modifyForm.reason.trim()) {
    message.warning('请填写申请修改理由');
    return;
  }
  modifyLoading.value = true;
  try {
    await modifyPlanApi({
      planId: modifyForm.planId,
      reason: modifyForm.reason,
      monthlyTargets: modifyForm.monthlyTargets,
    });
    message.success('修改申请已提交，等待审批');
    modifyVisible.value = false;
    loadData();
  } catch (e: any) {
    message.error(e?.message || '提交失败');
  } finally {
    modifyLoading.value = false;
  }
};

const ACTION_LABEL: Record<number, string> = {
  1: '提交申请',
  2: '审批通过',
  3: '驳回',
  4: '申请修改',
};

const STATUS_LABEL: Record<number, string> = {
  0: '草稿',
  1: '待审批',
  2: '已通过',
  3: '已驳回',
};

// ---- Table Columns ----
const columns: TableColumnsType = [
  { title: 'ID', dataIndex: 'id', width: 60 },
  { title: '员工', dataIndex: 'employeeName', width: 100 },
  { title: '年份', dataIndex: 'year', width: 70 },
  {
    title: '合同总额', dataIndex: 'totalContractTarget', width: 120,
    customRender: ({ text }: any) => text ? `¥${text.toLocaleString()}` : '-',
  },
  {
    title: '回款总额', dataIndex: 'totalPaymentTarget', width: 120,
    customRender: ({ text }: any) => text ? `¥${text.toLocaleString()}` : '-',
  },
  {
    title: '状态', dataIndex: 'status', width: 80,
    customRender: ({ text }: any) => {
      const cfg = STATUS_MAP[text as number] || { text: '未知', color: 'default' };
      return h(Tag, { color: cfg.color }, { default: () => cfg.text });
    },
  },
  { title: '版本', dataIndex: 'version', width: 60 },
  {
    title: '操作', key: 'actions', width: 260,
    customRender: ({ record }: any) => {
      const btns: any[] = [];
      if (record.status === 0 || record.status === 3) {
        btns.push(h(Button, { type: 'link', onClick: () => handleSubmit(record.id) }, { default: () => '提交' }));
      }
      btns.push(h(Button, { type: 'link', onClick: () => openDetail(record.id) }, { default: () => '详情' }));
      if (record.status === 1) {
        btns.push(h(Button, { type: 'link', style: { color: '#52c41a' }, onClick: () => openReview(record.id, 'approve') }, { default: () => '通过' }));
        btns.push(h(Button, { type: 'link', danger: true, onClick: () => openReview(record.id, 'reject') }, { default: () => '驳回' }));
      }
      if (record.status === 2) {
        btns.push(h(Button, { type: 'link', onClick: () => openModify(record.id) }, { default: () => '申请修改' }));
      }
      return h(Space, null, { default: () => btns });
    },
  },
];

// ---- Init ----
onMounted(() => {
  loadData();
});
</script>

<template>
  <div>
    <!-- Filter Bar -->
    <div class="mb-4 flex items-center justify-between">
      <Space>
        <span>年份：</span>
        <Select v-model:value="filters.year" style="width: 100px" @change="loadData">
          <SelectOption v-for="y in [currentYear - 1, currentYear, currentYear + 1]" :key="y" :value="y">{{ y }}</SelectOption>
        </Select>
        <span>状态：</span>
        <Select v-model:value="filters.status" style="width: 120px" allowClear @change="loadData">
          <SelectOption :value="0">草稿</SelectOption>
          <SelectOption :value="1">待审批</SelectOption>
          <SelectOption :value="2">已通过</SelectOption>
          <SelectOption :value="3">已驳回</SelectOption>
        </Select>
      </Space>
      <Button type="primary" @click="openCreateModal">+ 新申报</Button>
    </div>

    <!-- Table -->
    <Table
      :columns="columns"
      :data-source="plans"
      :loading="loading"
      row-key="id"
      :pagination="{ pageSize: 20 }"
      bordered
      size="middle"
    />

    <!-- Create Modal -->
    <Modal
      v-model:visible="createVisible"
      title="业绩目标计划申报"
      width="800px"
      :footer="null"
      destroy-on-close
    >
      <Form layout="vertical">
        <Form.Item label="目标年份" required>
          <Select v-model:value="createForm.year" style="width: 120px">
            <SelectOption v-for="y in [currentYear + 1, currentYear + 2]" :key="y" :value="y">{{ y }}</SelectOption>
          </Select>
        </Form.Item>
        <Form.Item label="月度目标明细" required>
          <table class="w-full border-collapse">
            <thead>
              <tr class="bg-gray-50">
                <th class="border px-2 py-1 text-left">月份</th>
                <th class="border px-2 py-1 text-right">合同金额目标(元)</th>
                <th class="border px-2 py-1 text-right">回款金额目标(元)</th>
                <th class="border px-2 py-1 text-right">合同数量目标</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(mt, idx) in createForm.monthlyTargets" :key="mt.month">
                <td class="border px-2 py-1">{{ mt.month }}月</td>
                <td class="border px-2 py-1">
                  <InputNumber v-model:value="mt.contractTargetAmount" :min="0" :precision="2" style="width: 100%" />
                </td>
                <td class="border px-2 py-1">
                  <InputNumber v-model:value="mt.paymentTargetAmount" :min="0" :precision="2" style="width: 100%" />
                </td>
                <td class="border px-2 py-1">
                  <InputNumber v-model:value="mt.contractTargetCount" :min="0" :precision="0" style="width: 100%" />
                </td>
              </tr>
            </tbody>
          </table>
        </Form.Item>
        <div class="flex justify-end gap-3">
          <Button @click="createVisible = false">取消</Button>
          <Button type="primary" :loading="createLoading" @click="handleCreate">保存草稿</Button>
        </div>
      </Form>
    </Modal>

    <!-- Detail Modal -->
    <Modal
      v-model:visible="detailVisible"
      title="计划详情"
      width="700px"
      :footer="null"
      destroy-on-close
    >
      <template v-if="detailData">
        <Descriptions bordered size="small" :column="2" class="mb-4">
          <Descriptions.Item label="员工">{{ detailData.employeeName || '-' }}</Descriptions.Item>
          <Descriptions.Item label="年份">{{ detailData.year }}</Descriptions.Item>
          <Descriptions.Item label="状态">
            <Tag :color="STATUS_MAP[detailData.status]?.color">{{ STATUS_MAP[detailData.status]?.text }}</Tag>
          </Descriptions.Item>
          <Descriptions.Item label="版本">v{{ detailData.version }}</Descriptions.Item>
          <Descriptions.Item v-if="detailData.applyReason" label="申请理由" :span="2">{{ detailData.applyReason }}</Descriptions.Item>
        </Descriptions>

        <h4 class="mb-2 font-medium">月度目标明细</h4>
        <Table
          :data-source="detailData.monthlyTargets || []"
          :columns="[
            { title: '月份', dataIndex: 'month', width: 60 },
            { title: '合同金额', dataIndex: 'contractTargetAmount', customRender: ({ text }: any) => text ? `¥${text.toLocaleString()}` : '-' },
            { title: '回款金额', dataIndex: 'paymentTargetAmount', customRender: ({ text }: any) => text ? `¥${text.toLocaleString()}` : '-' },
            { title: '合同数量', dataIndex: 'contractTargetCount' },
          ]"
          row-key="month"
          :pagination="false"
          size="small"
          bordered
          class="mb-4"
        />

        <h4 class="mb-2 font-medium">审批记录</h4>
        <Timeline v-if="detailData.approvalLogs?.length">
          <TimelineItem v-for="log in detailData.approvalLogs" :key="log.id">
            <template #dot>
              <span :class="log.action === 2 ? 'text-green-500' : log.action === 3 ? 'text-red-500' : 'text-blue-500'">
                ●
              </span>
            </template>
            <div>
              <strong>{{ log.operatorName }}</strong>
              <span class="mx-1 text-gray-400">{{ ACTION_LABEL[log.action] || '操作' }}</span>
              <Tag class="ml-1">{{ STATUS_LABEL[log.previousStatus] }}</Tag>
              <ArrowRightOutlined class="mx-1 text-gray-400" />
              <Tag>{{ STATUS_LABEL[log.newStatus] }}</Tag>
              <div v-if="log.reason" class="text-gray-500 text-sm mt-1">原因：{{ log.reason }}</div>
              <div class="text-gray-400 text-xs mt-1">{{ log.createTime }}</div>
            </div>
          </TimelineItem>
        </Timeline>
        <div v-else class="text-gray-400">暂无审批记录</div>
      </template>
    </Modal>

    <!-- Approve/Reject Modal -->
    <Modal
      v-model:visible="reviewVisible"
      :title="reviewAction === 'approve' ? '审批通过' : '驳回'"
      :on-ok="handleReview"
      :confirm-loading="reviewLoading"
      ok-text="确认"
      cancel-text="取消"
    >
      <Form layout="vertical">
        <Form.Item :label="reviewAction === 'approve' ? '审批意见（可选）' : '驳回原因'">
          <Input.TextArea
            v-model:value="reviewForm.reason"
            :placeholder="reviewAction === 'approve' ? '可选填写审批意见' : '请填写驳回原因'"
            :rows="3"
          />
        </Form.Item>
      </Form>
    </Modal>

    <!-- Modify Modal -->
    <Modal
      v-model:visible="modifyVisible"
      title="申请修改年度目标计划"
      width="800px"
      :footer="null"
      destroy-on-close
    >
      <Form layout="vertical">
        <Form.Item label="申请修改理由" required>
          <Input.TextArea v-model:value="modifyForm.reason" placeholder="请详细说明修改原因" :rows="3" />
        </Form.Item>
        <Form.Item label="修改后的月度目标">
          <table class="w-full border-collapse">
            <thead>
              <tr class="bg-gray-50">
                <th class="border px-2 py-1 text-left">月份</th>
                <th class="border px-2 py-1 text-right">合同金额目标(元)</th>
                <th class="border px-2 py-1 text-right">回款金额目标(元)</th>
                <th class="border px-2 py-1 text-right">合同数量目标</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(mt, idx) in modifyForm.monthlyTargets" :key="mt.month">
                <td class="border px-2 py-1">{{ mt.month }}月</td>
                <td class="border px-2 py-1">
                  <InputNumber v-model:value="mt.contractTargetAmount" :min="0" :precision="2" style="width: 100%" />
                </td>
                <td class="border px-2 py-1">
                  <InputNumber v-model:value="mt.paymentTargetAmount" :min="0" :precision="2" style="width: 100%" />
                </td>
                <td class="border px-2 py-1">
                  <InputNumber v-model:value="mt.contractTargetCount" :min="0" :precision="0" style="width: 100%" />
                </td>
              </tr>
            </tbody>
          </table>
        </Form.Item>
        <div class="flex justify-end gap-3">
          <Button @click="modifyVisible = false">取消</Button>
          <Button type="primary" :loading="modifyLoading" @click="handleModify">提交修改申请</Button>
        </div>
      </Form>
    </Modal>
  </div>
</template>