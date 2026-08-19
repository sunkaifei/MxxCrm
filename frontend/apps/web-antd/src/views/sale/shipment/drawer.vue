<script lang="ts" setup>
import type { VbenFormSchema } from '@vben/common-ui';

import { computed, h, ref, watch } from 'vue';

import { useVbenForm } from '@vben/common-ui';
import { LucidePencil } from '@vben/icons';

import {
  Button,
  Empty,
  InputNumber,
  message,
  Modal,
  Progress,
  Spin,
  Table,
  Tabs,
  Tag,
  Tooltip,
} from 'ant-design-vue';

import { useVbenDrawer } from '#/adapter/drawer';
import {
  createShipmentApi,
  getOrderInfoApi,
  getShipmentInfoApi,
  getShipmentListApi,
  updateShipmentApi,
} from '#/api';

import OrderSelectModal from '../components/OrderSelectModal.vue';
import ShipmentEditLogTimeline from '../components/ShipmentEditLogTimeline.vue';

const props = withDefaults(defineProps<{ row?: any }>(), { row: () => ({}) });

// ============ 状态与加载 ============
const isFullscreen = ref(false);
const loading = ref(false);
const submitting = ref(false);
const items = ref<any[]>([]);
const historyShipments = ref<any[]>([]);
const onlyPending = ref(false);

// ============ 订单选择器（从发货管理新增时使用） ============
const selectedOrderId = ref<null | number>(null);
const selectedOrderRow = ref<any>(null);
const orderSelectVisible = ref(false);
const hasInitRow = ref(false);

// ============ 选项卡状态 ============
const activeTab = ref<'history' | 'ship'>('ship');

// 订单是否已全部发完（用于自动隐藏发货选项卡）
const isOrderFullyShipped = computed(() => {
  return (
    items.value.length > 0 &&
    items.value.every((item) => getMaxShipQty(item) === 0)
  );
});

// ============ 发货记录详情展开 ============
// 缓存：shipmentId -> items 列表
const shipmentDetailsCache = ref<Record<number, any[]>>({});
const expandedShipmentIds = ref<Set<number>>(new Set());
const detailLoading = ref<Set<number>>(new Set());

async function toggleShipmentDetail(ship: any) {
  const shipId = Number(ship.id);
  if (expandedShipmentIds.value.has(shipId)) {
    // 收起
    expandedShipmentIds.value.delete(shipId);
    expandedShipmentIds.value = new Set(expandedShipmentIds.value);
    return;
  }
  // 展开
  expandedShipmentIds.value.add(shipId);
  expandedShipmentIds.value = new Set(expandedShipmentIds.value);
  // 已缓存则不再请求
  if (shipmentDetailsCache.value[shipId]) return;
  detailLoading.value.add(shipId);
  detailLoading.value = new Set(detailLoading.value);
  try {
    const res: any = await getShipmentInfoApi(shipId);
    const data = res?.data ?? res ?? {};
    shipmentDetailsCache.value = {
      ...shipmentDetailsCache.value,
      [shipId]: data.items || [],
    };
  } catch (error) {
    console.error('[发货详情] 加载失败:', error);
    shipmentDetailsCache.value = {
      ...shipmentDetailsCache.value,
      [shipId]: [],
    };
  } finally {
    detailLoading.value.delete(shipId);
    detailLoading.value = new Set(detailLoading.value);
  }
}

// ============ 订单状态映射 ============
const orderStatusColorMap: Record<number, string> = {
  1: 'default',
  2: 'blue',
  3: 'blue',
  4: 'orange',
  5: 'cyan',
  6: 'purple',
  7: 'red',
  8: 'cyan',
  9: 'green',
  10: 'blue',
};
const orderStatusLabelMap: Record<number, string> = {
  1: '草稿',
  2: '待确认',
  3: '已确认',
  4: '备货中',
  5: '部分发货',
  6: '已发货',
  7: '已取消',
  8: '已交付',
  9: '已签收',
  10: '已完成',
};

const shippingMethodMap: Record<number, { color: string; label: string }> = {
  1: { label: '快递', color: 'blue' },
  2: { label: '物流', color: 'cyan' },
  3: { label: '自提', color: 'orange' },
  4: { label: '送货上门', color: 'green' },
  5: { label: '其他', color: 'default' },
};

const shipmentStatusMap: Record<number, { color: string; label: string }> = {
  1: { label: '待发货', color: 'default' },
  2: { label: '已发货', color: 'processing' },
  3: { label: '已签收', color: 'green' },
  4: { label: '已取消', color: 'red' },
};

