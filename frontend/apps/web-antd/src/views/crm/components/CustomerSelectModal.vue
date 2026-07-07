<script lang="ts" setup>
/**
 * 客户选择弹窗组件
 * 复用客户列表页的表格样式，用于在合同/订单/报价等场景中选择客户
 *
 * 权限逻辑（与 customer/index.vue 保持一致）：
 * - 业务员(dataScope=3/5)：只能看到"我的客户"Tab
 * - 管理人员(dataScope=1/2/4)：可以看到"我的客户"和"下属客户"两个Tab
 *
 * 用法：
 * <CustomerSelectModal v-model:visible="visible" @select="onSelect" />
 */
import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, ref, watch } from 'vue';

import { LucideSearch } from '@vben/icons';
import { useAccessStore, useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  Input,
  Modal,
  Tabs,
  Tag,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getCustomerListApi } from '#/api/core/crm/customer';

const accessStore = useAccessStore();
const userStore = useUserStore();

// data_scope 决定可见的 Tab（与 customer/index.vue 保持一致）
// 1=全部数据 → my+subordinate  2=自定义 → my+subordinate
// 3=本部门 → my  4=本部门及以下 → my+subordinate  5=仅本人 → my
const dataScope = computed(() => {
  const scope = (userStore.userInfo as any)?.dataScope;
  const roles = userStore.userInfo?.roles ?? [];
  if (roles.includes('super_admin') || roles.includes('system_admin')) return 1;
  return typeof scope === 'number' ? scope : 5;
});

const props = withDefaults(defineProps<{
  /** 弹窗是否可见 */
  visible: boolean;
  /** 额外的过滤条件（如 customerId） */
  extraParams?: Record<string, any>;
  /** 弹窗宽度 */
  width?: string | number;
}>(), {
  width: '860px',
});

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void;
  (e: 'select', row: any): void;
}>();

// 内部可见状态
const innerVisible = computed({
  get: () => props.visible,
  set: (val) => emit('update:visible', val),
});

// Tab 列表 - 根据权限动态显示
const allTabList = [
  { key: 'my', label: '我的客户' },
  { key: 'subordinate', label: '下属客户' },
];
const tabList = computed(() => {
  const scope = dataScope.value;
  // 业务员(5/3)只有"我的客户"，管理人员有"我的客户"+"下属客户"
  if (scope === 3 || scope === 5) {
    return allTabList.filter(t => t.key === 'my');
  }
  return allTabList; // 1/2/4 显示全部Tab
});

// 当前激活的 Tab
const activeTab = ref('my');

// 根据 dataScope 设置默认 Tab，权限变化时自动切换
watch(dataScope, (scope) => {
  if (scope === 3 || scope === 5) {
    activeTab.value = 'my';
  }
}, { immediate: true });

// 确保 activeTab 始终在允许范围内
watch(tabList, (tabs) => {
  const keys = tabs.map(t => t.key);
  if (!keys.includes(activeTab.value) && keys.length > 0) {
    activeTab.value = keys[0];
  }
}, { immediate: true });

function handleTabChange(key: string) {
  activeTab.value = key;
  gridApi.query();
}

// 搜索表单
const keywords = ref('');

// 等级颜色映射
const levelColorMap: Record<string, string> = {
  1: 'default', 2: 'red', 3: 'orange', 4: 'blue', 5: 'green',
};
const levelLabelMap: Record<string, string> = {
  1: '无级别', 2: '重点客户', 3: '优质客户', 4: '普通客户', 5: '其他',
};
const industryLabelMap: Record<number, string> = {
  1: '零售', 2: '批发', 3: '制造', 4: '贸易代理',
  5: '电商', 6: '微商', 7: '社交电商', 8: '其他',
};
const sourceLabelMap: Record<number, string> = {
  1: '官网', 2: '展会', 3: '社交媒体', 4: '客户转介',
  5: '陌生拜访', 6: '海关数据', 7: '邮件营销', 8: '阿里国际站',
  9: 'Amazon', 10: 'TikTok', 11: '微信', 12: '其他',
};

