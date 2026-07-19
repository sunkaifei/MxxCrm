<script lang="ts" setup>
import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, ref, computed, onMounted, onUnmounted, watch } from 'vue';

import { Page } from '@vben/common-ui';
import { LucideSearch, LucidePlus, LucideTrash2 } from '@vben/icons';
import { useAccessStore, useUserStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Card, Col, Dropdown, Drawer, Form, Input, Modal, Popconfirm, Row, Select, Tabs, Tag, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteLeadApi, getLeadListApi, addLeadToPoolApi, convertLeadToCustomerApi, performBackgroundCheckApi } from '#/api';
import { $t } from '#/locales';
import LeadDetail from './detail.vue';

const accessStore = useAccessStore();
const userStore = useUserStore();

// data_scope 决定可见的 Tab
// 1=全部数据 → 全部Tab  2=自定义 → my+subordinate+todayFollow
// 3=本部门 → my+todayFollow  4=本部门及以下 → my+subordinate+todayFollow
// 5=仅本人 → my+todayFollow
const dataScope = computed(() => {
  const scope = (userStore.userInfo as any)?.dataScope ?? (userStore.userInfo as any)?.data_scope;
  const roles = userStore.userInfo?.roles ?? [];
  if (roles.includes('super_admin') || roles.includes('system_admin')) return 1;
  return typeof scope === 'number' ? scope : 5;
});

// 列表类型选项卡：我的线索 / 下属线索 / 今日跟进线索
const activeTab = ref('my');
const allTabList = [
  { key: 'my', label: '我的线索' },
  { key: 'subordinate', label: '下属线索' },
  { key: 'todayFollow', label: '今日跟进线索' },
];
// 根据 data_scope 过滤可见的Tab
const tabList = computed(() => {
  const scope = dataScope.value;
  let allowedKeys: string[];
  switch (scope) {
    case 1:
    case 2:
    case 4:
      allowedKeys = ['my', 'subordinate', 'todayFollow'];
      break;
    case 3:
    case 5:
    default:
      allowedKeys = ['my', 'todayFollow'];
      break;
  }
  return allTabList.filter(t => allowedKeys.includes(t.key));
});
// 当Tab权限变化时，确保当前激活的Tab仍然可见
watch(tabList, (newTabs) => {
  const keys = newTabs.map(t => t.key);
  if (!keys.includes(activeTab.value) && keys.length > 0) {
    activeTab.value = keys[0];
  }
}, { immediate: true });

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

// ============ 统一详情抽屉 ============
const detailVisible = ref(false);
const detailId = ref<number | null>(null);
const detailCreate = ref(false);
const detailKey = ref(0);

const windowWidth = ref(window.innerWidth);
function handleResize() {
  windowWidth.value = window.innerWidth;
}
onMounted(() => {
  window.addEventListener('resize', handleResize);
});
onUnmounted(() => {
  window.removeEventListener('resize', handleResize);
});

const drawerWidth = computed(() => {
  if (windowWidth.value < 600) return '100%';
  if (windowWidth.value < 900) return '90%';
  if (windowWidth.value < 1200) return 850;
  return 1100;
});

function openCreate() {
  detailCreate.value = true;
  detailId.value = null;
  detailKey.value++;
  detailVisible.value = true;
}

function openDetail(row: any) {
  const id = row.id ?? row.id_;
  if (!id) { message.error('线索ID不存在'); return; }
  detailCreate.value = false;
  detailId.value = Number(id);
  detailKey.value++;
  detailVisible.value = true;
}

function closeDetail() {
  detailVisible.value = false;
  detailId.value = null;
  detailCreate.value = false;
}

function handleDetailSaved(newId?: number) {
  if (newId && detailCreate.value) {
    // 创建后切换到编辑模式
    detailCreate.value = false;
    detailId.value = newId;
    detailKey.value++;
  }
  gridApi.query();
}

// ============ 表格 ============
const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, export: true, refresh: true, zoom: true },
  height: 'auto',
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true },
  rowConfig: { height: 'auto' },
  stripe: true,
  checkboxConfig: { checkField: 'checked', trigger: 'row' },

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }) => {
        const values = searchForm.value;
        const result = await getLeadListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          listType: activeTab.value,
          companyName: values.companyName || undefined,
          source: values.source,
        });
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
    { type: 'checkbox', width: 50 },
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: '公司名称', field: 'companyName', minWidth: 180, headerAlign: 'center', align: 'left', slots: { default: 'companyName' } },
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
      title: '负责人', field: 'assignee', width: 90,
      formatter: ({ cellValue }: any) => cellValue || '-',
    },
    {
      title: $t('ui.table.createTime'), field: 'createTime', slots: { default: 'createTime' }, width: 160,
    },
    {
      title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 220,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions });

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

async function handleAddToPool(row: any) {
  Modal.confirm({
    title: '退回公海线索',
    content: `确定将线索"${row.companyName}"退回公海线索吗？`,
    onOk: async () => {
      try {
        await addLeadToPoolApi(row.id);
        message.success('已退回公海线索');
        gridApi.query();
      } catch {
        message.error('操作失败');
      }
    },
  });
}

// 验证手机号格式
function isValidMobile(mobile: string): boolean {
  if (!mobile || !mobile.trim()) return false;
  const trimmed = mobile.trim();
  const cnReg = /^1[3-9]\d{9}$/;
  const intlReg = /^\+?[\d\s-]{7,20}$/;
  return cnReg.test(trimmed) || intlReg.test(trimmed);
}

