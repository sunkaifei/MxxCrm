<script lang="ts" setup>
import { computed, onMounted, ref } from 'vue';

import { Page } from '@vben/common-ui';
import { LucideBanknote, LucideCheckCircle, LucideShoppingCart, LucideTarget } from '@vben/icons';

import { Card, Progress, Select, Tabs, Tag } from 'ant-design-vue';

interface PerformanceItem {
  salesperson: string;
  department: string;
  orderCount: number;
  salesAmount: number;
  collectionAmount: number;
  achievementRate: number;
}

const selectedYear = ref('2024');
const selectedMonth = ref('6');
const selectedDept = ref<string | undefined>(undefined);
const activeTab = ref('detail');

const summary = ref({
  salesAmount: 0,
  orderCount: 0,
  achievementRate: 0,
  collectionAmount: 0,
  targetAmount: 0,
});

const details = ref<PerformanceItem[]>([]);

const ranking = computed(() =>
  [...details.value].sort((a, b) => b.salesAmount - a.salesAmount),
);

const topThree = computed(() => ranking.value.slice(0, 3));
const maxAmount = computed(() => ranking.value[0]?.salesAmount || 1);

onMounted(() => {
  // TODO: fetch from API
  summary.value = { salesAmount: 1250000, orderCount: 45, achievementRate: 83.3, collectionAmount: 980000, targetAmount: 1500000 };
  details.value = [
    { salesperson: '张三', department: '外贸一部', orderCount: 20, salesAmount: 520000, collectionAmount: 420000, achievementRate: 104 },
    { salesperson: '李四', department: '外贸一部', orderCount: 12, salesAmount: 380000, collectionAmount: 310000, achievementRate: 76 },
    { salesperson: '王五', department: '外贸二部', orderCount: 8, salesAmount: 250000, collectionAmount: 200000, achievementRate: 50 },
    { salesperson: '赵六', department: '外贸二部', orderCount: 5, salesAmount: 100000, collectionAmount: 50000, achievementRate: 25 },
  ];
});

function formatCurrency(val: number) {
  return `¥${val.toLocaleString()}`;
}
</script>