// ============ 抽屉样式 ============
const drawerClass = computed(() => [
  'sale-shipment-drawer',
  { 'sale-shipment-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

// ============ 物流信息表单 ============
const shippingMethodOptions = [
  { label: '快递', value: 1 },
  { label: '物流', value: 2 },
  { label: '自提', value: 3 },
  { label: '送货上门', value: 4 },
  { label: '其他', value: 5 },
];

// 常用物流公司（带"其他/自定义"选项，Select 配 showSearch 支持搜索）
const logisticsCompanyOptions = [
  { label: '顺丰速运', value: '顺丰速运' },
  { label: '德邦快递', value: '德邦快递' },
  { label: '中通快递', value: '中通快递' },
  { label: '圆通速递', value: '圆通速递' },
  { label: '申通快递', value: '申通快递' },
  { label: '韵达快递', value: '韵达快递' },
  { label: '京东物流', value: '京东物流' },
  { label: '极兔速递', value: '极兔速递' },
  { label: 'EMS', value: 'EMS' },
  { label: '百世快递', value: '百世快递' },
  { label: '邮政包裹', value: '邮政包裹' },
  { label: '其他', value: '其他' },
];

const formSchema: VbenFormSchema[] = [
  {
    component: 'DatePicker',
    fieldName: 'shipmentDate',
    label: '发货日期',
    rules: 'required',
    componentProps: {
      placeholder: '请选择',
      style: 'width:100%',
      valueFormat: 'YYYY-MM-DD',
      // 日历弹窗渲染到 body，z-index 高于 Modal(2100)
      dropdownStyle: { zIndex: 2200 },
      dropdownClassName: 'shipment-select-dropdown',
      popupStyle: { zIndex: 2200 },
      popupClassName: 'shipment-select-dropdown',
    },
  },
  {
    component: 'Select',
    fieldName: 'logisticsCompany',
    label: '物流公司',
    componentProps: {
      placeholder: '请选择物流公司',
      options: logisticsCompanyOptions,
      showSearch: true,
      allowClear: true,
      // 下拉渲染到 body（默认），并通过 dropdownStyle 提升 z-index 高于 Modal(2100)
      dropdownStyle: { zIndex: 2200 },
      dropdownClassName: 'shipment-select-dropdown',
      filterOption: (input: string, option: any) =>
        String(option?.label ?? '')
          .toLowerCase()
          .includes(input.toLowerCase()),
    },
  },
  {
    component: 'Input',
    fieldName: 'trackingNo',
    label: '物流单号',
    componentProps: { placeholder: '物流运单号' },
  },
  {
    component: 'Select',
    fieldName: 'shippingMethod',
    label: '配送方式',
    defaultValue: 1,
    componentProps: {
      placeholder: '请选择',
      options: shippingMethodOptions,
      allowClear: true,
      // 下拉渲染到 body（默认），并通过 dropdownStyle 提升 z-index 高于 Modal(2100)
      dropdownStyle: { zIndex: 2200 },
      dropdownClassName: 'shipment-select-dropdown',
    },
  },
  {
    component: 'Input',
    fieldName: 'receiverName',
    label: '收货人',
    componentProps: { placeholder: '收货人姓名' },
  },
  {
    component: 'Input',
    fieldName: 'receiverPhone',
    label: '收货电话',
    componentProps: { placeholder: '收货人电话' },
  },
  {
    component: 'Textarea',
    fieldName: 'shippingAddress',
    label: '收货地址',
    componentProps: { placeholder: '详细收货地址', rows: 2 },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'Textarea',
    fieldName: 'remark',
    label: '备注',
    componentProps: {
      placeholder: '备注信息',
      rows: 3,
      showCount: true,
      maxlength: 500,
    },
    wrapperClass: 'col-span-2',
  },
];

const [Form, formApi] = useVbenForm({
  schema: formSchema,
  wrapperClass: 'grid-cols-2 gap-4',
  compact: true,
  commonConfig: { componentProps: { class: 'w-full' } },
  showDefaultActions: false,
});

// ============ 商品明细 ============
function getItemStatus(item: any): { color: string; label: string } {
  const qty = Number(item.quantity) || 0;
  const delivered = Number(item.deliveredQuantity) || 0;
  if (delivered === 0) return { label: '未发货', color: 'default' };
  if (delivered >= qty) return { label: '已发完', color: 'green' };
  return { label: '部分发货', color: 'blue' };
}

function getProgressPercent(item: any): number {
  const qty = Number(item.quantity) || 0;
  const delivered = Number(item.deliveredQuantity) || 0;
  if (qty === 0) return 0;
  return Math.min(100, Math.round((delivered / qty) * 100));
}

function getProgressStatus(item: any): 'active' | 'normal' | 'success' {
  const qty = Number(item.quantity) || 0;
  const delivered = Number(item.deliveredQuantity) || 0;
  if (delivered >= qty && qty > 0) return 'success';
  if (delivered > 0) return 'active';
  return 'normal';
}

function getMaxShipQty(item: any): number {
  const qty = Number(item.quantity) || 0;
  const delivered = Number(item.deliveredQuantity) || 0;
  return Math.max(0, qty - delivered);
}

// 一键填满：所有未发完的商品填入未发货数量
function fillAllRemaining() {
  items.value.forEach((item) => {
    if (!item._disabled) {
      item.shipQuantity = getMaxShipQty(item);
    }
  });
}

// 清空发货数量
function clearAllShipQty() {
  items.value.forEach((item) => {
    item.shipQuantity = 0;
  });
}

// 过滤后的显示列表
const displayItems = computed(() => {
  if (!onlyPending.value) return items.value;
  return items.value.filter((it) => getMaxShipQty(it) > 0);
});

const itemColumns = [
  {
    title: '产品信息',
    dataIndex: 'productName',
    key: 'productName',
    width: 220,
  },
  {
    title: '订单数量',
    dataIndex: 'quantity',
    key: 'quantity',
    width: 100,
    align: 'center' as const,
  },
  {
    title: '已发货',
    dataIndex: 'deliveredQuantity',
    key: 'deliveredQuantity',
    width: 90,
    align: 'center' as const,
  },
  {
    title: '未发货',
    dataIndex: 'remaining',
    key: 'remaining',
    width: 90,
    align: 'center' as const,
  },
  { title: '发货进度', dataIndex: 'progress', key: 'progress', width: 180 },
  {
    title: '本次发货',
    dataIndex: 'shipQuantity',
    key: 'shipQuantity',
    width: 150,
  },
];

// 发货记录详情中显示的商品明细列（只读）
const detailItemColumns = [
  {
    title: '商品名称',
    dataIndex: 'productName',
    key: 'productName',
    ellipsis: true,
  },
  {
    title: '发货数量',
    dataIndex: 'quantity',
    key: 'quantity',
    width: 120,
    align: 'center' as const,
  },
];

// ============ 统计汇总 ============
const totalOrderQty = computed(() =>
  items.value.reduce((sum, item) => sum + (Number(item.quantity) || 0), 0),
);

const totalDeliveredQty = computed(() =>
  items.value.reduce(
    (sum, item) => sum + (Number(item.deliveredQuantity) || 0),
    0,
  ),
);

const totalRemainingQty = computed(() =>
  items.value.reduce((sum, item) => sum + getMaxShipQty(item), 0),
);

const totalShipQty = computed(() =>
  items.value.reduce((sum, item) => sum + (Number(item.shipQuantity) || 0), 0),
);

const overallProgress = computed(() => {
  if (totalOrderQty.value === 0) return 0;
  return Math.min(
    100,
    Math.round((totalDeliveredQty.value / totalOrderQty.value) * 100),
  );
});

const willCompleteAfterShip = computed(() => {
  return (
    items.value.every((item) => {
      const remaining = getMaxShipQty(item);
      const ship = Number(item.shipQuantity) || 0;
      return remaining === 0 || ship >= remaining;
    }) && totalShipQty.value > 0
  );
});

// ============ 订单选择弹窗（用于发货管理新增时选择订单） ============

// 打开订单选择弹窗
function openOrderSelectModal() {
  orderSelectVisible.value = true;
}

// 从弹窗选择订单后回调
async function handleOrderSelect(order: any) {
  selectedOrderId.value = Number(order.id);
  selectedOrderRow.value = order;
  await loadOrderInfo(selectedOrderId.value, selectedOrderRow.value);
}

// 更换订单（仅在从发货管理新增时可用）
function resetOrderSelection() {
  selectedOrderId.value = null;
  selectedOrderRow.value = null;
  items.value = [];
  historyShipments.value = [];
  formApi.resetForm();
  formApi.setValues({
    shipmentDate: new Date().toISOString().slice(0, 10),
  });
  // 重新弹出订单选择弹窗
  openOrderSelectModal();
}

// ============ 数据加载 ============
// 从发货管理列表进入（编辑/查看发货单）：先查发货单详情，拿到 orderId + 物流信息，再加载订单
async function loadShipmentForEdit(shipmentId: number) {
  loading.value = true;
  try {
    const res: any = await getShipmentInfoApi(shipmentId);
    const data = res?.data ?? res ?? {};
    const orderId = Number(data.orderId);
    if (!orderId) {
      message.error('发货单未关联订单');
      loading.value = false;
      return;
    }
    selectedOrderId.value = orderId;
    // 回显物流信息到表单（loadOrderInfo 内部会跳过表单填充）
    formApi.setValues({
      shipmentDate: data.shipmentDate,
      logisticsCompany: data.logisticsCompany,
      trackingNo: data.trackingNo,
      shippingMethod: data.shippingMethod,
      receiverName: data.receiverName,
      receiverPhone: data.receiverPhone,
      shippingAddress: data.shippingAddress,
      remark: data.remark,
    });
    // 加载订单商品 + 历史发货单（loadOrderInfo 内部会更新 selectedOrderRow 用于卡片展示）
    await loadOrderInfo(orderId, undefined, true);

    // 从发货单进入时默认切到发货记录 tab 并展开当前发货单
    activeTab.value = 'history';
    // 预先缓存当前发货单的商品明细（避免重复请求）
    if (data.items && Array.isArray(data.items)) {
      shipmentDetailsCache.value = {
        ...shipmentDetailsCache.value,
        [shipmentId]: data.items,
      };
    }
    expandedShipmentIds.value.add(shipmentId);
    expandedShipmentIds.value = new Set(expandedShipmentIds.value);

    // 从发货列表点"编辑"进入：直接打开修改弹窗，无需用户再手动点击"修改"按钮
    // data 中包含 id/orderId/customerId/shipmentNo 等字段，可直接作为 editingShipment
    openEditModal({
      ...data,
      id: shipmentId,
      orderId: Number(data.orderId),
      customerId: Number(data.customerId),
    });
  } catch (error) {
    console.error('[发货编辑] 加载发货单详情失败:', error);
  } finally {
    // 注意：loadOrderInfo 内部已有 finally 会清 loading，
    // 但若出错在 loadOrderInfo 之前，需要这里兜底
    loading.value = false;
  }
}

async function loadOrderInfo(
  orderId: number,
  rowOverride?: any,
  skipFormFill = false,
) {
  loading.value = true;
  try {
    const [infoResp, historyResp]: any = await Promise.all([
      getOrderInfoApi(orderId),
      getShipmentListApi({ orderId, page: 1, pageSize: 100 }),
    ]);

    // requestClient 配置了 responseReturn: 'data'，返回的就是 data 本身
    const data = infoResp && typeof infoResp === 'object' ? infoResp : {};
    const allItems = Array.isArray(data.items) ? data.items : [];

    // 更新 selectedOrderRow 以便在订单卡片中显示订单号、客户名等
    // 优先使用用户从 Modal 选择的 row 数据（rowOverride），data 作为补充
    if (data.id || data.orderNo || orderId) {
      const baseRow = rowOverride || selectedOrderRow.value || {};
      selectedOrderRow.value = {
        ...baseRow,
        id: orderId,
        orderNo: baseRow.orderNo || data.orderNo,
        title: baseRow.title || data.title,
        customerName: baseRow.customerName || data.customerName,
        customerId: data.customerId || baseRow.customerId,
        receiverName: data.receiverName || baseRow.receiverName,
        receiverPhone: data.receiverPhone || baseRow.receiverPhone,
        shippingAddress: data.shippingAddress || baseRow.shippingAddress,
        shippingMethod: data.shippingMethod || baseRow.shippingMethod,
        totalAmount: data.totalAmount ?? baseRow.totalAmount,
        orderStatus: data.orderStatus ?? baseRow.orderStatus,
        currency: data.currency ?? baseRow.currency,
      };
    }

    // 合并订单基础信息：优先使用 API 详情返回，其次使用列表 row 数据
    const orderRow = rowOverride || selectedOrderRow.value || props.row || {};

    // 仅显示实物商品（product_type=1），过滤已发完的也可通过 onlyPending 切换
    items.value = allItems
      .filter((it: any) => Number(it.productType) === 1)
      .map((it: any) => {
        const qty = Number(it.quantity) || 0;
        const delivered = Number(it.deliveredQuantity) || 0;
        const maxShip = Math.max(0, qty - delivered);
        return {
          id: it.id,
          productName: it.productName || '-',
          productCode: it.productCode || '',
          spec: it.spec || '',
          unit: it.unit || '',
          quantity: qty,
          deliveredQuantity: delivered,
          shipQuantity: 0,
          _disabled: maxShip === 0,
        };
      });

    // 加载历史发货单
    const historyData =
      historyResp && typeof historyResp === 'object' ? historyResp : {};
    historyShipments.value = historyData?.items ?? historyData?.list ?? [];

    // 编辑模式已经回显过表单，跳过表单填充
    if (!skipFormFill) {
      // 默认填充收货信息（API 详情优先，否则回退到列表 row）
      // shippingMethod 默认为 1（快递），避免 0/null 显示异常
      formApi.setValues({
        shipmentDate: new Date().toISOString().slice(0, 10),
        receiverName: data.receiverName || orderRow.receiverName,
        receiverPhone: data.receiverPhone || orderRow.receiverPhone,
        shippingAddress: data.shippingAddress || orderRow.shippingAddress,
        shippingMethod: data.shippingMethod || 1,
      });
    }

    // 根据发货状态自动切换选项卡
    // 已全部发完 → 直接显示发货记录
    // 还有未发完商品 → 显示发货选项卡
    const hasRemaining = items.value.some((it) => getMaxShipQty(it) > 0);
    activeTab.value = hasRemaining ? 'ship' : 'history';
  } catch (error) {
    console.error('[发货] 加载订单详情失败:', error);
    items.value = [];
    historyShipments.value = [];
    if (!skipFormFill) {
      formApi.setValues({
        shipmentDate: new Date().toISOString().slice(0, 10),
      });
    }
  } finally {
    loading.value = false;
  }
}

// ============ 提交 ============
async function handleSubmit() {
  try {
    // 先尝试 validate，再兜底 getValues（防止 TabPane 销毁导致 validate 返回空）
    let values: Record<string, any> = {};
    try {
      const result = await formApi.validate();
      values = result.values || {};
      if (!result.valid) {
        message.warning('请完善物流信息');
        return;
      }
    } catch (error) {
      console.warn('[订单提交] validate 异常，回退 getValues:', error);
    }
    // 兜底：如果 validate 返回空对象，尝试 getValues
    if (!values || Object.keys(values).length === 0) {
      try {
        const fallback = await formApi.getValues();
        if (fallback && Object.keys(fallback).length > 0) {
          values = fallback;
        }
      } catch (error) {
        console.warn('[订单提交] getValues 失败:', error);
      }
    }

    // 必填校验：发货日期是必填字段
    if (!values.shipmentDate) {
      message.warning('请选择发货日期');
      return;
    }

    const shipItems = items.value
      .filter((it) => Number(it.shipQuantity) > 0)
      .map((it) => ({
        orderItemId: it.id,
        productName: it.productName,
        quantity: Number(it.shipQuantity),
      }));

    if (shipItems.length === 0) {
      message.warning('请至少填写一条本次发货数量');
      return;
    }

    // 校验是否超出未发货数量
    const overLimit = items.value.find((it) => {
      const ship = Number(it.shipQuantity) || 0;
      return ship > getMaxShipQty(it);
    });
    if (overLimit) {
      message.error(`「${overLimit.productName}」发货数量超出未发货数量`);
      return;
    }

    submitting.value = true;
    const orderId = selectedOrderId.value ?? Number(props.row?.id);
    const orderRow = selectedOrderRow.value || props.row || {};
    // 明确列出所有要提交的字段，避免依赖 values 的展开
    const data = {
      shipmentDate: values.shipmentDate,
      logisticsCompany: values.logisticsCompany || undefined,
      trackingNo: values.trackingNo || undefined,
      shippingMethod: values.shippingMethod || undefined,
      receiverName: values.receiverName || undefined,
      receiverPhone: values.receiverPhone || undefined,
      shippingAddress: values.shippingAddress || undefined,
      remark: values.remark || undefined,
      orderId,
      customerId: Number(orderRow.customerId) || undefined,
      items: shipItems,
    };
    await createShipmentApi(data);
    message.success(
      willCompleteAfterShip.value ? '发货成功，订单商品已全部发完' : '发货成功',
    );
    drawerApi.setData({ needRefresh: true });
    // 清空详情缓存（因为有新发货单加入）
    shipmentDetailsCache.value = {};
    expandedShipmentIds.value = new Set();
    // 刷新订单数据（更新已发货数量 + 刷新历史发货记录）
    await loadOrderInfo(orderId, selectedOrderRow.value);
    // 切换到发货记录 tab，让用户看到刚提交的记录
    activeTab.value = 'history';
  } catch (error) {
    console.error('[发货] 提交失败:', error);
    message.error('操作失败');
  } finally {
    submitting.value = false;
  }
}

// ============ 修改发货单弹窗（行内入口） ============
const editModalVisible = ref(false);
const editSubmitting = ref(false);
const editingShipment = ref<any>(null);
// 时间轴刷新触发器：每次修改成功后递增
const logRefreshKey = ref(0);

// 修改弹窗使用独立的表单实例（避免与主表单冲突）
const editFormSchema: VbenFormSchema[] = [
  {
    component: 'DatePicker',
    fieldName: 'shipmentDate',
    label: '发货日期',
    rules: 'required',
    componentProps: {
      placeholder: '请选择',
      style: 'width:100%',
      valueFormat: 'YYYY-MM-DD',
      // 日历弹窗渲染到 body，z-index 高于 Modal(2100)
      dropdownStyle: { zIndex: 2200 },
      dropdownClassName: 'shipment-select-dropdown',
      popupStyle: { zIndex: 2200 },
      popupClassName: 'shipment-select-dropdown',
    },
  },
  {
    component: 'Select',
    fieldName: 'logisticsCompany',
    label: '物流公司',
    componentProps: {
      placeholder: '请选择物流公司',
      options: logisticsCompanyOptions,
      showSearch: true,
      allowClear: true,
      // 下拉渲染到 body（默认），并通过 dropdownStyle 提升 z-index 高于 Modal(2100)
      dropdownStyle: { zIndex: 2200 },
      dropdownClassName: 'shipment-select-dropdown',
      filterOption: (input: string, option: any) =>
        String(option?.label ?? '')
          .toLowerCase()
          .includes(input.toLowerCase()),
    },
  },
  {
    component: 'Input',
    fieldName: 'trackingNo',
    label: '物流单号',
    componentProps: { placeholder: '物流运单号' },
  },
  {
    component: 'Select',
    fieldName: 'shippingMethod',
    label: '配送方式',
    defaultValue: 1,
    componentProps: {
      placeholder: '请选择',
      options: shippingMethodOptions,
      allowClear: true,
      // 下拉渲染到 body（默认），并通过 dropdownStyle 提升 z-index 高于 Modal(2100)
      dropdownStyle: { zIndex: 2200 },
      dropdownClassName: 'shipment-select-dropdown',
    },
  },
  {
    component: 'Input',
    fieldName: 'receiverName',
    label: '收货人',
    componentProps: { placeholder: '收货人姓名' },
  },
  {
    component: 'Input',
    fieldName: 'receiverPhone',
    label: '收货电话',
    componentProps: { placeholder: '收货人电话' },
  },
  {
    component: 'Textarea',
    fieldName: 'shippingAddress',
    label: '收货地址',
    componentProps: { placeholder: '详细收货地址', rows: 2 },
    wrapperClass: 'col-span-2',
  },
  {
    component: 'Textarea',
    fieldName: 'remark',
    label: '备注',
    componentProps: {
      placeholder: '备注信息',
      rows: 3,
      showCount: true,
      maxlength: 500,
    },
    wrapperClass: 'col-span-2',
  },
];

const [EditForm, editFormApi] = useVbenForm({
  schema: editFormSchema,
  wrapperClass: 'grid-cols-2 gap-4',
  compact: true,
  commonConfig: { componentProps: { class: 'w-full' } },
  showDefaultActions: false,
});

function openEditModal(ship: any) {
  editingShipment.value = ship;
  editModalVisible.value = true;
  // 回显当前发货单数据（shippingMethod 为 0/null 时默认显示快递=1）
  editFormApi.resetForm();
  editFormApi.setValues({
    shipmentDate: ship.shipmentDate,
    logisticsCompany: ship.logisticsCompany,
    trackingNo: ship.trackingNo,
    shippingMethod: ship.shippingMethod || 1,
    receiverName: ship.receiverName,
    receiverPhone: ship.receiverPhone,
    shippingAddress: ship.shippingAddress,
    remark: ship.remark,
  });
}

async function handleEditSubmit() {
  if (!editingShipment.value) return;

  // 跟主表单相同的问题：editFormApi.validate() 在 Modal/TabPane 内可能返回空 values
  // 需要用 getValues() 兜底，确保物流字段被收集
  let vals: Record<string, any> = {};
  try {
    const result = await editFormApi.validate();
    vals = (result?.values || {}) as Record<string, any>;
    if (!result?.valid) {
      message.warning('请完善必填字段');
      return;
    }
  } catch (error) {
    console.warn('[发货修改] validate 异常，回退 getValues:', error);
  }
  // validate 返回空时用 getValues 兜底
  if (!vals || Object.keys(vals).length === 0) {
    try {
      const fallback = await editFormApi.getValues();
      if (fallback && Object.keys(fallback).length > 0) {
        vals = fallback as Record<string, any>;
      }
    } catch (error) {
      console.warn('[发货修改] getValues 失败:', error);
    }
  }

  editSubmitting.value = true;
  try {
    const payload = {
      id: Number(editingShipment.value.id),
      shipmentDate: vals.shipmentDate,
      logisticsCompany: vals.logisticsCompany || undefined,
      trackingNo: vals.trackingNo || undefined,
      shippingMethod: vals.shippingMethod || undefined,
      receiverName: vals.receiverName || undefined,
      receiverPhone: vals.receiverPhone || undefined,
      shippingAddress: vals.shippingAddress || undefined,
      remark: vals.remark || undefined,
      // 不修改 items，避免影响已发数量；如需修改 items，应在专门入口处理
      orderId: Number(editingShipment.value.orderId) || undefined,
      customerId: Number(editingShipment.value.customerId) || undefined,
    };
    await updateShipmentApi(payload);
    message.success('修改成功');
    editModalVisible.value = false;
    // 触发时间轴刷新
    logRefreshKey.value += 1;
    // 刷新发货记录列表
    if (selectedOrderId.value) {
      await loadOrderInfo(selectedOrderId.value, selectedOrderRow.value, true);
    }
    // 设置当前修改的发货单为展开状态
    if (editingShipment.value?.id) {
      expandedShipmentIds.value.add(Number(editingShipment.value.id));
      expandedShipmentIds.value = new Set(expandedShipmentIds.value);
    }
  } catch (error) {
    console.error('[发货修改] 提交失败:', error);
  } finally {
    editSubmitting.value = false;
  }
}

function handleEditCancel() {
  editModalVisible.value = false;
  editingShipment.value = null;
}

const [Drawer, drawerApi] = useVbenDrawer({
  onCancel() {
    drawerApi.close();
  },
  async onConfirm() {
    await handleSubmit();
  },
  onOpenChange(isOpen) {
    if (isOpen) {
      isFullscreen.value = false;
      onlyPending.value = false;
      formApi.resetForm();
      items.value = [];
      historyShipments.value = [];
      // 重置订单选择器
      selectedOrderId.value = null;
      selectedOrderRow.value = null;
      hasInitRow.value = false;
      // 重置选项卡和详情缓存
      activeTab.value = 'ship';
      shipmentDetailsCache.value = {};
      expandedShipmentIds.value = new Set();
      detailLoading.value = new Set();

      // 关键修复：优先用 drawerApi.getData() 获取数据
      // 直接读 props.row 在 connectedComponent 模式下存在竞态——
      // setData 后、open() 触发 onOpenChange 时 props.row 可能还没更新，导致走到 else 弹出订单选择
      const drawerData = drawerApi.getData() || {};
      const row = drawerData.row || props.row || {};

      // 场景判断（按可靠性排序）：
      // 1. 发货管理编辑：row.id 是发货单 ID，有 shipmentNo（或 orderId/logisticsCompany 等发货单字段）
      // 2. 订单列表点"发货"：row.id 是订单 ID，有 orderNo 无 shipmentNo
      // 3. 新增：row 为空
      if (row.id && row.shipmentNo) {
        // 发货管理编辑：先查发货单详情拿 orderId + 物流信息，再加载订单
        hasInitRow.value = true;
        loading.value = true; // 防止异步期间订单选择器闪现
        void loadShipmentForEdit(Number(row.id));
      } else if (row.id && row.orderNo) {
        // 订单列表进入：直接加载订单信息
        hasInitRow.value = true;
        loading.value = true;
        selectedOrderId.value = Number(row.id);
        selectedOrderRow.value = row;
        void loadOrderInfo(selectedOrderId.value, row);
      } else if (row.id) {
        // 兜底：有 id 但无明显标识，默认按发货单处理（因为发货管理是最常用入口）
        console.warn(
          '[发货Drawer] 无法明确识别入口场景，默认按发货单处理:',
          row,
        );
        hasInitRow.value = true;
        loading.value = true;
        void loadShipmentForEdit(Number(row.id));
      } else {
        // 新增：弹出订单选择弹窗
        openOrderSelectModal();
      }
    }
  },
});

// 监听 submitting 状态，更新抽屉 loading
watch(submitting, (val) => {
  drawerApi.setState({ confirmLoading: val });
});
</script>

<template>
  <Drawer
    title="分批发货分配"
    :class="drawerClass"
    :destroy-on-close="true"
    :z-index="2000"
  >
    <template #extra>
      <Tooltip :title="isFullscreen ? '退出全屏' : '全屏'">
        <button
          type="button"
          class="sale-shipment-drawer__fs-btn"
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

    <!-- ============ 加载中（编辑场景下异步加载发货单/订单详情期间显示，防止订单选择器闪现）============ -->
    <div v-if="loading && !selectedOrderId" class="shipment-loading">
      <Spin size="large" tip="正在加载发货单信息..." />
    </div>

    <!-- ============ 订单选择器（新增场景且未加载时显示）============ -->
    <div v-else-if="!selectedOrderId" class="shipment-order-picker">
      <div class="shipment-order-picker__icon">
        <svg
          viewBox="0 0 24 24"
          width="36"
          height="36"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M9 11l3 3L22 4" />
          <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" />
        </svg>
      </div>
      <div class="shipment-order-picker__title">请先选择需要发货的订单</div>
      <div class="shipment-order-picker__desc">
        仅显示「已确认 / 备货中 / 部分发货」状态的可发货订单
      </div>
      <Button type="primary" size="large" @click="openOrderSelectModal">
        选择订单
      </Button>
    </div>

    <!-- ============ 订单概览卡片（选择订单后显示） ============ -->
    <template v-else>
      <div class="shipment-order-card">
        <div class="shipment-order-card__header">
          <div class="shipment-order-card__title">
            <span class="shipment-order-card__no">{{
              (selectedOrderRow || props.row)?.orderNo || '-'
            }}</span>
            <Tag
              v-if="(selectedOrderRow || props.row)?.orderStatus"
              :color="
                orderStatusColorMap[
                  (selectedOrderRow || props.row)?.orderStatus
                ] || 'default'
              "
              class="shipment-order-card__status"
            >
              {{
                orderStatusLabelMap[
                  (selectedOrderRow || props.row)?.orderStatus
                ] || '-'
              }}
            </Tag>
            <a
              v-if="!hasInitRow"
              class="shipment-order-card__change"
              @click="resetOrderSelection"
              >更换订单</a
            >
          </div>
          <div class="shipment-order-card__title-text">
            {{ (selectedOrderRow || props.row)?.title || '-' }}
          </div>
        </div>
        <div class="shipment-order-card__body">
          <div class="shipment-order-card__info">
            <div class="shipment-meta">
              <span class="shipment-meta__label">客户</span>
              <span class="shipment-meta__value">{{
                (selectedOrderRow || props.row)?.customerName || '-'
              }}</span>
            </div>
            <div class="shipment-meta">
              <span class="shipment-meta__label">收货人</span>
              <span class="shipment-meta__value">{{
                (selectedOrderRow || props.row)?.receiverName || '-'
              }}</span>
            </div>
            <div class="shipment-meta">
              <span class="shipment-meta__label">联系电话</span>
              <span class="shipment-meta__value">{{
                (selectedOrderRow || props.row)?.receiverPhone || '-'
              }}</span>
            </div>
            <div class="shipment-meta">
              <span class="shipment-meta__label">订单金额</span>
              <span class="shipment-meta__value shipment-meta__value--amount">
                ¥
                {{
                  (
                    selectedOrderRow || props.row
                  )?.totalAmount?.toLocaleString?.() ??
                  (selectedOrderRow || props.row)?.totalAmount ??
                  0
                }}
              </span>
            </div>
          </div>
          <div class="shipment-order-card__progress">
            <Progress
              type="circle"
              :percent="overallProgress"
              :size="86"
              :stroke-color="overallProgress === 100 ? '#52c41a' : '#1890ff'"
            >
              <template #default="{ percent }">
                <div class="shipment-progress-inner">
                  <span class="shipment-progress-inner__percent"
                    >{{ percent }}%</span
                  >
                  <span class="shipment-progress-inner__label">已发</span>
                </div>
              </template>
            </Progress>
            <div class="shipment-progress-stats">
              <div class="shipment-progress-stat">
                <span class="shipment-progress-stat__num">{{
                  totalOrderQty
                }}</span>
                <span class="shipment-progress-stat__label">订单总数</span>
              </div>
              <div class="shipment-progress-stat">
                <span
                  class="shipment-progress-stat__num shipment-progress-stat__num--done"
                  >{{ totalDeliveredQty }}</span
                >
                <span class="shipment-progress-stat__label">已发货</span>
              </div>
              <div class="shipment-progress-stat">
                <span
                  class="shipment-progress-stat__num shipment-progress-stat__num--pending"
                  >{{ totalRemainingQty }}</span
                >
                <span class="shipment-progress-stat__label">待发货</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- ============ 选项卡：发货 / 发货记录 ============ -->
      <Tabs v-model:active-key="activeTab" class="shipment-tabs">
        <!-- ====== 发货选项卡（订单全部发完时自动隐藏）====== -->
        <Tabs.TabPane
          v-if="!isOrderFullyShipped"
          key="ship"
          tab="本次发货"
          force-render
        >
          <!-- 发货商品分配 -->
          <div class="shipment-section">
            <div class="shipment-section__header">
              <div class="shipment-section__title">
                <span class="shipment-section__bar"></span>
                <span>发货商品分配</span>
                <Tag color="blue" class="shipment-section__count">
                  {{ displayItems.length }} 项
                </Tag>
              </div>
              <div class="shipment-section__actions">
                <a class="shipment-link" @click="onlyPending = !onlyPending">
                  {{ onlyPending ? '显示全部' : '仅显示未发完' }}
                </a>
                <a
                  class="shipment-link shipment-link--primary"
                  @click="fillAllRemaining"
                  >一键填满</a
                >
                <a class="shipment-link" @click="clearAllShipQty">清空</a>
              </div>
            </div>

            <Table
              :columns="itemColumns"
              :data-source="displayItems"
              :pagination="false"
              :loading="loading"
              :row-key="(record: any) => record.id"
              :scroll="{ x: 830 }"
              size="small"
              bordered
              :row-class-name="
                (record: any) => (record._disabled ? 'shipment-row--done' : '')
              "
            >
              <template #bodyCell="{ column, record }">
                <template v-if="column.key === 'productName'">
                  <div class="shipment-product">
                    <div class="shipment-product__name">
                      {{ record.productName }}
                    </div>
                    <div class="shipment-product__meta">
                      <span v-if="record.productCode"
                        >编码: {{ record.productCode }}</span
                      >
                      <span v-if="record.spec"> | 规格: {{ record.spec }}</span>
                      <span v-if="record.unit"> | {{ record.unit }}</span>
                    </div>
                  </div>
                </template>
                <template v-else-if="column.key === 'quantity'">
                  <span class="shipment-num">{{ record.quantity }}</span>
                </template>
                <template v-else-if="column.key === 'deliveredQuantity'">
                  <span class="shipment-num shipment-num--done">{{
                    record.deliveredQuantity
                  }}</span>
                </template>
                <template v-else-if="column.key === 'remaining'">
                  <span
                    class="shipment-num"
                    :class="{
                      'shipment-num--pending': getMaxShipQty(record) > 0,
                    }"
                  >
                    {{ getMaxShipQty(record) }}
                  </span>
                </template>
                <template v-else-if="column.key === 'progress'">
                  <div class="shipment-progress-cell">
                    <Progress
                      :percent="getProgressPercent(record)"
                      :status="getProgressStatus(record)"
                      size="small"
                      :stroke-color="
                        getProgressStatus(record) === 'success'
                          ? '#52c41a'
                          : '#1890ff'
                      "
                    />
                    <Tag
                      :color="getItemStatus(record).color"
                      class="shipment-progress-cell__tag"
                    >
                      {{ getItemStatus(record).label }}
                    </Tag>
                  </div>
                </template>
                <template v-else-if="column.key === 'shipQuantity'">
                  <InputNumber
                    v-model:value="record.shipQuantity"
                    :min="0"
                    :max="getMaxShipQty(record)"
                    :precision="0"
                    :disabled="record._disabled"
                    style="width: 130px"
                    placeholder="发货数量"
                  />
                  <div
                    v-if="getMaxShipQty(record) > 0"
                    class="shipment-ship-hint"
                  >
                    可发 {{ getMaxShipQty(record) }}
                  </div>
                </template>
              </template>

              <template #emptyText>
                <Empty description="暂无可发货商品" />
              </template>

              <template #footer>
                <div class="shipment-summary-bar">
                  <div class="shipment-summary-bar__left">
                    <span class="shipment-summary-bar__label"
                      >本次发货总数量</span
                    >
                    <span
                      class="shipment-summary-bar__value shipment-summary-bar__value--primary"
                    >
                      {{ totalShipQty }}
                    </span>
                    <span class="shipment-summary-bar__unit">件</span>
                  </div>
                  <div class="shipment-summary-bar__right">
                    <template v-if="totalShipQty > 0">
                      <span
                        v-if="willCompleteAfterShip"
                        class="shipment-summary-bar__tip shipment-summary-bar__tip--success"
                      >
                        ✓ 发货后该订单将全部发完
                      </span>
                      <span v-else class="shipment-summary-bar__tip">
                        发货后剩余 {{ totalRemainingQty - totalShipQty }} 件待发
                      </span>
                    </template>
                  </div>
                </div>
              </template>
            </Table>
          </div>

          <!-- 物流信息表单 -->
          <div class="shipment-section">
            <div class="shipment-section__header">
              <div class="shipment-section__title">
                <span class="shipment-section__bar"></span>
                <span>物流与收货信息</span>
              </div>
            </div>
            <Form />
          </div>
        </Tabs.TabPane>

        <!-- ====== 发货记录选项卡 ====== -->
        <Tabs.TabPane
          key="history"
          :tab="`发货记录 (${historyShipments.length})`"
        >
          <div
            v-if="historyShipments.length === 0"
            class="shipment-empty-history"
          >
            <Empty
              description="暂无发货记录"
              :image-style="{ height: '60px' }"
            />
          </div>
          <div v-else class="shipment-history-list">
            <div
              v-for="ship in historyShipments"
              :key="ship.id"
              class="shipment-history-item"
              :class="{
                'shipment-history-item--expanded': expandedShipmentIds.has(
                  Number(ship.id),
                ),
              }"
            >
              <div
                class="shipment-history-item__header-row"
                @click="toggleShipmentDetail(ship)"
              >
                <div class="shipment-history-item__main">
                  <div class="shipment-history-item__header">
                    <span class="shipment-history-item__no">{{
                      ship.shipmentNo || `#${ship.id}`
                    }}</span>
                    <Tag
                      v-if="ship.status"
                      :color="shipmentStatusMap[ship.status]?.color"
                      class="shipment-history-item__status"
                    >
                      {{ shipmentStatusMap[ship.status]?.label || '-' }}
                    </Tag>
                    <Tag
                      v-if="ship.shippingMethod"
                      :color="shippingMethodMap[ship.shippingMethod]?.color"
                    >
                      {{ shippingMethodMap[ship.shippingMethod]?.label }}
                    </Tag>
                  </div>
                  <div class="shipment-history-item__meta">
                    <span>发货日期：{{ ship.shipmentDate || '-' }}</span>
                    <span v-if="ship.logisticsCompany">
                      | 物流：{{ ship.logisticsCompany }}</span
                    >
                    <span v-if="ship.trackingNo">
                      | 单号：{{ ship.trackingNo }}</span
                    >
                  </div>
                  <div class="shipment-history-item__receiver">
                    收货人：{{ ship.receiverName || '-' }} · 电话：{{
                      ship.receiverPhone || '-'
                    }}
                  </div>
                </div>
                <div class="shipment-history-item__qty">
                  <div class="shipment-history-item__qty-num">
                    {{ ship.totalQuantity ?? 0 }}
                  </div>
                  <div class="shipment-history-item__qty-label">发货数量</div>
                </div>
                <div class="shipment-history-item__actions" @click.stop>
                  <Tooltip title="修改物流/收货信息">
                    <Button
                      type="link"
                      size="small"
                      :icon="h(LucidePencil)"
                      @click="openEditModal(ship)"
                    />
                  </Tooltip>
                </div>
                <div class="shipment-history-item__toggle">
                  <span v-if="detailLoading.has(Number(ship.id))"
                    >加载中...</span
                  >
                  <a v-else>
                    {{
                      expandedShipmentIds.has(Number(ship.id))
                        ? '收起'
                        : '查看详情'
                    }}
                  </a>
                </div>
              </div>

              <!-- 展开的详情：商品明细 + 完整物流信息 -->
              <div
                v-if="expandedShipmentIds.has(Number(ship.id))"
                class="shipment-history-detail"
              >
                <div class="shipment-history-detail__section">
                  <div class="shipment-history-detail__title">商品明细</div>
                  <Table
                    :columns="detailItemColumns"
                    :data-source="shipmentDetailsCache[Number(ship.id)] || []"
                    :pagination="false"
                    :loading="detailLoading.has(Number(ship.id))"
                    :row-key="(record: any) => record.id"
                    size="small"
                    bordered
                  >
                    <template #emptyText>
                      <Empty
                        description="暂无商品明细"
                        :image-style="{ height: '50px' }"
                      />
                    </template>
                  </Table>
                </div>

                <div class="shipment-history-detail__section">
                  <div class="shipment-history-detail__title">收货信息</div>
                  <div class="shipment-history-detail__grid">
                    <div class="shipment-history-detail__item">
                      <span class="shipment-history-detail__label">收货人</span>
                      <span class="shipment-history-detail__value">{{
                        ship.receiverName || '-'
                      }}</span>
                    </div>
                    <div class="shipment-history-detail__item">
                      <span class="shipment-history-detail__label"
                        >联系电话</span
                      >
                      <span class="shipment-history-detail__value">{{
                        ship.receiverPhone || '-'
                      }}</span>
                    </div>
                    <div
                      class="shipment-history-detail__item shipment-history-detail__item--full"
                    >
                      <span class="shipment-history-detail__label"
                        >收货地址</span
                      >
                      <span class="shipment-history-detail__value">{{
                        ship.shippingAddress || '-'
                      }}</span>
                    </div>
                    <div class="shipment-history-detail__item">
                      <span class="shipment-history-detail__label"
                        >物流公司</span
                      >
                      <span class="shipment-history-detail__value">{{
                        ship.logisticsCompany || '-'
                      }}</span>
                    </div>
                    <div class="shipment-history-detail__item">
                      <span class="shipment-history-detail__label"
                        >物流单号</span
                      >
                      <span class="shipment-history-detail__value">{{
                        ship.trackingNo || '-'
                      }}</span>
                    </div>
                    <div class="shipment-history-detail__item">
                      <span class="shipment-history-detail__label"
                        >配送方式</span
                      >
                      <span class="shipment-history-detail__value">
                        {{
                          ship.shippingMethod
                            ? shippingMethodMap[ship.shippingMethod]?.label ||
                              '-'
                            : '-'
                        }}
                      </span>
                    </div>
                    <div class="shipment-history-detail__item">
                      <span class="shipment-history-detail__label"
                        >发货日期</span
                      >
                      <span class="shipment-history-detail__value">{{
                        ship.shipmentDate || '-'
                      }}</span>
                    </div>
                    <div
                      v-if="ship.remark"
                      class="shipment-history-detail__item shipment-history-detail__item--full"
                    >
                      <span class="shipment-history-detail__label">备注</span>
                      <span class="shipment-history-detail__value">{{
                        ship.remark
                      }}</span>
                    </div>
                  </div>
                </div>

                <!-- 操作追溯时间轴：显示该发货单的所有修改记录 -->
                <div class="shipment-history-detail__section">
                  <ShipmentEditLogTimeline
                    :shipment-id="Number(ship.id)"
                    :refresh-key="logRefreshKey"
                  />
                </div>
              </div>
            </div>
          </div>
        </Tabs.TabPane>
      </Tabs>
    </template>
    <!-- ============ 订单选择弹窗 ============ -->
    <OrderSelectModal
      v-model:visible="orderSelectVisible"
      @select="handleOrderSelect"
    />

    <!-- ============ 修改发货单弹窗（行内入口）============ -->
    <Modal
      :open="editModalVisible"
      title="修改发货单 · 物流与收货信息"
      width="680px"
      :z-index="2100"
      :destroy-on-close="true"
      :mask-closable="false"
      :confirm-loading="editSubmitting"
      ok-text="保存修改"
      cancel-text="取消"
      @ok="handleEditSubmit"
      @cancel="handleEditCancel"
    >
      <div v-if="editingShipment" class="shipment-edit-modal__header">
        <Tag color="blue">
          {{ editingShipment.shipmentNo || `#${editingShipment.id}` }}
        </Tag>
        <span class="shipment-edit-modal__hint">
          修改后会自动记录到操作追溯时间轴，便于追责
        </span>
      </div>
      <EditForm />
    </Modal>
  </Drawer>
</template>

<style>
/* ============ 下拉层 z-index 提升（高于 Modal 的 2100）============ */
.shipment-select-dropdown {
  z-index: 2200 !important;
}

/* ============ 抽屉容器 ============ */
.sale-shipment-drawer {
  width: 78vw !important;
  max-width: 1400px;
}

.sale-shipment-drawer--fullscreen {
  width: 100vw !important;
  max-width: 100vw;
}

.sale-shipment-drawer__fs-btn {
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

.sale-shipment-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgb(0 0 0 / 6%);
}

/* ============ 加载中容器 ============ */
.shipment-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 80px 0;
}