const gridOptions: VxeGridProps = {
  toolbarConfig: { refresh: true, zoom: true },
  pagerConfig: {},
  height: 420,
  cellConfig: { isHover: true },
  rowConfig: { height: 'auto' },
  stripe: true,

  proxyConfig: {
    autoLoad: false,
    ajax: {
      query: async ({ page }) => {
        const result = await getCustomerListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          keywords: keywords.value || undefined,
          listType: activeTab.value,
          ...props.extraParams,
        });
        return result;
      },
    },
  },

  columns: [
    { title: '#', type: 'seq', width: 50 },
    { title: '编号', field: 'customerNo', width: 140, headerAlign: 'center', align: 'center' },
    { title: '公司名称', field: 'companyName', minWidth: 200, align: 'left', slots: { default: 'companyName' } },
    {
      title: '等级', field: 'level', width: 80, slots: { default: 'levelSlot' },
    },
    {
      title: '行业', field: 'industry', width: 90,
      formatter: ({ cellValue }: any) => industryLabelMap[cellValue] || cellValue || '-',
    },
    {
      title: '来源', field: 'source', width: 100,
      formatter: ({ cellValue }: any) => sourceLabelMap[cellValue] || cellValue || '-',
    },
    {
      title: '联系人', field: 'contactCount', width: 70, align: 'center',
      formatter: ({ cellValue }: any) => cellValue ?? '-',
    },
    {
      title: '商机数', field: 'opportunityCount', width: 70, align: 'center',
      formatter: ({ cellValue }: any) => cellValue ?? '-',
    },
    {
      title: '负责人', field: 'assigneeName', width: 90,
      formatter: ({ cellValue }: any) => cellValue || '-',
    },
    {
      title: '创建时间', field: 'createTime', width: 150, slots: { default: 'createdAt' },
    },
    {
      title: '操作', field: 'action', fixed: 'right', slots: { default: 'action' }, width: 80,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions });

/** 搜索 */
function handleSearch() {
  gridApi.query();
}

/** 重置搜索 */
function handleReset() {
  keywords.value = '';
  gridApi.query();
}

/** 选择客户 */
function handleSelect(row: any) {
  emit('select', row);
}

/** 双击行也触发选择 */
function handleRowDblClick({ row }: { row: any }) {
  handleSelect(row);
}

// 弹窗打开时自动加载数据
watch(() => props.visible, (val) => {
  if (val) {
    keywords.value = '';
    // 根据权限重置默认 Tab
    const scope = dataScope.value;
    activeTab.value = (scope === 3 || scope === 5) ? 'my' : 'my';
    // 延迟一帧确保 DOM 就绪
    setTimeout(() => gridApi.query(), 100);
  }
});
</script>

<template>
  <Modal
    :open="innerVisible"
    title="选择客户"
    :width="width"
    :footer="null"
    :destroy-on-close="true"
    @cancel="innerVisible = false"
  >
    <!-- Tab 切换：我的客户 / 下属客户 -->
    <Tabs v-model:activeKey="activeTab" @change="handleTabChange" class="mb-3">
      <Tabs.TabPane v-for="tab in tabList" :key="tab.key" :tab="tab.label" />
    </Tabs>

    <!-- 搜索栏 -->
    <div class="flex items-center gap-2 mb-3">
      <Input
        v-model:value="keywords"
        placeholder="输入客户名称/编号搜索"
        allow-clear
        class="flex-1"
        @press-enter="handleSearch"
      >
        <template #prefix>
          <LucideSearch class="w-4 h-4 text-gray-400" />
        </template>
      </Input>
      <Button type="primary" :icon="h(LucideSearch)" @click="handleSearch">搜索</Button>
      <Button @click="handleReset">重置</Button>
    </div>

    <!-- 客户列表表格 -->
    <Grid @row-dblclick="handleRowDblClick">
      <template #companyName="{ row }">
        <div>
          <span class="text-blue-600 font-medium">{{ row.companyName || '-' }}</span>
          <div v-if="row.tags && row.tags.length" class="mt-0.5 flex flex-wrap gap-1">
            <Tag
              v-for="tag in row.tags"
              :key="tag.id"
              :color="tag.tagColor || 'blue'"
              class="!mr-0 !mb-0"
              style="font-size: 11px; line-height: 16px;"
            >
              {{ tag.tagName }}
            </Tag>
          </div>
        </div>
      </template>

      <template #levelSlot="{ row }">
        <Tag :color="levelColorMap[row.level] || 'default'" size="small">
          {{ levelLabelMap[row.level] || row.level || '-' }}
        </Tag>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #action="{ row }">
        <Button type="primary" size="small" @click="handleSelect(row)">选择</Button>
      </template>
    </Grid>

    <div class="mt-2 text-xs text-gray-400 text-right">
      提示：双击行可快速选择
    </div>
  </Modal>
</template>

<style scoped>
/* 行悬停高亮 - 可选择状态 */
:deep(.vxe-table--body-wrapper) {
  cursor: pointer;
}
:deep(.vxe-table--body-wrapper .vxe-body--row:hover td) {
  background-color: #e6f4ff !important;
}
</style>
