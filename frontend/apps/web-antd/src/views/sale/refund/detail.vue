<script lang="ts" setup>
import { ref, watch } from 'vue';

import { formatDateTime } from '@vben/utils';

import {
  Card,
  Descriptions,
  DescriptionsItem,
  Empty,
  Skeleton,
  Table,
  TabPane,
  Tabs,
  Tag,
} from 'ant-design-vue';

import { getRefundInfoApi } from '#/api';

const props = defineProps<{ id?: number | string }>();

const loading = ref(true);
const detail = ref<any>({});
const items = ref<any[]>([]);
const payments = ref<any[]>([]);

// 退货状态映射：1=草稿,2=待审批,3=审批通过,4=待收货,5=已收货,6=质检中,7=已完成,8=已驳回,9=已取消
const refundStatusColorMap: Record<number, string> = {
  1: 'default',
  2: 'processing',
  3: 'success',
  4: 'warning',
  5: 'cyan',
  6: 'orange',
  7: 'green',
  8: 'error',
  9: 'default',
};
const refundStatusLabelMap: Record<number, string> = {
  1: '草稿',
  2: '待审批',
  3: '审批通过',
  4: '待收货',
  5: '已收货',
  6: '质检中',
  7: '已完成',
  8: '已驳回',
  9: '已取消',
};

// 审批状态：0=草稿,1=待审批,2=审批中,3=已通过,4=已驳回
const approvalStatusColorMap: Record<number, string> = {
  0: 'default',
  1: 'processing',
  2: 'warning',
  3: 'success',
  4: 'error',
};
const approvalStatusLabelMap: Record<number, string> = {
  0: '草稿',
  1: '待审批',
  2: '审批中',
  3: '已通过',
  4: '已驳回',
};

// 退货类型
const refundTypeLabelMap: Record<number, string> = {
  1: '整单退货',
  2: '部分退货',
};

// 质检结果
const qualityCheckResultLabelMap: Record<number, string> = {
  0: '未质检',
  1: '合格',
  2: '不合格',
};
const qualityCheckResultColorMap: Record<number, string> = {
  0: 'default',
  1: 'success',
  2: 'error',
};

// 退款方式：1=原路退回, 2=银行转账, 3=现金, 4=其他
const paymentMethodLabelMap: Record<number, string> = {
  1: '原路退回',
  2: '银行转账',
  3: '现金',
  4: '其他',
};

const itemColumns = [
  {
    title: '#',
    width: 45,
    key: 'seq',
    customRender: ({ index }: any) => index + 1,
    align: 'center' as const,
  },
  {
    title: '产品名称',
    dataIndex: 'productName',
    key: 'productName',
    width: 200,
    ellipsis: true,
  },
  { title: '规格', dataIndex: 'spec', key: 'spec', width: 120 },
  {
    title: '单位',
    dataIndex: 'unit',
    key: 'unit',
    width: 60,
    align: 'center' as const,
  },
  {
    title: '退货数量',
    dataIndex: 'refundQty',
    key: 'refundQty',
    width: 100,
    align: 'right' as const,
    customRender: ({ text }: any) => Number(text || 0).toFixed(0),
  },
  {
    title: '单价',
    dataIndex: 'unitPrice',
    key: 'unitPrice',
    width: 100,
    align: 'right' as const,
    customRender: ({ text }: any) => Number(text || 0).toFixed(2),
  },
  {
    title: '退货金额',
    dataIndex: 'refundAmount',
    key: 'refundAmount',
    width: 120,
    align: 'right' as const,
    customRender: ({ text }: any) => `¥ ${Number(text || 0).toFixed(2)}`,
  },
];