async function handleConvertToCustomer(row: any) {
  if (row.convertedToCustomerId) {
    message.warning('该线索已转为客户');
    return;
  }
  if (!row.companyName || !row.companyName.trim()) {
    message.error('公司名称不能为空');
    return;
  }
  if (!row.contactName || !row.contactName.trim()) {
    message.error('联系人姓名不能为空');
    return;
  }
  if (!row.mobile || !row.mobile.trim()) {
    message.error('联系人手机号不能为空');
    return;
  }
  if (!isValidMobile(row.mobile)) {
    message.error('手机号格式不正确');
    return;
  }

  Modal.confirm({
    title: '一键转客户',
    content: `确定将线索"${row.companyName}"转为客户吗？转换后将自动创建客户和联系人。`,
    onOk: async () => {
      try {
        await convertLeadToCustomerApi(row.id);
        message.success('转客户成功');
        gridApi.query();
      } catch {
        // 全局拦截器会处理错误消息
      }
    },
  });
}

function handleMoreMenuClick({ key }: { key: string }, row: any) {
  switch (key) {
    case 'convert':
      handleConvertToCustomer(row);
      break;
    case 'pool':
      handleAddToPool(row);
      break;
    case 'delete':
      Modal.confirm({
        title: $t('ui.text.do_you_want_delete', { moduleName: $t('page.crm.lead.title') }),
        okText: $t('ui.button.ok'),
        cancelText: $t('ui.button.cancel'),
        onOk: () => handleDelete(row),
      });
      break;
  }
}

async function handleAIAssessment(row: any) {
  if (!row.companyName || !row.companyName.trim()) {
    message.error('公司名称不能为空');
    return;
  }

  console.log(`[AI评估] 开始评估: companyName=${row.companyName}, leadId=${row.id}`);

  const loading = Modal.info({
    title: 'AI评估中',
    content: '正在通过AI获取企业工商信息和风险评估报告，请稍候...',
    closable: false,
  });

  try {
    const res = await performBackgroundCheckApi({
      company_name: row.companyName,
      lead_id: row.id,
    });
    console.log('[AI评估] 后端返回结果:', res);
    loading.destroy();
    message.success('AI评估完成，结果已保存');
    openDetail(row);
  } catch (err: any) {
    loading.destroy();
    const msg = err?.message || err?.msg || '评估失败，请检查API配置是否正常';
    console.error('[AI评估] 失败:', err);
    message.error(msg);
  }
}

function handleDeleteConfirm(row: any) {
  Modal.confirm({
    title: $t('ui.text.do_you_want_delete', { moduleName: $t('page.crm.lead.title') }),
    okText: $t('ui.button.ok'),
    cancelText: $t('ui.button.cancel'),
    onOk: () => handleDelete(row),
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
            @click="openCreate"
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

      <template #action="{ row }">
        <span class="action-btns">
          <a class="action-btn" @click="() => openDetail(row)">详情/跟进</a>
          <Dropdown :trigger="['click']">
            <a class="action-btn more-btn">更多 ▾</a>
            <template #overlay>
              <div class="lead-more-menu">
                <div
                  class="more-menu-item"
                  @click="() => handleAIAssessment(row)"
                >
                  <span>一键评估</span>
                </div>
                <div
                  class="more-menu-item"
                  :class="{ disabled: row.status === 3 || !!row.convertedToCustomerId }"
                  @click="() => ! (row.status === 3 || row.convertedToCustomerId) && handleConvertToCustomer(row)"
                >
                  <span>一键转客户</span>
                </div>
                <div
                  class="more-menu-item"
                  :class="{ disabled: row.status === 4 }"
                  @click="() => row.status !== 4 && handleAddToPool(row)"
                >
                  <span>退回到公海</span>
                </div>
                <div v-if="activeTab !== 'subordinate' && accessStore.hasAccessCode('crm:lead:delete')" class="more-menu-divider" />
                <div
                  v-if="activeTab !== 'subordinate' && accessStore.hasAccessCode('crm:lead:delete')"
                  class="more-menu-item danger"
                  @click="() => handleDeleteConfirm(row)"
                >
                  <span>删除</span>
                </div>
              </div>
            </template>
          </Dropdown>
        </span>
      </template>
    </Grid>

    <!-- 统一详情/新建/编辑/跟进 抽屉 -->
    <Drawer
      v-model:open="detailVisible"
      :width="drawerWidth"
      placement="right"
      :destroy-on-close="false"
      :mask-closable="false"
      :closable="true"
      :title="detailCreate ? '新建线索' : '线索详情'"
      :body-style="{ padding: 0, overflow: 'auto', height: '100%' }"
      @close="closeDetail"
    >
      <LeadDetail
        v-if="detailVisible"
        :key="detailKey"
        :id="detailId"
        :create="detailCreate"
        @saved="handleDetailSaved"
      />
    </Drawer>
  </Page>
</template>

<style scoped>
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
.more-btn {
  white-space: nowrap;
}
.lead-more-menu {
  min-width: 130px;
  background: #fff;
  border: 1px solid #e8e8e8;
  border-radius: 6px;
  box-shadow: 0 3px 10px rgba(0, 0, 0, 0.08);
  padding: 4px 0;
}
.more-menu-item {
  padding: 8px 14px;
  cursor: pointer;
  font-size: 13px;
  color: #333;
  transition: background 0.2s;
}
.more-menu-item:hover {
  background: #f5f5f5;
}
.more-menu-item.danger {
  color: #ff4d4f;
}
.more-menu-item.danger:hover {
  background: #fff1f0;
}
.more-menu-item.disabled {
  color: #bbb;
  cursor: not-allowed;
}
.more-menu-item.disabled:hover {
  background: transparent;
}
.more-menu-divider {
  height: 1px;
  background: #f0f0f0;
  margin: 4px 0;
}
:deep(.vxe-table--fixed-right-wrapper .vxe-body--column .vxe-cell) {
  display: flex !important;
  align-items: center !important;
  justify-content: center !important;
  height: 100% !important;
}
</style>