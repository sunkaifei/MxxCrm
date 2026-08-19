<script lang="ts" setup>
import { computed, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';

import {
  Card,
  Descriptions,
  DescriptionsItem,
  Statistic,
  Table,
  TabPane,
  Tabs,
  Tag,
} from 'ant-design-vue';

import { getCheckInfoApi } from '#/api/core/product/check';
import { $t } from '#/locales';

const drawerData = ref<{ row?: any }>({});
const mainInfo = ref<any>({});
const allItems = ref<any[]>([]);
const activeTab = ref('all');

const [Drawer, drawerApi] = useVbenDrawer({
  onCancel() {
    drawerApi.close();
  },
  onOpenChange(isOpen: boolean) {
    if (isOpen) {
      drawerData.value = drawerApi.getData<{ row?: any }>() || {};
      mainInfo.value = {};
      allItems.value = [];
      activeTab.value = 'all';
      if (drawerData.value.row?.id) {
        loadDetail(drawerData.value.row.id);
      }
    }
  },
});

async function loadDetail(id: number) {
  try {
    const resp = await getCheckInfoApi(id);
    const data = resp?.data ?? resp;
    if (!data) return;
    const main = data.main ?? data;
    mainInfo.value = {
      ...main,
      warehouseName: main.warehouse_name ?? main.warehouseName ?? '',
      stocktakeNo: main.stocktake_no ?? main.stocktakeNo ?? main.checkNo ?? '',
      status: main.status ?? 0,
    };
    // 字段映射：snake_case → camelCase，product_sku 存的实际是产品编号
    const rawItems = data.items ?? [];
    allItems.value = rawItems.map((item: any) => {
      let actualQuantity: null | number = null;
      if (item.actual_quantity !== null && item.actual_quantity !== undefined) {
        actualQuantity = Number(item.actual_quantity);
      } else if (
        item.actualQuantity !== null &&
        item.actualQuantity !== undefined
      ) {
        actualQuantity = Number(item.actualQuantity);
      }
      return {
        ...item,
        productName: item.product_name ?? item.productName ?? '',
        productCode: item.product_code ?? item.productCode ?? '',
        productSku: item.product_sku ?? item.productSku ?? '',
        systemQuantity: Number(
          item.system_quantity ?? item.systemQuantity ?? 0,
        ),
        actualQuantity,
        difference:
          item.difference !== null && item.difference !== undefined
            ? Number(item.difference)
            : Number(item.difference ?? 0),
      };
    });
  } catch (error) {
    console.error('[盘点详情] 加载失败:', error);
  }
}

// 汇总统计
const surplusCount = computed(
  () =>
    allItems.value.filter((i) => {
      const diff = Number(i.difference ?? 0);
      return diff > 0;
    }).length,
);

const shortageCount = computed(
  () =>
    allItems.value.filter((i) => {
      const diff = Number(i.difference ?? 0);
      return diff < 0;
    }).length,
);

const matchCount = computed(
  () =>
    allItems.value.filter((i) => {
      const diff = Number(i.difference ?? 0);
      return diff === 0;
    }).length,
);

// 按 Tab 筛选
const filteredItems = computed(() => {
  if (activeTab.value === 'surplus') {
    return allItems.value.filter((i) => Number(i.difference ?? 0) > 0);
  }
  if (activeTab.value === 'shortage') {
    return allItems.value.filter((i) => Number(i.difference ?? 0) < 0);
  }
  if (activeTab.value === 'match') {
    return allItems.value.filter((i) => Number(i.difference ?? 0) === 0);
  }
  return allItems.value;
});

function getDiffTag(diff: number) {
  if (diff > 0)
    return {
      label: $t('page.product.inventory.check.type.surplus'),
      color: 'success',
    };
  if (diff < 0)
    return {
      label: $t('page.product.inventory.check.type.shortage'),
      color: 'error',
    };
  return {
    label: $t('page.product.inventory.check.type.match'),
    color: 'default',
  };
}

const columns = [
  {
    title: $t('page.product.inventory.check.field.productCode'),
    dataIndex: 'productCode',
    width: 120,
    ellipsis: true,
  },
  {
    title: $t('page.product.inventory.check.field.productName'),
    dataIndex: 'productName',
    width: 150,
    ellipsis: true,
  },
  {
    title: $t('page.product.inventory.check.field.productSku'),
    dataIndex: 'productSku',
    width: 120,
    ellipsis: true,
  },
  {
    title: $t('page.product.inventory.check.field.systemQuantity'),
    dataIndex: 'systemQuantity',
    width: 100,
    align: 'right' as const,
    customRender: ({ text }: any) =>
      text !== null && text !== undefined ? Number(text) : '0',
  },
  {
    title: $t('page.product.inventory.check.field.actualQuantity'),
    dataIndex: 'actualQuantity',
    width: 100,
    align: 'right' as const,
    customRender: ({ text }: any) =>
      text !== null && text !== undefined ? Number(text) : '-',
  },
  {
    title: $t('page.product.inventory.check.field.difference'),
    dataIndex: 'difference',
    width: 100,
    align: 'right' as const,
    customRender: ({ text }: any) => {
      if (text === null || text === undefined) return '-';
      const num = Number(text);
      return num > 0 ? `+${num}` : `${num}`;
    },
  },
  {
    title: $t('page.product.inventory.check.field.differenceType'),
    dataIndex: 'differenceType',
    width: 90,
    align: 'center' as const,
  },
  {
    title: $t('page.product.inventory.check.field.remark'),
    dataIndex: 'remark',
    width: 150,
    ellipsis: true,
  },
];
</script>

<template>
  <Drawer
    class="check-detail-drawer"
    :title="$t('page.product.inventory.check.drawer.detailTitle')"
    :show-confirm-button="false"
    :cancel-text="$t('ui.button.close')"
    width="80%"
  >
    <!-- 汇总卡片 -->
    <div class="check-detail-summary">
      <Card size="small" class="summary-card">
        <Statistic
          :title="$t('page.product.inventory.check.field.totalItems')"
          :value="allItems.length"
        />
      </Card>
      <Card size="small" class="summary-card summary-surplus">
        <Statistic
          :title="$t('page.product.inventory.check.type.surplus')"
          :value="surplusCount"
          :value-style="{ color: '#00b42a' }"
        />
      </Card>
      <Card size="small" class="summary-card summary-shortage">
        <Statistic
          :title="$t('page.product.inventory.check.type.shortage')"
          :value="shortageCount"
          :value-style="{ color: '#f53f3f' }"
        />
      </Card>
      <Card size="small" class="summary-card summary-match">
        <Statistic
          :title="$t('page.product.inventory.check.type.match')"
          :value="matchCount"
          :value-style="{ color: '#86909c' }"
        />
      </Card>
    </div>

    <!-- 基本信息 -->
    <Descriptions
      v-if="mainInfo.id"
      :column="3"
      size="small"
      bordered
      class="check-detail-desc"
    >
      <DescriptionsItem
        :label="$t('page.product.inventory.check.field.checkNo')"
      >
        {{ mainInfo.stocktakeNo }}
      </DescriptionsItem>
      <DescriptionsItem
        :label="$t('page.product.inventory.check.field.warehouse')"
      >
        {{ mainInfo.warehouseName || drawerData.row?.warehouseName || '-' }}
      </DescriptionsItem>
      <DescriptionsItem
        :label="$t('page.product.inventory.check.field.status')"
      >
        <Tag>
          {{
            $t(`page.product.inventory.check.status.${mainInfo.status ?? 0}`)
          }}
        </Tag>
      </DescriptionsItem>
    </Descriptions>

    <!-- 明细表格 + Tab 筛选 -->
    <Tabs v-model:active-key="activeTab" class="check-detail-tabs">
      <TabPane
        key="all"
        :tab="`${$t('page.product.inventory.check.tab.all')} (${allItems.length})`"
      />
      <TabPane
        key="surplus"
        :tab="`${$t('page.product.inventory.check.type.surplus')} (${surplusCount})`"
      />
      <TabPane
        key="shortage"
        :tab="`${$t('page.product.inventory.check.type.shortage')} (${shortageCount})`"
      />
      <TabPane
        key="match"
        :tab="`${$t('page.product.inventory.check.type.match')} (${matchCount})`"
      />
    </Tabs>

    <Table
      :columns="columns"
      :data-source="filteredItems"
      :row-key="(record) => record.id"
      :pagination="false"
      size="small"
      :scroll="{ y: 'calc(100vh - 420px)' }"
      bordered
    >
      <template #bodyCell="{ column, record }">
        <template v-if="column.dataIndex === 'differenceType'">
          <Tag
            :color="getDiffTag(Number(record.difference ?? 0)).color"
            :bordered="false"
          >
            {{ getDiffTag(Number(record.difference ?? 0)).label }}
          </Tag>
        </template>
      </template>
    </Table>
  </Drawer>
</template>

<style>
.check-detail-drawer {
  width: 80vw !important;
}

.check-detail-summary {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  margin-bottom: 16px;
}

.check-detail-summary .summary-card {
  text-align: center;
}

.check-detail-desc {
  margin-bottom: 16px;
}

.check-detail-tabs {
  margin-bottom: 8px;
}
</style>