const paymentColumns = [
  {
    title: '#',
    width: 45,
    key: 'seq',
    customRender: ({ index }: any) => index + 1,
    align: 'center' as const,
  },
  { title: '退款单号', dataIndex: 'paymentNo', key: 'paymentNo', width: 170 },
  {
    title: '退款方式',
    dataIndex: 'paymentMethod',
    key: 'paymentMethod',
    width: 100,
    customRender: ({ text }: any) => paymentMethodLabelMap[text] || '-',
  },
  {
    title: '退款金额',
    dataIndex: 'paymentAmount',
    key: 'paymentAmount',
    width: 120,
    align: 'right' as const,
    customRender: ({ text }: any) => `¥ ${Number(text || 0).toFixed(2)}`,
  },
  {
    title: '退款时间',
    dataIndex: 'paymentTime',
    key: 'paymentTime',
    width: 160,
    customRender: ({ text }: any) => (text ? formatDateTime(text) : '-'),
  },
  {
    title: '退款账号',
    dataIndex: 'paymentAccount',
    key: 'paymentAccount',
    width: 150,
    ellipsis: true,
  },
  {
    title: '第三方交易号',
    dataIndex: 'transactionNo',
    key: 'transactionNo',
    width: 150,
    ellipsis: true,
  },
  {
    title: '备注',
    dataIndex: 'remark',
    key: 'remark',
    width: 200,
    ellipsis: true,
  },
];

async function loadDetail(id: number) {
  loading.value = true;
  try {
    const res: any = await getRefundInfoApi(id);
    const data = res?.data ?? res ?? {};
    detail.value = data;
    items.value = data.items || [];
    payments.value = data.payments || [];
  } catch (error) {
    console.error('[退货单详情] 加载失败:', error);
    detail.value = {};
    items.value = [];
    payments.value = [];
  } finally {
    loading.value = false;
  }
}

watch(
  () => props.id,
  (val) => {
    if (val) loadDetail(Number(val));
  },
  { immediate: true },
);
</script>

