<script lang="ts" setup>
import type { Key } from 'ant-design-vue/es/table/interface';

import type { VbenFormSchema } from '@vben/common-ui';

import { computed, ref } from 'vue';

import { useVbenForm } from '@vben/common-ui';

import {
  Select as ASelect,
  Button,
  Input,
  InputNumber,
  message,
  Modal,
  Radio,
  RadioGroup,
  Table,
  TabPane,
  Tabs,
  Tooltip,
} from 'ant-design-vue';

import { useVbenDrawer } from '#/adapter/drawer';
import {
  createRefundApi,
  getOrderInfoApi,
  getOrderListApi,
  getRefundInfoApi,
  getWarehouseListApi,
  updateRefundApi,
} from '#/api';

// drawerData 在 onOpenChange 中手动赋值，避免引用尚未定义的 drawerApi
const drawerData = ref<{ create: boolean; row: any }>({
  create: true,
  row: {},
});
const isEdit = computed(() => !drawerData.value.create);

const activeTab = ref('basic');
const isFullscreen = ref(false);
const submitting = ref(false);

// ===== 订单选择 =====
const orderModalVisible = ref(false);
const orderList = ref<any[]>([]);
const orderLoading = ref(false);
const orderKeyword = ref('');
const orderSelectedKeys = ref<Key[]>([]);
const orderSelectedRow = ref<any | null>(null);
const orderInfo = ref<{
  customerId?: number;
  customerName?: string;
  id?: number;
  orderNo?: string;
  title?: string;
}>({});

// ===== 退货明细 =====
const items = ref<any[]>([]);
// 退货类型：1=整单退货, 2=部分退货
const refundType = ref<number>(2);

// ===== 金额 =====
const restockingFee = ref(0);

// ===== 仓库 =====
const warehouseList = ref<any[]>([]);
const warehouseId = ref<number | undefined>(undefined);

// ===== 收货信息 =====
const receiver = ref('');
const receiverPhone = ref('');
const receiverAddress = ref('');

// ===== 备注 =====
const remark = ref('');
const title = ref('');
const refundReason = ref('');