/* ============ 订单选择器（未选订单时） ============ */
.shipment-order-picker {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 56px 24px 48px;
  margin-bottom: 16px;
  text-align: center;
  background: linear-gradient(180deg, #fafbff 0%, #f0f5ff 100%);
  border: 1px dashed #adc6ff;
  border-radius: 10px;
}

.shipment-order-picker__icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 64px;
  height: 64px;
  margin-bottom: 16px;
  color: #1890ff;
  background: #e6f7ff;
  border-radius: 50%;
}

.shipment-order-picker__title {
  margin-bottom: 6px;
  font-size: 16px;
  font-weight: 600;
  color: #1f1f1f;
}

.shipment-order-picker__desc {
  margin-bottom: 24px;
  font-size: 12px;
  color: #8c8c8c;
}

/* 更换订单链接 */
.shipment-order-card__change {
  margin-left: auto;
  font-size: 12px;
  font-weight: normal;
  color: #1890ff;
  text-decoration: underline;
  cursor: pointer;
}

.shipment-order-card__change:hover {
  color: #40a9ff;
}

/* ============ 订单概览卡片 ============ */
.shipment-order-card {
  position: relative;
  margin-bottom: 16px;
  overflow: hidden;
  background: linear-gradient(135deg, #e6f7ff 0%, #f0f9ff 50%, #f6ffed 100%);
  border: 1px solid #d6e4ff;
  border-radius: 8px;
}

.shipment-order-card::before {
  position: absolute;
  top: 0;
  bottom: 0;
  left: 0;
  width: 4px;
  content: '';
  background: linear-gradient(180deg, #1890ff 0%, #52c41a 100%);
}

.shipment-order-card__header {
  padding: 14px 20px 6px;
  border-bottom: 1px solid rgb(24 144 255 / 10%);
}

.shipment-order-card__title {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 4px;
}

.shipment-order-card__no {
  font-size: 16px;
  font-weight: 600;
  color: #1f1f1f;
  letter-spacing: 0.3px;
}

.shipment-order-card__status {
  margin-left: 4px;
}

.shipment-order-card__title-text {
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 13px;
  color: #595959;
  white-space: nowrap;
}

.shipment-order-card__body {
  display: flex;
  gap: 24px;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
}

.shipment-order-card__info {
  display: grid;
  flex: 1;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px 32px;
}

.shipment-meta {
  display: flex;
  gap: 8px;
  align-items: baseline;
}

.shipment-meta__label {
  min-width: 56px;
  font-size: 12px;
  color: #8c8c8c;
}

.shipment-meta__value {
  font-size: 14px;
  font-weight: 500;
  color: #262626;
}

.shipment-meta__value--amount {
  font-size: 15px;
  font-weight: 600;
  color: #cf1322;
}

.shipment-order-card__progress {
  display: flex;
  gap: 20px;
  align-items: center;
  padding-left: 24px;
  border-left: 1px solid rgb(24 144 255 / 15%);
}

.shipment-progress-inner {
  display: flex;
  flex-direction: column;
  align-items: center;
  line-height: 1.2;
}

.shipment-progress-inner__percent {
  font-size: 18px;
  font-weight: 600;
  color: #1890ff;
}

.shipment-progress-inner__label {
  margin-top: 2px;
  font-size: 11px;
  color: #8c8c8c;
}

.shipment-progress-stats {
  display: flex;
  gap: 16px;
}

.shipment-progress-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  min-width: 56px;
}

.shipment-progress-stat__num {
  font-size: 20px;
  font-weight: 600;
  line-height: 1.2;
  color: #262626;
}

.shipment-progress-stat__num--done {
  color: #52c41a;
}

.shipment-progress-stat__num--pending {
  color: #fa8c16;
}

.shipment-progress-stat__label {
  margin-top: 2px;
  font-size: 11px;
  color: #8c8c8c;
}

/* ============ 通用区块 ============ */
.shipment-section {
  margin-bottom: 20px;
}

.shipment-section__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 2px;
  margin-bottom: 12px;
}

