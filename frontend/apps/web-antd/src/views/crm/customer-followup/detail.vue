<script lang="ts" setup>
import { h, ref, computed, watch } from 'vue';
import { Card, Descriptions, Tabs, Tag, Button, Row, Col, Space, Popconfirm, Divider, Avatar, Dropdown, Menu, MenuItem, Skeleton, Tooltip, Timeline, Empty } from 'ant-design-vue';
import { LucideFilePenLine, LucideUserPlus, LucideMoreHorizontal, LucideBuilding2, LucidePhone, LucideMail, LucideMapPin, LucideGlobe } from '@vben/icons';
import { getCustomerInfoApi, getCustomerContactsApi, getCustomerAssignHistoryApi, getFollowupInfoApi } from '#/api';
import { getCustomerEditLogApi, type CustomerEditLogVO } from '#/api/core/crm/customer-edit-log';
import { useVbenDrawer } from '@vben/common-ui';
import ContactDrawer from '../contact/drawer.vue';
import TagSelector from '../components/TagSelector.vue';
import { formatDateTime } from '@vben/utils';

const props = defineProps<{ id: number }>();
const emit = defineEmits<{
  (e: 'edit', customer: any): void;
}>();

const loading = ref(true);
const customer = ref<any>({});
const contacts = ref<any[]>([]);
const historyContacts = ref<any[]>([]);
const assignHistory = ref<any[]>([]);
const activeTab = ref('followups');
const customerId = ref<number | null>(null);

const [ContactEditDrawer, contactEditDrawerApi] = useVbenDrawer({
  connectedComponent: ContactDrawer,
  onClosed() {
    if (contactEditDrawerApi.getData()?.needRefresh) loadContacts();
  },
});

const levelColor = computed(() => {
  const map: Record<string, string> = { 1: 'default', 2: 'red', 3: 'orange', 4: 'blue', 5: 'green' };
  return map[customer.value.level] || 'blue';
});

const levelLabel = computed(() => {
  const map: Record<string, string> = { 1: '无级别', 2: '重点客户', 3: '优质客户', 4: '普通客户', 5: '其他' };
  return map[customer.value.level] || customer.value.level || '-';
});

const initials = computed(() => {
  const name = customer.value.companyName || customer.value.shortName || '?';
  return name.slice(0, 2).toUpperCase();
});

const roleLabel: Record<number, string> = { 0: '决策人', 1: '影响者', 2: '使用者', 3: '其他' };
const roleColor: Record<number, string> = { 0: 'red', 1: 'blue', 2: 'green', 3: 'default' };

// 行业映射 - 后端存储数值
const industryLabelMap: Record<number, string> = {
  1: '零售', 2: '批发', 3: '制造', 4: '贸易代理',
  5: '电商', 6: '微商', 7: '社交电商', 8: '其他',
};

// 来源映射 - 后端存储数值
const sourceLabelMap: Record<number, string> = {
  1: '官网', 2: '展会', 3: '社交媒体', 4: '客户转介',
  5: '陌生拜访', 6: '海关数据', 7: '邮件营销', 8: '阿里国际站',
  9: 'Amazon', 10: 'TikTok', 11: '微信', 12: '其他',
};

// 等级映射
const levelLabelMap: Record<number, string> = { 1: '无级别', 2: '重点客户', 3: '优质客户', 4: '普通客户', 5: '其他' };
const levelColorMap: Record<number, string> = { 1: 'default', 2: 'red', 3: 'orange', 4: 'blue', 5: 'green' };

// 币种映射
const currencyLabelMap: Record<number, string> = { 1: '人民币', 2: '美元', 3: '欧元', 4: '英镑', 5: '日元', 6: '港币', 7: '澳元' };

