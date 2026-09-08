<script lang="ts" setup>
import type { VxeGridProps } from '#/adapter/vxe-table';

import { computed, h, reactive, ref } from 'vue';

import { Page } from '@vben/common-ui';
import {
  LucideClock,
  LucideHistory,
  LucideMoreHorizontal,
  LucideSearch,
  LucideTrendingUp,
  LucideUndo2,
  LucideUsers,
} from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import {
  Button,
  Drawer,
  Dropdown,
  Form,
  Input,
  Menu,
  MenuItem,
  message,
  Modal,
  Spin,
  Tag,
  Textarea,
  Timeline,
  TimelineItem,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import {
  applyWorkbenchApi,
  claimWorkbenchApi,
  getCustomerPoolStatsMineApi,
  getWorkbenchCustomersApi,
  getWorkbenchPoolsApi,
  getWorkbenchTraceApi,
} from '#/api';
import { PageUsageGuide } from '#/components/PageUsageGuide';
import { $t } from '#/locales';

import CustomerDetailDrawer from '../components/CustomerDetailDrawer.vue';

// 公海客户使用说明步骤数（与 i18n 中 page.crm.customerPool.guide.steps 数组对齐）
const guideStepCount = 5;

const accessStore = useAccessStore();

const sourceLabelMap: Record<number, string> = {
  1: '官网',
  2: '展会',
  3: '社交媒体',
  4: '客户转介',
  5: '陌生拜访',
  6: '海关数据',
  7: '邮件营销',
  8: '阿里国际站',
  9: 'Amazon',
  10: 'TikTok',
  11: '微信',
  12: '其他',
};

const industryLabelMap: Record<number, string> = {
  1: '零售',
  2: '批发',
  3: '制造',
  4: '贸易代理',
  5: '电商',
  6: '微商',
  7: '社交电商',
  8: '其他',
};

const levelColorMap: Record<number, string> = {
  1: 'default',
  2: 'red',
  3: 'orange',
  4: 'blue',
  5: 'green',
};
const levelLabelMap: Record<number, string> = {
  1: '无级别',
  2: '重点客户',
  3: '优质客户',
  4: '普通客户',
  5: '其他',
};

// ===== 池 Tab 与页面级配置 =====
interface MyPool {
  claimMode?: number;
  customerCount?: number;
  id?: number | string;
  isDefault?: number;
  isManager?: boolean;
  name?: string;
}
const pools = ref<MyPool[]>([]);
const activePoolId = ref<number | undefined>(undefined);
const pageMeta = reactive({
  allowDetail: false,
  canClaim: false,
  canViewDetail: false,
  claimMode: undefined as number | undefined,
  maskFields: [] as string[],
  releaseBackTo: 1,
});

const loadPools = async () => {
  try {
    const list = (await getWorkbenchPoolsApi()) as any[];
    pools.value = Array.isArray(list) ? list : [];
  } catch {
    pools.value = [];
  }
};

const activePool = computed(() =>
  pools.value.find((p) => String(p.id) === String(activePoolId.value)),
);
// 单池视角用池上的领取模式；跨池视图用后端回显（可能为空=不展示领取/申领）
const effectiveClaimMode = computed(() => {
  if (activePoolId.value !== undefined) {
    return (
      activePool.value?.claimMode ?? pageMeta.claimMode ?? undefined
    );
  }
  return pageMeta.claimMode ?? undefined;
});
const canClaimRow = computed(
  () => pageMeta.canClaim && effectiveClaimMode.value === 1,
);
const canApplyRow = computed(() => effectiveClaimMode.value === 2);

// ===== 个人公海概览（stats/mine） =====
const statsMine = reactive({
  data: null as null | {
    avgHoldingDays?: number;
    firstResponseAvgHours?: number;
    followupTimelyRate?: number;
    holdingCount?: number;
    monthClaimed?: number;
    monthReleased?: number;
    monthRecycled?: number;
  },
  loaded: false,
});

const loadStatsMine = async () => {
  try {
    const data = (await getCustomerPoolStatsMineApi()) as any;
    if (data && typeof data === 'object') {
      statsMine.data = data;
      statsMine.loaded = true;
    }
  } catch {
    // 无权限或接口异常时静默隐藏概览条
  }
};

const timelyRateText = computed(() => {
  const v = statsMine.data?.followupTimelyRate;
  if (v === undefined || v === null || Number.isNaN(v)) return '-';
  return `${Math.round(v)}%`;
});

const statsCards = computed(() => {
  const d = statsMine.data;
  return [
    {
      icon: LucideUsers,
      key: 'holding',
      label: '在持客户',
      value: d?.holdingCount ?? 0,
      tone: 'primary',
    },
    {
      icon: LucideTrendingUp,
      key: 'claimed',
      label: '本月领取',
      value: d?.monthClaimed ?? 0,
      tone: 'green',
    },
    {
      icon: LucideUndo2,
      key: 'released',
      label: '本月退回',
      value: d?.monthReleased ?? 0,
      tone: 'orange',
    },
    {
      icon: LucideClock,
      key: 'timely',
      label: '跟进及时率',
      value: timelyRateText.value,
      tone: 'blue',
    },
  ];
});

// ===== 脱敏（后端可能已脱敏，本函数幂等） =====
const maskFields = computed(() => pageMeta.maskFields || []);
function maskValue(field: string, val?: string) {
  if (!val) return '-';
  if (!maskFields.value.includes(field)) return val;
  if (field === 'personalEmail') {
    const [name, domain] = val.split('@');
    return `${(name || '').slice(0, 1)}***@${domain || ''}`;
  }
  if (val.length <= 4) return '****';
  return `${val.slice(0, 3)}****${val.slice(-4)}`;
}

// ===== 查询 =====
const searchForm = reactive({ keywords: '' });

// ===== 客户详情 =====
const detailVisible = ref(false);
const detailId = ref<null | number>(null);
function openDetail(row: any) {
  if (!pageMeta.allowDetail || !pageMeta.canViewDetail) {
    message.warning('当前池配置不允许查看客户详情');
    return;
  }
  const id = row.id ?? row.id_;
  if (!id) {
    message.error('客户ID不存在');
    return;
  }
  detailId.value = Number(id);
  detailVisible.value = true;
}
function handleDetailCreated() {
  detailVisible.value = false;
  detailId.value = null;
  gridApi.query();
}

// ===== 列表 =====
const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, refresh: true, zoom: true },
  pagerConfig: {},
  rowConfig: { height: 'auto' as any },
  stripe: true,
  checkboxConfig: { reserve: true },
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }) => {
        const result = (await getWorkbenchCustomersApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          poolId: activePoolId.value,
          keywords: searchForm.keywords || undefined,
        })) as any;
        pageMeta.allowDetail = !!result?.allowDetail;
        pageMeta.canViewDetail = !!result?.canViewDetail;
        pageMeta.canClaim = !!result?.canClaim;
        pageMeta.claimMode = result?.claimMode ?? undefined;
        pageMeta.maskFields = Array.isArray(result?.maskFields)
          ? result.maskFields
          : [];
        pageMeta.releaseBackTo = result?.releaseBackTo ?? 1;
        // 无数据 600px（避免空态列表塌缩），有数据按内容自适应
        const items = (result as any)?.items ?? [];
        const gridEl = gridApi.grid?.$el as HTMLElement | undefined;
        if (gridEl) {
          if (items.length === 0) {
            gridEl.style.setProperty('height', '600px', 'important');
          } else {
            gridEl.style.removeProperty('height');
          }
        }
        return result;
      },
    },
  },
  columns: [
    { type: 'checkbox', width: 46 },
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    {
      title: '公司名称',
      field: 'companyName',
      minWidth: 200,
      slots: { default: 'companyName' },
    },
    { title: '联系人', field: 'personName', width: 100 },
    {
      title: '等级',
      field: 'level',
      width: 96,
      slots: { default: 'level' },
    },
    {
      title: '来源',
      field: 'source',
      width: 100,
      formatter: ({ cellValue }: any) =>
        sourceLabelMap[cellValue] || cellValue || '-',
    },
    {
      title: '行业',
      field: 'industry',
      width: 100,
      formatter: ({ cellValue }: any) =>
        industryLabelMap[cellValue] || cellValue || '-',
    },
    {
      title: '手机',
      field: 'personalMobile',
      width: 136,
      slots: { default: 'personalMobile' },
    },
    {
      title: '邮箱',
      field: 'personalEmail',
      minWidth: 168,
      slots: { default: 'personalEmail' },
    },
    {
      title: '进入公海时间',
      field: 'enteredPoolAt',
      width: 152,
      slots: { default: 'enteredPoolAt' },
    },
    {
      title: '被退回/回收次数',
      field: 'releaseCount',
      width: 110,
      slots: { default: 'releaseCount' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      width: 150,
      slots: { default: 'action' },
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions });

function handleSearch() {
  gridApi.query();
}

function handleReset() {
  searchForm.keywords = '';
  gridApi.query();
}

function handlePoolSelect(key: number | undefined) {
  activePoolId.value = key;
  gridApi.query();
}

// ===== 领取（单个/批量） =====
const selectedRows = computed(() => gridApi.grid?.getCheckboxRecords() || []);

async function doClaim(ids: number[]) {
  if (ids.length === 0) return;
  const results = (await claimWorkbenchApi(ids)) as any[];
  const list = Array.isArray(results) ? results : [];
  const failList = list.filter((r) => !r.success);
  if (list.length > 1 || failList.length > 0) {
    Modal.info({
      title: `领取完成：成功 ${list.length - failList.length} 条，失败 ${failList.length} 条`,
      width: 520,
      content: () =>
        h(
          'div',
          { style: 'max-height:320px;overflow:auto' },
          list.map((r) =>
            h(
              'div',
              { key: r.customerId },
              `客户 #${r.customerId}：${r.success ? '✅' : '❌'} ${r.message || ''}`,
            ),
          ),
        ),
    });
  } else if (list.length === 1 && failList.length === 1) {
    message.error(list[0]?.message || '领取失败');
  } else {
    message.success('领取成功');
  }
  gridApi.query();
  loadPools();
}

function handleClaim(row: any) {
  Modal.confirm({
    title: '领取客户',
    content: `确定领取客户"${row.companyName || row.shortName || row.id}"吗？领取后转入您的私海并开始保护期计时。`,
    onOk: async () => {
      try {
        await doClaim([Number(row.id)]);
      } catch {
        // 错误提示由拦截器处理
      }
    },
  });
}

function handleBatchClaim() {
  const ids = (selectedRows.value || []).map((r: any) => Number(r.id));
  if (ids.length === 0) {
    message.warning('请先勾选要领取的客户');
    return;
  }
  Modal.confirm({
    title: '批量领取',
    content: `确定领取选中的 ${ids.length} 个客户吗？`,
    onOk: async () => {
      try {
        await doClaim(ids);
      } catch {
        // 错误提示由拦截器处理
      }
    },
  });
}

// ===== 申领 =====
const applyVisible = ref(false);
const applyForm = reactive({ customerId: 0, reason: '' });
const applyTargetName = ref('');

function openApply(row: any) {
  applyForm.customerId = Number(row.id);
  applyForm.reason = '';
  applyTargetName.value = row.companyName || row.shortName || '';
  applyVisible.value = true;
}

async function submitApply() {
  try {
    await applyWorkbenchApi({
      customerId: applyForm.customerId,
      reason: applyForm.reason || undefined,
    });
    message.success('申领已提交，等待池管理员审批');
    applyVisible.value = false;
    gridApi.query();
  } catch {
    // 错误提示由拦截器处理
  }
}

// ===== 流水轨迹 =====
const traceVisible = ref(false);
const traceLoading = ref(false);
const traceCustomerName = ref('');
const traceList = ref<any[]>([]);

async function openTrace(row: any) {
  traceCustomerName.value =
    row.companyName || row.shortName || `客户 #${row.id}`;
  traceVisible.value = true;
  traceLoading.value = true;
  try {
    const list = (await getWorkbenchTraceApi(Number(row.id))) as any[];
    traceList.value = Array.isArray(list) ? list : [];
  } catch {
    traceList.value = [];
  } finally {
    traceLoading.value = false;
  }
}

function traceColor(actionType?: number) {
  if (actionType === 1 || actionType === 5) return 'green';
  if (actionType === 2 || actionType === 6) return 'red';
  if (actionType === 9) return 'red';
  if (actionType === 10) return 'green';
  return 'blue';
}

// 初始化
loadPools();
loadStatsMine();
</script>

<template>
  <Page>
    <!-- 个人公海概览 -->
    <div v-if="statsMine.loaded && statsMine.data" class="pool-stats-strip">
      <div v-for="card in statsCards" :key="card.key" class="pool-stat-card">
        <div class="pool-stat-icon" :class="`tone-${card.tone}`">
          <component :is="card.icon" class="pool-stat-svg" />
        </div>
        <div class="pool-stat-body">
          <div class="pool-stat-value">{{ card.value }}</div>
          <div class="pool-stat-label">{{ card.label }}</div>
        </div>
      </div>
    </div>

    <PageUsageGuide
      :title="$t('page.crm.customerPool.guide.title')"
      :brief="$t('page.crm.customerPool.guide.brief')"
      :expand-text="$t('page.crm.customerPool.guide.expand')"
      :collapse-text="$t('page.crm.customerPool.guide.collapse')"
    >
      <div v-for="i in guideStepCount" :key="i" class="page-guide-step-item">
        <div class="page-guide-step-index">{{ i }}</div>
        <div class="page-guide-step-content">
          <div class="page-guide-step-title">
            {{ $t(`page.crm.customerPool.guide.steps[${i - 1}].title`) }}
          </div>
          <div class="page-guide-step-desc">
            {{ $t(`page.crm.customerPool.guide.steps[${i - 1}].desc`) }}
          </div>
        </div>
      </div>
    </PageUsageGuide>

    <!-- 池选择 + 搜索 + 批量操作（筛选卡） -->
    <Card :bordered="false" class="customer-pool-filter-card mb-4">
      <!-- 池选择器：全部 + 各池（横向滚动，数量徽标） -->
      <div class="pool-switcher">
        <button
          class="pool-chip"
          :class="{ active: activePoolId === undefined }"
          type="button"
          @click="() => handlePoolSelect(undefined)"
        >
          <span class="pool-chip-name">全部我的池</span>
          <span class="pool-chip-count">{{
            pools.reduce((s, p) => s + (p.customerCount || 0), 0)
          }}</span>
        </button>
        <button
          v-for="p in pools"
          :key="String(p.id)"
          class="pool-chip"
          :class="{ active: String(p.id) === String(activePoolId) }"
          type="button"
          @click="() => handlePoolSelect(Number(p.id))"
        >
          <span class="pool-chip-name">{{ p.name }}</span>
          <Tag v-if="p.isDefault === 1" class="pool-chip-default" color="blue">
            默认
          </Tag>
          <span class="pool-chip-count">{{ p.customerCount ?? 0 }}</span>
        </button>
      </div>

      <!-- 领取模式提示（轻量一行） -->
      <div class="pool-mode-line">
        <div
          v-if="effectiveClaimMode === 2"
          class="pool-mode-chip info"
        >
          <span class="pool-mode-dot" />
          <span>本池领取需申领审批：勾选目标后提交申领，池管理员审批通过即可成为负责人</span>
        </div>
        <div
          v-else-if="effectiveClaimMode === 3"
          class="pool-mode-chip warning"
        >
          <span class="pool-mode-dot" />
          <span>本池已停用自主领取，客户由管理员分配</span>
        </div>
      </div>

      <div class="pool-toolbar">
        <Form
          :model="searchForm"
          layout="inline"
          class="customer-pool-search-form"
          @submit.prevent="handleSearch"
        >
          <Form.Item>
            <Input
              v-model:value="searchForm.keywords"
              placeholder="公司名 / 简称 / 联系人"
              allow-clear
              class="pool-search-input"
              @press-enter="handleSearch"
            >
              <template #prefix>
                <LucideSearch class="pool-search-icon" />
              </template>
            </Input>
          </Form.Item>
          <Form.Item>
            <Button type="primary" @click="handleSearch">搜索</Button>
          </Form.Item>
          <Form.Item>
            <Button @click="handleReset">重置</Button>
          </Form.Item>
        </Form>

        <div class="pool-toolbar-right">
          <span
            v-if="!canClaimRow && effectiveClaimMode === 1"
            class="pool-claim-hint"
          >
            当前不可领取（可能不为本池成员或已达限额）
          </span>
          <Button
            v-if="accessStore.hasAccessCode('crm:customer:batch-claim')"
            :disabled="!canClaimRow"
            type="primary"
            @click="handleBatchClaim"
          >
            批量领取
          </Button>
        </div>
      </div>
    </Card>

    <Grid
      table-title="公海客户"
      class="customer-pool-grid-card"
    >
      <template #companyName="{ row }">
        <a
          v-if="pageMeta.allowDetail && pageMeta.canViewDetail"
          class="company-name-link"
          @click="() => openDetail(row)"
        >
          {{ row.companyName || row.shortName || '-' }}
        </a>
        <span v-else class="company-name-text">{{
          row.companyName || row.shortName || '-'
        }}</span>
        <div v-if="row.shortName" class="company-name-sub">
          {{ row.shortName }}
        </div>
      </template>

      <template #level="{ row }">
        <Tag :color="levelColorMap[row.level] || 'default'">
          {{ levelLabelMap[row.level] || row.level || '-' }}
        </Tag>
      </template>

      <template #personalMobile="{ row }">
        <span class="contact-cell">{{ maskValue('mobile', row.personalMobile) }}</span>
      </template>

      <template #personalEmail="{ row }">
        <span class="contact-cell">{{ maskValue('email', row.personalEmail) }}</span>
      </template>

      <template #enteredPoolAt="{ row }">
        <span class="time-cell">{{ formatDateTime(row.enteredPoolAt) }}</span>
      </template>

      <template #releaseCount="{ row }">
        <Tag
          :color="(row.releaseCount || 0) > 0 ? 'orange' : 'default'"
        >
          {{ row.releaseCount ?? 0 }}
        </Tag>
      </template>

      <template #action="{ row }">
        <div class="row-actions">
          <Button
            v-if="canClaimRow && accessStore.hasAccessCode('crm:customer:claim')"
            size="small"
            type="primary"
            @click="() => handleClaim(row)"
          >
            领取
          </Button>
          <Button
            v-else-if="
              canApplyRow && accessStore.hasAccessCode('crm:customer:apply')
            "
            size="small"
            type="primary"
            @click="() => openApply(row)"
          >
            申领
          </Button>
          <Dropdown
            v-if="
              accessStore.hasAccessCode('crm:customer:trace') ||
              (pageMeta.allowDetail && pageMeta.canViewDetail)
            "
          >
            <Button class="row-more-btn" size="small" type="text">
              <LucideMoreHorizontal class="row-more-icon" />
            </Button>
            <template #overlay>
              <Menu>
                <MenuItem
                  v-if="accessStore.hasAccessCode('crm:customer:trace')"
                  key="trace"
                  @click="() => openTrace(row)"
                >
                  <span class="menu-item-with-icon">
                    <LucideHistory class="menu-item-icon" />
                    流转轨迹
                  </span>
                </MenuItem>
                <MenuItem
                  v-if="pageMeta.allowDetail && pageMeta.canViewDetail"
                  key="detail"
                  @click="() => openDetail(row)"
                >
                  <span class="menu-item-with-icon">
                    <LucideUsers class="menu-item-icon" />
                    客户详情
                  </span>
                </MenuItem>
              </Menu>
            </template>
          </Dropdown>
        </div>
      </template>

      <template #empty>
        <div class="pool-empty">
          <div class="pool-empty-icon">
            <LucideSearch />
          </div>
          <div class="pool-empty-title">
            {{
              searchForm.keywords
                ? '未找到匹配的公海客户'
                : '当前公海暂无客户'
            }}
          </div>
          <div class="pool-empty-desc">
            {{
              searchForm.keywords
                ? '试试更换关键词或切换公海池'
                : '客户被退回后会自动进入公海，可在此认领'
            }}
          </div>
          <Button
            v-if="searchForm.keywords"
            size="small"
            type="primary"
            @click="handleReset"
          >
            清除搜索
          </Button>
        </div>
      </template>
    </Grid>

    <!-- 申领弹窗 -->
    <Modal
      v-model:visible="applyVisible"
      title="提交申领"
      @ok="submitApply"
    >
      <div class="apply-target">
        申领客户：
        <span class="font-medium">{{ applyTargetName }}</span>
        <span class="apply-target-tip">审批通过后自动成为负责人</span>
      </div>
      <Textarea
        v-model:value="applyForm.reason"
        :rows="3"
        placeholder="申领理由（可选）"
      />
    </Modal>

    <!-- 流水轨迹抽屉 -->
    <Drawer
      v-model:visible="traceVisible"
      :title="`公海流水轨迹 - ${traceCustomerName}`"
      width="min(520px, 92vw)"
    >
      <Spin :spinning="traceLoading">
        <Timeline v-if="traceList.length > 0">
          <TimelineItem
            v-for="item in traceList"
            :key="item.id"
            :color="traceColor(item.actionType)"
          >
            <div class="font-medium">
              {{ item.actionLabel || '操作' }}
              <span class="ml-1 text-xs text-gray-400">
                {{ item.poolName || '' }}
              </span>
            </div>
            <div class="text-xs text-gray-500">
              <div v-if="item.adminName">负责人：{{ item.adminName }}</div>
              <div v-if="item.operatedByName">操作人：{{ item.operatedByName }}</div>
              <div v-if="item.startTime">
                归属开始：{{ formatDateTime(item.startTime) }}
              </div>
              <div v-if="item.endTime">
                归属结束：{{ formatDateTime(item.endTime) }}
              </div>
              <div v-if="item.reason">原因：{{ item.reason }}</div>
              <div>记录时间：{{ formatDateTime(item.createTime) }}</div>
            </div>
          </TimelineItem>
        </Timeline>
        <div v-else-if="!traceLoading" class="py-8 text-center text-gray-400">
          暂无轨迹记录
        </div>
      </Spin>
    </Drawer>

    <CustomerDetailDrawer
      v-model:visible="detailVisible"
      :id="detailId ?? undefined"
      @created="handleDetailCreated"
    />
  </Page>
</template>

<style scoped>
/* ===== 个人概览条 ===== */
.pool-stats-strip {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12px;
  margin-bottom: 16px;
}

.pool-stat-card {
  display: flex;
  gap: 12px;
  align-items: center;
  padding: 14px 16px;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 12px;
  box-shadow: 0 1px 2px hsl(0deg 0% 0% / 3%);
  transition:
    border-color 0.2s ease,
    transform 0.2s ease,
    box-shadow 0.2s ease;
}

.pool-stat-card:hover {
  border-color: hsl(var(--primary) / 30%);
  box-shadow: 0 4px 12px hsl(var(--primary) / 8%);
  transform: translateY(-1px);
}

.pool-stat-icon {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  border-radius: 10px;
}

.pool-stat-svg {
  width: 18px;
  height: 18px;
}

.pool-stat-icon.tone-primary {
  color: hsl(var(--primary));
  background: hsl(var(--primary) / 10%);
}

.pool-stat-icon.tone-green {
  color: hsl(152deg 69% 31%);
  background: hsl(152deg 69% 31% / 10%);
}

.pool-stat-icon.tone-orange {
  color: hsl(27deg 90% 45%);
  background: hsl(27deg 90% 45% / 10%);
}

.pool-stat-icon.tone-blue {
  color: hsl(221deg 83% 53%);
  background: hsl(221deg 83% 53% / 10%);
}

.pool-stat-body {
  min-width: 0;
}

.pool-stat-value {
  font-size: 20px;
  font-weight: 700;
  line-height: 1.2;
  color: hsl(var(--foreground));
  letter-spacing: -0.02em;
}

.pool-stat-label {
  margin-top: 2px;
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

/* ===== 筛选卡片 ===== */
.customer-pool-filter-card {
  margin-bottom: 16px;
}

.customer-pool-grid-card {
  margin-top: 16px;
}

/* ===== 池选择器 ===== */
.pool-switcher {
  display: flex;
  gap: 8px;
  padding-bottom: 12px;
  margin-bottom: 12px;
  overflow-x: auto;
  border-bottom: 1px solid hsl(var(--border));
  scrollbar-width: thin;
}

.pool-chip {
  display: inline-flex;
  flex-shrink: 0;
  gap: 8px;
  align-items: center;
  padding: 6px 14px;
  font-size: 13px;
  line-height: 1;
  color: hsl(var(--muted-foreground));
  background: hsl(var(--muted));
  border: 1px solid transparent;
  border-radius: 999px;
  cursor: pointer;
  transition:
    background 0.2s ease,
    color 0.2s ease,
    border-color 0.2s ease,
    box-shadow 0.2s ease;
}

.pool-chip:hover {
  color: hsl(var(--foreground));
  border-color: hsl(var(--primary) / 30%);
}

.pool-chip.active {
  color: hsl(var(--primary));
  font-weight: 600;
  background: hsl(var(--primary) / 10%);
  border-color: hsl(var(--primary) / 40%);
  box-shadow: 0 0 0 3px hsl(var(--primary) / 8%);
}

.pool-chip-name {
  max-width: 180px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pool-chip-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 20px;
  height: 18px;
  padding: 0 6px;
  font-size: 11px;
  font-weight: 600;
  color: hsl(var(--muted-foreground));
  background: hsl(var(--card));
  border-radius: 999px;
}

.pool-chip.active .pool-chip-count {
  color: hsl(var(--primary));
  background: hsl(var(--card));
}

.pool-chip-default {
  height: 16px;
  margin: 0;
  padding: 0 6px;
  font-size: 11px;
  line-height: 16px;
}

/* ===== 领取模式提示 ===== */
.pool-mode-line {
  display: flex;
  align-items: center;
  margin-bottom: 12px;
}

.pool-mode-chip {
  display: inline-flex;
  gap: 8px;
  align-items: center;
  padding: 6px 12px;
  font-size: 12px;
  line-height: 1.5;
  border-radius: 8px;
}

.pool-mode-chip.info {
  color: hsl(221deg 83% 45%);
  background: hsl(221deg 83% 53% / 8%);
}

.pool-mode-chip.warning {
  color: hsl(27deg 90% 42%);
  background: hsl(27deg 90% 45% / 8%);
}

.pool-mode-dot {
  flex-shrink: 0;
  width: 6px;
  height: 6px;
  background: currentcolor;
  border-radius: 50%;
}

/* ===== 工具条 ===== */
.pool-toolbar {
  display: flex;
  gap: 16px;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
}

.customer-pool-search-form :deep(.ant-form-item) {
  margin-bottom: 0;
}

.customer-pool-search-form :deep(.ant-form-item-control) {
  flex: 1;
}

.pool-search-input {
  width: 260px;
}

.pool-search-icon {
  width: 14px;
  height: 14px;
  color: hsl(var(--muted-foreground));
}

.pool-toolbar-right {
  display: flex;
  gap: 12px;
  align-items: center;
}

.pool-claim-hint {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

/* ===== 表格单元格 ===== */
.company-name-link {
  font-weight: 600;
  color: hsl(var(--primary));
  text-decoration: none;
  transition: opacity 0.15s ease;
}

.company-name-link:hover {
  text-decoration: underline;
  text-underline-offset: 3px;
  opacity: 0.85;
}

.company-name-text {
  font-weight: 600;
  color: hsl(var(--foreground));
}

.company-name-sub {
  margin-top: 2px;
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.contact-cell {
  font-variant-numeric: tabular-nums;
  color: hsl(var(--foreground) / 85%);
}

.time-cell {
  font-variant-numeric: tabular-nums;
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

/* ===== 行内操作 ===== */
.row-actions {
  display: inline-flex;
  gap: 6px;
  align-items: center;
}

.row-more-btn {
  width: 24px;
  height: 24px;
  padding: 0;
  color: hsl(var(--muted-foreground));
}

.row-more-icon {
  width: 16px;
  height: 16px;
}

.menu-item-with-icon {
  display: inline-flex;
  gap: 8px;
  align-items: center;
}

.menu-item-icon {
  width: 14px;
  height: 14px;
}

/* ===== 空态 ===== */
.pool-empty {
  display: flex;
  flex-direction: column;
  gap: 8px;
  align-items: center;
  padding: 48px 16px;
}

.pool-empty-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 48px;
  height: 48px;
  color: hsl(var(--muted-foreground));
  background: hsl(var(--muted));
  border-radius: 50%;
}

.pool-empty-icon :deep(svg) {
  width: 22px;
  height: 22px;
}

.pool-empty-title {
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--foreground));
}

.pool-empty-desc {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

/* ===== 申领弹窗 ===== */
.apply-target {
  padding: 10px 12px;
  margin-bottom: 12px;
  font-size: 13px;
  background: hsl(var(--muted));
  border-radius: 8px;
}

.apply-target-tip {
  margin-left: 8px;
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

/* ===== 固定列居中 ===== */
:deep(.vxe-table--fixed-right-wrapper .vxe-body--column .vxe-cell) {
  display: flex !important;
  align-items: center !important;
  justify-content: center !important;
  height: 100% !important;
}

/* ===== 响应式 ===== */
@media (max-width: 1100px) {
  .pool-stats-strip {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 640px) {
  .pool-stats-strip {
    grid-template-columns: 1fr;
  }

  .pool-search-input {
    width: 100%;
  }

  .pool-toolbar {
    align-items: stretch;
    flex-direction: column;
  }

  .pool-toolbar-right {
    justify-content: flex-end;
  }
}
</style>