.shipment-section__title {
  display: flex;
  gap: 8px;
  align-items: center;
  font-size: 14px;
  font-weight: 600;
  color: #1f1f1f;
}

.shipment-section__bar {
  display: inline-block;
  width: 3px;
  height: 14px;
  background: #1890ff;
  border-radius: 2px;
}

.shipment-section__count {
  margin-left: 4px;
}

.shipment-section__actions {
  display: flex;
  gap: 12px;
  align-items: center;
}

.shipment-link {
  font-size: 12px;
  color: #595959;
  cursor: pointer;
  transition: color 0.2s;
}

.shipment-link:hover {
  color: #1890ff;
}

.shipment-link--primary {
  font-weight: 500;
  color: #1890ff;
}

/* ============ 商品明细表格 ============ */
.shipment-product {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.shipment-product__name {
  font-size: 13px;
  font-weight: 500;
  color: #262626;
}

.shipment-product__meta {
  font-size: 11px;
  color: #8c8c8c;
}

.shipment-num {
  font-size: 14px;
  font-weight: 500;
  color: #262626;
}

.shipment-num--done {
  color: #52c41a;
}

.shipment-num--pending {
  font-weight: 600;
  color: #fa8c16;
}

.shipment-progress-cell {
  display: flex;
  gap: 8px;
  align-items: center;
}

.shipment-progress-cell__tag {
  flex-shrink: 0;
  margin: 0;
}

.shipment-ship-hint {
  margin-top: 2px;
  font-size: 11px;
  color: #fa8c16;
  text-align: center;
}

.shipment-row--done {
  background-color: #f6ffed;
}

.shipment-row--done:hover > td {
  background-color: #f0ffe0 !important;
}

/* ============ 底部汇总栏 ============ */
.shipment-summary-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px;
}

