<script lang="ts" setup>
import { ref, watch } from 'vue';

import {
  Descriptions,
  DescriptionsItem,
  Drawer,
  Empty,
  Spin,
  Steps,
  Step,
  TabPane,
  Tabs,
  Timeline,
  TimelineItem,
} from 'ant-design-vue';

import { getEntitlementUsageApi } from '#/api/core/sale/entitlement';

const props = defineProps<{
  detail: any;
  loading: boolean;
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:visible', v: boolean): void;
}>();

const activeTab = ref('base');
const usageList = ref<any[]>([]);

function statusText(s: any) {
  const map: Record<number, string> = {
    1: '待激活',
    2: '生效中',
    3: '已暂停',
    4: '已到期',
    5: '已取消',
  };
  return map[Number(s)] || String(s ?? '-');
}

const actionColor: Record<number, string> = {
  1: 'green',
  2: 'blue',
  3: 'green',
  4: 'orange',
  5: 'red',
  6: 'blue',
  7: 'red',
  8: 'green',
  9: 'purple',
  10: 'gray',
};

watch(
  () => props.visible,
  async (v) => {
    if (v && props.detail?.id) {
      // P1.3：配额流水按需加载
      try {
        const res: any = await getEntitlementUsageApi(props.detail.id);
        usageList.value = res?.data || [];
      } catch {
        usageList.value = [];
      }
    }
  },
);

</script>

<template>
  <Drawer
    :open="visible"
    title="权益详情"
    width="640px"
    :destroy-on-close="true"
    @close="() => emit('update:visible', false)"
  >
    <Spin :spinning="loading">
      <Empty v-if="!detail || !detail.id" description="暂无数据" />
      <Tabs v-else v-model:active-key="activeTab">
        <TabPane key="base" tab="基础信息">
          <Descriptions :column="2" size="small" bordered>
            <DescriptionsItem label="权益编号" :span="2">
              {{ detail.entitlementNo || '-' }}
            </DescriptionsItem>
            <DescriptionsItem label="客户">
              {{ detail.customerName || detail.customerId || '-' }}
            </DescriptionsItem>
            <DescriptionsItem label="关联订单">
              {{ detail.orderNo || detail.orderId || '-' }}
            </DescriptionsItem>
            <DescriptionsItem label="商品名">
              {{ detail.productName || '-' }}
            </DescriptionsItem>
            <DescriptionsItem label="权益类型">
              {{ detail.entitlementTypeName || '-' }}
            </DescriptionsItem>
            <DescriptionsItem label="状态">
              {{ statusText(detail.status) }}
            </DescriptionsItem>
            <DescriptionsItem label="剩余天数">
              {{ detail.remainingDays ?? '-' }}
            </DescriptionsItem>
            <DescriptionsItem label="开始日期">
              {{ detail.startDate || '-' }}
            </DescriptionsItem>
            <DescriptionsItem label="结束日期">
              {{ detail.endDate || '-' }}
            </DescriptionsItem>
            <DescriptionsItem label="服务时长（月）">
              {{ detail.durationMonths ?? '-' }}
            </DescriptionsItem>
            <DescriptionsItem label="续约次数">
              {{ detail.renewCount ?? 0 }}
            </DescriptionsItem>
            <DescriptionsItem label="自动续约">
              {{ detail.autoRenew === 1 ? '是' : '否' }}
            </DescriptionsItem>
            <DescriptionsItem label="总配额">
              {{ detail.totalQuota ?? '-' }}
            </DescriptionsItem>
            <DescriptionsItem label="已用配额">
              {{ detail.usedQuota ?? '-' }}
            </DescriptionsItem>
            <DescriptionsItem label="剩余配额">
              {{ detail.remainingQuota ?? '-' }}
            </DescriptionsItem>
            <DescriptionsItem label="SLA 等级">
              {{ detail.slaLevel || '-' }}
            </DescriptionsItem>
            <DescriptionsItem label="创建时间" :span="2">
              {{ detail.createTime || '-' }}
            </DescriptionsItem>
          </Descriptions>

          <div v-if="(detail.renewChain || []).length > 0" class="mt-4">
            <h6 class="mb-2 font-semibold">续约链（前序权益）</h6>
            <Steps direction="vertical" size="small" :current="0">
              <Step
                v-for="c in detail.renewChain"
                :key="c.id"
                :title="`#${c.id} ${c.entitlementNo || ''}`"
                :description="`状态：${statusText(c.status)}｜结束：${c.endDate || '-'}｜续约 ${c.renewCount ?? 0} 次`"
              />
            </Steps>
          </div>
        </TabPane>

        <TabPane key="logs" tab="操作日志">
          <Empty v-if="(detail.logs || []).length === 0" description="暂无操作记录" />
          <Timeline v-else>
            <TimelineItem
              v-for="lg in detail.logs"
              :key="lg.id"
              :color="actionColor[lg.action] || 'blue'"
            >
              <p class="font-medium">
                {{ lg.actionName || lg.action }}
                <span v-if="lg.toStatus" class="text-gray-400">
                  → {{ statusText(lg.toStatus) }}
                </span>
              </p>
              <p class="text-xs text-gray-400">
                {{ lg.operatorName ? `操作人：${lg.operatorName}｜` : '' }}
                {{ lg.createTime }}
              </p>
              <p v-if="lg.remark" class="text-xs text-gray-500">{{ lg.remark }}</p>
            </TimelineItem>
          </Timeline>
        </TabPane>

        <TabPane key="usage" tab="配额流水">
          <Empty v-if="usageList.length === 0" description="暂无配额流水" />
          <Timeline v-else>
            <TimelineItem v-for="u in usageList" :key="u.id">
              <p>
                {{ Number(u.changeAmount) > 0 ? '消耗' : '返还' }}
                {{ Math.abs(Number(u.changeAmount)) }}
                （{{ u.beforeQuota }} → {{ u.afterQuota }}）
              </p>
              <p class="text-xs text-gray-400">{{ u.createTime }}</p>
            </TimelineItem>
          </Timeline>
        </TabPane>
      </Tabs>
    </Spin>
  </Drawer>
</template>
