<script lang="ts" setup>
import { h } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideSend, LucideTrash2, LucideUndo2 } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Popconfirm, Tag } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import {
  deleteNoticeApi,
  getNoticeListApi,
  publishNoticeApi,
  revokeNoticeApi,
  NOTICE_PUBLISH_STATUS,
  NOTICE_PUBLISH_STATUS_OPTIONS,
  NOTICE_TYPE_OPTIONS,
} from '#/api';
import { $t } from '#/locales';

import NoticeDrawer from './drawer.vue';

const accessStore = useAccessStore();

// 类型/目标类型/状态 → 标签 + 颜色 的映射
const TYPE_MAP: Record<number, { label: string; color: string }> = {
  1: { label: '通知', color: 'blue' },
  2: { label: '公告', color: 'purple' },
  3: { label: '系统消息', color: 'cyan' },
};

const TARGET_TYPE_MAP: Record<number, { label: string; color: string }> = {
  1: { label: '全体', color: 'green' },
  2: { label: '指定', color: 'orange' },
};

const STATUS_MAP: Record<number, { label: string; color: string }> = {
  [NOTICE_PUBLISH_STATUS.UNPUBLISHED]: { label: '未发布', color: 'default' },
  [NOTICE_PUBLISH_STATUS.PUBLISHED]: { label: '已发布', color: 'success' },
  [NOTICE_PUBLISH_STATUS.REVOKED]: { label: '已撤回', color: 'warning' },
};

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'title',
      label: '公告标题',
      componentProps: {
        placeholder: '请输入公告标题',
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'type',
      label: '公告类型',
      componentProps: {
        options: NOTICE_TYPE_OPTIONS,
        placeholder: '请选择公告类型',
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'publishStatus',
      label: '发布状态',
      componentProps: {
        options: NOTICE_PUBLISH_STATUS_OPTIONS.map((o) => ({
          label: o.label,
          value: o.value,
        })),
        placeholder: '请选择发布状态',
        allowClear: true,
      },
    },
  ],
};

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
  cellConfig: {},
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getNoticeListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          title: formValues.title,
          type: formValues.type,
          status: formValues.publishStatus,
        });
      },
    },
  },

  columns: [
    {
      title: $t('ui.table.seq'),
      type: 'seq',
      width: 70,
    },
    {
      title: '公告标题',
      field: 'title',
      minWidth: 200,
    },
    {
      title: '公告类型',
      field: 'type',
      width: 100,
      slots: { default: 'type' },
    },
    {
      title: '目标用户',
      field: 'targetType',
      width: 100,
      slots: { default: 'targetType' },
    },
    {
      title: '发布状态',
      field: 'publishStatus',
      width: 100,
      slots: { default: 'publishStatus' },
    },
    {
      title: '发布人',
      field: 'publisherName',
      width: 120,
    },
    {
      title: '发布时间',
      field: 'publishTime',
      width: 160,
      slots: { default: 'publishTime' },
    },
    {
      title: $t('ui.table.createTime'),
      field: 'createTime',
      width: 160,
      slots: { default: 'createdAt' },
    },
    {
      title: $t('ui.table.action'),
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 200,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [Drawer, drawerApi] = useVbenDrawer({
  connectedComponent: NoticeDrawer,
  onClosed() {
    const data = drawerApi.getData();
    if (data && data.needRefresh) {
      gridApi.query();
    }
  },
});

function openDrawer(create: boolean, row?: any) {
  drawerApi.setData({
    create,
    row,
  });
  drawerApi.open();
}

function handleCreate() {
  openDrawer(true);
}

function handleEdit(row: any) {
  openDrawer(false, row);
}

async function handleDelete(row: any) {
  row.pending = true;
  try {
    await deleteNoticeApi(row.id);
    window.$message.success($t('ui.notification.delete_success'));
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handlePublish(row: any) {
  row.pending = true;
  try {
    await publishNoticeApi(row.id);
    window.$message.success('公告已发布');
  } catch {
    // 错误由全局拦截器处理
  } finally {
    row.pending = false;
    gridApi.query();
  }
}

async function handleRevoke(row: any) {
  row.pending = true;
  try {
    await revokeNoticeApi(row.id);
    window.$message.success('公告已撤回');
  } catch {
    // 错误由全局拦截器处理
  } finally {
    row.pending = false;
    gridApi.query();
  }
}
</script>

<template>
  <Page auto-content-height>
    <Grid>
      <template #toolbar-tools>
        <Button
          v-if="accessStore.hasAccessCode('system:notice:add')"
          type="primary"
          class="mr-2"
          @click="handleCreate"
        >
          {{ $t('page.system.notice.button.create') }}
        </Button>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createTime) }}
      </template>

      <template #publishTime="{ row }">
        {{ formatDateTime(row.publishTime) }}
      </template>

      <template #type="{ row }">
        <Tag :color="TYPE_MAP[row.type]?.color || 'default'">
          {{ TYPE_MAP[row.type]?.label || row.type }}
        </Tag>
      </template>

      <template #targetType="{ row }">
        <Tag :color="TARGET_TYPE_MAP[row.targetType]?.color || 'default'">
          {{ TARGET_TYPE_MAP[row.targetType]?.label || '未知' }}
        </Tag>
      </template>

      <template #publishStatus="{ row }">
        <span class="status-badge" :class="`status-badge--${STATUS_MAP[row.publishStatus]?.color || 'default'}`">
          <span class="status-badge__dot"></span>
          {{ STATUS_MAP[row.publishStatus]?.label || '未知' }}
        </span>
      </template>

      <template #action="{ row }">
        <div class="flex items-center gap-1">
          <!-- 编辑/查看（已发布状态进入只读查看模式） -->
          <Button
            v-if="accessStore.hasAccessCode('system:notice:update')"
            type="link"
            size="small"
            :icon="h(LucideFilePenLine)"
            @click="handleEdit(row)"
          >
            {{ row.publishStatus === 1 ? '查看' : '编辑' }}
          </Button>

          <!-- 发布（仅未发布或已撤回状态可发布） -->
          <Popconfirm
            v-if="
              accessStore.hasAccessCode('system:notice:publish') &&
              row.publishStatus !== 1
            "
            title="确认发布该公告？"
            ok-text="发布"
            cancel-text="取消"
            @confirm="handlePublish(row)"
          >
            <Button type="link" size="small" :icon="h(LucideSend)">
              发布
            </Button>
          </Popconfirm>

          <!-- 撤回（仅已发布状态可撤回） -->
          <Popconfirm
            v-if="
              accessStore.hasAccessCode('system:notice:revoke') &&
              row.publishStatus === 1
            "
            title="确认撤回该公告？撤回后用户将无法查看"
            ok-text="撤回"
            cancel-text="取消"
            @confirm="handleRevoke(row)"
          >
            <Button type="link" size="small" danger :icon="h(LucideUndo2)">
              撤回
            </Button>
          </Popconfirm>

          <!-- 删除 -->
          <Popconfirm
            :title="
              $t('ui.text.do_you_want_delete', {
                moduleName: $t('page.system.notice.module'),
              })
            "
            :ok-text="$t('ui.button.ok')"
            :cancel-text="$t('ui.button.cancel')"
            @confirm="handleDelete(row)"
          >
            <Button
              v-if="accessStore.hasAccessCode('system:notice:delete')"
              type="link"
              size="small"
              danger
              :icon="h(LucideTrash2)"
            />
          </Popconfirm>
        </div>
      </template>
    </Grid>
    <Drawer />
  </Page>
</template>

<style scoped>
/* 发布状态徽标 */
.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 2px 10px;
  font-size: 12px;
  border-radius: 10px;
  font-weight: 500;
  line-height: 1.5;
}

.status-badge__dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  display: inline-block;
}

/* 已发布 - 绿色 */
.status-badge--success {
  background: #f6ffed;
  color: #389e0d;
  border: 1px solid #b7eb8f;
}
.status-badge--success .status-badge__dot {
  background: #52c41a;
  box-shadow: 0 0 0 2px rgba(82, 196, 26, 0.2);
}

/* 已撤回 - 橙色 */
.status-badge--warning {
  background: #fff7e6;
  color: #d46b08;
  border: 1px solid #ffd591;
}
.status-badge--warning .status-badge__dot {
  background: #faad14;
}

/* 未发布 - 灰色 */
.status-badge--default {
  background: #fafafa;
  color: #8c8c8c;
  border: 1px solid #d9d9d9;
}
.status-badge--default .status-badge__dot {
  background: #bfbfbf;
}
</style>