.shipment-summary-bar__left {
  display: flex;
  gap: 6px;
  align-items: baseline;
}

.shipment-summary-bar__label {
  font-size: 13px;
  color: #595959;
}

.shipment-summary-bar__value {
  font-size: 20px;
  font-weight: 600;
  color: #262626;
}

.shipment-summary-bar__value--primary {
  color: #1890ff;
}

.shipment-summary-bar__unit {
  font-size: 12px;
  color: #8c8c8c;
}

.shipment-summary-bar__right {
  flex: 1;
  text-align: right;
}

.shipment-summary-bar__tip {
  font-size: 12px;
  color: #595959;
}

.shipment-summary-bar__tip--success {
  font-weight: 500;
  color: #52c41a;
}

/* ============ 历史发货记录 ============ */
.shipment-empty-history {
  padding: 16px 0;
  text-align: center;
}

.shipment-history-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: none;
  padding-right: 4px;
}

.shipment-history-item {
  position: relative;
  overflow: hidden;
  background: linear-gradient(90deg, rgb(15 41 66 / 2%) 0%, transparent 30%);
  border: 1px solid #e5e7eb;
  border-left: 3px solid #0f2942;
  border-radius: 4px;
  transition: all 0.25s ease;
}

.shipment-history-item::before {
  position: absolute;
  top: 0;
  right: 0;
  width: 28px;
  height: 28px;
  pointer-events: none;
  content: '';
  background: linear-gradient(
    225deg,
    rgb(245 158 11 / 18%) 0%,
    transparent 60%
  );
}