<template>
  <div class="p-4">
    <Skeleton v-if="loading" active />
    <template v-else>
      <!-- 顶部状态卡 -->
      <Card class="mb-4" :bordered="true">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-4">
            <div class="text-lg font-semibold">
              {{ detail.refundNo || '-' }}
            </div>
            <Tag
              :color="refundStatusColorMap[detail.refundStatus] ?? 'default'"
            >
              {{ refundStatusLabelMap[detail.refundStatus] ?? '-' }}
            </Tag>
            <Tag
              :color="
                approvalStatusColorMap[detail.approvalStatus] ?? 'default'
              "
            >
              审批：{{ approvalStatusLabelMap[detail.approvalStatus] ?? '-' }}
            </Tag>
            <Tag color="blue">
              {{
                refundTypeLabelMap[detail.refundType] ||
                detail.refundType ||
                '-'
              }}
            </Tag>
          </div>
          <div class="text-sm text-gray-500">
            创建时间：{{
              detail.createTime ? formatDateTime(detail.createTime) : '-'
            }}
          </div>
        </div>
      </Card>

      <Tabs default-active-key="basic">
        <TabPane key="basic" tab="基本信息">
          <Card title="基本信息" class="mb-4" :bordered="true">
            <Descriptions :column="2" bordered size="small">
              <DescriptionsItem label="退货单号">
                {{ detail.refundNo || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="退货标题">
                {{ detail.title || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="关联订单">
                {{ detail.orderNo || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="客户名称">
                {{ detail.customerName || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="退货类型">
                <Tag color="blue">
                  {{ refundTypeLabelMap[detail.refundType] || '-' }}
                </Tag>
              </DescriptionsItem>
              <DescriptionsItem label="负责人">
                {{ detail.ownerUserName || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="退货原因" :span="2">
                {{ detail.refundReason || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="备注" :span="2">
                {{ detail.remark || '-' }}
              </DescriptionsItem>
            </Descriptions>
          </Card>

          <Card title="金额信息" class="mb-4" :bordered="true">
            <Descriptions :column="2" bordered size="small">
              <DescriptionsItem label="退货总金额">
                <span class="font-medium"
                  >¥ {{ Number(detail.totalAmount ?? 0).toFixed(2) }}</span
                >
              </DescriptionsItem>
              <DescriptionsItem label="折让金额">
                <span class="text-orange-500"
                  >¥ {{ Number(detail.restockingFee ?? 0).toFixed(2) }}</span
                >
              </DescriptionsItem>
              <DescriptionsItem label="应退金额">
                <span class="font-medium text-red-500"
                  >¥ {{ Number(detail.refundAmount ?? 0).toFixed(2) }}</span
                >
              </DescriptionsItem>
              <DescriptionsItem label="已退金额">
                <span class="font-medium text-green-500"
                  >¥ {{ Number(detail.refundedAmount ?? 0).toFixed(2) }}</span
                >
              </DescriptionsItem>
            </Descriptions>
          </Card>

          <Card title="收货信息" class="mb-4" :bordered="true">
            <Descriptions :column="2" bordered size="small">
              <DescriptionsItem label="收货人">
                {{ detail.receiver || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="联系电话">
                {{ detail.receiverPhone || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="收货地址" :span="2">
                {{ detail.receiverAddress || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="物流单号">
                {{ detail.logisticsNo || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="物流公司">
                {{ detail.logisticsCompany || '-' }}
              </DescriptionsItem>
            </Descriptions>
          </Card>

          <Card title="质检信息" class="mb-4" :bordered="true">
            <Descriptions :column="2" bordered size="small">
              <DescriptionsItem label="质检结果">
                <Tag
                  :color="
                    qualityCheckResultColorMap[detail.qualityCheckResult] ??
                    'default'
                  "
                >
                  {{
                    qualityCheckResultLabelMap[detail.qualityCheckResult] ??
                    '未质检'
                  }}
                </Tag>
              </DescriptionsItem>
              <DescriptionsItem label="质检备注">
                {{ detail.qualityCheckRemark || '-' }}
              </DescriptionsItem>
            </Descriptions>
          </Card>

          <Card title="系统信息" :bordered="true">
            <Descriptions :column="2" bordered size="small">
              <DescriptionsItem label="创建人ID">
                {{ detail.createBy || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="创建时间">
                {{
                  detail.createTime ? formatDateTime(detail.createTime) : '-'
                }}
              </DescriptionsItem>
              <DescriptionsItem label="更新人ID">
                {{ detail.updateBy || '-' }}
              </DescriptionsItem>
              <DescriptionsItem label="更新时间">
                {{
                  detail.updateTime ? formatDateTime(detail.updateTime) : '-'
                }}
              </DescriptionsItem>
            </Descriptions>
          </Card>
        </TabPane>

        <TabPane key="items" :tab="`退货明细 (${items.length})`">
          <Card :bordered="true">
            <Empty v-if="items.length === 0" description="暂无退货明细" />
            <Table
              v-else
              :columns="itemColumns"
              :data-source="items"
              :pagination="false"
              size="small"
              :row-key="(record: any) => record.id || record.orderItemId"
              bordered
            >
              <template #summary>
                <Table.Summary fixed>
                  <Table.SummaryRow>
                    <Table.SummaryCell :index="0" :col-span="6" align="right">
                      <span class="font-medium">合计：</span>
                    </Table.SummaryCell>
                    <Table.SummaryCell :index="6" align="right">
                      <span class="font-medium text-red-500">
                        ¥ {{ Number(detail.totalAmount ?? 0).toFixed(2) }}
                      </span>
                    </Table.SummaryCell>
                  </Table.SummaryRow>
                </Table.Summary>
              </template>
            </Table>
          </Card>
        </TabPane>

        <TabPane key="payments" :tab="`退款记录 (${payments.length})`">
          <Card :bordered="true">
            <Empty v-if="payments.length === 0" description="暂无退款记录" />
            <Table
              v-else
              :columns="paymentColumns"
              :data-source="payments"
              :pagination="false"
              size="small"
              :row-key="(record: any) => record.id"
              bordered
            >
              <template #summary>
                <Table.Summary fixed>
                  <Table.SummaryRow>
                    <Table.SummaryCell :index="0" :col-span="2" align="right">
                      <span class="font-medium">累计退款：</span>
                    </Table.SummaryCell>
                    <Table.SummaryCell :index="2" align="right" :col-span="1">
                      <span class="font-medium text-green-500">
                        ¥ {{ Number(detail.refundedAmount ?? 0).toFixed(2) }}
                      </span>
                    </Table.SummaryCell>
                    <Table.SummaryCell :index="3" :col-span="5">
                      <span class="text-gray-500 ml-2">
                        应退：¥
                        {{ Number(detail.refundAmount ?? 0).toFixed(2) }}，
                        待退：¥
                        {{
                          (
                            Number(detail.refundAmount ?? 0) -
                            Number(detail.refundedAmount ?? 0)
                          ).toFixed(2)
                        }}
                      </span>
                    </Table.SummaryCell>
                  </Table.SummaryRow>
                </Table.Summary>
              </template>
            </Table>
          </Card>
        </TabPane>
      </Tabs>
    </template>
  </div>
</template>
