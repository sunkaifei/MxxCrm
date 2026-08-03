<script lang="ts" setup>
import { h, ref } from 'vue';
import { useVbenVxeGrid } from '#/adapter/vxe-table';
import type { VxeGridProps } from '#/adapter/vxe-table';
import { Page } from '@vben/common-ui';
import type { VbenFormProps } from '@vben/common-ui';
import { LucideEye, LucideTruck, LucideTrash2 } from '@vben/icons';
import {
  Button,
  Tag,
  Modal,
  message,
  Select,
  Input,
  Textarea,
  Spin,
} from 'ant-design-vue';
import { orderApi } from '#/api/core/website/order';

defineOptions({ name: 'WebsiteOrder' });

const formOptions: VbenFormProps = {
  collapsed: false,
  showCollapseButton: false,
  submitOnEnter: true,
  schema: [
    {
      component: 'Select',
      fieldName: 'status',
      label: '订单状态',
      componentProps: {
        options: [
          { label: '全部', value: '' },
          { label: '待付款', value: 0 },
          { label: '待发货', value: 1 },
          { label: '待收货', value: 2 },
          { label: '已完成', value: 3 },
          { label: '已取消', value: 4 },
          { label: '已关闭', value: 5 },
        ],
        placeholder: '请选择订单状态',
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'payStatus',
      label: '支付状态',
      componentProps: {
        options: [
          { label: '全部', value: '' },
          { label: '未支付', value: 0 },
          { label: '已支付', value: 1 },
          { label: '已退款', value: 2 },
          { label: '部分退款', value: 3 },
        ],
        placeholder: '请选择支付状态',
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'shipStatus',
      label: '发货状态',
      componentProps: {
        options: [
          { label: '全部', value: '' },
          { label: '未发货', value: 0 },
          { label: '部分发货', value: 1 },
          { label: '已发货', value: 2 },
          { label: '已签收', value: 3 },
        ],
        placeholder: '请选择发货状态',
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'orderNo',
      label: '订单号',
      componentProps: {
        placeholder: '请输入订单号',
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
        return await orderApi.list({
          page: page.currentPage,
          pageSize: page.pageSize,
          status: formValues.status ?? undefined,
          payStatus: formValues.payStatus ?? undefined,
          shipStatus: formValues.shipStatus ?? undefined,
          orderNo: formValues.orderNo || undefined,
        });
      },
      delete: async ({ body }) => {
        await orderApi.batchDelete(body.removeRecords);
      },
    },
  },
  columns: [
    { title: '序号', type: 'seq', width: 70 },
    { title: '订单号', field: 'orderNo', width: 200 },
    { title: '用户ID', field: 'userId', width: 90 },
    { title: '商品金额', field: 'totalAmount', width: 110 },
    { title: '实付金额', field: 'payAmount', width: 110 },
    {
      title: '订单状态',
      field: 'status',
      width: 100,
      slots: { default: 'status' },
    },
    {
      title: '支付状态',
      field: 'payStatus',
      width: 100,
      slots: { default: 'payStatus' },
    },
    {
      title: '发货状态',
      field: 'shipStatus',
      width: 100,
      slots: { default: 'shipStatus' },
    },
    { title: '收货人', field: 'consigneeName', width: 110 },
    { title: '下单时间', field: 'createTime', width: 170 },
    {
      title: '操作',
      field: 'action',
      fixed: 'right',
      slots: { default: 'action' },
      width: 220,
    },
  ],
};

const [Grid, gridApi] = useVbenVxeGrid({ gridOptions, formOptions });

// 详情弹窗
const detailVisible = ref(false);
const detailData = ref<any>({});
const detailLoading = ref(false);

async function handleView(row: any) {
  detailVisible.value = true;
  detailLoading.value = true;
  try {
    detailData.value = await orderApi.detail(row.id);
  } catch {
    detailData.value = row;
  } finally {
    detailLoading.value = false;
  }
}

// 发货弹窗
const shipVisible = ref(false);
const shipRow = ref<any>({});
const shipForm = ref({
  deliveryNo: '',
  deliveryCompany: '',
  deliveryType: 1,
  remark: '',
});

function openShipModal(row: any) {
  shipRow.value = row;
  shipForm.value = {
    deliveryNo: '',
    deliveryCompany: '',
    deliveryType: 1,
    remark: '',
  };
  shipVisible.value = true;
}

async function handleShip() {
  if (!shipForm.value.deliveryNo) {
    message.warning('请输入物流单号');
    return;
  }
  if (!shipForm.value.deliveryCompany) {
    message.warning('请输入物流公司');
    return;
  }
  try {
    await orderApi.ship(shipRow.value.id, shipForm.value);
    message.success('发货成功');
    shipVisible.value = false;
    gridApi.query();
  } catch {
    message.error('发货失败');
  }
}

// 删除
async function handleDelete(row: any) {
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除订单"${row.orderNo}"吗？`,
    okType: 'danger',
    onOk: async () => {
      try {
        await orderApi.batchDelete([row.id]);
        message.success('删除成功');
        gridApi.query();
      } catch {
        message.error('删除失败');
      }
    },
  });
}

function formatAmount(val: any): string {
  if (val === null || val === undefined || val === '') return '—';
  return `¥${Number(val).toFixed(2)}`;
}
</script>

<template>
  <Page auto-content-height>
    <Grid table-title="订单管理">
      <template #status="{ row }">
        <Tag v-if="row.status === 0" color="orange">待付款</Tag>
        <Tag v-else-if="row.status === 1" color="blue">待发货</Tag>
        <Tag v-else-if="row.status === 2" color="cyan">待收货</Tag>
        <Tag v-else-if="row.status === 3" color="success">已完成</Tag>
        <Tag v-else-if="row.status === 4" color="default">已取消</Tag>
        <Tag v-else-if="row.status === 5" color="red">已关闭</Tag>
        <Tag v-else color="default">未知</Tag>
      </template>

      <template #payStatus="{ row }">
        <Tag v-if="row.payStatus === 0" color="default">未支付</Tag>
        <Tag v-else-if="row.payStatus === 1" color="success">已支付</Tag>
        <Tag v-else-if="row.payStatus === 2" color="orange">已退款</Tag>
        <Tag v-else-if="row.payStatus === 3" color="warning">部分退款</Tag>
        <Tag v-else color="default">未知</Tag>
      </template>

      <template #shipStatus="{ row }">
        <Tag v-if="row.shipStatus === 0" color="default">未发货</Tag>
        <Tag v-else-if="row.shipStatus === 1" color="warning">部分发货</Tag>
        <Tag v-else-if="row.shipStatus === 2" color="blue">已发货</Tag>
        <Tag v-else-if="row.shipStatus === 3" color="success">已签收</Tag>
        <Tag v-else color="default">未知</Tag>
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
          v-if="row.status === 1"
          type="primary"
          link
          :icon="h(LucideTruck)"
          @click="() => openShipModal(row)"
        >
          发货
        </Button>
        <Button
          type="primary"
          link
          danger
          :icon="h(LucideTrash2)"
          @click="() => handleDelete(row)"
        >
          删除
        </Button>
      </template>
    </Grid>

    <!-- 详情弹窗 -->
    <Modal
      v-model:open="detailVisible"
      title="订单详情"
      width="860px"
      :footer="null"
    >
      <Spin :spinning="detailLoading">
      <div class="space-y-4">
        <div class="grid grid-cols-2 gap-x-8 gap-y-3">
          <div class="flex">
            <span class="w-24 text-gray-500">订单号：</span>
            <span>{{ detailData.orderNo || '—' }}</span>
          </div>
          <div class="flex">
            <span class="w-24 text-gray-500">用户ID：</span>
            <span>{{ detailData.userId || '—' }}</span>
          </div>
          <div class="flex">
            <span class="w-24 text-gray-500">商品金额：</span>
            <span>{{ formatAmount(detailData.totalAmount) }}</span>
          </div>
          <div class="flex">
            <span class="w-24 text-gray-500">优惠金额：</span>
            <span>{{ formatAmount(detailData.discountAmount) }}</span>
          </div>
          <div class="flex">
            <span class="w-24 text-gray-500">运费：</span>
            <span>{{ formatAmount(detailData.shippingFee) }}</span>
          </div>
          <div class="flex">
            <span class="w-24 text-gray-500">实付金额：</span>
            <span class="font-medium text-red-500">
              {{ formatAmount(detailData.payAmount) }}
            </span>
          </div>
          <div class="flex">
            <span class="w-24 text-gray-500">订单状态：</span>
            <Tag v-if="detailData.status === 0" color="orange">待付款</Tag>
            <Tag v-else-if="detailData.status === 1" color="blue">待发货</Tag>
            <Tag v-else-if="detailData.status === 2" color="cyan">待收货</Tag>
            <Tag v-else-if="detailData.status === 3" color="success">
              已完成
            </Tag>
            <Tag v-else-if="detailData.status === 4" color="default">
              已取消
            </Tag>
            <Tag v-else-if="detailData.status === 5" color="red">已关闭</Tag>
            <span v-else>—</span>
          </div>
          <div class="flex">
            <span class="w-24 text-gray-500">支付状态：</span>
            <Tag v-if="detailData.payStatus === 0" color="default">未支付</Tag>
            <Tag v-else-if="detailData.payStatus === 1" color="success">
              已支付
            </Tag>
            <Tag v-else-if="detailData.payStatus === 2" color="orange">
              已退款
            </Tag>
            <Tag v-else-if="detailData.payStatus === 3" color="warning">
              部分退款
            </Tag>
            <span v-else>—</span>
          </div>
          <div class="flex">
            <span class="w-24 text-gray-500">发货状态：</span>
            <Tag v-if="detailData.shipStatus === 0" color="default">未发货</Tag>
            <Tag v-else-if="detailData.shipStatus === 1" color="warning">
              部分发货
            </Tag>
            <Tag v-else-if="detailData.shipStatus === 2" color="blue">
              已发货
            </Tag>
            <Tag v-else-if="detailData.shipStatus === 3" color="success">
              已签收
            </Tag>
            <span v-else>—</span>
          </div>
          <div class="flex">
            <span class="w-24 text-gray-500">支付时间：</span>
            <span>{{ detailData.payTime || '—' }}</span>
          </div>
          <div class="flex">
            <span class="w-24 text-gray-500">发货时间：</span>
            <span>{{ detailData.shipTime || '—' }}</span>
          </div>
          <div class="flex">
            <span class="w-24 text-gray-500">完成时间：</span>
            <span>{{ detailData.finishTime || '—' }}</span>
          </div>
          <div class="flex">
            <span class="w-24 text-gray-500">交易单号：</span>
            <span>{{ detailData.transactionId || '—' }}</span>
          </div>
          <div class="flex">
            <span class="w-24 text-gray-500">下单时间：</span>
            <span>{{ detailData.createTime || '—' }}</span>
          </div>
        </div>

        <div class="border-t pt-3">
          <h4 class="mb-2 font-semibold">收货信息</h4>
          <div class="grid grid-cols-2 gap-x-8 gap-y-2 text-sm">
            <div class="flex">
              <span class="w-24 text-gray-500">收货人：</span>
              <span>{{ detailData.consigneeName || '—' }}</span>
            </div>
            <div class="flex">
              <span class="w-24 text-gray-500">联系电话：</span>
              <span>{{ detailData.consigneePhone || '—' }}</span>
            </div>
            <div class="flex">
              <span class="w-24 text-gray-500">邮编：</span>
              <span>{{ detailData.consigneeZipcode || '—' }}</span>
            </div>
            <div class="col-span-2 flex">
              <span class="w-24 shrink-0 text-gray-500">收货地址：</span>
              <span>
                {{ detailData.consigneeProvince }}
                {{ detailData.consigneeCity }}
                {{ detailData.consigneeDistrict }}
                {{ detailData.consigneeAddress }}
              </span>
            </div>
          </div>
        </div>

        <div class="border-t pt-3">
          <h4 class="mb-2 font-semibold">备注</h4>
          <div class="space-y-1 text-sm">
            <div class="flex">
              <span class="w-24 text-gray-500">买家备注：</span>
              <span>{{ detailData.buyerRemark || '—' }}</span>
            </div>
            <div class="flex">
              <span class="w-24 text-gray-500">卖家备注：</span>
              <span>{{ detailData.sellerRemark || '—' }}</span>
            </div>
            <div class="flex">
              <span class="w-24 text-gray-500">取消原因：</span>
              <span>{{ detailData.cancelReason || '—' }}</span>
            </div>
          </div>
        </div>

        <div class="border-t pt-3">
          <h4 class="mb-2 font-semibold">商品明细</h4>
          <table class="w-full text-sm">
            <thead>
              <tr class="border-b bg-gray-50 text-gray-600">
                <th class="px-3 py-2 text-left">商品名称</th>
                <th class="px-3 py-2 text-left">SKU编码</th>
                <th class="px-3 py-2 text-right">单价</th>
                <th class="px-3 py-2 text-right">数量</th>
                <th class="px-3 py-2 text-right">小计</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="item in detailData.items || []"
                :key="item.id"
                class="border-b"
              >
                <td class="px-3 py-2">{{ item.productName || '—' }}</td>
                <td class="px-3 py-2">{{ item.skuCode || '—' }}</td>
                <td class="px-3 py-2 text-right">
                  {{ formatAmount(item.price) }}
                </td>
                <td class="px-3 py-2 text-right">{{ item.quantity }}</td>
                <td class="px-3 py-2 text-right">
                  {{ formatAmount(item.totalAmount) }}
                </td>
              </tr>
              <tr v-if="!(detailData.items && detailData.items.length)">
                <td colspan="5" class="px-3 py-4 text-center text-gray-400">
                  暂无商品明细
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
      </Spin>
    </Modal>

    <!-- 发货弹窗 -->
    <Modal
      v-model:open="shipVisible"
      title="订单发货"
      width="520px"
      @ok="handleShip"
    >
      <div class="space-y-4 py-2">
        <div class="flex items-center gap-3">
          <span class="w-24">订单号：</span>
          <span>{{ shipRow.orderNo || '—' }}</span>
        </div>
        <div class="flex items-center gap-3">
          <span class="w-24">收货人：</span>
          <span>{{ shipRow.consigneeName || '—' }}</span>
        </div>
        <div class="flex items-center gap-3">
          <span class="w-24"><span class="text-red-500">*</span> 物流单号：</span>
          <Input
            v-model:value="shipForm.deliveryNo"
            placeholder="请输入物流单号"
            style="flex: 1"
          />
        </div>
        <div class="flex items-center gap-3">
          <span class="w-24">
            <span class="text-red-500">*</span> 物流公司：
          </span>
          <Input
            v-model:value="shipForm.deliveryCompany"
            placeholder="请输入物流公司"
            style="flex: 1"
          />
        </div>
        <div class="flex items-center gap-3">
          <span class="w-24">配送方式：</span>
          <Select
            v-model:value="shipForm.deliveryType"
            style="flex: 1"
            :options="[
              { label: '快递', value: 1 },
              { label: '自提', value: 2 },
              { label: '同城', value: 3 },
            ]"
          />
        </div>
        <div class="flex items-start gap-3">
          <span class="w-24 shrink-0">备注：</span>
          <Textarea
            v-model:value="shipForm.remark"
            placeholder="请输入发货备注"
            :rows="3"
            style="flex: 1"
          />
        </div>
      </div>
    </Modal>
  </Page>
</template>