.shipment-history-item:hover {
  background: linear-gradient(
    90deg,
    rgb(15 41 66 / 4%) 0%,
    rgb(245 158 11 / 3%) 50%,
    transparent 100%
  );
  border-color: #f59e0b;
  box-shadow: 0 2px 8px rgb(245 158 11 / 8%);
  transform: translateX(2px);
}

.shipment-history-item--expanded {
  background: linear-gradient(
    90deg,
    rgb(15 41 66 / 5%) 0%,
    rgb(245 158 11 / 4%) 100%
  );
  border-color: #f59e0b;
}

.shipment-history-item__no {
  font-family: 'JetBrains Mono', 'Cascadia Code', Menlo, Consolas, monospace;
  font-size: 13px;
  font-weight: 600;
  color: #0f2942;
  letter-spacing: 0.5px;
}

.shipment-history-item__header-row {
  display: flex;
  align-items: stretch;
  justify-content: space-between;
  padding: 12px 14px;
  cursor: pointer;
  user-select: none;
}

.shipment-history-item__main {
  flex: 1;
  min-width: 0;
}

.shipment-history-item__header {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 4px;
}

.shipment-history-item__status {
  margin: 0;
}

.shipment-history-item__meta {
  margin-bottom: 2px;
  font-size: 12px;
  color: #595959;
}

