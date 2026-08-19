<script lang="ts" setup>
import { onMounted, reactive, ref } from 'vue';

import { Page } from '@vben/common-ui';

import {
  Card,
  Input,
  RangePicker,
  Select,
  SelectOption,
  Table,
} from 'ant-design-vue';

import { getAuditListApi } from '#/api/core/system/audit';
import { $t } from '#/locales';

/**
 * 审计日志查询页（append-only 只读）
 * 对应文档：docs/统计性能优化与审计日志开发设计文档.md（5.4）
 */

const loading = ref(false);
const tableData = ref<any[]>([]);
const total = ref(0);

const filters = reactive({
  module: undefined as string | undefined,
  action: undefined as string | undefined,
  keyword: '',
  page: 1,
  page_size: 20,
  range: null as any,
  user_id: undefined as number | undefined,
});

// 模块/动作选项（与后端埋点清单一致）
const moduleOptions = [
  { label: $t('page.system.audit.allModules'), value: '' },
  { label: '合同', value: 'contract' },
  { label: '回款', value: 'payment' },
  { label: '客户', value: 'customer' },
  { label: '权限', value: 'auth' },
];
const actionOptions = [
  { label: $t('page.system.audit.allActions'), value: '' },
  { label: 'create', value: 'create' },
  { label: 'update', value: 'update' },
  { label: 'delete', value: 'delete' },
  { label: 'confirm', value: 'confirm' },
  { label: 'transfer', value: 'transfer' },
  { label: 'grant', value: 'grant' },
];

const columns = [
  {
    title: $t('page.system.audit.time') || '时间',
    dataIndex: 'create_time',
    width: 170,
  },
  {
    title: $t('page.system.audit.user'),
    dataIndex: 'user_name',
    width: 110,
  },
  {
    title: $t('page.system.audit.module'),
    dataIndex: 'module',
    width: 90,
  },
  {
    title: $t('page.system.audit.action'),
    dataIndex: 'action',
    width: 90,
  },
  { title: $t('page.system.audit.summary'), dataIndex: 'summary' },
  {
    title: $t('page.system.audit.target'),
    dataIndex: 'target_id',
    width: 100,
    customRender: ({ record }: any) =>
      `${record.target_type || ''}#${record.target_id || 0}`,
  },
  { title: $t('page.system.audit.ip'), dataIndex: 'ip', width: 130 },
];

const loadData = async () => {
  loading.value = true;
  try {
    const params: any = {
      action: filters.action || undefined,
      keyword: filters.keyword || undefined,
      module: filters.module || undefined,
      page: filters.page,
      page_size: filters.page_size,
      user_id: filters.user_id,
    };
    if (filters.range && filters.range[0] && filters.range[1]) {
      params.start_date =
        filters.range[0].format?.('YYYY-MM-DD') ?? filters.range[0];
      params.end_date =
        filters.range[1].format?.('YYYY-MM-DD') ?? filters.range[1];
    }
    const res: any = await getAuditListApi(params);
    const d = res?.data ?? res;
    tableData.value = d ?? [];
    total.value = res?.meta?.total ?? tableData.value.length;
  } catch (error) {
    console.error('load audit list failed', error);
  } finally {
    loading.value = false;
  }
};

const handleSearch = () => {
  filters.page = 1;
  loadData();
};

const handlePageChange = (page: number, pageSize: number) => {
  filters.page = page;
  filters.page_size = pageSize;
  loadData();
};

const expandedRowRender = (record: any) => {
  const block = (title: string, json: any) =>
    json
      ? `<div style="margin-bottom:8px"><div style="font-weight:600;margin-bottom:4px">${title}</div><pre style="max-height:240px;overflow:auto;background:var(--ant-color-fill-2);padding:8px;border-radius:6px;font-size:12px;margin:0">${JSON.stringify(json, null, 2)}</pre></div>`
      : '';
  return (
    block($t('page.system.audit.before'), record.before_json) +
    block($t('page.system.audit.after'), record.after_json)
  );
};

onMounted(() => {
  loadData();
});
</script>

<template>
  <Page auto-content-height>
    <div class="p-4">
      <h2 class="mb-4 text-lg font-bold">
        {{ $t('page.system.audit.title') }}
      </h2>

      <Card class="mb-4">
        <div class="flex flex-wrap items-center gap-3">
          <Select
            v-model:value="filters.module"
            :placeholder="$t('page.system.audit.module')"
            style="width: 140px"
            @change="handleSearch"
          >
            <SelectOption
              v-for="opt in moduleOptions"
              :key="opt.value"
              :value="opt.value"
            >
              {{ opt.label }}
            </SelectOption>
          </Select>
          <Select
            v-model:value="filters.action"
            :placeholder="$t('page.system.audit.action')"
            style="width: 140px"
            @change="handleSearch"
          >
            <SelectOption
              v-for="opt in actionOptions"
              :key="opt.value"
              :value="opt.value"
            >
              {{ opt.label }}
            </SelectOption>
          </Select>
          <RangePicker
            v-model:value="filters.range"
            :placeholder="[$t('page.system.audit.timeRange'), '']"
            style="width: 260px"
            @change="handleSearch"
          />
          <Input
            v-model:value="filters.keyword"
            :placeholder="$t('page.system.audit.keyword')"
            allow-clear
            style="width: 200px"
            @press-enter="handleSearch"
          />
        </div>
      </Card>

      <Card>
        <Table
          :columns="columns"
          :data-source="tableData"
          :loading="loading"
          :pagination="{
            current: filters.page,
            pageSize: filters.page_size,
            total,
            showSizeChanger: true,
            showTotal: (t: number) => `${t}`,
            onChange: handlePageChange,
            onShowSizeChange: handlePageChange,
          }"
          row-key="id"
          size="small"
        >
          <template #expandedRowRender="{ record }">
            <!-- eslint-disable-next-line vue/no-v-html -- 审计详情结构化渲染，可信来源 -->
            <div v-html="expandedRowRender(record)"></div>
          </template>
        </Table>
      </Card>
    </div>
  </Page>
</template>
