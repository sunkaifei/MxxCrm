<script lang="ts" setup>
import { computed, h, ref } from 'vue';

import { Page } from '@vben/common-ui';

import { Button, Card, Select, Table, Tag } from 'ant-design-vue';
import type { TableColumnsType } from 'ant-design-vue';

import {
  getContractListApi,
  getContractPaymentPlanApi,
} from '#/api';
import { $t } from '#/locales';

interface PlanRow {
  id: number;
  contractId: number | null;
  contractNo: string;
  customerName: string;
  stageName: string;
  paymentType: number | null;
  planAmount: number;
  receivedAmount: number;
  unappliedAmount: number;
  planDate: string;
  actualDate: string;
  status: number | null;
  remark: string;
}

const loading = ref(false);
const planList = ref<PlanRow[]>([]);
const contractOptions = ref<any[]>([]);
const selectedContractId = ref<number | undefined>(undefined);

const paymentTypeMap: Record<number, { label: string; color: string }> = {
  1: { label: '预付款', color: 'blue' },
  2: { label: '进度款', color: 'cyan' },
  3: { label: '到货款', color: 'orange' },
  4: { label: '验收款', color: 'purple' },
  5: { label: '质保金', color: 'geekblue' },
  6: { label: '尾款', color: 'magenta' },
};

const statusMap: Record<number, { label: string; color: string }> = {
  0: { label: '未开始', color: 'default' },
  1: { label: '部分回款', color: 'orange' },
  4: { label: '已完成', color: 'green' },
};

const summary = computed(() => {
  const totalPlan = planList.value.reduce(
    (sum, row) => sum + Number(row.planAmount || 0),
    0,
  );
  const totalReceived = planList.value.reduce(
    (sum, row) => sum + Number(row.receivedAmount || 0),
    0,
  );
  const totalUnapplied = planList.value.reduce(
    (sum, row) => sum + Number(row.unappliedAmount || 0),
    0,
  );
  return { totalPlan, totalReceived, totalUnapplied };
});

async function loadContractOptions() {
  try {
    loading.value = true;
    const result: any = await getContractListApi({
      page: 1,
      pageSize: 1000,
    });
    const list = result?.data?.items || result?.items || result?.list || [];
    contractOptions.value = list.map((item: any) => ({
      value: item.id,
      label: item.contractNo || item.title || `合同#${item.id}`,
      raw: item,
    }));
  } catch (e) {
    console.error('加载合同列表失败:', e);
  } finally {
    loading.value = false;
  }
}

async function loadPlans(contractId: number) {
  loading.value = true;
  try {
    const result: any = await getContractPaymentPlanApi(contractId);
    const rawList = result?.data || result?.items || result || [];
    const contractInfo = contractOptions.value.find(
      (c) => c.value === contractId,
    );
    const contractNo = contractInfo?.raw?.contractNo || `合同#${contractId}`;
    const customerName =
      contractInfo?.raw?.customerName || contractInfo?.raw?.partyA || '';

    planList.value = (Array.isArray(rawList) ? rawList : []).map(
      (p: any) => ({
        id: Number(p.id),
        contractId: p.contractId ?? contractId,
        contractNo,
        customerName,
        stageName: p.stageName || '',
        paymentType: p.paymentType ?? null,
        planAmount: Number(p.planAmount || 0),
        receivedAmount: Number(p.receivedAmount || 0),
        unappliedAmount:
          Number(p.planAmount || 0) - Number(p.receivedAmount || 0),
        planDate: p.planDate || '',
        actualDate: p.actualDate || '',
        status: p.status ?? null,
        remark: p.remark || '',
      }),
    );
  } catch (e) {
    console.error('加载回款计划失败:', e);
    planList.value = [];
  } finally {
    loading.value = false;
  }
}

function handleContractChange(value: number | undefined) {
  selectedContractId.value = value;
  if (value) {
    loadPlans(value);
  } else {
    planList.value = [];
  }
}