.shipment-history-item__receiver {
  font-size: 12px;
  color: #8c8c8c;
}

.shipment-history-item__qty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-width: 80px;
  padding: 4px 12px;
  margin-left: 12px;
  border-left: 1px dashed #d9d9d9;
}

.shipment-history-item__qty-num {
  font-size: 20px;
  font-weight: 600;
  line-height: 1.2;
  color: #1890ff;
}

.shipment-history-item__qty-label {
  margin-top: 2px;
  font-size: 11px;
  color: #8c8c8c;
}

.shipment-history-item__toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 70px;
  padding-left: 12px;
  margin-left: 12px;
  font-size: 12px;
  color: #1890ff;
  border-left: 1px dashed #d9d9d9;
}

.shipment-history-item__toggle a {
  font-weight: 500;
  color: #1890ff;
}

.shipment-history-item__toggle a:hover {
  color: #40a9ff;
}

/* 修改按钮区 */
.shipment-history-item__actions {
  display: flex;
  gap: 4px;
  align-items: center;
  padding: 0 8px;
  margin-left: 8px;
  border-left: 1px dashed #d9d9d9;
}

.shipment-history-item__actions :deep(.ant-btn) {
  padding: 0 6px;
  color: #f59e0b;
}

.shipment-history-item__actions :deep(.ant-btn:hover) {
  color: #d97706;
  background: rgb(245 158 11 / 10%);
}

