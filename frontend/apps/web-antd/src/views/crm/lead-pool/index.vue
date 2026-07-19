<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';
import type { VxeGridProps } from '#/adapter/vxe-table';

import { ref } from 'vue';

import { Page } from '@vben/common-ui';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Drawer, Modal, Popconfirm, Tag, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { getLeadPoolListApi, claimLeadApi, deleteLeadPoolApi } from '#/api';
import { $t } from '#/locales';
import LeadDetail from '../lead/detail.vue';

const accessStore = useAccessStore();

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
const statusColorMap: Record<number, string> = {
  1: 'blue', 2: 'cyan', 3: 'green', 4: 'default',
  5: 'orange', 6: 'default', 7: 'processing', 8: 'success',
};

// ============ 统一详情抽屉 ============
const detailVisible = ref(false);
const detailId = ref<number | null>(null);
const detailCreate = ref(false);
const detailKey = ref(0);

function openDetail(row: any) {
  const id = row.id ?? row.id_;
  if (!id) { message.error('线索ID不存在'); return; }
  detailCreate.value = false;
  detailId.value = Number(id);
  detailKey.value++;
  detailVisible.value = true;
}

function openEdit(row: any) {
  openDetail(row);
}

function openCreate() {
  detailCreate.value = true;
  detailId.value = null;
  detailKey.value++;
  detailVisible.value = true;
}

function closeDetail() {
  detailVisible.value = false;
  detailId.value = null;
  detailCreate.value = false;
}

function handleDetailSaved() {
  gridApi.query();
}

async function handleClaim(row: any) {
  Modal.confirm({
    title: '领取线索',
    content: `确定领取线索"${row.companyName}"吗？领取后将转为您的客户。`,
    onOk: async () => {
      try {
        await claimLeadApi(row.id);
        message.success('领取成功，已转为客户');
        gridApi.query();
      } catch {
        // 错误提示由 requestClient 拦截器处理，无需重复提示
      }
    },
  });
}

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'companyName',
      label: '公司名称',
      componentProps: { placeholder: '输入公司名称搜索', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'source',
      label: '来源',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: [
          { label: '官网', value: 'website' },
          { label: '展会', value: 'exhibition' },
          { label: '社交媒体', value: 'social' },
          { label: '客户转介', value: 'referral' },
          { label: '陌生拜访', value: 'cold_call' },
          { label: '海关数据', value: 'customs' },
          { label: '邮件营销', value: 'email' },
          { label: '阿里国际站', value: 'alibaba' },
          { label: 'Amazon', value: 'amazon' },
          { label: 'TikTok', value: 'tiktok' },
          { label: '微信', value: 'wechat' },
          { label: '其他', value: 'other' },
        ],
      },
    },
    {
      component: 'Input',
      fieldName: 'industry',
      label: '行业',
      componentProps: { placeholder: '输入行业', allowClear: true },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, export: true, refresh: true, zoom: true },
  exportConfig: {},
  pagerConfig: {},
  cellConfig: { isHover: true },
  rowConfig: { height: 'auto' },
  stripe: true,
  checkboxConfig: { checkField: 'checked', trigger: 'row' },

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        const result = await getLeadPoolListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          listType: 'pool',
          ...formValues,
        });
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
    { type: 'checkbox', width: 50 },
    { title: $t('ui.table.seq'), type: 'seq', width: 60, headerAlign: 'center' },
    { title: '公司名称', field: 'companyName', minWidth: 180, headerAlign: 'center', align: 'left', slots: { default: 'companyName' } },
    { title: '联系人', field: 'contactName', width: 100, headerAlign: 'center' },
    {
      title: '来源', field: 'source', width: 100, headerAlign: 'center',
      formatter: ({ cellValue }: any) => sourceLabelMap[cellValue] || cellValue || '-',
    },
    {
      title: '状态', field: 'status', width: 90, headerAlign: 'center',
      slots: { default: 'status' },
    },
    { title: '行业', field: 'industry', width: 90, headerAlign: 'center', formatter: ({ cellValue }: any) => industryLabelMap[cellValue] || cellValue || '-' },
    { title: '国家', field: 'country', width: 80, headerAlign: 'center' },
    { title: '创建人', field: 'createdByName', width: 90, headerAlign: 'center' },
    {
      title: $t('ui.table.createTime'), field: 'createTime', slots: { default: 'createdAt' }, width: 160, headerAlign: 'center',
    },
    {
      title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 240, headerAlign: 'center',
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteLeadPoolApi([row.id]);
    message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}
</script>

<template>
  <Page>
    <Grid :table-title="$t('page.crm.leadPool.title')">
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('crm:lead:create')"
          type="primary"
          class="mr-2"
          @click="openCreate"
        >
          {{ $t('page.crm.leadPool.button.create') }}
        </Button>
      </template>

      <template #createdAt="{ row }">{{ formatDateTime(row.createTime) }}</template>

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

      <template #status="{ row }">
        <Tag :color="statusColorMap[row.status] || 'default'">{{ statusLabelMap[row.status] || row.status || '-' }}</Tag>
      </template>

      <template #action="{ row }">
        <span class="action-btns">
          <a class="action-btn" @click="() => handleClaim(row)">领取</a>
          <a
            v-if="accessStore.hasAccessCode('crm:lead:edit')"
            class="action-btn"
            @click="() => openEdit(row)"
          >编辑</a>
          <a class="action-btn" @click="() => openDetail(row)">详情</a>
          <Popconfirm
            :title="$t('ui.text.do_you_want_delete', { moduleName: $t('page.crm.leadPool.title') })"
            :ok-text="$t('ui.button.ok')"
            :cancel-text="$t('ui.button.cancel')"
            @confirm="handleDelete(row)"
          >
            <a
              v-if="accessStore.hasAccessCode('crm:lead-pool:delete')"
              class="action-btn danger"
            >删除</a>
          </Popconfirm>
        </span>
      </template>
    </Grid>

    <!-- 统一详情/新建/编辑/跟进 抽屉 -->
    <Drawer
      v-model:open="detailVisible"
      :width="1100"
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
.action-btn.danger {
  color: #ff4d4f;
}
.action-btn.danger:hover {
  color: #ff7875;
}
:deep(.vxe-table--fixed-right-wrapper .vxe-body--column .vxe-cell) {
  display: flex !important;
  align-items: center !important;
  justify-content: center !important;
  height: 100% !important;
}
</style>