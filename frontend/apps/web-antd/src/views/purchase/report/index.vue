<script lang="ts" setup>
import { ref } from 'vue';

import { Page } from '@vben/common-ui';

import { Card, DatePicker, Statistic, Table, Tabs, Tag } from 'ant-design-vue';
import dayjs from 'dayjs';

import {
  getPurchaseReportByBrandApi,
  getPurchaseReportByDepartmentApi,
  getPurchaseReportByProductApi,
  getPurchaseReportBySupplierApi,
  getPurchaseReportSummaryApi,
} from '#/api';

const { RangePicker } = DatePicker;

const activeKey = ref('summary');
const dateRange = ref<[dayjs.Dayjs, dayjs.Dayjs]>([dayjs().subtract(30, 'day'), dayjs()]);
const loading = ref(false);

// 汇总数据
const summaryData = ref<Record<string, any>>({});
// 按供应商统计
const supplierData = ref<any[]>([]);
// 按产品统计
const productData = ref<any[]>([]);
// 按部门统计
const departmentData = ref<any[]>([]);
// 按品牌统计
const brandData = ref<any[]>([]);

async function fetchData() {
  loading.value = true;
  try {
    const params = {
      startDate: dateRange.value[0]?.format('YYYY-MM-DD'),
      endDate: dateRange.value[1]?.format('YYYY-MM-DD'),
    };

    const [summaryRes, supplierRes, productRes, departmentRes, brandRes] = await Promise.all([
      getPurchaseReportSummaryApi(params),
      getPurchaseReportBySupplierApi(params),
      getPurchaseReportByProductApi(params),
      getPurchaseReportByDepartmentApi(params),
      getPurchaseReportByBrandApi(params),
    ]);

    summaryData.value = summaryRes;
    supplierData.value = Array.isArray(supplierRes) ? supplierRes : supplierRes?.records ?? [];
    productData.value = Array.isArray(productRes) ? productRes : productRes?.records ?? [];
    departmentData.value = Array.isArray(departmentRes) ? departmentRes : departmentRes?.records ?? [];
    brandData.value = Array.isArray(brandRes) ? brandRes : brandRes?.records ?? [];
  } finally {
    loading.value = false;
  }
}

function onRangeChange(dates: [dayjs.Dayjs, dayjs.Dayjs] | null) {
  if (dates) {
    dateRange.value = dates;
    fetchData();
  }
}

fetchData();

// 汇总统计卡片配置
const summaryCards = [
  { key: 'totalOrderCount', label: '总采购单数' },
  { key: 'totalAmount', label: '总金额', prefix: '¥' },
  { key: 'totalTax', label: '总税额', prefix: '¥' },
  { key: 'totalDiscount', label: '总折扣', prefix: '¥' },
  { key: 'totalFreight', label: '总运费', prefix: '¥' },
];

// 各状态数量映射
function statusKeys(data: Record<string, any>): { key: string; label: string; value: number }[] {
  const statusMap: Record<string, string> = {
    pendingCount: '待审核',
    approvedCount: '已审核',
    completedCount: '已完成',
    cancelledCount: '已取消',
  };
  return Object.entries(statusMap)
    .filter(([k]) => k in data)
    .map(([k, label]) => ({ key: k, label, value: data[k] ?? 0 }));
}

// 供应商统计列
const supplierColumns = [
  { title: '供应商ID', dataIndex: 'supplierId', key: 'supplierId', width: 120 },
  { title: '供应商名称', dataIndex: 'supplierName', key: 'supplierName', minWidth: 160 },
  { title: '采购单数', dataIndex: 'orderCount', key: 'orderCount', width: 100 },
  { title: '总金额', dataIndex: 'totalAmount', key: 'totalAmount', width: 140 },
];

// 产品统计列
const productColumns = [
  { title: '产品ID', dataIndex: 'productId', key: 'productId', width: 100 },
  { title: '产品名称', dataIndex: 'productName', key: 'productName', minWidth: 160 },
  { title: 'SKU', dataIndex: 'sku', key: 'sku', width: 140 },
  { title: '总数量', dataIndex: 'totalQuantity', key: 'totalQuantity', width: 100 },
  { title: '总金额', dataIndex: 'totalAmount', key: 'totalAmount', width: 140 },
];

