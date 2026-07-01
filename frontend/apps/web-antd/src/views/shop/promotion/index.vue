<script lang="ts" setup>
import { h, ref, reactive } from 'vue';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { Page } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideEye, LucidePlus, LucideEdit, LucideTrash2 } from '@vben/icons';
import {
  Button,
  Tag,
  Modal,
  message,
  Form,
  Input,
  Select,
  DatePicker,
  Space,
  Descriptions,
} from 'ant-design-vue';
import { promotionApi } from '#/api';

const modalVisible = ref(false);
const detailModalVisible = ref(false);
const detailData = ref<any>(null);
const isEdit = ref(false);

const formData = reactive({
  id: undefined,
  shopId: undefined,
  title: '',
  type: 1,
  rules: { steps: [{ full: 200, minus: 20 }] },
  startTime: '',
  endTime: '',
  status: 0,
});

const typeOptions = [
  { label: '满减', value: 1 },
  { label: '折扣', value: 2 },
  { label: '限时抢购', value: 3 },
  { label: '新人专享', value: 4 },
];

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Select',
      fieldName: 'status',
      label: '活动状态',
      componentProps: {
        options: [
          { label: '未开始', value: 0 },
          { label: '进行中', value: 1 },
          { label: '已结束', value: 2 },
          { label: '已关闭', value: 3 },
        ],
        placeholder: '请选择状态',
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'title',
      label: '活动标题',
      componentProps: {
        placeholder: '请输入活动标题',
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
  cellConfig: {
    isHover: true,
  },
  stripe: true,

  proxyConfig: {
    autoLoad: true,
    ajax: {
      query: async ({ page }, formValues) => {
        return await promotionApi.list({
          page: page.currentPage,
          pageSize: page.pageSize,
          status: formValues.status,
          shopId: formValues.shopId,
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
      title: '活动标题',
      field: 'title',
      width: 200,
    },
    {
      title: '活动类型',
      field: 'type',
      slots: { default: 'type' },
      width: 120,
    },
    {
      title: '所属店铺',
      field: 'shopName',
      width: 150,
    },
    {
      title: '活动时间',
      field: 'timeRange',
      slots: { default: 'timeRange' },
      width: 240,
    },
    {
      title: '状态',
      field: 'status',
      slots: { default: 'status' },
      width: 100,
    },
    {
      title: '创建时间',
      field: 'createTime',
      width: 160,
    },
    {
      title: '操作',
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 200,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

function getTypeText(type: number) {
  const map: Record<number, string> = {
    1: '满减',
    2: '折扣',
    3: '限时抢购',
    4: '新人专享',
  };
  return map[type] || '未知';
}

function getTypeColor(type: number) {
  const map: Record<number, string> = {
    1: 'red',
    2: 'orange',
    3: 'purple',
    4: 'cyan',
  };
  return map[type] || 'default';
}

function getStatusText(status: number) {
  const map: Record<number, string> = {
    0: '未开始',
    1: '进行中',
    2: '已结束',
    3: '已关闭',
  };
  return map[status] || '未知';
}

function getStatusColor(status: number) {
  const map: Record<number, string> = {
    0: 'default',
    1: 'success',
    2: 'info',
    3: 'error',
  };
  return map[status] || 'default';
}

function openModal(row?: any) {
  if (row) {
    isEdit.value = true;
    Object.assign(formData, row);
  } else {
    isEdit.value = false;
    Object.assign(formData, {
      id: undefined,
      shopId: undefined,
      title: '',
      type: 1,
      rules: { steps: [{ full: 200, minus: 20 }] },
      startTime: '',
      endTime: '',
      status: 0,
    });
  }
  modalVisible.value = true;
}

async function handleSave() {
  try {
    if (isEdit.value) {
      await promotionApi.update(formData);
      message.success('修改成功');
    } else {
      await promotionApi.save(formData);
      message.success('创建成功');
    }
    modalVisible.value = false;
    gridApi.query();
  } catch (e) {
    console.error('Save promotion error:', e);
  }
}

async function handleDelete(row: any) {
  Modal.confirm({
    title: '删除活动',
    content: '确定删除该促销活动吗？',
    async onOk() {
      try {
        await promotionApi.delete({ id: row.id });
        message.success('删除成功');
        gridApi.query();
      } catch (e) {
        console.error('Delete promotion error:', e);
      }
    },
  });
}

async function viewDetail(row: any) {
  detailData.value = await promotionApi.detail(row.id);
  detailModalVisible.value = true;
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="促销管理">
      <template #toolbar-left>
        <Button type="primary" :icon="h(LucidePlus)" @click="openModal">
          新增活动
        </Button>
      </template>

      <template #type="{ row }">
        <Tag :color="getTypeColor(row.type)">
          {{ getTypeText(row.type) }}
        </Tag>
      </template>

      <template #timeRange="{ row }">
        <span
          >{{ row.startTime || '-' }}<br />至<br />{{
            row.endTime || '-'
          }}</span
        >
      </template>

      <template #status="{ row }">
        <Tag :color="getStatusColor(row.status)">
          {{ getStatusText(row.status) }}
        </Tag>
      </template>

      <template #action="{ row }">
        <Button
          type="primary"
          link
          :icon="h(LucideEye)"
          @click="() => viewDetail(row)"
        >
          详情
        </Button>
        <Button
          type="primary"
          link
          :icon="h(LucideEdit)"
          @click="() => openModal(row)"
        >
          编辑
        </Button>
        <Button
          type="default"
          link
          :icon="h(LucideTrash2)"
          @click="() => handleDelete(row)"
        >
          删除
        </Button>
      </template>
    </Grid>

    <Modal
      v-model:open="modalVisible"
      :title="isEdit ? '编辑促销活动' : '新增促销活动'"
      width="600"
      @ok="handleSave"
    >
      <Form
        :model="formData"
        :label-col="{ span: 8 }"
        :wrapper-col="{ span: 16 }"
      >
        <Form.Item label="活动标题" name="title" required>
          <Input v-model="formData.title" placeholder="请输入活动标题" />
        </Form.Item>
        <Form.Item label="活动类型" name="type" required>
          <Select v-model="formData.type" :options="typeOptions" />
        </Form.Item>
        <Form.Item label="所属店铺" name="shopId">
          <Input v-model="formData.shopId" placeholder="空则为平台活动" />
        </Form.Item>
        <Form.Item label="活动时间">
          <Space>
            <DatePicker
              v-model="formData.startTime"
              show-time
              format="YYYY-MM-DD HH:mm:ss"
              placeholder="开始时间"
            />
            <span>至</span>
            <DatePicker
              v-model="formData.endTime"
              show-time
              format="YYYY-MM-DD HH:mm:ss"
              placeholder="结束时间"
            />
          </Space>
        </Form.Item>
        <Form.Item label="活动规则" name="rules">
          <Input.TextArea
            v-model="formData.rules"
            :rows="4"
            placeholder='例如: {"steps":[{"full":200,"minus":20},{"full":500,"minus":60}]}'
          />
        </Form.Item>
      </Form>
    </Modal>

    <Modal v-model:open="detailModalVisible" title="活动详情" width="700">
      <Descriptions :column="2" bordered>
        <Descriptions.Item label="活动标题">{{
          detailData?.title || '-'
        }}</Descriptions.Item>
        <Descriptions.Item label="状态">
          <Tag :color="getStatusColor(detailData?.status)">{{
            getStatusText(detailData?.status)
          }}</Tag>
        </Descriptions.Item>
        <Descriptions.Item label="活动类型">{{
          getTypeText(detailData?.type)
        }}</Descriptions.Item>
        <Descriptions.Item label="所属店铺">{{
          detailData?.shopName || '平台活动'
        }}</Descriptions.Item>
        <Descriptions.Item label="开始时间">{{
          detailData?.startTime || '-'
        }}</Descriptions.Item>
        <Descriptions.Item label="结束时间">{{
          detailData?.endTime || '-'
        }}</Descriptions.Item>
        <Descriptions.Item label="活动规则" :span="2">
          <pre class="bg-gray-50 p-4 rounded text-gray-700 text-sm">{{
            JSON.stringify(detailData?.rules, null, 2)
          }}</pre>
        </Descriptions.Item>
      </Descriptions>
    </Modal>
  </Page>
</template>