/* 修改弹窗头 */
.shipment-edit-modal__header {
  display: flex;
  gap: 12px;
  align-items: center;
  padding: 8px 12px;
  margin-bottom: 16px;
  background: linear-gradient(
    90deg,
    rgb(15 41 66 / 4%) 0%,
    rgb(245 158 11 / 6%) 100%
  );
  border-left: 3px solid #f59e0b;
  border-radius: 0 4px 4px 0;
}

.shipment-edit-modal__hint {
  font-size: 12px;
  color: #6b7280;
}

/* ============ 发货记录详情展开 ============ */
.shipment-history-detail {
  padding: 12px 16px 16px;
  background: #fff;
  border-top: 1px dashed #d9d9d9;
}

.shipment-history-detail__section {
  margin-bottom: 14px;
}

.shipment-history-detail__section:last-child {
  margin-bottom: 0;
}

.shipment-history-detail__title {
  padding-left: 8px;
  margin-bottom: 8px;
  font-size: 13px;
  font-weight: 600;
  color: #1f1f1f;
  border-left: 3px solid #1890ff;
}

.shipment-history-detail__grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px 24px;
  padding: 12px 14px;
  background: #fafafa;
  border-radius: 4px;
}

.shipment-history-detail__item {
  display: flex;
  gap: 8px;
  align-items: baseline;
  min-width: 0;
}

.shipment-history-detail__item--full {
  grid-column: 1 / -1;
}

.shipment-history-detail__label {
  flex-shrink: 0;
  min-width: 60px;
  font-size: 12px;
  color: #8c8c8c;
}

.shipment-history-detail__value {
  font-size: 13px;
  font-weight: 500;
  color: #262626;
  word-break: break-all;
}

/* ============ 响应式 ============ */
@media (max-width: 1200px) {
  .sale-shipment-drawer {
    width: 92vw !important;
  }

  .shipment-order-card__body {
    flex-direction: column;
    align-items: stretch;
  }

  .shipment-order-card__progress {
    justify-content: space-around;
    padding-top: 12px;
    padding-left: 0;
    border-top: 1px solid rgb(24 144 255 / 15%);
    border-left: none;
  }

  .shipment-order-card__info {
    grid-template-columns: 1fr;
  }
}

/* ============ 滚动条美化 ============ */
.shipment-history-list::-webkit-scrollbar {
  width: 6px;
}

.shipment-history-list::-webkit-scrollbar-track {
  background: transparent;
}

.shipment-history-list::-webkit-scrollbar-thumb {
  background: #d9d9d9;
  border-radius: 3px;
}

.shipment-history-list::-webkit-scrollbar-thumb:hover {
  background: #bfbfbf;
}
</style>