<template>
  <Page auto-content-height>
    <div class="p-4">
      <!-- 顶部筛选 -->
      <div class="flex items-center gap-3 mb-4">
        <Select v-model:value="selectedYear" style="width: 100px" :options="[
          { label: '2024年', value: '2024' }, { label: '2025年', value: '2025' }, { label: '2026年', value: '2026' },
        ]" />
        <Select v-model:value="selectedMonth" style="width: 80px" :options="[
          { label: '1月', value: '1' }, { label: '2月', value: '2' }, { label: '3月', value: '3' },
          { label: '4月', value: '4' }, { label: '5月', value: '5' }, { label: '6月', value: '6' },
          { label: '7月', value: '7' }, { label: '8月', value: '8' }, { label: '9月', value: '9' },
          { label: '10月', value: '10' }, { label: '11月', value: '11' }, { label: '12月', value: '12' },
        ]" />
        <Select v-model:value="selectedDept" style="width: 140px" placeholder="全部部门" allow-clear :options="[
          { label: '全部部门', value: '' }, { label: '外贸一部', value: 'dept1' }, { label: '外贸二部', value: 'dept2' },
        ]" />
      </div>

      <!-- 统计卡片 -->
      <div class="grid grid-cols-4 gap-4 mb-6">
        <Card size="small" class="text-center">
          <div class="flex items-center justify-center gap-2 mb-1">
            <LucideBanknote class="w-4 h-4 text-blue-500" />
            <span class="text-xs text-gray-500">本月销售额</span>
          </div>
          <div class="text-xl font-bold text-blue-600">{{ formatCurrency(summary.salesAmount) }}</div>
        </Card>
        <Card size="small" class="text-center">
          <div class="flex items-center justify-center gap-2 mb-1">
            <LucideShoppingCart class="w-4 h-4 text-green-500" />
            <span class="text-xs text-gray-500">本月订单数</span>
          </div>
          <div class="text-xl font-bold text-green-600">{{ summary.orderCount }} 笔</div>
        </Card>
        <Card size="small" class="text-center">
          <div class="flex items-center justify-center gap-2 mb-1">
            <LucideTarget class="w-4 h-4 text-orange-500" />
            <span class="text-xs text-gray-500">目标达成率</span>
          </div>
          <div class="text-xl font-bold text-orange-600">{{ summary.achievementRate }}%</div>
        </Card>
        <Card size="small" class="text-center">
          <div class="flex items-center justify-center gap-2 mb-1">
            <LucideCheckCircle class="w-4 h-4 text-purple-500" />
            <span class="text-xs text-gray-500">本月回款</span>
          </div>
          <div class="text-xl font-bold text-purple-600">{{ formatCurrency(summary.collectionAmount) }}</div>
        </Card>
      </div>

      <!-- Tab 内容 -->
      <Tabs v-model:activeKey="activeTab">
        <Tabs.TabPane key="detail" tab="业绩明细">
          <div class="overflow-x-auto">
            <table class="w-full text-sm border-collapse">
              <thead>
                <tr class="bg-gray-50">
                  <th class="p-2 text-left border">#</th>
                  <th class="p-2 text-left border">销售员</th>
                  <th class="p-2 text-left border">部门</th>
                  <th class="p-2 text-right border">订单数</th>
                  <th class="p-2 text-right border">销售额</th>
                  <th class="p-2 text-right border">回款额</th>
                  <th class="p-2 text-center border">达成率</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(item, idx) in details" :key="idx" class="hover:bg-gray-50">
                  <td class="p-2 border">{{ idx + 1 }}</td>
                  <td class="p-2 border font-medium">{{ item.salesperson }}</td>
                  <td class="p-2 border">{{ item.department }}</td>
                  <td class="p-2 text-right border">{{ item.orderCount }}</td>
                  <td class="p-2 text-right border text-blue-600">{{ formatCurrency(item.salesAmount) }}</td>
                  <td class="p-2 text-right border text-purple-600">{{ formatCurrency(item.collectionAmount) }}</td>
                  <td class="p-2 text-center border">
                    <Progress :percent="item.achievementRate" :size="16" :stroke-width="6"
                      :stroke-color="item.achievementRate >= 80 ? '#52c41a' : item.achievementRate >= 50 ? '#faad14' : '#ff4d4f'"
                      :show-info="false" />
                    <span class="text-xs ml-1">{{ item.achievementRate }}%</span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </Tabs.TabPane>

        <Tabs.TabPane key="ranking" tab="个人排名">
          <div class="space-y-3">
            <div v-for="(item, idx) in ranking" :key="idx" class="flex items-center gap-3 p-3 bg-gray-50 rounded-lg">
              <div class="flex items-center justify-center w-8 h-8 rounded-full text-white font-bold text-sm"
                :class="idx === 0 ? 'bg-yellow-500' : idx === 1 ? 'bg-gray-400' : idx === 2 ? 'bg-orange-400' : 'bg-gray-300'">
                {{ idx + 1 }}
              </div>
              <div class="flex-1">
                <div class="flex justify-between mb-1">
                  <span class="font-medium">{{ item.salesperson }}</span>
                  <span class="text-blue-600 font-semibold">{{ formatCurrency(item.salesAmount) }}</span>
                </div>
                <Progress :percent="(item.salesAmount / maxAmount) * 100" :size="8" :stroke-width="8"
                  :stroke-color="idx === 0 ? '#fadb14' : idx === 1 ? '#bfbfbf' : idx === 2 ? '#ffa940' : '#d9d9d9'"
                  :show-info="false" />
                <div class="text-xs text-gray-400 mt-0.5">{{ item.department }} · {{ item.orderCount }}单</div>
              </div>
            </div>
          </div>
        </Tabs.TabPane>
      </Tabs>
    </div>
  </Page>
</template>