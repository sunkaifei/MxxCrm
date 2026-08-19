<script lang="ts" setup>
import type { TableColumnsType } from 'ant-design-vue';

import { computed, ref } from 'vue';

import {
  Button,
  InputNumber,
  message,
  Popconfirm,
  Table,
  Tag,
} from 'ant-design-vue';

import { useVbenDrawer } from '#/adapter/drawer';
import {
  applyPaymentApi,
  cancelPaymentApplicationApi,
  getPaymentApplicationsApi,
  getPaymentUnappliedApi,
} from '#/api/core/sale/payment';

interface PlanRow {
  id: null | number;
  stageName: string;
  planAmount: number;
  receivedAmount: number;
  unappliedAmount: number;
  applyAmount: number;
}

interface ApplicationRow {
  id: number;
  planId: null | number;
  applyAmount: number;
  createTime: string;
}

const loading = ref(false);
const submitting = ref(false);

// 回款基本信息
const paymentInfo = ref<{
  amount: number;
  appliedAmount: number;
  contractId: null | number;
  paymentId: null | number;
  unappliedAmount: number;
}>({
  paymentId: null,
  amount: 0,
  appliedAmount: 0,
  unappliedAmount: 0,
  contractId: null,
});

// 可核销计划列表
const planRows = ref<PlanRow[]>([]);

// 已有核销明细
const applications = ref<ApplicationRow[]>([]);

const totalApplyAmount = computed(() =>
  planRows.value.reduce((sum, row) => sum + Number(row.applyAmount || 0), 0),
);

const remainingAfterApply = computed(
  () => paymentInfo.value.unappliedAmount - totalApplyAmount.value,
);

const planColumns: TableColumnsType = [
  { title: '期次名称', dataIndex: 'stageName', width: 160 },
  {
    title: '计划金额',
    dataIndex: 'planAmount',
    width: 120,
    align: 'right',
    customRender: ({ text }: any) =>
      `¥${Number(text || 0).toLocaleString('zh-CN', {
        minimumFractionDigits: 2,
        maximumFractionDigits: 2,
      })}`,
  },
  {
    title: '已收金额',
    dataIndex: 'receivedAmount',
    width: 120,
    align: 'right',
    customRender: ({ text }: any) =>
      `¥${Number(text || 0).toLocaleString('zh-CN', {
        minimumFractionDigits: 2,
        maximumFractionDigits: 2,
      })}`,
  },
  {
    title: '未收金额',
    dataIndex: 'unappliedAmount',
    width: 120,
    align: 'right',
    customRender: ({ text }: any) =>
      `¥${Number(text || 0).toLocaleString('zh-CN', {
        minimumFractionDigits: 2,
        maximumFractionDigits: 2,
      })}`,
  },
  {
    title: '本次核销金额',
    dataIndex: 'applyAmount',
    width: 160,
  },
];

const applicationColumns: TableColumnsType = [
  {
    title: '核销ID',
    dataIndex: 'id',
    width: 80,
  },
  {
    title: '计划ID',
    dataIndex: 'planId',
    width: 100,
    customRender: ({ text }: any) => text || '-',
  },
  {
    title: '核销金额',
    dataIndex: 'applyAmount',
    width: 140,
    align: 'right',
    customRender: ({ text }: any) =>
      `¥${Number(text || 0).toLocaleString('zh-CN', {
        minimumFractionDigits: 2,
        maximumFractionDigits: 2,
      })}`,
  },
  {
    title: '核销时间',
    dataIndex: 'createTime',
    width: 180,
  },
  {
    title: '操作',
    dataIndex: 'action',
    width: 100,
  },
];

