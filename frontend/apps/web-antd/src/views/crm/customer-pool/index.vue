<script lang="ts" setup>
import type { VxeGridProps } from '#/adapter/vxe-table';

import { reactive, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Drawer, Form, Input, Modal, Row, Col, Select, Tag, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getCustomerPoolListApi, claimCustomerApi } from '#/api';
import { $t } from '#/locales';
import CustomerDrawer from '../customer/drawer.vue';
import CustomerDetail from '../customer/detail.vue';

const accessStore = useAccessStore();

const sourceLabelMap: Record<number, string> = {
  1: '官网', 2: '展会', 3: '社交媒体', 4: '客户转介',
  5: '陌生拜访', 6: '海关数据', 7: '邮件营销', 8: '阿里国际站',
  9: 'Amazon', 10: 'TikTok', 11: '微信', 12: '其他',
};

const levelColorMap: Record<number, string> = {
  1: 'default', 2: 'red', 3: 'orange', 4: 'blue', 5: 'green',
};
const levelLabelMap: Record<number, string> = {
  1: '无级别', 2: '重点客户', 3: '优质客户', 4: '普通客户', 5: '其他',
};

const industryLabelMap: Record<number, string> = {
  1: '零售', 2: '批发', 3: '制造', 4: '贸易代理',
  5: '电商', 6: '微商', 7: '社交电商', 8: '其他',
};

const detailVisible = ref(false);
const detailId = ref<number | null>(null);

function openDetail(row: any) {
  const id = row.id ?? row.id_;
  if (!id) { message.error('客户ID不存在'); return; }
  detailId.value = Number(id);
  detailVisible.value = true;
}
function closeDetail() { detailVisible.value = false; detailId.value = null; }
function handleDetailEdit(customer: any) { closeDetail(); openDrawer(false, customer); }

const searchForm = reactive({
  companyName: '',
  level: undefined as number | undefined,
  country: '',
  source: undefined as number | undefined,
  industry: undefined as number | undefined,
});

function handleSearch() {
  gridApi.query();
}
function handleReset() {
  searchForm.companyName = '';
  searchForm.level = undefined;
  searchForm.country = '';
  searchForm.source = undefined;
  searchForm.industry = undefined;
  gridApi.query();
}

const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, refresh: true, zoom: true },
  pagerConfig: {},
  cellConfig: { isHover: true },
  rowConfig: { height: 'auto' },
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }) => {
        const result = await getCustomerPoolListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          companyName: searchForm.companyName || undefined,
          level: searchForm.level,
          country: searchForm.country || undefined,
          source: searchForm.source,
          industry: searchForm.industry,
        });
        // 无数据 280px，有数据按内容自适应
        const items = (result as any)?.items ?? [];
        const gridEl = gridApi.grid?.$el as HTMLElement | undefined;
        if (gridEl) {
          gridEl.style.height = items.length === 0 ? '280px' : '';
        }
        // 等DOM渲染完成后同步固定列行高并居中内容
        const syncFixedColumn = (retry = 0) => {
          const $el = gridApi.grid?.$el as HTMLElement | undefined;
          if (!$el) return;
          const mainBody = $el.querySelector('.vxe-table--body-wrapper tbody');
          const fixedRightBody = $el.querySelector('.vxe-table--fixed-right-wrapper tbody');
          if (!mainBody || !fixedRightBody) {
            if (retry < 3) setTimeout(() => syncFixedColumn(retry + 1), 200);
            return;
          }
          const rows1 = mainBody.querySelectorAll('tr.vxe-body--row');
          const rows2 = fixedRightBody.querySelectorAll('tr.vxe-body--row');
          const len = Math.min(rows1.length, rows2.length);
          if (len === 0) return;
          for (let i = 0; i < len; i++) {
            const h = (rows1[i] as HTMLElement).offsetHeight;
            if (h === 0) continue;
            (rows2[i] as HTMLElement).style.height = h + 'px';
            const tds = (rows2[i] as HTMLElement).querySelectorAll('td');
            tds.forEach((td: Element) => {
              const cell = td.querySelector('.vxe-cell');
              if (cell) {
                (cell as HTMLElement).style.display = 'flex';
                (cell as HTMLElement).style.alignItems = 'center';
                (cell as HTMLElement).style.justifyContent = 'center';
                (cell as HTMLElement).style.height = h + 'px';
              }
            });
          }
        };
        requestAnimationFrame(() => {
          syncFixedColumn();
          setTimeout(() => syncFixedColumn(), 200);
          setTimeout(() => syncFixedColumn(), 500);
        });
        return result;
      },
    },
  },

  columns: [
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: '编号', field: 'customerNo', width: 150, headerAlign: 'center', align: 'center', slots: { default: 'customerNo' } },
    { title: '公司名称', field: 'companyName', minWidth: 200, headerAlign: 'center', align: 'left', slots: { default: 'companyName' } },
    {
      title: '等级', field: 'level', width: 80, slots: { default: 'level' },
    },
    { title: '国家', field: 'country', width: 80 },
    {
      title: '来源', field: 'source', width: 100,
      formatter: ({ cellValue }: any) => sourceLabelMap[cellValue] || cellValue || '-',
    },
    { title: '录入人', field: 'createdByName', width: 90 },
    {
      title: $t('ui.table.createTime'), field: 'createTime', slots: { default: 'createdAt' }, width: 160,
    },
    {
      title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 80,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions });

const [FormDrawer, drawerApi] = useVbenDrawer({
  connectedComponent: CustomerDrawer,
  onClosed() { if (drawerApi.getData()?.needRefresh) gridApi.query(); },
});

function openDrawer(create: boolean, row?: any) { drawerApi.setData({ create, row }); drawerApi.open(); }

async function handleClaim(row: any) {
  Modal.confirm({
    title: '领取客户',
    content: `确定领取客户"${row.companyName}"吗？领取后将转入您的客户管理。`,
    onOk: async () => {
      try {
        await claimCustomerApi(Number(row.id));
        message.success('领取成功');
        gridApi.query();
      } catch {
        // 错误提示由拦截器处理
      }
    },
  });
}
</script>

<template>
  <Page>
    <div class="pool-search-card">
      <Form :model="searchForm" layout="horizontal" :label-col="{ style: { width: '100px' } }">
        <div class="pool-search-form-wrapper">
        <Row :gutter="24">
          <Col :xs="24" :sm="24" :md="12">
            <Form.Item label="客户名称" name="companyName">
              <Input v-model:value="searchForm.companyName" placeholder="请输入客户名称" allow-clear />
            </Form.Item>
          </Col>
          <Col :xs="24" :sm="24" :md="12">
            <Form.Item label="客户级别" name="level">
              <Select v-model:value="searchForm.level" placeholder="请选择客户级别" allow-clear>
                <Select.Option :value="1">无级别</Select.Option>
                <Select.Option :value="2">重点客户</Select.Option>
                <Select.Option :value="3">优质客户</Select.Option>
                <Select.Option :value="4">普通客户</Select.Option>
                <Select.Option :value="5">其他</Select.Option>
              </Select>
            </Form.Item>
          </Col>
          <Col :xs="24" :sm="24" :md="12">
            <Form.Item label="客户行业" name="industry">
              <Select v-model:value="searchForm.industry" placeholder="请选择客户行业" allow-clear>
                <Select.Option :value="1">零售</Select.Option>
                <Select.Option :value="2">批发</Select.Option>
                <Select.Option :value="3">制造</Select.Option>
                <Select.Option :value="4">贸易代理</Select.Option>
                <Select.Option :value="5">电商</Select.Option>
                <Select.Option :value="6">微商</Select.Option>
                <Select.Option :value="7">社交电商</Select.Option>
                <Select.Option :value="8">其他</Select.Option>
              </Select>
            </Form.Item>
          </Col>
          <Col :xs="24" :sm="24" :md="12">
            <Form.Item label="客户来源" name="source">
              <Select v-model:value="searchForm.source" placeholder="请选择客户来源" allow-clear>
                <Select.Option :value="1">官网</Select.Option>
                <Select.Option :value="2">展会</Select.Option>
                <Select.Option :value="3">社交媒体</Select.Option>
                <Select.Option :value="4">客户转介</Select.Option>
                <Select.Option :value="5">陌生拜访</Select.Option>
                <Select.Option :value="6">海关数据</Select.Option>
                <Select.Option :value="7">邮件营销</Select.Option>
                <Select.Option :value="8">阿里国际站</Select.Option>
                <Select.Option :value="9">Amazon</Select.Option>
                <Select.Option :value="10">TikTok</Select.Option>
                <Select.Option :value="11">微信</Select.Option>
                <Select.Option :value="12">其他</Select.Option>
              </Select>
            </Form.Item>
          </Col>
          <Col :xs="24" :sm="24" :md="12">
            <Form.Item label="国家" name="country">
              <Input v-model:value="searchForm.country" placeholder="请输入国家" allow-clear />
            </Form.Item>
          </Col>
        </Row>
        </div>
        <div class="pool-search-actions">
          <Button type="primary" @click="handleSearch">
            <template #icon>
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
            </template>
            搜索
          </Button>
          <Button @click="handleReset">
            <template #icon>
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/><path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/><path d="M8 16H3v5"/></svg>
            </template>
            刷新
          </Button>
        </div>
      </Form>
    </div>

    <Grid table-title="公海客户">
      <template #createdAt="{ row }">{{ formatDateTime(row.createTime) }}</template>

      <template #customerNo="{ row }">{{ row.customerNo || '-' }}</template>

      <template #companyName="{ row }">
        <div>
          <a class="cursor-pointer text-blue-600 hover:text-blue-800" @click="() => openDetail(row)">{{ row.companyName }}</a>
          <div v-if="row.tags && row.tags.length" class="mt-1 flex flex-wrap gap-1">
            <Tag
              v-for="tag in row.tags"
              :key="tag.id"
              :color="tag.tagColor || 'blue'"
              class="!mr-0 !mb-1"
              style="font-size: 12px; line-height: 18px;"
            >
              {{ tag.tagName }}
            </Tag>
          </div>
        </div>
      </template>

      <template #level="{ row }">
        <Tag :color="levelColorMap[row.level] || 'default'">{{ levelLabelMap[row.level] || row.level || '-' }}</Tag>
      </template>

      <template #action="{ row }">
        <span class="action-btns">
          <a v-if="accessStore.hasAccessCode('crm:customer:update')" class="action-btn" @click="() => handleClaim(row)">领取</a>
        </span>
      </template>
    </Grid>
    <FormDrawer />

    <Drawer v-model:open="detailVisible" :width="1000" placement="right" :destroy-on-close="true" :mask-closable="true" :closable="true" title="客户详情" :body-style="{ padding: 0, maxHeight: 'calc(100vh - 110px)', overflow: 'auto' }" @close="closeDetail">
      <CustomerDetail v-if="detailId" :id="detailId" @edit="handleDetailEdit" />
    </Drawer>
  </Page>