const columns: TableColumnsType = [
  { title: $t('ui.table.seq'), dataIndex: 'seq', width: 70, customRender: ({ index }: any) => index + 1 },
  { title: '合同编号', dataIndex: 'contractNo', width: 160 },
  { title: '客户名称', dataIndex: 'customerName', width: 140 },
  { title: '期次名称', dataIndex: 'stageName', width: 140 },
  {
    title: '款项类型',
    dataIndex: 'paymentType',
    width: 100,
    customRender: ({ text }: any) => {
      const item = text != null ? paymentTypeMap[text] : undefined;
      if (item) {
        return h(
          Tag,
          { color: item.color },
          { default: () => item.label },
        );
      }
      return h('span', { class: 'text-gray-300' }, '-');
    },
  },
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
    customRender: ({ text }: any) => {
      const val = Number(text || 0);
      const cls = val > 0 ? 'text-orange-600 font-medium' : 'text-gray-400';
      return h(
        'span',
        { class: cls },
        `¥${val.toLocaleString('zh-CN', {
          minimumFractionDigits: 2,
          maximumFractionDigits: 2,
        })}`,
      );
    },
  },
  { title: '计划日期', dataIndex: 'planDate', width: 120 },
  { title: '实际日期', dataIndex: 'actualDate', width: 120 },
  {
    title: '状态',
    dataIndex: 'status',
    width: 100,
    customRender: ({ text }: any) => {
      const item = text != null ? statusMap[text] : undefined;
      if (item) {
        return h(
          Tag,
          { color: item.color },
          { default: () => item.label },
        );
      }
      return h('span', { class: 'text-gray-300' }, '-');
    },
  },
  { title: '备注', dataIndex: 'remark', width: 200 },
];

// 初始化加载合同选项
loadContractOptions();
</script>

<template>
  <Page>
    <Card class="mb-3" :bordered="false" size="small">
      <div class="flex items-center gap-3 flex-wrap">
        <span class="text-sm font-medium">选择合同：</span>
        <Select
          v-model:value="selectedContractId"
          :options="contractOptions"
          :loading="loading"
          show-search
          allow-clear
          placeholder="请选择合同查看回款计划"
          style="width: 320px"
          :filter-option="
            (input: string, option: any) =>
              String(option?.label ?? '')
                .toLowerCase()
                .includes(input.toLowerCase())
          "
          @change="(value: any) => handleContractChange(value ?? undefined)"
        />
        <Button
          v-if="selectedContractId"
          type="link"
          @click="() => loadPlans(selectedContractId!)"
        >
          刷新
        </Button>
        <span class="text-xs text-gray-400 ml-2">
          说明：当前后端按合同查询回款计划，需先选择合同。
        </span>
      </div>
    </Card>

    <Card :bordered="false">
      <div class="mb-3 flex items-center justify-between">
        <span class="font-medium">
          回款计划列表
          <span v-if="planList.length > 0" class="text-gray-400 text-xs ml-2">
            （共 {{ planList.length }} 条）
          </span>
        </span>
      </div>

      <Table
        :columns="columns"
        :data-source="planList"
        :loading="loading"
        :pagination="false"
        bordered
        size="small"
        :row-key="(record: any) => record.id"
        :scroll="{ x: 1500 }"
      >
        <template #emptyText>
          <span class="text-gray-400">
            {{ selectedContractId ? '该合同暂无回款计划' : '请先选择合同以查看回款计划' }}
          </span>
        </template>
      </Table>

      <!-- 底部汇总 -->
      <div
        v-if="planList.length > 0"
        class="mt-4 grid grid-cols-3 gap-3"
      >
        <div class="rounded-lg bg-blue-50 p-3">
          <div class="text-xs text-gray-500">总计划金额</div>
          <div class="text-lg font-bold text-blue-600 mt-1">
            ¥{{ summary.totalPlan.toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 }) }}
          </div>
        </div>
        <div class="rounded-lg bg-green-50 p-3">
          <div class="text-xs text-gray-500">总已收金额</div>
          <div class="text-lg font-bold text-green-600 mt-1">
            ¥{{ summary.totalReceived.toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 }) }}
          </div>
        </div>
        <div class="rounded-lg bg-orange-50 p-3">
          <div class="text-xs text-gray-500">总未收金额</div>
          <div class="text-lg font-bold text-orange-600 mt-1">
            ¥{{ summary.totalUnapplied.toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 }) }}
          </div>
        </div>
      </div>
    </Card>
  </Page>
</template>