async function loadData(paymentId: number) {
  loading.value = true;
  try {
    const [unappliedRes, appRes]: any = await Promise.all([
      getPaymentUnappliedApi(paymentId),
      getPaymentApplicationsApi(paymentId),
    ]);

    const unapplied = unappliedRes?.data ?? unappliedRes ?? {};
    paymentInfo.value = {
      paymentId: Number(unapplied.paymentId ?? paymentId),
      amount: Number(unapplied.amount ?? 0),
      appliedAmount: Number(unapplied.appliedAmount ?? 0),
      unappliedAmount: Number(unapplied.unappliedAmount ?? 0),
      contractId: unapplied.contractId ?? null,
    };

    const plans = unapplied.plans ?? [];
    planRows.value = plans.map((p: any) => ({
      id: p.id ?? null,
      stageName: p.stageName ?? '',
      planAmount: Number(p.planAmount ?? 0),
      receivedAmount: Number(p.receivedAmount ?? 0),
      unappliedAmount: Number(p.unappliedAmount ?? 0),
      applyAmount: 0,
    }));

    const appList = appRes?.data ?? appRes ?? [];
    applications.value = (Array.isArray(appList) ? appList : []).map(
      (a: any) => ({
        id: Number(a.id),
        planId: a.planId ?? null,
        applyAmount: Number(a.applyAmount ?? 0),
        createTime: a.createTime ?? '',
      }),
    );
  } catch (error) {
    console.error('加载核销数据失败:', error);
  } finally {
    loading.value = false;
  }
}

async function handleApply() {
  if (!paymentInfo.value.paymentId) {
    message.warning('回款ID无效');
    return;
  }

  const validItems = planRows.value.filter(
    (row) =>
      row.id !== null && row.id !== undefined && Number(row.applyAmount) > 0,
  );

  if (validItems.length === 0) {
    message.warning('请填写至少一条核销明细');
    return;
  }

  // 单条核销金额 ≤ 计划未收金额
  for (const item of validItems) {
    if (Number(item.applyAmount) > Number(item.unappliedAmount)) {
      message.warning(
        `期次「${item.stageName}」核销金额不能超过未收金额 ¥${item.unappliedAmount.toFixed(2)}`,
      );
      return;
    }
  }

  // 核销总额 ≤ 未核销金额
  if (totalApplyAmount.value > paymentInfo.value.unappliedAmount) {
    message.warning(
      `核销总额 ¥${totalApplyAmount.value.toFixed(2)} 超过未核销金额 ¥${paymentInfo.value.unappliedAmount.toFixed(2)}`,
    );
    return;
  }

  submitting.value = true;
  try {
    await applyPaymentApi({
      paymentId: paymentInfo.value.paymentId,
      applications: validItems.map((row) => ({
        planId: row.id,
        applyAmount: Number(row.applyAmount),
      })),
    });
    message.success('核销成功');
    drawerApi.setData({ needRefresh: true });
    drawerApi.close();
  } catch {
    message.error('核销失败');
  } finally {
    submitting.value = false;
  }
}

async function handleCancelApplication(id: number) {
  try {
    await cancelPaymentApplicationApi(id);
    message.success('取消核销成功');
    if (paymentInfo.value.paymentId) {
      await loadData(paymentInfo.value.paymentId);
    }
    drawerApi.setData({ needRefresh: true });
  } catch {
    message.error('取消核销失败');
  }
}

const [Drawer, drawerApi] = useVbenDrawer({
  onCancel() {
    drawerApi.close();
  },
  async onConfirm() {
    await handleApply();
  },
  onOpenChange(isOpen) {
    if (isOpen) {
      const data = drawerApi.getData<{ row?: any }>();
      const paymentId = data?.row?.id;
      if (paymentId) {
        loadData(paymentId);
      }
    } else {
      // 关闭时重置
      planRows.value = [];
      applications.value = [];
      paymentInfo.value = {
        paymentId: null,
        amount: 0,
        appliedAmount: 0,
        unappliedAmount: 0,
        contractId: null,
      };
    }
  },
});
</script>

