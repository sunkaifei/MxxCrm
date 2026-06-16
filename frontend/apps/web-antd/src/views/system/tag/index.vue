<script lang="ts" setup>import { h, ref } from 'vue';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { $t } from '#/locales';
import { Page, useVbenDrawer } from '@vben/common-ui';
import { LucideFilePenLine, LucideTrash2, LucideLayers, LucideTag } from '@vben/icons';
import { Button, Popconfirm, Tag, Space, Select } from 'ant-design-vue';
import TagDrawer from './drawer.vue';
import { deleteTagApi, getTagListApi, batchDeleteTagApi } from '#/api';
import { getTagGroupListApi } from '#/api/core/system/tag_group';
import { formatDateTime } from '@vben/utils';
const groupOptions = ref<any[]>([{ label: $t('ui.all'), value: '' }]);
const selectedGroupId = ref('');
async function loadGroups() {
 const res = await getTagGroupListApi();
 if (res.code === 200 && res.data) {
 groupOptions.value = [
 { label: $t('ui.all'), value: '' },
 ...res.data.map((item: any) => ({
 label: item.groupName,
 value: item.id,
 })),
 ];
 }
}
loadGroups();
const formOptions = {
 collapsed: false,
 showCollapseButton: false,
 submitOnEnter: true,
 schema: [
 {
 component: 'Input',
 fieldName: 'tagName',
 label: $t('page.system.tag.name'),
 componentProps: {
 placeholder: $t('ui.placeholder.input'),
 allowClear: true,
 },
 },
 ],
};
const gridOptions: VxeGridProps = {
 toolbarConfig: {
 custom: true,
 refresh: true,
 zoom: true,
 },
 height: 'auto',
 pagerConfig: {},
 rowConfig: {
 isHover: true,
 },
 stripe: true,
 proxyConfig: {
 autoLoad: true,
 ajax: {
 query: async ({ page }, formValues) => {
 return await getTagListApi({
 page: page.currentPage,
 pageSize: page.pageSize,
 tagName: formValues.tagName,
 groupId: selectedGroupId.value || undefined,
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
 title: $t('page.system.tag.group'),
 field: 'groupName',
 width: 120,
 slots: { default: 'groupName' },
 },
 {
 title: $t('page.system.tag.name'),
 field: 'tagName',
 slots: { default: 'tagName' },
 },
 {
 title: $t('page.system.tag.color'),
 field: 'tagColor',
 width: 100,
 slots: { default: 'tagColor' },
 },
 {
 title: $t('ui.form.description'),
 field: 'description',
 minWidth: 150,
 },
 {
 title: $t('page.system.tag.global'),
 field: 'isGlobal',
 width: 100,
 slots: { default: 'isGlobal' },
 },
 {
 title: $t('ui.table.createTime'),
 field: 'createdAt',
 width: 180,
 slots: { default: 'createdAt' },
 },
 {
 title: $t('ui.table.action'),
 field: 'action',
 fixed: 'right',
 slots: { default: 'action' },
 width: 120,
 },
 ],
};
const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });
const [Drawer, drawerApi] = useVbenDrawer({
 connectedComponent: TagDrawer,
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
 await deleteTagApi(row.id);
 window.$message.success($t('ui.notification.delete_success'));
 }
 finally {
 row.pending = false;
 gridApi.query();
 }
}
async function handleBatchDelete() {
 const selectedRows = gridApi.getCheckboxRecords();
 if (!selectedRows || selectedRows.length === 0) {
 window.$message.warning($t('ui.notification.select_row'));
 return;
 }
 const ids = selectedRows.map((row: any) => row.id);
 try {
 await batchDeleteTagApi({ ids });
 window.$message.success($t('ui.notification.delete_success'));
 }
 finally {
 gridApi.query();
 }
}
function handleGroupChange(value: any) {
 selectedGroupId.value = value;
 gridApi.query();
}
</script>

<template>
  <Page auto-content-height>
    <Grid :table-title="$t('page.system.tag.title')">
      <template #toolbar-tools>
        <Select
          v-model:value="selectedGroupId"
          :options="groupOptions"
          class="mr-2"
          style="width: 160px"
          @change="handleGroupChange"
        />
        <Button
          class="mr-2"
          v-access:code="['system:tag:save']"
          type="primary"
          @click="handleCreate"
        >
          {{ $t('page.system.tag.button.create') }}
        </Button>
        <Popconfirm
          :title="$t('ui.text.do_you_want_delete', { moduleName: $t('page.system.tag.module') })"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="handleBatchDelete"
        >
          <template #reference>
            <Button
              type="danger"
              v-access:code="['system:tag:delete']"
              :icon="h(LucideTrash2)"
            >
              {{ $t('ui.button.batch_delete') }}
            </Button>
          </template>
        </Popconfirm>
      </template>

      <template #groupName="{ row }">
        <Tag :color="row.groupColor || '#1890ff'">
          {{ row.groupName || $t('page.system.tag.ungrouped') }}
        </Tag>
      </template>

      <template #tagName="{ row }">
        <Tag :color="row.tagColor || '#1890ff'">
          {{ row.tagName }}
        </Tag>
      </template>

      <template #tagColor="{ row }">
        <div class="flex items-center">
          <div
            class="w-6 h-6 rounded-full mr-2"
            :style="{ backgroundColor: row.tagColor || '#1890ff' }"
          />
          <span class="text-sm">{{ row.tagColor }}</span>
        </div>
      </template>

      <template #isGlobal="{ row }">
        <Tag :color="row.isGlobal ? 'green' : 'default'">
          {{ row.isGlobal ? $t('ui.switch.yes') : $t('ui.switch.no') }}
        </Tag>
      </template>

      <template #createdAt="{ row }">
        {{ formatDateTime(row.createdAt) }}
      </template>

      <template #action="{ row }">
        <Button
          type="primary"
          link
          v-access:code="['system:tag:update']"
          :icon="h(LucideFilePenLine)"
          @click="() => handleEdit(row)"
        />

        <Popconfirm
          :title="$t('ui.text.do_you_want_delete', { moduleName: $t('page.system.tag.module') })"
          :ok-text="$t('ui.button.ok')"
          :cancel-text="$t('ui.button.cancel')"
          @confirm="() => handleDelete(row)"
        >
          <template #reference>
            <Button
              type="danger"
              v-access:code="['system:tag:delete']"
              link
              :icon="h(LucideTrash2)"
            />
          </template>
        </Popconfirm>
      </template>
    </Grid>
    <Drawer />
  </Page>
</template>
