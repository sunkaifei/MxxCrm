<script lang="ts" setup>
import { ref } from 'vue';

import { Page } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { formatDateTime } from '@vben/utils';

import { Button, Drawer, Empty, Tag, Timeline } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { getEditLogListApi } from '#/api/core/system/edit-log';

defineOptions({ name: 'EditLog' });

// 业务类型映射
const businessTypeMap: Record<number, { label: string; color: string }> = {
  1: { label: '报价单', color: 'blue' },
  2: { label: '订单', color: 'green' },
  3: { label: '合同', color: 'purple' },
};

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Select',
      fieldName: 'businessType',
      label: '业务类型',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: [
          { label: '报价单', value: 1 },
          { label: '订单', value: 2 },
          { label: '合同', value: 3 },
        ],
      },
    },
    {
      component: 'Input',
      fieldName: 'keyword',
      label: '关键词',
      componentProps: {
        placeholder: '编号/标题/操作人',
        allowClear: true,
      },
    },
  ],
};

// 详情抽屉
const detailVisible = ref(false);
const currentLog = ref<any>(null);

function openDetail(row: any) {
  currentLog.value = row;
  detailVisible.value = true;
}

const gridOptions: VxeGridProps = {
  toolbarConfig: {
    custom: true,
    export: true,
    refresh: true,
    zoom: true,
  },
  height: 'auto',
  exportConfig: {},
  pagerConfig: {},
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getEditLogListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          businessType: formValues.businessType,
          keyword: formValues.keyword,
        });
      },
    },
  },

  columns: [
    {
      title: '序号',
      type: 'seq',
      width: 70,
    },
    {
      title: '业务类型',
      field: 'businessType',
      width: 100,
      slots: { default: 'businessType' },
    },
    {
      title: '业务编号',
      field: 'businessNo',
      width: 180,
    },
    {
      title: '业务标题',
      field: 'businessTitle',
      minWidth: 200,
    },
    {
      title: '操作人',
      field: 'editorName',
      width: 120,
    },
    {
      title: '修改字段数',
      field: 'content',
      width: 100,
      slots: { default: 'fieldCount' },
    },
    {
      title: '操作时间',
      field: 'editTime',
      width: 180,
      slots: { default: 'editTime' },
    },
    {
      title: '操作',
      field: 'action',
      width: 100,
      fixed: 'right',
      slots: { default: 'action' },
    },
  ],
};

const [Grid] = useVbenVxeGrid({ gridOptions, formOptions });
</script>

<template>
  <Page auto-content-height>
    <Grid>
      <template #businessType="{ row }">
        <Tag :color="businessTypeMap[row.businessType]?.color || 'default'">
          {{ businessTypeMap[row.businessType]?.label || '-' }}
        </Tag>
      </template>

      <template #fieldCount="{ row }">
        <Tag color="blue">
          {{ Array.isArray(row.content) ? row.content.length : 0 }}
        </Tag>
      </template>

      <template #editTime="{ row }">
        {{ row.editTime ? formatDateTime(row.editTime) : '-' }}
      </template>

      <template #action="{ row }">
        <Button type="link" size="small" @click="openDetail(row)">
          查看详情
        </Button>
      </template>
    </Grid>
  </Page>

  <!-- 修改详情抽屉 -->
  <Drawer
    v-model:open="detailVisible"
    title="修改详情"
    width="560px"
    :footer="null"
  >
    <div v-if="currentLog" class="edit-log-detail">
      <div class="detail-header">
        <div class="flex items-center gap-2 mb-2">
          <Tag :color="businessTypeMap[currentLog.businessType]?.color || 'default'">
            {{ businessTypeMap[currentLog.businessType]?.label || '-' }}
          </Tag>
          <span class="text-sm text-gray-500">{{ currentLog.businessNo }}</span>
        </div>
        <div class="text-base font-semibold text-gray-800 mb-1">
          {{ currentLog.businessTitle || '-' }}
        </div>
        <div class="text-xs text-gray-400">
          操作人：{{ currentLog.editorName || '未知' }}
          <span class="mx-2">·</span>
          {{ currentLog.editTime ? formatDateTime(currentLog.editTime) : '-' }}
        </div>
      </div>

      <div class="detail-timeline">
        <div class="text-sm font-semibold text-gray-700 mb-3">
          修改字段（{{ currentLog.content?.length || 0 }}项）
        </div>
        <Timeline v-if="currentLog.content && currentLog.content.length > 0">
          <Timeline.Item
            v-for="(item, idx) in currentLog.content"
            :key="idx"
            color="blue"
          >
            <div class="change-item">
              <div class="change-field">
                <Tag color="blue">{{ item.fieldLabel }}</Tag>
              </div>
              <div class="change-values mt-1">
                <template v-if="item.old && item.new">
                  <span class="old-val">{{ item.old }}</span>
                  <span class="arrow">→</span>
                  <span class="new-val">{{ item.new }}</span>
                </template>
                <template v-else-if="item.new">
                  <span class="text-green-600">新增：{{ item.new }}</span>
                </template>
                <template v-else-if="item.old">
                  <span class="text-red-500">删除：{{ item.old }}</span>
                </template>
              </div>
            </div>
          </Timeline.Item>
        </Timeline>
        <Empty v-else description="暂无修改内容" />
      </div>
    </div>
  </Drawer>
</template>

<style scoped>
.edit-log-detail {
  padding: 0;
}
.detail-header {
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-color-base, #f0f0f0);
  margin-bottom: 16px;
}
.change-item {
  padding: 6px 0;
}
.change-field {
  margin-bottom: 2px;
}
.change-values {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  padding-left: 4px;
}
.old-val {
  color: var(--text-color-secondary, #999);
  text-decoration: line-through;
}
.arrow {
  color: var(--text-color-secondary, #999);
}
.new-val {
  color: #52c41a;
  font-weight: 500;
}
</style>
