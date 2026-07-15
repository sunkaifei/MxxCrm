<script lang="ts" setup>
import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideSearch, LucidePlus, LucideFilePenLine, LucideTrash2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Card, Col, Drawer, Form, Input, Modal, Popconfirm, Row, Select, Tabs, Tag, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteLeadApi, getLeadListApi, addLeadToPoolApi } from '#/api';
import { $t } from '#/locales';
import LeadDrawerComp from './drawer.vue';
import LeadDetail from './detail.vue';
import LeadFollowupDrawer from './followup-drawer.vue';

const accessStore = useAccessStore();

// 列表类型选项卡：我的线索 / 下属线索 / 今日跟进线索
const activeTab = ref('my');
const tabList = [
  { key: 'my', label: '我的线索' },
  { key: 'subordinate', label: '下属线索' },
  { key: 'todayFollow', label: '今日跟进线索' },
];

function handleTabChange(key: string) {
  activeTab.value = key;
  gridApi.query();
}

// 搜索表单
const searchForm = ref({
  companyName: '',
  source: undefined as string | undefined,
});

function handleSearch() {
  gridApi.query();
}

function handleReset() {
  searchForm.value = {
    companyName: '',
    source: undefined,
  };
  gridApi.query();
}

const sourceLabelMap: Record<string, string> = {
  website: '官网', exhibition: '展会', social: '社交媒体', referral: '客户转介',
  cold_call: '陌生拜访', customs: '海关数据', email: '邮件营销', alibaba: '阿里国际站',
  amazon: 'Amazon', tiktok: 'TikTok', wechat: '微信', other: '其他',
};

const industryLabelMap: Record<number, string> = {
  1: '零售', 2: '批发', 3: '制造', 4: '贸易代理',
  5: '电商', 6: '微商', 7: '社交电商', 8: '其他',
};

const statusLabelMap: Record<number, string> = {
  1: '新客', 2: '跟进中', 3: '已成交', 4: '无效线索',
  5: '已回收', 6: '未核查', 7: '核查中', 8: '有效线索',
};

const detailVisible = ref(false);
const detailId = ref<number | null>(null);

const followupVisible = ref(false);
const followupId = ref<number | null>(null);

function openDetail(row: any) {
  const id = row.id ?? row.id_;
  if (!id) { message.error('线索ID不存在'); return; }
  detailId.value = Number(id);
  detailVisible.value = true;
}
function closeDetail() { detailVisible.value = false; detailId.value = null; }
function handleDetailEdit(lead: any) { closeDetail(); openDrawer(false, lead); }

function openFollowup(row: any) {
  const id = row.id ?? row.id_;
  if (!id) { message.error('线索ID不存在'); return; }
  followupId.value = Number(id);
  followupVisible.value = true;
}
function closeFollowup() { followupVisible.value = false; followupId.value = null; }

const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, export: true, refresh: true, zoom: true },
  height: 'auto',
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true },
  stripe: true,
  checkboxConfig: { checkField: 'checked', trigger: 'row' },

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }) => {
        const values = searchForm.value;
        return await getLeadListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          listType: activeTab.value,
          companyName: values.companyName || undefined,
          source: values.source,
        });
      },
    },
  },

  columns: [
    { type: 'checkbox', width: 50 },
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: '公司名称', field: 'companyName', minWidth: 180, slots: { default: 'companyName' } },
    { title: '所属行业', field: 'industry', width: 100, formatter: ({ cellValue }: any) => industryLabelMap[cellValue] || cellValue || '-' },
    { title: '联系人', field: 'contactName', width: 100 },
    { title: '状态', field: 'status', width: 90, formatter: ({ cellValue }: any) => statusLabelMap[cellValue] || cellValue || '-' },
    {
      title: '来源', field: 'source', width: 100,
      formatter: ({ cellValue }: any) => sourceLabelMap[cellValue] || cellValue || '-',
    },
    { title: '邮箱', field: 'email', width: 160 },
    { title: '手机', field: 'mobile', width: 130 },
    { title: '国家', field: 'country', width: 80 },
    {
      title: $t('ui.table.createTime'), field: 'createTime', slots: { default: 'createTime' }, width: 160,
    },
    {
      title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 280,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions });

const [FormDrawer, drawerApi] = useVbenDrawer({
  connectedComponent: LeadDrawerComp,
  onClosed() { if (drawerApi.getData()?.needRefresh) gridApi.query(); },
});

function openDrawer(create: boolean, row?: any) { drawerApi.setData({ create, row }); drawerApi.open(); }
function handleCreate() { openDrawer(true); }
function handleEdit(row: any) { openDrawer(false, row); }

async function handleDelete(row: any) {
  row.pending = true;
  try { await deleteLeadApi([row.id]); message.success($t('ui.notification.delete_success')); }
  finally { row.pending = false; gridApi.query(); }
}

async function handleBatchDelete() {
  const records = gridApi.grid?.getCheckboxRecords();
  if (!records?.length) { message.warning('请先选择要删除的线索'); return; }
  Modal.confirm({
    title: '批量删除',
    content: `确定批量删除 ${records.length} 条线索？`,
    onOk: async () => {
      try {
        const ids = records.map((r: any) => r.id);
        await deleteLeadApi(ids);
        message.success(`已删除 ${records.length} 条线索`);
        gridApi.query();
      } catch { /* ignore */ }
    },
  });
}

