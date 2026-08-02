<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, onMounted, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  Card,
  Image as AImage,
  Statistic,
  Tabs,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getVisitListApi, getVisitStatisticsApi } from '#/api';
import { $t } from '#/locales';
import VisitDetailDrawer from './detail-drawer.vue';

const userStore = useUserStore();

// 全部拜访 Tab 显示条件：超级管理员 / 系统管理员 / data_scope=全部数据
const canViewAll = computed(() => {
  const roles = userStore.userInfo?.roles ?? [];
  const dataScope =
    (userStore.userInfo as any)?.dataScope ??
    (userStore.userInfo as any)?.data_scope;
  if (roles.includes('super_admin') || roles.includes('system_admin'))
    return true;
  return dataScope === 1;
});

// 下属拜访 Tab 显示条件：超级管理员 / 系统管理员 / 数据权限含部门（2/3/4）
const canViewSubordinate = computed(() => {
  const roles = userStore.userInfo?.roles ?? [];
  const dataScope =
    (userStore.userInfo as any)?.dataScope ??
    (userStore.userInfo as any)?.data_scope;
  if (roles.includes('super_admin') || roles.includes('system_admin'))
    return true;
  return dataScope === 2 || dataScope === 3 || dataScope === 4;
});

const allTabList = [
  { key: 'all', label: '全部拜访' },
  { key: 'my', label: '我的拜访' },
  { key: 'subordinate', label: '下属拜访' },
];

const tabList = computed(() => {
  const keys: string[] = [];
  if (canViewAll.value) keys.push('all');
  keys.push('my');
  if (canViewSubordinate.value) keys.push('subordinate');
  return allTabList.filter((t) => keys.includes(t.key));
});

const activeTab = ref('my');

function handleTabChange(key: string | number) {
  activeTab.value = String(key);
  gridApi.query();
}

// ========== 顶部统计卡片 ==========
const statistics = ref({
  totalVisits: 0,
  todayVisits: 0,
  weekVisits: 0,
  monthVisits: 0,
  uniqueCustomers: 0,
});
const statisticsLoading = ref(false);

async function loadStatistics() {
  statisticsLoading.value = true;
  try {
    const res: any = await getVisitStatisticsApi();
    statistics.value = {
      totalVisits: Number(res?.totalVisits ?? 0),
      todayVisits: Number(res?.todayVisits ?? 0),
      weekVisits: Number(res?.weekVisits ?? 0),
      monthVisits: Number(res?.monthVisits ?? 0),
      uniqueCustomers: Number(res?.uniqueCustomers ?? 0),
    };
  } catch {
    // 全局拦截器处理
  } finally {
    statisticsLoading.value = false;
  }
}

// ========== 工具函数 ==========

// 解析签到照片字段（后端 visit_photos 为 JSON 数组）
function parsePhotos(row: any): string[] {
  const raw = row.visitPhotos ?? row.visit_photos;
  if (!raw) return [];
  if (Array.isArray(raw)) return raw.filter((u: any) => !!u);
  if (typeof raw === 'string') {
    try {
      const parsed = JSON.parse(raw);
      return Array.isArray(parsed) ? parsed.filter((u: any) => !!u) : [];
    } catch {
      // 单个 URL 字符串
      return raw ? [raw] : [];
    }
  }
  return [];
}

// 拜访时长：签退时间 - 签到时间
function formatDuration(row: any): string {
  const start = row.checkInTime ?? row.check_in_time;
  const end = row.checkOutTime ?? row.check_out_time;
  if (!start || !end) return '-';
  const startMs = new Date(start).getTime();
  const endMs = new Date(end).getTime();
  if (Number.isNaN(startMs) || Number.isNaN(endMs) || endMs <= startMs)
    return '-';
  const diffMs = endMs - startMs;
  const minutes = Math.floor(diffMs / 60000);
  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  if (hours > 0) return `${hours}小时${mins}分钟`;
  return `${mins}分钟`;
}

