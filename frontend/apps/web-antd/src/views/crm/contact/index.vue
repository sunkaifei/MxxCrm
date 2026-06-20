<script lang="ts" setup>
import type { VbenFormProps } from '@vben/common-ui';
import type { VxeGridProps } from '#/adapter/vxe-table';

import { h, ref } from 'vue';

import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2, LucideEye } from '@vben/icons';
import { useAccessStore } from '@vben/stores';
import { formatDateTime } from '@vben/utils';

import { Button, Popconfirm, Drawer, Modal, message } from 'ant-design-vue';

import { useVbenVxeGrid } from '#/adapter/vxe-table';
import { deleteContactApi, getContactListApi } from '#/api';
import { $t } from '#/locales';
import ContactDrawer from './drawer.vue';
import ContactDetail from './detail.vue';

const accessStore = useAccessStore();

// 角色映射 - 对齐后端 role_type
const roleColorMap: Record<string, string> = {
  decision_maker: 'red', influencer: 'orange', user: 'blue', other: 'default',
};
const roleLabelMap: Record<string, string> = {
  decision_maker: '决策人', influencer: '影响者', user: '使用者', other: '其他',
};

// 详情抽屉
const detailVisible = ref(false);
const detailId = ref<number | null>(null);

function openDetail(row: any) {
  const id = row.id ?? row.id_;
  if (!id) { message.error('联系人ID不存在'); return; }
  detailId.value = Number(id);
  detailVisible.value = true;
}
function closeDetail() { detailVisible.value = false; detailId.value = null; }
function handleDetailEdit(contact: any) { closeDetail(); openDrawer(false, contact); }

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Input',
      fieldName: 'name',
      label: '姓名',
      componentProps: { placeholder: '输入姓名搜索', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'email',
      label: '邮箱',
      componentProps: { placeholder: '输入邮箱', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'phone',
      label: '手机号',
      componentProps: { placeholder: '输入手机号', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'roleType',
      label: '角色',
      componentProps: {
        placeholder: '全部',
        allowClear: true,
        options: [
          { label: '决策人', value: 'decision_maker' },
          { label: '影响者', value: 'influencer' },
          { label: '使用者', value: 'user' },
          { label: '其他', value: 'other' },
        ],
      },
    },
  ],
};

const gridOptions: VxeGridProps = {
  toolbarConfig: { custom: true, export: true, refresh: true, zoom: true },
  height: 'auto',
  exportConfig: {},
  pagerConfig: {},
  rowConfig: { isHover: true },
  stripe: true,
  checkboxConfig: { checkField: 'checked', trigger: 'row' },

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await getContactListApi({
          page: page.currentPage,
          pageSize: page.pageSize,
          ...formValues,
        });
      },
    },
  },

  columns: [
    { type: 'checkbox', width: 50 },
    { title: $t('ui.table.seq'), type: 'seq', width: 60 },
    { title: '姓名', field: 'name', width: 120, slots: { default: 'name' } },
    { title: '当前公司', field: 'currentCompany', minWidth: 160 },
    { title: '职位', field: 'title', width: 120 },
    {
      title: '角色', field: 'roleType', width: 90,
      cellRender: {
        name: 'Tag',
        options: [
          { value: 'decision_maker', label: '决策人', color: 'red' },
          { value: 'influencer', label: '影响者', color: 'orange' },
          { value: 'user', label: '使用者', color: 'blue' },
          { value: 'other', label: '其他', color: 'default' },
        ],
      },
    },
    {
      title: '首要', field: 'isPrimary', width: 60, align: 'center',
      formatter: ({ cellValue }: any) => cellValue ? '★' : '-',
    },
    { title: '邮箱', field: 'email', width: 180 },
    { title: '手机', field: 'mobile', width: 130 },
    {
      title: $t('ui.table.createTime'), field: 'createTime', slots: { default: 'createdAt' }, width: 160,
    },
    {
      title: $t('ui.table.action'), field: 'action', fixed: 'right', slots: { default: 'action' }, width: 140,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

const [FormDrawer, drawerApi] = useVbenDrawer({
  connectedComponent: ContactDrawer,
  onClosed() { if (drawerApi.getData()?.needRefresh) gridApi.query(); },
});

function openDrawer(create: boolean, row?: any) { drawerApi.setData({ create, row }); drawerApi.open(); }
function handleCreate() { openDrawer(true); }
function handleEdit(row: any) { openDrawer(false, row); }

async function handleDelete(row: any) {
  row.pending = true;
  try { await deleteContactApi(row.id); message.success($t('ui.notification.delete_success')); }
  finally { row.pending = false; gridApi.query(); }
}

async function handleBatchDelete() {
  const records = gridApi.grid?.getCheckboxRecords();
  if (!records?.length) { message.warning('请先选择要删除的联系人'); return; }
  Modal.confirm({
    title: '批量删除',
    content: `确定批量删除 ${records.length} 个联系人？`,
    onOk: async () => {
      try {
        await Promise.all(records.map((r: any) => deleteContactApi(r.id)));
        message.success(`已删除 ${records.length} 个联系人`);
        gridApi.query();
      } catch { /* ignore */ }
    },
  });
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.crm.contact.title')">
      <template #toolbar-tools>
        <Button v-if="accessStore.hasAccessCode('crm:contact:create')" type="primary" class="mr-2" @click="handleCreate">
          {{ $t('page.crm.contact.button.create') }}
        </Button>
        <Button @click="handleBatchDelete" class="mr-2" danger ghost>批量删除</Button>
      </template>

      <template #createdAt="{ row }">{{ formatDateTime(row.createdAt) }}</template>

      <template #name="{ row }">
        <a class="cursor-pointer text-blue-600 hover:text-blue-800" @click="() => openDetail(row)">{{ row.name }}</a>
      </template>

      <template #action="{ row }">
        <Button type="link" :icon="h(LucideEye)" @click="() => openDetail(row)" />
        <Button v-if="accessStore.hasAccessCode('crm:contact:edit')" type="link" :icon="h(LucideFilePenLine)" @click="() => handleEdit(row)" />
        <Popconfirm :title="$t('ui.text.do_you_want_delete', { moduleName: $t('page.crm.contact.title') })" :ok-text="$t('ui.button.ok')" :cancel-text="$t('ui.button.cancel')" @confirm="handleDelete(row)">
          <Button v-if="accessStore.hasAccessCode('crm:contact:delete')" type="link" danger :icon="h(LucideTrash2)" />
        </Popconfirm>
      </template>
    </Grid>
    <FormDrawer />

    <Drawer v-model:open="detailVisible" :width="1000" placement="right" :destroy-on-close="true" :mask-closable="true" :closable="true" title="联系人详情" :body-style="{ padding: 0, maxHeight: 'calc(100vh - 110px)', overflow: 'auto' }" @close="closeDetail">
      <ContactDetail v-if="detailId" :id="detailId" @edit="handleDetailEdit" />
    </Drawer>
  </Page>
</template>