const drawerClass = computed(() => [
  'sale-refund-drawer',
  { 'sale-refund-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

// 加载仓库列表（仅退货仓 warehouse_type=4）
async function loadWarehouseList() {
  try {
    const res: any = await getWarehouseListApi({ page: 1, pageSize: 100 });
    const data = res?.data ?? res ?? {};
    const rawList = data.list || data.items || data.rows || [];
    // 仅显示退货仓（warehouse_type=4），若无类型字段则全部展示
    warehouseList.value = rawList.filter(
      (w: any) =>
        !w.warehouseType || w.warehouseType === 4 || w.warehouse_type === 4,
    );
    // 默认选择第一个
    if (warehouseList.value.length > 0 && !warehouseId.value) {
      warehouseId.value = warehouseList.value[0].id;
    }
  } catch (error) {
    console.error('[退货单] 加载仓库列表失败:', error);
    warehouseList.value = [];
  }
}

// 加载订单列表（仅已发货5/已签收9/已完成10状态）
async function loadOrderList() {
  orderLoading.value = true;
  try {
    const res: any = await getOrderListApi({
      page: 1,
      pageSize: 100,
      keywords: orderKeyword.value || undefined,
    });
    const data = res?.data ?? res ?? {};
    const rawList = data.list || data.items || data.rows || [];
    orderList.value = rawList.filter((o: any) =>
      [5, 9, 10].includes(Number(o.orderStatus)),
    );
  } finally {
    orderLoading.value = false;
  }
}

function openOrderModal() {
  orderKeyword.value = '';
  orderSelectedKeys.value = [];
  orderSelectedRow.value = null;
  orderModalVisible.value = true;
  loadOrderList();
}

function handleOrderSelect(record: any) {
  orderSelectedKeys.value = [record.id];
  orderSelectedRow.value = record;
}

async function handleOrderConfirm() {
  if (!orderSelectedRow.value) return;
  const row = orderSelectedRow.value;
  orderInfo.value = {
    id: row.id,
    orderNo: row.orderNo,
    title: row.title,
    customerId: row.customerId,
    customerName: row.customerName,
  };
  // 默认标题
  if (!title.value) {
    title.value = `${row.orderNo}-退货`;
  }
  // 默认收货信息（退货收货地址即我方仓库收货地址，可由用户修改）
  orderModalVisible.value = false;
  // 加载订单明细
  await loadOrderDetail(row.id);
}

async function loadOrderDetail(orderId: number) {
  try {
    const res: any = await getOrderInfoApi(orderId);
    const data = res?.data ?? res ?? {};
    const orderItems = data.items || [];
    // 初始化退货明细：默认勾选全部，退货数量=可退数量
    items.value = orderItems.map((it: any) => {
      const delivered = Number(
        it.deliveredQuantity ?? it.delivered_quantity ?? it.quantity ?? 0,
      );
      const unitPrice = Number(it.unitPrice ?? it.unit_price ?? 0);
      return {
        orderItemId: it.id,
        productId: it.productId ?? it.product_id,
        productName: it.productName || it.product_name || '',
        spec: it.spec || '',
        unit: it.unit || '',
        deliveredQty: delivered,
        refundQty: refundType.value === 1 ? delivered : 0,
        unitPrice,
        refundAmount: refundType.value === 1 ? delivered * unitPrice : 0,
        selected: refundType.value === 1,
      };
    });
  } catch (error) {
    console.error('[退货单] 加载订单明细失败:', error);
    items.value = [];
  }
}

function clearOrder() {
  orderInfo.value = {};
  items.value = [];
}

// 切换退货类型
function handleRefundTypeChange(e: any) {
  refundType.value = e?.target?.value ?? e;
  // 整单退货：全部勾选，退货数量=已发货数量；部分退货：取消勾选，退货数量清零
  items.value =
    refundType.value === 1
      ? items.value.map((it) => ({
          ...it,
          selected: true,
          refundQty: it.deliveredQty,
          refundAmount: it.deliveredQty * it.unitPrice,
        }))
      : items.value.map((it) => ({
          ...it,
          selected: false,
          refundQty: 0,
          refundAmount: 0,
        }));
}

function updateItemAmount(index: number) {
  const it = items.value[index];
  it.refundAmount = Number(it.refundQty || 0) * Number(it.unitPrice || 0);
}

function toggleItemSelection(index: number, checked: boolean) {
  items.value[index].selected = checked;
  if (checked) {
    // 默认填入1
    items.value[index].refundQty = items.value[index].refundQty || 1;
    updateItemAmount(index);
  } else {
    items.value[index].refundQty = 0;
    items.value[index].refundAmount = 0;
  }
}

const totalAmount = computed(() => {
  return items.value
    .filter((it) => it.selected)
    .reduce((sum, it) => sum + Number(it.refundAmount || 0), 0);
});

const refundAmount = computed(() => {
  return Math.max(0, totalAmount.value - (Number(restockingFee.value) || 0));
});

// 加载退货单详情
async function loadRefundDetail(refundId: number) {
  try {
    const res: any = await getRefundInfoApi(refundId);
    const data = res?.data ?? res ?? {};
    orderInfo.value = {
      id: data.orderId,
      orderNo: data.orderNo,
      customerId: data.customerId,
      customerName: data.customerName,
    };
    title.value = data.title || '';
    refundReason.value = data.refundReason || '';
    refundType.value = data.refundType ?? 2;
    restockingFee.value = Number(data.restockingFee ?? 0);
    warehouseId.value = data.warehouseId ?? undefined;
    receiver.value = data.receiver || '';
    receiverPhone.value = data.receiverPhone || '';
    receiverAddress.value = data.receiverAddress || '';
    remark.value = data.remark || '';
    // 加载订单明细，再合并退货明细数据
    if (data.orderId) {
      await loadOrderDetail(Number(data.orderId));
      // 用退货单保存的明细数据覆盖
      const savedItems = data.items || [];
      const savedMap = new Map<number, any>();
      for (const si of savedItems) {
        savedMap.set(Number(si.orderItemId), si);
      }
      items.value = items.value.map((it) => {
        const saved = savedMap.get(Number(it.orderItemId));
        if (saved) {
          return {
            ...it,
            refundQty: Number(saved.refundQty ?? 0),
            unitPrice: Number(saved.unitPrice ?? it.unitPrice ?? 0),
            refundAmount: Number(saved.refundAmount ?? 0),
            selected: true,
          };
        }
        return { ...it, refundQty: 0, refundAmount: 0, selected: false };
      });
    }
  } catch (error) {
    console.error('[退货单] 加载详情失败:', error);
  }
}

const basicFormSchema: VbenFormSchema[] = [
  {
    component: 'Input',
    fieldName: 'title',
    label: '退货标题',
    rules: 'required',
    componentProps: { placeholder: '请输入退货标题' },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'Input',
    fieldName: 'refundReason',
    label: '退货原因',
    componentProps: { placeholder: '请输入退货原因' },
    wrapperClass: 'col-span-2',
  },
];

const [BasicForm, basicFormApi] = useVbenForm({
  schema: basicFormSchema,
  wrapperClass: 'grid-cols-2 gap-4',
  compact: true,
  commonConfig: { componentProps: { class: 'w-full' } },
  showDefaultActions: false,
});

const itemColumns = [
  {
    title: '#',
    width: 45,
    key: 'seq',
    customRender: ({ index }: any) => index + 1,
    align: 'center' as const,
  },
  { title: '产品信息', dataIndex: 'productName', key: 'product', width: 220 },
  { title: '规格', dataIndex: 'spec', key: 'spec', width: 110 },
  {
    title: '单位',
    dataIndex: 'unit',
    key: 'unit',
    width: 55,
    align: 'center' as const,
  },
  {
    title: '已发货',
    dataIndex: 'deliveredQty',
    key: 'deliveredQty',
    width: 80,
    align: 'right' as const,
  },
  { title: '退货数量', key: 'refundQty', width: 100, align: 'center' as const },
  {
    title: '单价',
    dataIndex: 'unitPrice',
    key: 'unitPrice',
    width: 95,
    align: 'right' as const,
  },
  {
    title: '退货金额',
    dataIndex: 'refundAmount',
    key: 'refundAmount',
    width: 105,
    align: 'right' as const,
  },
];

// 订单选择表格列定义
const orderColumns = [
  { title: '订单号', dataIndex: 'orderNo', width: 150 },
  { title: '订单标题', dataIndex: 'title', width: 200, ellipsis: true },
  { title: '客户名称', dataIndex: 'customerName', width: 140, ellipsis: true },
  {
    title: '订单金额',
    key: 'totalAmount',
    width: 120,
    customRender: ({ record }: any) =>
      `¥ ${Number(record.totalAmount || 0).toLocaleString()}`,
  },
  {
    title: '订单状态',
    dataIndex: 'orderStatus',
    width: 100,
    customRender: ({ record }: any) => {
      const map: Record<number, string> = {
        5: '已发货',
        9: '已签收',
        10: '已完成',
      };
      return map[record.orderStatus] || '-';
    },
  },
];

async function handleSubmit() {
  try {
    // 1. 表单验证
    let validResult;
    try {
      validResult = await basicFormApi.validate();
    } catch (error) {
      console.error('[退货单提交] 表单验证异常:', error);
      activeTab.value = 'basic';
      message.warning('请完善基本信息');
      return;
    }
    if (!validResult?.valid) {
      activeTab.value = 'basic';
      message.warning('请完善必填项');
      return;
    }

    // 2. 校验订单选择
    if (!orderInfo.value.id) {
      message.error('请选择关联的销售订单');
      activeTab.value = 'basic';
      return;
    }

    // 3. 校验退货明细
    const selectedItems = items.value.filter(
      (it) => it.selected && Number(it.refundQty || 0) > 0,
    );
    if (selectedItems.length === 0) {
      message.error('请至少添加一条退货明细');
      activeTab.value = 'items';
      return;
    }

    // 4. 校验退货数量不超过已发货数量
    for (const it of selectedItems) {
      if (Number(it.refundQty) > Number(it.deliveredQty)) {
        message.error(
          `产品 [${it.productName}] 退货数量 ${it.refundQty} 超过已发货数量 ${it.deliveredQty}`,
        );
        activeTab.value = 'items';
        return;
      }
    }

    // 5. 收集数据
    const basicValues = await basicFormApi.getValues();
    const submitItems = selectedItems.map((it) => ({
      orderItemId: it.orderItemId,
      productId: it.productId,
      productName: it.productName,
      spec: it.spec,
      unit: it.unit,
      refundQty: Number(it.refundQty),
      unitPrice: Number(it.unitPrice),
      refundAmount: Number(it.refundAmount),
    }));

    const data = {
      title: basicValues.title,
      refundReason: basicValues.refundReason,
      orderId: orderInfo.value.id,
      customerId: orderInfo.value.customerId,
      customerName: orderInfo.value.customerName,
      refundType: refundType.value,
      restockingFee: Number(restockingFee.value) || 0,
      warehouseId: warehouseId.value,
      receiver: receiver.value || undefined,
      receiverPhone: receiverPhone.value || undefined,
      receiverAddress: receiverAddress.value || undefined,
      remark: remark.value || undefined,
      items: submitItems,
    };

    submitting.value = true;
    const submitData = isEdit.value
      ? { ...data, id: drawerData.value.row.id }
      : data;

    if (isEdit.value) {
      await updateRefundApi(submitData);
      message.success('更新成功');
    } else {
      await createRefundApi(submitData);
      message.success('创建成功');
    }
    closeDrawer();
  } catch (error) {
    console.error('[退货单提交] 提交失败:', error);
    message.error('操作失败');
  } finally {
    submitting.value = false;
  }
}

function closeDrawer() {
  drawerApi.close();
  drawerApi.setData({ needRefresh: true });
}

const [Drawer, drawerApi] = useVbenDrawer({
  onConfirm: handleSubmit,
  onOpenChange(isOpen) {
    if (isOpen) {
      const data = drawerApi.getData() as { create?: boolean; row?: any };
      drawerData.value = { create: data?.create ?? true, row: data?.row ?? {} };
      isFullscreen.value = false;
      activeTab.value = 'basic';
      // 重置
      orderInfo.value = {};
      items.value = [];
      refundType.value = 2;
      restockingFee.value = 0;
      warehouseId.value = undefined;
      receiver.value = '';
      receiverPhone.value = '';
      receiverAddress.value = '';
      remark.value = '';
      title.value = '';
      refundReason.value = '';
      basicFormApi.resetForm();
      // 加载仓库列表
      loadWarehouseList();
      if (!drawerData.value.create && drawerData.value.row?.id) {
        loadRefundDetail(Number(drawerData.value.row.id));
      }
    }
  },
});
</script>

<template>
  <Drawer
    :title="isEdit ? '修改退货单' : '新建退货单'"
    :class="drawerClass"
    :destroy-on-close="true"
    :z-index="2000"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? '退出全屏' : '全屏'">
        <button
          type="button"
          class="sale-refund-drawer__fs-btn"
          @click="toggleFullscreen"
        >
          <svg
            v-if="!isFullscreen"
            viewBox="0 0 24 24"
            width="16"
            height="16"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M8 3H5a2 2 0 0 0-2 2v3" />
            <path d="M21 8V5a2 2 0 0 0-2-2h-3" />
            <path d="M3 16v3a2 2 0 0 0 2 2h3" />
            <path d="M16 21h3a2 2 0 0 0 2-2v-3" />
          </svg>
          <svg
            v-else
            viewBox="0 0 24 24"
            width="16"
            height="16"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M8 3v3a2 2 0 0 1-2 2H3" />
            <path d="M21 8h-3a2 2 0 0 1-2-2V3" />
            <path d="M3 16h3a2 2 0 0 1 2 2v3" />
            <path d="M16 21v-3a2 2 0 0 1 2-2h3" />
          </svg>
        </button>
      </Tooltip>
    </template>
    <Tabs v-model:active-key="activeTab">
      <TabPane key="basic" tab="基本信息">
        <BasicForm />
        <!-- 关联订单 -->
        <div class="flex items-center gap-2 mt-2 px-1">
          <span class="text-sm text-gray-500 shrink-0" style="width: 82px"
            >关联订单：</span
          >
          <div class="flex-1">
            <a
              v-if="orderInfo.id"
              class="text-blue-600 cursor-pointer"
              @click="openOrderModal"
            >
              {{ orderInfo.orderNo || `订单 #${orderInfo.id}` }}
              <span v-if="orderInfo.customerName" class="ml-2 text-gray-500">
                （{{ orderInfo.customerName }}）
              </span>
            </a>
            <a
              v-else
              class="text-blue-600 cursor-pointer"
              @click="openOrderModal"
            >
              选择销售订单
            </a>
          </div>
          <Button
            v-if="orderInfo.id"
            type="link"
            size="small"
            danger
            @click="clearOrder"
          >
            清除
          </Button>
        </div>
        <!-- 退货类型 -->
        <div class="flex items-center gap-2 mt-3 px-1">
          <span class="text-sm text-gray-500 shrink-0" style="width: 82px"
            >退货类型：</span
          >
          <RadioGroup :value="refundType" @change="handleRefundTypeChange">
            <Radio :value="1">整单退货</Radio>
            <Radio :value="2">部分退货</Radio>
          </RadioGroup>
        </div>
        <!-- 入库仓库 -->
        <div class="flex items-center gap-2 mt-3 px-1">
          <span class="text-sm text-gray-500 shrink-0" style="width: 82px"
            >入库仓库：</span
          >
          <ASelect
            v-model:value="warehouseId"
            placeholder="请选择退货仓"
            style="flex: 1; width: 100%"
            allow-clear
            :options="
              warehouseList.map((w) => ({
                label: w.warehouseName || w.name,
                value: w.id,
              }))
            "
          />
        </div>
        <!-- 收货信息 -->
        <div class="mt-4 p-3 bg-gray-50 rounded">
          <div class="text-sm font-medium mb-2">
            收货信息（仓库接收退货的地址）
          </div>
          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="text-xs text-gray-500">收货人</label>
              <Input v-model:value="receiver" placeholder="收货人姓名" />
            </div>
            <div>
              <label class="text-xs text-gray-500">联系电话</label>
              <Input v-model:value="receiverPhone" placeholder="联系电话" />
            </div>
            <div class="col-span-2">
              <label class="text-xs text-gray-500">收货地址</label>
              <Input v-model:value="receiverAddress" placeholder="收货地址" />
            </div>
          </div>
        </div>
        <!-- 备注 -->
        <div class="mt-3 px-1">
          <label class="text-sm text-gray-500">备注：</label>
          <Input.TextArea
            v-model:value="remark"
            placeholder="备注信息"
            :rows="2"
          />
        </div>
      </TabPane>
      <TabPane key="items" tab="退货明细">
        <!-- 空状态 -->
        <div v-if="items.length === 0" class="py-12 text-center">
          <div class="mb-4 text-gray-400">请先选择关联销售订单</div>
          <Button type="primary" @click="openOrderModal">选择订单</Button>
        </div>
        <template v-else>
          <div class="mb-3 flex justify-between items-center">
            <span class="text-sm text-gray-500">
              共 {{ items.length }} 项，
              <template v-if="refundType === 1">
                <span class="text-orange-500"
                  >整单退货：全部明细按已发货数量退货</span
                >
              </template>
              <template v-else>
                <span class="text-blue-500"
                  >部分退货：勾选要退货的明细，并填写退货数量</span
                >
              </template>
            </span>
          </div>
          <Table
            :columns="itemColumns"
            :data-source="items"
            :pagination="false"
            size="small"
            :scroll="{ x: 950 }"
            :row-key="(_: any, index) => String(index)"
            bordered
          >
            <template #bodyCell="{ column, record, index }">
              <template v-if="column.key === 'product'">
                <div class="flex flex-col">
                  <span class="font-medium">{{
                    record.productName || '-'
                  }}</span>
                  <span v-if="record.productId" class="text-xs text-gray-400"
                    >ID: {{ record.productId }}</span
                  >
                </div>
              </template>
              <template v-else-if="column.key === 'deliveredQty'">
                {{ Number(record.deliveredQty || 0).toFixed(0) }}
              </template>
              <template v-else-if="column.key === 'refundQty'">
                <div class="flex items-center justify-center gap-1">
                  <input
                    v-if="refundType === 2"
                    type="checkbox"
                    :checked="record.selected"
                    class="cursor-pointer"
                    @change="
                      (e) =>
                        toggleItemSelection(
                          index,
                          (e.target as HTMLInputElement).checked,
                        )
                    "
                  />
                  <InputNumber
                    v-model:value="record.refundQty"
                    :min="0"
                    :max="Number(record.deliveredQty) || 0"
                    :precision="0"
                    style="width: 80px"
                    size="small"
                    :disabled="refundType === 2 && !record.selected"
                    @change="() => updateItemAmount(index)"
                  />
                </div>
              </template>
              <template v-else-if="column.key === 'unitPrice'">
                {{ Number(record.unitPrice || 0).toFixed(2) }}
              </template>
              <template v-else-if="column.key === 'refundAmount'">
                <span class="font-medium text-red-500">
                  {{ Number(record.refundAmount || 0).toFixed(2) }}
                </span>
              </template>
            </template>
          </Table>
          <!-- 金额汇总 -->
          <div class="mt-4 flex flex-col items-end gap-2 pr-4">
            <div class="flex items-center gap-2">
              <span class="w-32 text-right text-gray-500">退货总金额：</span>
              <span class="w-32 text-right font-medium">{{
                totalAmount.toFixed(2)
              }}</span>
            </div>
            <div class="flex items-center gap-2">
              <span class="w-24 text-right text-gray-500">折让金额：</span>
              <InputNumber
                v-model:value="restockingFee"
                :min="0"
                :precision="2"
                class="w-32"
              />
            </div>
            <div class="flex items-center gap-2 border-t pt-2">
              <span class="w-24 text-right font-medium">应退金额：</span>
              <span class="w-32 text-right text-lg font-bold text-red-500">
                {{ refundAmount.toFixed(2) }}
              </span>
            </div>
          </div>
        </template>
      </TabPane>
    </Tabs>

    <!-- 订单选择弹窗 -->
    <Modal
      v-model:open="orderModalVisible"
      title="选择需要退货的销售订单"
      width="900px"
      :z-index="2100"
      :destroy-on-close="true"
      :mask-closable="false"
      ok-text="确认选择"
      cancel-text="取消"
      :ok-button-props="{ disabled: !orderSelectedRow }"
      @ok="handleOrderConfirm"
    >
      <div class="mb-3 flex items-center gap-2">
        <Input
          v-model:value="orderKeyword"
          placeholder="搜索订单号 / 客户名称 / 订单标题"
          allow-clear
          style="width: 360px"
          @press-enter="loadOrderList"
        />
        <Button type="primary" @click="loadOrderList">搜索</Button>
        <span class="ml-2 text-xs text-gray-400">
          仅显示「已发货 / 已签收 / 已完成」状态的可退货订单
        </span>
      </div>
      <Table
        :columns="orderColumns"
        :data-source="orderList"
        :loading="orderLoading"
        :pagination="false"
        :row-key="(record: any) => record.id"
        :row-selection="{
          type: 'radio',
          selectedRowKeys: orderSelectedKeys,
          onChange: (keys: Key[], rows: any[]) => {
            orderSelectedKeys = keys;
            orderSelectedRow = rows[0] || null;
          },
        }"
        :custom-row="
          (record: any) => ({
            onClick: () => handleOrderSelect(record),
            style: { cursor: 'pointer' },
          })
        "
        size="small"
        :scroll="{ y: 400 }"
      />
    </Modal>
  </Drawer>
</template>

<style>
.sale-refund-drawer {
  width: 75vw !important;
}

.sale-refund-drawer--fullscreen {
  width: 100vw !important;
}

.sale-refund-drawer__fs-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  margin-right: 8px;
  color: rgb(0 0 0 / 45%);
  cursor: pointer;
  background: transparent;
  border: none;
  border-radius: 4px;
  transition: all 0.2s;
}

.sale-refund-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgb(0 0 0 / 6%);
}
</style>