// 部门统计列
const departmentColumns = [
  { title: '部门ID', dataIndex: 'deptId', key: 'deptId', width: 100 },
  { title: '部门名称', dataIndex: 'deptName', key: 'deptName', minWidth: 160 },
  { title: '采购单数', dataIndex: 'orderCount', key: 'orderCount', width: 100 },
  { title: '总金额', dataIndex: 'totalAmount', key: 'totalAmount', width: 140 },
];

// 品牌统计列
const brandColumns = [
  { title: '品牌ID', dataIndex: 'brandId', key: 'brandId', width: 100 },
  { title: '品牌名称', dataIndex: 'brandName', key: 'brandName', minWidth: 160 },
  { title: '明细数', dataIndex: 'detailCount', key: 'detailCount', width: 100 },
  { title: '总金额', dataIndex: 'totalAmount', key: 'totalAmount', width: 140 },
];
</script>

<template>
  <Page auto-content-height>
    <div class="purchase-report">
      <Card :bordered="false" class="mb-4">
        <div class="flex items-center gap-4">
          <span class="text-gray-600 text-sm">统计时间段</span>
          <RangePicker
            :value="dateRange"
            :allow-clear="false"
            @change="onRangeChange"
          />
        </div>
      </Card>

      <Card :bordered="false">
        <Tabs v-model:activeKey="activeKey">
          <Tabs.TabPane key="summary" tab="汇总统计">
            <div v-if="summaryData && Object.keys(summaryData).length > 0">
              <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4 mb-6">
                <Card
                  v-for="card in summaryCards"
                  :key="card.key"
                  size="small"
                  :bordered="false"
                  class="bg-gray-50"
                >
                  <Statistic
                    :title="card.label"
                    :value="summaryData[card.key] ?? 0"
                    :prefix="card.prefix"
                  />
                </Card>
              </div>

              <div v-if="statusKeys(summaryData).length > 0">
                <h4 class="text-base font-medium mb-3">各状态数量</h4>
                <div class="flex flex-wrap gap-4">
                  <Tag
                    v-for="item in statusKeys(summaryData)"
                    :key="item.key"
                    color="blue"
                    class="text-sm px-4 py-1"
                  >
                    {{ item.label }}：{{ item.value }}
                  </Tag>
                </div>
              </div>
            </div>
            <div v-else-if="!loading" class="text-center text-gray-400 py-8">
              暂无数据
            </div>
          </Tabs.TabPane>

          <Tabs.TabPane key="supplier" tab="按供应商统计">
            <Table
              :columns="supplierColumns"
              :data-source="supplierData"
              :loading="loading"
              :pagination="{ pageSize: 20, showSizeChanger: true, showTotal: (total: number) => `共 ${total} 条` }"
              :row-key="(record: any) => record.supplierId"
              bordered
              size="middle"
            />
          </Tabs.TabPane>

          <Tabs.TabPane key="product" tab="按产品统计">
            <Table
              :columns="productColumns"
              :data-source="productData"
              :loading="loading"
              :pagination="{ pageSize: 20, showSizeChanger: true, showTotal: (total: number) => `共 ${total} 条` }"
              :row-key="(record: any) => record.productId"
              bordered
              size="middle"
            />
          </Tabs.TabPane>

          <Tabs.TabPane key="department" tab="按部门统计">
            <Table
              :columns="departmentColumns"
              :data-source="departmentData"
              :loading="loading"
              :pagination="{ pageSize: 20, showSizeChanger: true, showTotal: (total: number) => `共 ${total} 条` }"
              :row-key="(record: any) => record.deptId"
              bordered
              size="middle"
            />
          </Tabs.TabPane>

          <Tabs.TabPane key="brand" tab="按品牌统计">
            <Table
              :columns="brandColumns"
              :data-source="brandData"
              :loading="loading"
              :pagination="{ pageSize: 20, showSizeChanger: true, showTotal: (total: number) => `共 ${total} 条` }"
              :row-key="(record: any) => record.brandId"
              bordered
              size="middle"
            />
          </Tabs.TabPane>
        </Tabs>
      </Card>
    </div>
  </Page>
</template>

<style scoped>
.purchase-report :deep(.ant-statistic-title) {
  margin-bottom: 4px;
  font-size: 13px;
}
</style>