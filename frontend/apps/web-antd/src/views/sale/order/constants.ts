/**
 * 订单相关状态映射常量
 *
 * 数据源：与 index.vue 列表页保持一致，供详情页/抽屉页复用
 */

// 订单状态
export const orderStatusOptions = [
  { label: '草稿', value: 1 },
  { label: '待确认', value: 2 },
  { label: '已确认', value: 3 },
  { label: '备货中', value: 4 },
  { label: '部分发货', value: 5 },
  { label: '已发货', value: 6 },
  { label: '已取消', value: 7 },
  { label: '已交付', value: 8 },
  { label: '已签收', value: 9 },
  { label: '已完成', value: 10 },
  { label: '已作废', value: 11 },
];

export const orderStatusColorMap: Record<number, string> = {
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
  11: 'red',
};

export const orderStatusLabelMap: Record<number, string> = {
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
  11: '已作废',
};

// 支付状态
export const paymentStatusOptions = [
  { label: '未支付', value: 1 },
  { label: '部分支付', value: 2 },
  { label: '已支付', value: 3 },
  { label: '已退款', value: 4 },
];

export const paymentStatusColorMap: Record<number, string> = {
  1: 'default',
  2: 'orange',
  3: 'green',
  4: 'red',
};

export const paymentStatusLabelMap: Record<number, string> = {
  1: '未支付',
  2: '部分支付',
  3: '已支付',
  4: '已退款',
};

// 审批状态（0=草稿, 1=待审批, 2=审批中, 3=已通过, 4=已驳回）
export const approvalStatusOptions = [
  { label: '草稿', value: 0 },
  { label: '待审批', value: 1 },
  { label: '审批中', value: 2 },
  { label: '已通过', value: 3 },
  { label: '已驳回', value: 4 },
];

export const approvalStatusColorMap: Record<number, string> = {
  0: 'default',
  1: 'processing',
  2: 'warning',
  3: 'success',
  4: 'error',
};

export const approvalStatusLabelMap: Record<number, string> = {
  0: '草稿',
  1: '待审批',
  2: '审批中',
  3: '已通过',
  4: '已驳回',
};

// 币种
export const currencyLabelMap: Record<number, string> = {
  1: 'CNY',
  2: 'USD',
  3: 'EUR',
  4: 'GBP',
  5: 'JPY',
  6: 'HKD',
};

export const currencySymbolMap: Record<number, string> = {
  1: '¥',
  2: '$',
  3: '€',
  4: '£',
  5: '¥',
  6: 'HK$',
};

// 支付方式
export const paymentMethodLabelMap: Record<number, string> = {
  1: '银行转账',
  2: '现金',
  3: '支票',
  4: '信用卡',
  5: '电汇',
  6: '承兑汇票',
  7: '其他',
};

// 配送方式
export const shippingMethodLabelMap: Record<number, string> = {
  1: '快递',
  2: '物流',
  3: '自提',
  4: '空运',
  5: '海运',
  6: '铁路',
  7: '其他',
};

// 工具函数：金额格式化
export function formatMoney(val: any): string {
  return Number(val || 0).toLocaleString('zh-CN', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
}