// 距客户距离格式化（米 / 千米）
function formatDistance(row: any): string {
  const dist = row.visitDistance ?? row.visit_distance;
  if (dist == null || Number.isNaN(Number(dist))) return '-';
  const d = Number(dist);
  if (d < 0) return '-';
  if (d < 1000) return `${d.toFixed(0)}米`;
  return `${(d / 1000).toFixed(2)}公里`;
}

// ========== 搜索表单 ==========
const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'customerName',
      label: '客户名称',
      componentProps: { placeholder: '请输入客户名称', allowClear: true },
    },
    {
      component: 'RangePicker',
      fieldName: 'checkInDateRange',
      label: '签到时间',
      componentProps: {
        placeholder: ['开始日期', '结束日期'],
        style: 'width:100%',
        valueFormat: 'YYYY-MM-DD',
      },
    },
    {
      component: 'Input',
      fieldName: 'ownerName',
      label: '负责人',
      componentProps: { placeholder: '请输入负责人姓名', allowClear: true },
    },
  ],
};

// ========== 表格 ==========
const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, export: true, refresh: true, zoom: true },
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true },
  rowConfig: { height: 'auto' },
  stripe: true,
  checkboxConfig: { checkMethod: () => true },
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const params: any = {
          page: page.currentPage,
          pageSize: page.pageSize,
          listType: activeTab.value,
        };
        if (formValues.customerName) params.customerName = formValues.customerName;
        if (formValues.ownerName) params.ownerName = formValues.ownerName;
        if (
          formValues.checkInDateRange &&
          formValues.checkInDateRange.length === 2
        ) {
          params.checkInStart = formValues.checkInDateRange[0];
          params.checkInEnd = formValues.checkInDateRange[1];
        }
        const result = await getVisitListApi(params);
        // 无数据 280px，有数据按内容自适应
        const items = (result as any)?.items ?? [];
        const gridEl = gridApi.grid?.$el as HTMLElement | undefined;
        if (gridEl) {
          gridEl.style.height = items.length === 0 ? '280px' : '';
        }
        return result;
      },
    },
  },
  columns: [
    { type: 'checkbox', width: 50 },
    { title: $t('ui.table.seq'), type: 'seq', width: 60, headerAlign: 'center' },
    {
      title: '客户名称',
      field: 'customerName',
      minWidth: 180,
      headerAlign: 'center',
      align: 'left',
      slots: { default: 'customerName' },
    },
    {
      title: '签到地址',
      field: 'visitAddress',
      minWidth: 220,
      headerAlign: 'center',
      align: 'left',
      slots: { default: 'visitAddress' },
    },
    {
      title: '签到时间',
      field: 'checkInTime',
      width: 160,
      headerAlign: 'center',
      slots: { default: 'checkInTime' },
    },
    {
      title: '签退时间',
      field: 'checkOutTime',
      width: 160,
      headerAlign: 'center',
      slots: { default: 'checkOutTime' },
    },
    {
      title: '拜访时长',
      field: 'duration',
      width: 120,
      headerAlign: 'center',
      slots: { default: 'duration' },
    },
    {
      title: '距客户距离',
      field: 'visitDistance',
      width: 110,
      headerAlign: 'center',
      slots: { default: 'distance' },
    },
    {
      title: '签到照片',
      field: 'visitPhotos',
      width: 110,
      headerAlign: 'center',
      slots: { default: 'photos' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      headerAlign: 'center',
      slots: { default: 'action' },
      width: 100,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

// ========== 详情抽屉 ==========
const [DetailDrawer, detailDrawerApi] = useVbenDrawer({
  connectedComponent: VisitDetailDrawer,
});

function handleView(row: any) {
  const id = row.id ?? row.id_;
  if (!id) return;
  detailDrawerApi.setData({ id: Number(id) });
  detailDrawerApi.open();
}

// 刷新统计：在表格查询后同步刷新统计
function handleRefresh() {
  gridApi.query();
  loadStatistics();
}

onMounted(() => {
  loadStatistics();
});
</script>

<template>
  <Page>
    <!-- 顶部统计卡片 -->
    <Card :bordered="false" class="mb-4" :loading="statisticsLoading">
      <div class="visit-stat-row">
        <div class="visit-stat-item">
          <Statistic title="总拜访次数" :value="statistics.totalVisits" />
        </div>
        <div class="visit-stat-item">
          <Statistic title="今日拜访" :value="statistics.todayVisits" :value-style="{ color: '#1677ff' }" />
        </div>
        <div class="visit-stat-item">
          <Statistic title="本周拜访" :value="statistics.weekVisits" :value-style="{ color: '#52c41a' }" />
        </div>
        <div class="visit-stat-item">
          <Statistic title="本月拜访" :value="statistics.monthVisits" :value-style="{ color: '#faad14' }" />
        </div>
        <div class="visit-stat-item">
          <Statistic title="拜访客户数" :value="statistics.uniqueCustomers" :value-style="{ color: '#722ed1' }" />
        </div>
      </div>
    </Card>

    <Grid table-title="外勤拜访记录">
      <template #form-header>
        <Tabs v-model:activeKey="activeTab" class="mb-3" @change="handleTabChange">
          <Tabs.TabPane
            v-for="tab in tabList"
            :key="tab.key"
            :tab="tab.label"
          />
        </Tabs>
      </template>

      <template #toolbar-tools>
        <Button class="mr-2" @click="handleRefresh">刷新统计</Button>
      </template>

      <template #customerName="{ row }">
        <a
          class="text-blue-600 cursor-pointer hover:text-blue-800"
          @click="() => handleView(row)"
        >
          {{ row.customerName || (row.leadName || '-') }}
        </a>
      </template>

      <template #visitAddress="{ row }">
        <span class="visit-address-text">{{ row.visitAddress || '-' }}</span>
      </template>

      <template #checkInTime="{ row }">
        {{ row.checkInTime ? formatDateTime(row.checkInTime) : '-' }}
      </template>

      <template #checkOutTime="{ row }">
        {{ row.checkOutTime ? formatDateTime(row.checkOutTime) : '-' }}
      </template>

      <template #duration="{ row }">
        <span class="visit-duration">{{ formatDuration(row) }}</span>
      </template>

      <template #distance="{ row }">
        {{ formatDistance(row) }}
      </template>

      <template #photos="{ row }">
        <AImage.PreviewGroup v-if="parsePhotos(row).length > 0">
          <div class="visit-photo-cell">
            <AImage
              :src="parsePhotos(row)[0]"
              :width="40"
              :height="40"
              class="visit-photo-thumb"
            />
            <AImage
              v-for="(url, idx) in parsePhotos(row).slice(1)"
              :key="idx"
              :src="url"
              class="visit-photo-hidden"
            />
            <span
              v-if="parsePhotos(row).length > 1"
              class="visit-photo-count"
            >+{{ parsePhotos(row).length - 1 }}</span>
          </div>
        </AImage.PreviewGroup>
        <span v-else class="text-gray-400">-</span>
      </template>

      <template #action="{ row }">
        <a
          class="text-blue-600 cursor-pointer mx-1"
          @click="() => handleView(row)"
        >查看详情</a>
      </template>
    </Grid>

    <DetailDrawer />
  </Page>
</template>

<style scoped>
.visit-stat-row {
  display: flex;
  flex-wrap: wrap;
  gap: 16px 24px;
}
.visit-stat-item {
  flex: 1 1 160px;
  min-width: 160px;
}
.visit-address-text {
  word-break: break-all;
  white-space: normal;
  line-height: 1.5;
}
.visit-duration {
  font-weight: 500;
  color: #1677ff;
}
.visit-photo-cell {
  position: relative;
  display: inline-flex;
  align-items: center;
}
.visit-photo-thumb {
  border-radius: 4px;
  border: 1px solid #e8e8e8;
  object-fit: cover;
}
.visit-photo-count {
  margin-left: 4px;
  font-size: 12px;
  color: #666;
}
.visit-photo-hidden {
  display: none;
}
</style>