async function handleFollow(row: any) {
  openFollowup(row);
}

async function handleAddToPool(row: any) {
  Modal.confirm({
    title: '退回公海线索',
    content: `确定将线索"${row.companyName}"退回公海线索吗？`,
    onOk: async () => {
      try {
        await addLeadToPoolApi(row.id);
        message.success('已退回公海线索');
        gridApi.query();
      } catch (e) {
        message.error('操作失败');
      }
    },
  });
}
</script>

<template>
  <Page auto-content-height>
    <Card :bordered="false" class="mb-[15px]">
      <Tabs v-model:activeKey="activeTab" @change="handleTabChange" class="mb-4">
        <Tabs.TabPane v-for="tab in tabList" :key="tab.key" :tab="tab.label" />
      </Tabs>

      <Form :model="searchForm" :label-col="{ style: { width: '90px' } }">
        <Row :gutter="[16, 12]" style="width: 100%">
          <Col :xs="24" :sm="24" :md="12">
            <Form.Item label="公司名称" name="companyName">
              <Input v-model:value="searchForm.companyName" placeholder="输入公司名称搜索" allow-clear style="width: 100%" />
            </Form.Item>
          </Col>
          <Col :xs="24" :sm="24" :md="12">
            <Form.Item label="来源" name="source">
              <Select v-model:value="searchForm.source" placeholder="全部" allow-clear style="width: 100%">
                <Select.Option value="website">官网</Select.Option>
                <Select.Option value="exhibition">展会</Select.Option>
                <Select.Option value="social">社交媒体</Select.Option>
                <Select.Option value="referral">客户转介</Select.Option>
                <Select.Option value="cold_call">陌生拜访</Select.Option>
                <Select.Option value="customs">海关数据</Select.Option>
                <Select.Option value="email">邮件营销</Select.Option>
                <Select.Option value="alibaba">阿里国际站</Select.Option>
                <Select.Option value="amazon">Amazon</Select.Option>
                <Select.Option value="tiktok">TikTok</Select.Option>
                <Select.Option value="wechat">微信</Select.Option>
                <Select.Option value="other">其他</Select.Option>
              </Select>
            </Form.Item>
          </Col>
        </Row>

        <div class="flex flex-wrap items-center gap-2 mt-3" style="margin-left: 90px">
          <Button type="default" :icon="h(LucideSearch)" @click="handleSearch">搜索</Button>
          <Button type="default" @click="handleReset">刷新</Button>
          <Button
            v-if="accessStore.hasAccessCode('crm:lead:create')"
            type="primary"
            :icon="h(LucidePlus)"
            @click="handleCreate"
          >
            新增线索
          </Button>
        </div>
      </Form>
    </Card>

    <Grid :table-title="$t('page.crm.lead.title')" style="margin-top: 15px">
      <template #toolbar-tools>
        <Button @click="handleBatchDelete" class="mr-2" danger ghost>批量删除</Button>
      </template>

      <template #createTime="{ row }">{{ formatDateTime(row.createTime) }}</template>

      <template #companyName="{ row }">
        <a class="cursor-pointer text-blue-600 hover:text-blue-800" @click="() => openDetail(row)">{{ row.companyName }}</a>
      </template>

      <template #action="{ row }">
        <Button type="link" @click="() => handleFollow(row)">跟进</Button>
        <Button v-if="row.status !== 4" type="link" @click="() => handleAddToPool(row)">退回公海线索</Button>
        <Button v-if="accessStore.hasAccessCode('crm:lead:edit')" type="link" :icon="h(LucideFilePenLine)" @click="() => handleEdit(row)" title="编辑" />
        <Popconfirm v-if="activeTab !== 'subordinate' && accessStore.hasAccessCode('crm:lead:delete')" :title="$t('ui.text.do_you_want_delete', { moduleName: $t('page.crm.lead.title') })" :ok-text="$t('ui.button.ok')" :cancel-text="$t('ui.button.cancel')" @confirm="handleDelete(row)">
          <Button type="link" danger :icon="h(LucideTrash2)" title="删除" />
        </Popconfirm>
      </template>
    </Grid>
    <FormDrawer />

    <Drawer v-model:open="detailVisible" :width="960" placement="right" :destroy-on-close="true" :mask-closable="true" :closable="true" title="线索详情" :body-style="{ padding: 0, maxHeight: 'calc(100vh - 110px)', overflow: 'auto' }" @close="closeDetail">
      <LeadDetail v-if="detailId" :id="detailId" @edit="handleDetailEdit" />
    </Drawer>

    <Drawer v-model:open="followupVisible" :width="1100" placement="right" :destroy-on-close="true" :mask-closable="true" :closable="true" title="跟进线索" :body-style="{ padding: 0, height: '100%' }" @close="closeFollowup">
      <LeadFollowupDrawer v-if="followupId" :id="followupId" />
    </Drawer>
  </Page>
</template>