</template>

<style scoped>
.pool-search-card {
  background: #fff;
  border: 1px solid #e8e8e8;
  border-radius: 0;
  padding: 20px 24px 0 24px;
  margin-bottom: 15px;
}
.pool-search-card :deep(.ant-form-item) {
  margin-bottom: 16px;
}
.pool-search-card :deep(.ant-form-item-label) {
  text-align: right;
  padding-right: 12px;
}
.pool-search-form-wrapper {
  width: 100%;
}
@media (min-width: 768px) {
  .pool-search-form-wrapper {
    width: 75%;
  }
}
.pool-search-actions {
  display: flex;
  gap: 12px;
  padding-bottom: 20px;
}
.action-btns {
  display: inline-flex;
  align-items: center;
  gap: 15px;
  font-size: 13px;
}
.action-btn {
  cursor: pointer;
  color: #1677ff;
  line-height: 1;
  text-decoration: none;
}
.action-btn:hover {
  color: #4096ff;
}
/* 固定列内容垂直居中 */
:deep(.vxe-table--fixed-right-wrapper td.vxe-body--column) {
  display: flex !important;
  align-items: center !important;
  justify-content: center !important;
  padding: 0 !important;
}
:deep(.vxe-table--fixed-right-wrapper td.vxe-body--column .vxe-cell) {
  display: inline-block;
}
</style>