// 操作日志字段值映射：将数值或代码显示为中文名
function getFieldValueLabel(field: string, value: string | null | undefined): string {
  if (value == null || value === '') return '';
  const numVal = Number(value);
  if (field === 'level') return levelLabelMap[numVal] || value;
  if (field === 'industry') return industryLabelMap[numVal] || value;
  if (field === 'source') return sourceLabelMap[numVal] || value;
  if (field === 'currency') return currencyLabelMap[numVal] || value;
  return value;
}

const statCards = computed(() => [
  { label: '成交总额', value: customer.value.stats?.totalRevenue ? '¥' + (customer.value.stats.totalRevenue / 10000).toFixed(1) + '万' : '-', color: 'text-blue-600', bg: 'bg-blue-50' },
  { label: '成交笔数', value: customer.value.stats?.orderCount ?? 0, color: 'text-green-600', bg: 'bg-green-50' },
  { label: '联系人', value: contacts.value.length, color: 'text-purple-600', bg: 'bg-purple-50' },
  { label: '商机数', value: customer.value.stats?.opportunityCount ?? 0, color: 'text-orange-600', bg: 'bg-orange-50' },
  { label: '信用额度', value: customer.value.creditLimit ? '¥' + (customer.value.creditLimit / 10000).toFixed(1) + '万' : '-', color: 'text-red-500', bg: 'bg-red-50' },
  { label: '账期', value: customer.value.creditDays ? customer.value.creditDays + '天' : '-', color: 'text-cyan-600', bg: 'bg-cyan-50' },
]);

// 通过跟进ID反查客户信息
const loadData = async () => {
  if (!props.id) return;
  loading.value = true;
  try {
    // 1. 先通过跟进ID获取跟进记录，拿到 customerId
    const followup = await getFollowupInfoApi(props.id);
    const cid = followup?.customerId;
    if (!cid) {
      loading.value = false;
      return;
    }
    customerId.value = Number(cid);

    // 2. 用 customerId 加载客户详情
    const result = await getCustomerInfoApi(Number(cid));
    customer.value = result || {};
    await Promise.all([loadContacts(), loadAssignHistory()]);
  } finally { loading.value = false; }
};

const loadAssignHistory = async () => {
  if (!customerId.value) return;
  try {
    const result = await getCustomerAssignHistoryApi(customerId.value);
    const list = Array.isArray(result) ? result
      : (Array.isArray(result?.data) ? result.data
      : []);
    assignHistory.value = list;
  } catch { /* ignore */ }
};

const loadContacts = async () => {
  if (!customerId.value) return;
  try {
    const result = await getCustomerContactsApi(customerId.value);
    contacts.value = result.current || [];
    historyContacts.value = result.history || [];
  } catch { /* ignore */ }
};

const handleAddContact = () => {
  contactEditDrawerApi.setData({ create: true, customerId: customerId.value });
  contactEditDrawerApi.open();
};

const handleViewContact = (contactId: number) => {
  console.log('view contact', contactId);
};

const handleUnbind = async (contactId: number) => {
  window.$message?.success('解绑成功');
  loadContacts();
};

const handleEdit = () => emit('edit', customer.value);

const followups = computed(() => customer.value?.followups || []);
const editLogs = ref<CustomerEditLogVO[]>([]);
const editLogLoading = ref(false);

const loadEditLogs = async () => {
  if (!customerId.value) return;
  editLogLoading.value = true;
  try {
    const result = await getCustomerEditLogApi({ customerId: customerId.value, page: 1, pageSize: 50 });
    editLogs.value = (result as any)?.items || [];
  } catch { /* ignore */ }
  finally { editLogLoading.value = false; }
};

watch(() => activeTab.value, (tab) => {
  if (tab === 'logs' && editLogs.value.length === 0) loadEditLogs();
});

watch(() => props.id, () => { if (props.id) loadData(); }, { immediate: true });
</script>

