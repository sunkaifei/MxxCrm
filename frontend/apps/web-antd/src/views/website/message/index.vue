<script lang="ts" setup>
import { h, onMounted, ref } from 'vue';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { Page } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideFilePenLine, LucideEye } from '@vben/icons';
import { Button, Tag, Modal, message, Select } from 'ant-design-vue';
import { messageApi } from '#/api';
import { getAdminOptionsApi } from '#/api/core/system/user';

defineOptions({ name: 'WebsiteMessage' });

const adminOptions = ref<any[]>([]);

async function loadAdminOptions() {
  try {
    const res: any = await getAdminOptionsApi();
    adminOptions.value = (res || []).map((item: any) => ({
      label: item.nickname || item.username || `用户${item.id}`,
      value: item.id,
    }));
  } catch {
    adminOptions.value = [];
  }
}

onMounted(() => {
  loadAdminOptions();
});

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Select',
      fieldName: 'status',
      label: '状态',
      componentProps: {
        options: [
          { label: '全部', value: '' },
          { label: '待处理', value: 0 },
          { label: '已转线索', value: 1 },
          { label: '已处理', value: 2 },
          { label: '已忽略', value: 3 },
        ],
        placeholder: '请选择状态',
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
  cellConfig: {},
  stripe: true,
  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await messageApi.list({
          page: page.currentPage,
          pageSize: page.pageSize,
          status: formValues.status ?? undefined,
        });
      },
      delete: async ({ body }) => {
        await messageApi.delete(body.removeRecords);
      },
    },
  },
  columns: [
    { title: '序号', type: 'seq', width: 70 },
    {
      title: '联系人',
      field: 'contactName',
      width: 120,
      slots: { default: 'contactName' },
    },
    { title: '联系电话', field: 'contactPhone', width: 140 },
    { title: '邮箱', field: 'contactEmail', width: 180 },
    {
      title: '留言内容',
      field: 'content',
      minWidth: 200,
      slots: { default: 'content' },
    },
    {
      title: '状态',
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
    {
      title: '转线索',
      field: 'convertedToLead',
      width: 90,
      slots: { default: 'convertedToLead' },
    },
    { title: '来源', field: 'source', width: 90 },
    { title: '提交时间', field: 'createTime', width: 170 },
    {
      title: '操作',
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 280,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

// 详情弹窗
const detailVisible = ref(false);
const detailData = ref<any>({});

function handleView(row: any) {
  detailData.value = row;
  detailVisible.value = true;
}

// 转线索弹窗
const convertVisible = ref(false);
const convertRow = ref<any>({});
const assignedTo = ref<number | undefined>(undefined);

function openConvertModal(row: any) {
  if (row.convertedToLead === 1) {
    message.warning('该留言已转线索，不能重复转换');
    return;
  }
  convertRow.value = row;
  assignedTo.value = undefined;
  convertVisible.value = true;
}

async function handleConvertLead() {
  if (!assignedTo.value) {
    message.warning('请选择负责人');
    return;
  }
  try {
    await messageApi.convertLead(convertRow.value.id, {
      assignedTo: assignedTo.value,
    });
    message.success('转线索成功');
    convertVisible.value = false;
    gridApi.query();
  } catch {
    message.error('转线索失败');
  }
}

// 更新状态
async function handleUpdateStatus(row: any, status: number) {
  const statusText = status === 2 ? '已处理' : '已忽略';
  Modal.confirm({
    title: '确认操作',
    content: `确定要将此留言标记为"${statusText}"吗？`,
    onOk: async () => {
      try {
        await messageApi.updateStatus(row.id, status);
        message.success('操作成功');
        gridApi.query();
      } catch {
        message.error('操作失败');
      }
    },
  });
}

// 删除
async function handleDelete(row: any) {
  Modal.confirm({
    title: '确认删除',
    content: '确定要删除该留言吗？',
    okType: 'danger',
    onOk: async () => {
      try {
        await messageApi.delete([row.id]);
        message.success('删除成功');
        gridApi.query();
      } catch {
        message.error('删除失败');
      }
    },
  });
}

function truncate(text: string | undefined, len = 40): string {
  if (!text) return '—';
  return text.length > len ? `${text.slice(0, len)}...` : text;
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="留言管理">
      <template #contactName="{ row }">
        <span>{{ row.contactName || '—' }}</span>
      </template>

      <template #content="{ row }">
        <span :title="row.content">{{ truncate(row.content) }}</span>
      </template>

      <template #status="{ row }">
        <Tag v-if="row.status === 0" color="orange">待处理</Tag>
        <Tag v-else-if="row.status === 1" color="blue">已转线索</Tag>
        <Tag v-else-if="row.status === 2" color="success">已处理</Tag>
        <Tag v-else-if="row.status === 3" color="default">已忽略</Tag>
        <Tag v-else color="default">未知</Tag>
      </template>

      <template #convertedToLead="{ row }">
        <Tag v-if="row.convertedToLead === 1" color="green">已转</Tag>
        <Tag v-else color="gray">未转</Tag>
      </template>

      <template #action="{ row }">
        <Button
          type="primary"
          link
          :icon="h(LucideEye)"
          @click="() => handleView(row)"
        >
          详情
        </Button>
        <Button
          v-if="row.convertedToLead !== 1"
          type="primary"
          link
          :icon="h(LucideFilePenLine)"
          @click="() => openConvertModal(row)"
        >
          转线索
        </Button>
        <Button
          v-if="row.status === 0"
          type="link"
          @click="() => handleUpdateStatus(row, 2)"
        >
          已处理
        </Button>
        <Button
          v-if="row.status === 0"
          type="link"
          @click="() => handleUpdateStatus(row, 3)"
        >
          忽略
        </Button>
        <Button type="primary" link danger @click="() => handleDelete(row)">
          删除
        </Button>
      </template>
    </Grid>

    <!-- 详情弹窗 -->
    <Modal
      v-model:open="detailVisible"
      title="留言详情"
      width="640px"
      :footer="null"
    >
      <div class="space-y-3">
        <div class="flex">
          <span class="w-24 text-gray-500">联系人：</span>
          <span>{{ detailData.contactName || '—' }}</span>
        </div>
        <div class="flex">
          <span class="w-24 text-gray-500">联系电话：</span>
          <span>{{ detailData.contactPhone || '—' }}</span>
        </div>
        <div class="flex">
          <span class="w-24 text-gray-500">联系邮箱：</span>
          <span>{{ detailData.contactEmail || '—' }}</span>
        </div>
        <div class="flex">
          <span class="w-24 text-gray-500">来源页面：</span>
          <span>{{ detailData.sourceUrl || '—' }}</span>
        </div>
        <div class="flex">
          <span class="w-24 text-gray-500">来源标识：</span>
          <span>{{ detailData.source || '—' }}</span>
        </div>
        <div class="flex">
          <span class="w-24 text-gray-500">提交时间：</span>
          <span>{{ detailData.createTime || '—' }}</span>
        </div>
        <div class="flex">
          <span class="w-24 text-gray-500">关联产品：</span>
          <span>{{ detailData.productId || '—' }}</span>
        </div>
        <div class="flex">
          <span class="w-24 text-gray-500">线索ID：</span>
          <span>{{ detailData.leadId || '—' }}</span>
        </div>
        <div>
          <span class="w-24 text-gray-500 inline-block">留言内容：</span>
          <div class="mt-2 rounded bg-gray-50 p-3 text-sm leading-relaxed">
            {{ detailData.content || '—' }}
          </div>
        </div>
      </div>
    </Modal>

    <!-- 转线索弹窗 -->
    <Modal
      v-model:open="convertVisible"
      title="留言转线索"
      width="480px"
      @ok="handleConvertLead"
    >
      <div class="space-y-4 py-2">
        <p class="text-sm text-gray-500">
          将留言转为CRM线索，分配给指定负责人跟进。
        </p>
        <div class="flex items-center gap-3">
          <span class="w-24">联系人：</span>
          <span>{{ convertRow.contactName || '—' }}</span>
        </div>
        <div class="flex items-center gap-3">
          <span class="w-24">电话：</span>
          <span>{{ convertRow.contactPhone || '—' }}</span>
        </div>
        <div class="flex items-center gap-3">
          <span class="w-24">分配给：</span>
          <Select
            v-model:value="assignedTo"
            style="flex: 1"
            placeholder="请选择负责人"
            :options="adminOptions"
            show-search
            :filter-option="
              (input: string, option: any) =>
                option.label?.toLowerCase().includes(input.toLowerCase())
            "
          />
        </div>
      </div>
    </Modal>
  </Page>
</template>