<template>
  <Drawer
    title="回款核销"
    :destroy-on-close="true"
    :width="960"
    :z-index="2000"
  >
    <div class="space-y-4">
      <!-- 回款金额概览 -->
      <div class="grid grid-cols-4 gap-3">
        <div class="rounded-lg bg-blue-50 p-3">
          <div class="text-xs text-gray-500">回款总金额</div>
          <div class="text-lg font-bold text-blue-600 mt-1">
            ¥{{
              paymentInfo.amount.toLocaleString('zh-CN', {
                minimumFractionDigits: 2,
                maximumFractionDigits: 2,
              })
            }}
          </div>
        </div>
        <div class="rounded-lg bg-green-50 p-3">
          <div class="text-xs text-gray-500">已核销金额</div>
          <div class="text-lg font-bold text-green-600 mt-1">
            ¥{{
              paymentInfo.appliedAmount.toLocaleString('zh-CN', {
                minimumFractionDigits: 2,
                maximumFractionDigits: 2,
              })
            }}
          </div>
        </div>
        <div class="rounded-lg bg-orange-50 p-3">
          <div class="text-xs text-gray-500">未核销金额</div>
          <div class="text-lg font-bold text-orange-600 mt-1">
            ¥{{
              paymentInfo.unappliedAmount.toLocaleString('zh-CN', {
                minimumFractionDigits: 2,
                maximumFractionDigits: 2,
              })
            }}
          </div>
        </div>
        <div class="rounded-lg bg-purple-50 p-3">
          <div class="text-xs text-gray-500">本次核销总额</div>
          <div class="text-lg font-bold text-purple-600 mt-1">
            ¥{{
              totalApplyAmount.toLocaleString('zh-CN', {
                minimumFractionDigits: 2,
                maximumFractionDigits: 2,
              })
            }}
          </div>
        </div>
      </div>

      <!-- 可核销计划表格 -->
      <div>
        <div class="flex items-center justify-between mb-2">
          <span class="font-medium">可核销计划</span>
          <Tag v-if="paymentInfo.contractId" color="blue">
            合同ID: {{ paymentInfo.contractId }}
          </Tag>
          <Tag v-else color="default"> 未关联合同，无可核销计划 </Tag>
        </div>
        <Table
          :columns="planColumns"
          :data-source="planRows"
          :loading="loading"
          :pagination="false"
          bordered
          size="small"
          :row-key="(record: any) => record.id ?? record.stageName"
          :scroll="{ x: 700 }"
        >
          <template #bodyCell="{ column, record, index }">
            <template v-if="column.dataIndex === 'applyAmount'">
              <InputNumber
                :value="record.applyAmount"
                :min="0"
                :max="record.unappliedAmount"
                :precision="2"
                style="width: 100%"
                placeholder="请输入核销金额"
                @update:value="
                  (val: any) => {
                    planRows[index]!.applyAmount = Number(val || 0);
                  }
                "
              />
            </template>
          </template>
          <template #emptyText>
            <span class="text-gray-400"
              >暂无可核销的计划（未关联合同或所有计划已核销完毕）</span
            >
          </template>
        </Table>
      </div>

      <!-- 核销汇总 -->
      <div
        v-if="planRows.length > 0"
        class="rounded-lg bg-gray-50 p-3 flex items-center justify-between"
      >
        <div class="text-sm text-gray-600">
          核销后剩余未核销金额：
          <span
            :class="
              remainingAfterApply < 0
                ? 'text-red-600 font-bold'
                : 'text-blue-600 font-bold'
            "
          >
            ¥{{
              remainingAfterApply.toLocaleString('zh-CN', {
                minimumFractionDigits: 2,
                maximumFractionDigits: 2,
              })
            }}
          </span>
        </div>
        <div v-if="remainingAfterApply < 0" class="text-xs text-red-500">
          核销总额超过未核销金额，请调整
        </div>
      </div>

      <!-- 已有核销明细 -->
      <div>
        <div class="font-medium mb-2">已有核销明细</div>
        <Table
          :columns="applicationColumns"
          :data-source="applications"
          :pagination="false"
          bordered
          size="small"
          :row-key="(record: any) => record.id"
        >
          <template #bodyCell="{ column, record }">
            <template v-if="column.dataIndex === 'action'">
              <Popconfirm
                title="确定要取消该核销记录吗？取消后将回滚相关金额。"
                ok-text="确认"
                cancel-text="取消"
                @confirm="() => handleCancelApplication(record.id)"
              >
                <Button type="link" danger size="small">取消核销</Button>
              </Popconfirm>
            </template>
          </template>
          <template #emptyText>
            <span class="text-gray-400">暂无核销记录</span>
          </template>
        </Table>
      </div>
    </div>
  </Drawer>
</template>