<template>
  <div class="p-4">
    <Skeleton :loading="loading" active>
      <!-- 头部信息卡片 -->
      <Card class="rounded-lg shadow-sm" :body-style="{ padding: '24px' }" style="margin-bottom: 16px;">
        <div class="flex items-start justify-between">
          <div class="flex items-start gap-5">
            <Avatar :size="64" :style="{ backgroundColor: '#1677ff', fontSize: '24px', fontWeight: 600 }">{{ initials }}</Avatar>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-3 mb-3">
                <h2 class="text-xl font-bold text-gray-800 m-0">{{ customer.companyName }}</h2>
                <Tag :color="levelColor" class="text-sm">{{ levelLabel }}</Tag>
                <Tag v-if="customer.customerNo" color="default" class="text-xs text-gray-400">{{ customer.customerNo }}</Tag>
              </div>
              <div class="flex items-center gap-5 text-sm text-gray-500 flex-wrap mb-3">
                <span v-if="customer.industry" class="flex items-center gap-1.5">
                  <LucideBuilding2 :size="14" class="text-gray-400" />{{ customer.industry }}
                </span>
                <span v-if="customer.country" class="flex items-center gap-1.5">
                  <LucideMapPin :size="14" class="text-gray-400" />{{ customer.country }}
                </span>
                <span v-if="customer.website" class="flex items-center gap-1.5">
                  <LucideGlobe :size="14" class="text-gray-400" /><a :href="customer.website" target="_blank" class="text-blue-500 hover:text-blue-600 hover:underline">{{ customer.website }}</a>
                </span>
                <span v-if="customer.assignedToName" class="flex items-center gap-1.5">
                  <LucideUserPlus :size="14" class="text-gray-400" />{{ customer.assignedToName }}
                </span>
                <span v-if="customer.cooperatedAt" class="flex items-center gap-1.5">
                  <span class="text-gray-400">合作:</span>{{ customer.cooperatedAt }}
                </span>
              </div>
              <div class="flex items-center gap-2">
                <TagSelector
                  entity-type="customer"
                  :entity-id="customerId"
                />
              </div>
            </div>
          </div>
          <div class="flex items-center gap-3">
            <Button type="primary" :icon="h(LucideFilePenLine)" @click="handleEdit">编辑</Button>
            <Dropdown>
              <Button :icon="h(LucideMoreHorizontal)" />
              <template #overlay>
                <Menu>
                  <MenuItem key="transfer">转移负责人</MenuItem>
                  <MenuItem key="merge">合并客户</MenuItem>
                  <MenuItem key="delete" danger>删除客户</MenuItem>
                </Menu>
              </template>
            </Dropdown>
          </div>
        </div>

        <div style="height: 10px;"></div>

        <!-- 基本信息 -->
        <Descriptions :column="2" bordered size="small" class="rounded-lg">
          <Descriptions.Item label="公司名称" class="text-gray-700">{{ customer.companyName }}</Descriptions.Item>
          <Descriptions.Item label="简称" class="text-gray-700">{{ customer.shortName || '-' }}</Descriptions.Item>
          <Descriptions.Item label="客户编号" class="text-gray-700">{{ customer.customerNo || '-' }}</Descriptions.Item>
          <Descriptions.Item label="行业" class="text-gray-700">{{ industryLabelMap[customer.industry] || customer.industry || '-' }}</Descriptions.Item>
          <Descriptions.Item label="国家" class="text-gray-700">{{ customer.country || '-' }}</Descriptions.Item>
          <Descriptions.Item label="区域" class="text-gray-700">{{ customer.region || '-' }}</Descriptions.Item>
          <Descriptions.Item label="地址" :span="2" class="text-gray-700">{{ customer.address || '-' }}</Descriptions.Item>
          <Descriptions.Item label="网站" class="text-gray-700">{{ customer.website || '-' }}</Descriptions.Item>
          <Descriptions.Item label="来源" class="text-gray-700">{{ sourceLabelMap[customer.source] || customer.source || '-' }}</Descriptions.Item>
          <Descriptions.Item label="等级" class="text-gray-700">
            <Tag :color="levelColor">{{ levelLabel }}</Tag>
          </Descriptions.Item>
          <Descriptions.Item label="币种" class="text-gray-700">{{ customer.currency ? (currencyLabelMap[Number(customer.currency)] || customer.currency) : '-' }}</Descriptions.Item>
          <Descriptions.Item label="信用额度" class="text-gray-700">{{ customer.creditLimit ? `¥${customer.creditLimit} / ${customer.creditDays || 0}天` : '-' }}</Descriptions.Item>
          <Descriptions.Item label="负责人" class="text-gray-700">{{ customer.assignedToName || '-' }}</Descriptions.Item>
          <Descriptions.Item label="合作时间" class="text-gray-700">{{ customer.cooperatedAt || '-' }}</Descriptions.Item>
          <Descriptions.Item label="备注" :span="2" class="text-gray-700">{{ customer.description || '-' }}</Descriptions.Item>
        </Descriptions>
      </Card>

      <!-- KPI 统计卡片 -->
      <Row :gutter="16" style="margin-bottom: 16px;">
        <Col v-for="stat in statCards" :key="stat.label" :span="4">
          <Card size="small" class="text-center rounded-lg hover:shadow-md transition-shadow" :body-style="{ padding: '20px 16px', backgroundColor: stat.bg }">
            <div class="text-2xl font-bold" :class="stat.color">{{ stat.value }}</div>
            <div class="text-xs text-gray-500 mt-2">{{ stat.label }}</div>
          </Card>
        </Col>
      </Row>

      <!-- Tab 内容区 -->
      <Card class="overflow-hidden" :body-style="{ padding: '0' }">
        <Tabs v-model:activeKey="activeTab" :tabBarStyle="{ paddingLeft: '30px' }" class="pt-4">
          <Tabs.TabPane key="followups" tab="跟进记录">
            <div class="p-4">
              <div class="mb-4">
                <span class="text-sm font-semibold text-gray-600">跟进记录</span>
              </div>
              <Timeline v-if="followups.length > 0">
                <Timeline.Item v-for="(item, index) in followups" :key="item.id || index">
                  <div class="flex items-start justify-between">
                    <div class="flex items-center gap-2">
                      <Avatar size="small">{{ item.createdByName?.charAt(0) || '?' }}</Avatar>
                      <span class="font-medium">{{ item.createdByName || '-' }}</span>
                    </div>
                    <span class="text-sm text-gray-400">{{ formatDateTime(item.createTime) }}</span>
                  </div>
                  <p class="mt-1 text-sm text-gray-600 whitespace-pre-wrap">{{ item.content || '-' }}</p>
                  <div v-if="item.nextFollowDate" class="mt-1 text-xs text-orange-500">
                    下次联系：{{ item.nextFollowDate }}
                  </div>
                </Timeline.Item>
              </Timeline>
              <Empty v-else description="暂无跟进记录" />
            </div>
          </Tabs.TabPane>

          <Tabs.TabPane key="contacts" :tab="`联系人 (${contacts.length})`">
            <div class="flex items-center justify-between mb-4 mt-2 px-2">
              <span class="text-sm font-semibold text-gray-600">当前在职</span>
              <Button size="small" type="primary" ghost :icon="h(LucideUserPlus)" @click="handleAddContact">添加联系人</Button>
            </div>
            <div v-if="contacts.length === 0" class="text-gray-400 text-center py-16 text-sm">暂无联系人</div>
            <div class="space-y-3 px-2">
              <Card v-for="c in contacts" :key="c.id" size="small" hoverable class="border-l-4 rounded-lg transition-shadow hover:shadow-sm" :class="c.isPrimary ? 'border-l-blue-500' : 'border-l-transparent'">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-4 flex-1">
                    <Avatar :size="40" :style="{ backgroundColor: c.isPrimary ? '#1677ff' : '#d9d9d9' }" class="flex-shrink-0">{{ c.name?.slice(0, 1) || '?' }}</Avatar>
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-2.5 flex-wrap mb-1.5">
                        <span class="font-semibold text-gray-800 cursor-pointer hover:text-blue-600 truncate" @click="handleViewContact(c.id)">{{ c.name }}</span>
                        <Tag :color="roleColor[c.roleType]" size="small">{{ roleLabel[c.roleType] || c.roleType }}</Tag>
                        <Tag v-if="c.isPrimary" color="gold" size="small">首要</Tag>
                        <Tag v-if="c.isBilling" color="purple" size="small">账单</Tag>
                        <Tag v-if="c.isShipping" color="cyan" size="small">收货</Tag>
                      </div>
                      <div class="text-xs text-gray-500 flex items-center gap-4 flex-wrap">
                        <span v-if="c.title" class="text-gray-600">{{ c.title }}</span>
                        <Tooltip v-if="c.email" :title="c.email">
                          <span class="flex items-center gap-1"><LucideMail :size="12" class="text-gray-400" />{{ c.email }}</span>
                        </Tooltip>
                        <Tooltip v-if="c.mobile" :title="c.mobile">
                          <span class="flex items-center gap-1"><LucidePhone :size="12" class="text-gray-400" />{{ c.mobile }}</span>
                        </Tooltip>
                      </div>
                    </div>
                  </div>
                  <Space size="small" class="flex-shrink-0">
                    <Button size="small" type="link" @click="handleViewContact(c.id)">详情</Button>
                    <Popconfirm title="确认解绑该联系人？" ok-text="确认" cancel-text="取消" @confirm="handleUnbind(c.id)">
                      <Button size="small" type="link" danger>解绑</Button>
                    </Popconfirm>
                  </Space>
                </div>
              </Card>
            </div>

            <template v-if="historyContacts.length > 0">
              <Divider class="!my-5" />
              <div class="flex items-center gap-2 mb-4 px-2">
                <span class="text-sm font-semibold text-gray-400">历史联系人</span>
                <Tag size="small" color="default" class="text-gray-400">{{ historyContacts.length }}人</Tag>
              </div>
              <div class="space-y-3 px-2 pb-[15px]">
                <Card v-for="c in historyContacts" :key="c.id" size="small" class="opacity-75 rounded-lg">
                  <div class="flex items-center justify-between">
                    <div class="flex items-center gap-4 flex-1">
                      <Avatar :size="36" :style="{ backgroundColor: '#d9d9d9' }" class="flex-shrink-0">{{ c.name?.slice(0, 1) || '?' }}</Avatar>
                      <div class="flex-1">
                        <div class="flex items-center gap-2 mb-1">
                          <span class="font-medium text-gray-600 cursor-pointer hover:text-blue-600" @click="handleViewContact(c.id)">{{ c.name }}</span>
                          <span class="text-xs text-gray-400">{{ c.title }}</span>
                        </div>
                        <div class="text-xs text-gray-400">
                          {{ c.boundAt }} ~ {{ c.unboundAt }}
                          <span v-if="c.notes" class="ml-2">| {{ c.notes }}</span>
                        </div>
                      </div>
                    </div>
                    <Button size="small" type="link" @click="handleViewContact(c.id)">详情</Button>
                  </div>
                </Card>
              </div>
            </template>
          </Tabs.TabPane>

          <Tabs.TabPane key="opportunities" :tab="`商机 (${customer.stats?.opportunityCount || 0})`">
            <div class="text-gray-400 text-center py-16 text-sm">商机模块开发中</div>
          </Tabs.TabPane>
          <Tabs.TabPane key="orders" :tab="`订单 (${customer.stats?.orderCount || 0})`">
            <div class="text-gray-400 text-center py-16 text-sm">订单模块开发中</div>
          </Tabs.TabPane>
          <Tabs.TabPane key="payments" tab="回款">
            <div class="text-gray-400 text-center py-16 text-sm">回款模块开发中</div>
          </Tabs.TabPane>
          <Tabs.TabPane key="assignHistory" :tab="`负责人记录 (${assignHistory.length})`">
            <div class="p-4">
              <Timeline v-if="assignHistory.length > 0">
                <Timeline.Item
                  v-for="(item, index) in assignHistory"
                  :key="item.id || index"
                  :color="item.endTime ? 'blue' : 'green'"
                >
                  <div class="flex items-start justify-between">
                    <div class="flex items-center gap-2">
                      <Avatar size="small" :style="{ backgroundColor: item.endTime ? '#d9d9d9' : '#52c41a' }">
                        {{ item.adminName?.charAt(0) || '?' }}
                      </Avatar>
                      <div>
                        <span class="font-medium">{{ item.adminName || '未知' }}</span>
                        <Tag v-if="!item.endTime" color="green" size="small" class="ml-2">服务中</Tag>
                        <Tag v-else color="default" size="small" class="ml-2">已结束</Tag>
                      </div>
                    </div>
                  </div>
                  <div class="mt-2 text-sm text-gray-500">
                    <span>{{ formatDateTime(item.startTime) }}</span>
                    <span v-if="item.endTime"> ~ {{ formatDateTime(item.endTime) }}</span>
                    <span v-else class="text-green-500"> ~ 至今</span>
                  </div>
                  <div v-if="item.remark" class="mt-1 text-xs text-gray-400">{{ item.remark }}</div>
                </Timeline.Item>
              </Timeline>
              <Empty v-else description="暂无负责人记录" />
            </div>
          </Tabs.TabPane>
          <Tabs.TabPane key="logs" tab="操作日志">
            <div class="p-4">
              <Skeleton :loading="editLogLoading" active :paragraph="{ rows: 4 }">
                <Timeline v-if="editLogs.length > 0">
                  <Timeline.Item v-for="log in editLogs" :key="log.id" color="blue">
                    <div class="flex items-start justify-between mb-2">
                      <div class="flex items-center gap-2">
                        <Avatar size="small" :style="{ backgroundColor: '#1677ff' }">
                          {{ log.editorName?.charAt(0) || '?' }}
                        </Avatar>
                        <span class="font-medium">{{ log.editorName || '未知' }}</span>
                      </div>
                      <span class="text-sm text-gray-400">{{ log.editTime ? formatDateTime(log.editTime) : '-' }}</span>
                    </div>
                    <div class="mt-2 space-y-1">
                      <div
                        v-for="(item, idx) in log.content"
                        :key="idx"
                        class="text-sm flex items-center gap-2 py-1 px-3 rounded bg-gray-50"
                      >
                        <Tag color="blue" size="small" class="!mr-0">{{ item.fieldLabel }}</Tag>
                        <template v-if="item.old !== null && item.new !== null">
                          <span class="text-gray-400 line-through">{{ getFieldValueLabel(item.field, item.old) }}</span>
                          <span class="text-gray-400">→</span>
                          <span class="text-green-600 font-medium">{{ getFieldValueLabel(item.field, item.new) }}</span>
                        </template>
                        <template v-else-if="item.new === null">
                          <span class="text-red-500">已删除：{{ getFieldValueLabel(item.field, item.old) }}</span>
                        </template>
                        <template v-else>
                          <span class="text-green-600 font-medium">{{ getFieldValueLabel(item.field, item.new) }}</span>
                        </template>
                      </div>
                    </div>
                  </Timeline.Item>
                </Timeline>
                <Empty v-else description="暂无修改记录" />
              </Skeleton>
            </div>
          </Tabs.TabPane>
        </Tabs>
      </Card>
      <ContactEditDrawer />
    </Skeleton>
  </div>
</template>
