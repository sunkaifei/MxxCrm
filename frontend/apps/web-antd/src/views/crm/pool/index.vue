<script lang="ts" setup>
/**
 * 公海池管理页（v3.0 公海优化 §6.1）
 * 池列表 + 池编辑抽屉 + 成员管理抽屉；权限码 crm:pool:*
 */
import type { VbenFormProps } from '@vben/common-ui';

import type { VxeGridProps } from '#/adapter/vxe-table';

import { ref } from 'vue';

import { Page } from '@vben/common-ui';

import {
  Button,
  Dropdown,
  Menu,
  Popconfirm,
  Tag,
  Tooltip,
} from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deletePoolApi, getPoolPageApi, updatePoolStatusApi } from '#/api/core/crm/pool';
import { $t } from '#/locales';

import PoolEditDrawer from './drawer.vue';
import PoolMemberDrawer from './member-drawer.vue';

const editVisible = ref(false);
const editPoolId = ref<string | null>(null);

const memberVisible = ref(false);
const memberPoolId = ref<string | null>(null);
const memberPoolName = ref('');

// 领取模式文案
const claimModeText: Record<number, string> = {
  1: '自主领取',
  2: '申领需审批',
  3: '仅自动分配',
};

const formOptions: VbenFormProps = {
  collapsed: false,
  schema: [
    {
      component: 'Input',
      componentProps: { allowClear: true, placeholder: '按池名称搜索' },
      fieldName: 'keywords',
      label: '关键词',
    },
    {
      component: 'Select',
      componentProps: {
        allowClear: true,
        options: [
          { label: '启用', value: 1 },
          { label: '停用', value: 2 },
        ],
        placeholder: '请选择状态',
      },
      fieldName: 'status',
      label: '状态',
    },
  ],
};

const gridOptions: VxeGridProps = {
  height: 'auto',
  stripe: true,
  columnConfig: { resizable: true },
  rowConfig: { keyField: 'id' },
  toolbarConfig: {
    custom: true,
    refresh: { code: 'query' },
    slots: {
      tools: 'toolbar-tools',
    },
  },
  pagerConfig: {},
  proxyConfig: {
    response: { result: 'items', total: 'total', list: 'items' },
    ajax: {
      query: async ({ page }, formValues) => {
        return await getPoolPageApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          ...formValues,
        });
      },
    },
  },
  columns: [
    { title: '#', type: 'seq', width: 50, fixed: 'left' },
    {
      title: '池名称',
      field: 'name',
      minWidth: 180,
      fixed: 'left',
      slots: { default: 'nameSlot' },
    },
    { title: '描述', field: 'description', minWidth: 200, showOverflow: true },
    {
      title: '领取模式',
      field: 'claimMode',
      width: 110,
      formatter: ({ row }: any) =>
        claimModeText[row?.config?.claimMode ?? 1] || '-',
    },
    { title: '成员数', field: 'memberCount', width: 80, align: 'center' },
    { title: '在池线索数', field: 'leadCount', width: 100, align: 'center' },
    {
      title: '状态',
      field: 'status',
      width: 80,
      align: 'center',
      slots: { default: 'statusSlot' },
    },
    { title: '排序', field: 'sort', width: 70, align: 'center' },
    { title: '创建时间', field: 'createTime', width: 160 },
    {
      title: '操作',
      field: 'action',
      width: 160,
      fixed: 'right',
      align: 'center',
      slots: { default: 'action' },
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

function openCreate() {
  editPoolId.value = null;
  editVisible.value = true;
}

function openEdit(row: any) {
  editPoolId.value = String(row.id);
  editVisible.value = true;
}

function openMembers(row: any) {
  memberPoolId.value = String(row.id);
  memberPoolName.value = row.name || '';
  memberVisible.value = true;
}

async function handleToggleStatus(row: any) {
  const target = row.status === 1 ? 2 : 1;
  await updatePoolStatusApi(row.id, target);
  window.$message.success(target === 1 ? '已启用' : '已停用');
  gridApi.query();
}

async function handleDelete(row: any) {
  await deletePoolApi([row.id]);
  window.$message.success($t('ui.notification.delete_success'));
  gridApi.query();
}

function handleSaved() {
  gridApi.query();
}

function handleMemberChanged() {
  gridApi.query();
}
</script>

<template>
  <Page auto-content-height>
    <Grid>
      <template #toolbar-tools>
        <Button
          v-access:code="['crm:pool:save']"
          type="primary"
          class="mr-2"
          @click="openCreate"
        >
          {{ $t('page.crm.pool.button.save') }}
        </Button>
      </template>

      <template #nameSlot="{ row }">
        <span>{{ row.name }}</span>
        <Tag v-if="row.isDefault === 1" color="blue" class="ml-1">默认池</Tag>
      </template>

      <template #statusSlot="{ row }">
        <Tag :color="row.status === 1 ? 'green' : 'default'">
          {{ row.status === 1 ? '启用' : '停用' }}
        </Tag>
      </template>

      <template #action="{ row }">
        <Tooltip title="编辑池及规则配置">
          <Button
            v-access:code="['crm:pool:update']"
            type="link"
            size="small"
            @click="openEdit(row)"
          >
            配置
          </Button>
        </Tooltip>
        <Button
          v-access:code="['crm:pool:member']"
          type="link"
          size="small"
          @click="openMembers(row)"
        >
          成员
        </Button>
        <Dropdown v-access:code="['crm:pool:update', 'crm:pool:delete']">
          <Button type="link" size="small">更多</Button>
          <template #overlay>
            <Menu>
              <Menu.Item v-access:code="['crm:pool:update']">
                <Popconfirm
                  :title="row.status === 1 ? '确认停用该池？' : '确认启用该池？'"
                  @confirm="handleToggleStatus(row)"
                >
                  <span>{{ row.status === 1 ? '停用' : '启用' }}</span>
                </Popconfirm>
              </Menu.Item>
              <Menu.Item v-access:code="['crm:pool:delete']">
                <Popconfirm
                  title="删除后池内线索与成员关系将一并处理，确认删除？"
                  @confirm="handleDelete(row)"
                >
                  <span class="text-red-600">删除</span>
                </Popconfirm>
              </Menu.Item>
            </Menu>
          </template>
        </Dropdown>
      </template>
    </Grid>

    <PoolEditDrawer
      v-model:visible="editVisible"
      :pool-id="editPoolId"
      @saved="handleSaved"
    />
    <PoolMemberDrawer
      v-model:visible="memberVisible"
      :pool-id="memberPoolId"
      :pool-name="memberPoolName"
      @changed="handleMemberChanged"
    />
  </Page>
</template>
