<script lang="ts" setup>
import { h, ref, computed, onMounted, watch } from 'vue';
import { Card, Descriptions, Tabs, Tag, Button, Row, Col, Space, Popconfirm, Divider, Avatar, Dropdown, Menu, MenuItem, Skeleton, Tooltip } from 'ant-design-vue';
import { LucideFilePenLine, LucideUserPlus, LucideMoreHorizontal, LucideBuilding2, LucidePhone, LucideMail, LucideMapPin, LucideGlobe } from '@vben/icons';
import { getCustomerInfoApi, getCustomerContactsApi } from '#/api';
import { useVbenDrawer } from '@vben/common-ui';
import ContactDrawer from '../contact/drawer.vue';
import TagSelector from '../components/TagSelector.vue';

const props = defineProps<{ id?: number | string }>();
const emit = defineEmits<{
  (e: 'edit', customer: any): void;
}>();

const loading = ref(true);
const customer = ref<any>({});
const contacts = ref<any[]>([]);
const historyContacts = ref<any[]>([]);
const activeTab = ref('basic');

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

const roleLabel: Record<string, string> = { decision_maker: '决策人', influencer: '影响者', user: '使用者', other: '其他' };
const roleColor: Record<string, string> = { decision_maker: 'red', influencer: 'blue', user: 'green', other: 'default' };

const statCards = computed(() => [
  { label: '成交总额', value: customer.value.stats?.totalRevenue ? '¥' + (customer.value.stats.totalRevenue / 10000).toFixed(1) + '万' : '-', color: 'text-blue-600', bg: 'bg-blue-50' },
  { label: '成交笔数', value: customer.value.stats?.orderCount ?? 0, color: 'text-green-600', bg: 'bg-green-50' },
  { label: '联系人', value: contacts.value.length, color: 'text-purple-600', bg: 'bg-purple-50' },
  { label: '商机数', value: customer.value.stats?.opportunityCount ?? 0, color: 'text-orange-600', bg: 'bg-orange-50' },
  { label: '信用额度', value: customer.value.creditLimit ? '¥' + (customer.value.creditLimit / 10000).toFixed(1) + '万' : '-', color: 'text-red-500', bg: 'bg-red-50' },
  { label: '账期', value: customer.value.creditDays ? customer.value.creditDays + '天' : '-', color: 'text-cyan-600', bg: 'bg-cyan-50' },
]);

const loadData = async () => {
  if (!props.id) return;
  loading.value = true;
  try {
    const result = await getCustomerInfoApi(Number(props.id));
    customer.value = result || {};
    await loadContacts();
  } finally { loading.value = false; }
};

const loadContacts = async () => {
  if (!props.id) return;
  try {
    const result = await getCustomerContactsApi(Number(props.id));
    contacts.value = result.current || [];
    historyContacts.value = result.history || [];
  } catch { /* ignore */ }
};

const handleAddContact = () => {
  contactEditDrawerApi.setData({ create: true, customerId: Number(props.id) });
  contactEditDrawerApi.open();
};

const handleViewContact = (contactId: number) => {
  // 可在此处触发事件让父级打开联系人详情弹窗，或自行打开
  console.log('view contact', contactId);
};

const handleUnbind = async (contactId: number) => {
  // TODO: 调用 unbindContactApi
  window.$message?.success('解绑成功');
  loadContacts();
};

const handleEdit = () => emit('edit', customer.value);

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
                <span v-if="customer.assignedTo?.name" class="flex items-center gap-1.5">
                  <LucideUserPlus :size="14" class="text-gray-400" />{{ customer.assignedTo.name }}
                </span>
                <span v-if="customer.cooperatedAt" class="flex items-center gap-1.5">
                  <span class="text-gray-400">合作:</span>{{ customer.cooperatedAt }}
                </span>
              </div>
              <div class="flex items-center gap-2">
                <TagSelector
                  entity-type="customer"
                  :entity-id="Number(props.id)"
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
          <Tabs.TabPane key="basic" tab="基本信息">
            <div class="p-4">
              <Descriptions :column="3" bordered size="small" class="rounded-lg">
                <Descriptions.Item label="公司名称" class="text-gray-700">{{ customer.companyName }}</Descriptions.Item>
                <Descriptions.Item label="简称" class="text-gray-700">{{ customer.shortName || '-' }}</Descriptions.Item>
                <Descriptions.Item label="客户编号" class="text-gray-700">{{ customer.customerNo || '-' }}</Descriptions.Item>
                <Descriptions.Item label="行业" class="text-gray-700">{{ customer.industry || '-' }}</Descriptions.Item>
                <Descriptions.Item label="国家" class="text-gray-700">{{ customer.country || '-' }}</Descriptions.Item>
                <Descriptions.Item label="区域" class="text-gray-700">{{ customer.region || '-' }}</Descriptions.Item>
                <Descriptions.Item label="地址" :span="2" class="text-gray-700">{{ customer.address || '-' }}</Descriptions.Item>
                <Descriptions.Item label="网站" class="text-gray-700">{{ customer.website || '-' }}</Descriptions.Item>
                <Descriptions.Item label="来源" class="text-gray-700">{{ customer.source || '-' }}</Descriptions.Item>
                <Descriptions.Item label="等级" class="text-gray-700">
                  <Tag :color="levelColor">{{ levelLabel }}</Tag>
                </Descriptions.Item>
                <Descriptions.Item label="币种" class="text-gray-700">{{ customer.currency || '-' }}</Descriptions.Item>
                <Descriptions.Item label="信用额度" class="text-gray-700">{{ customer.creditLimit ? `¥${customer.creditLimit} / ${customer.creditDays || 0}天` : '-' }}</Descriptions.Item>
                <Descriptions.Item label="负责人" class="text-gray-700">{{ customer.assignedTo?.name || '-' }}</Descriptions.Item>
                <Descriptions.Item label="合作时间" class="text-gray-700">{{ customer.cooperatedAt || '-' }}</Descriptions.Item>
                <Descriptions.Item label="标签" :span="2" class="text-gray-700">
                  <TagSelector
                    entity-type="customer"
                    :entity-id="Number(props.id)"
                  />
                </Descriptions.Item>
                <Descriptions.Item label="备注" :span="3" class="text-gray-700">{{ customer.description || '-' }}</Descriptions.Item>
              </Descriptions>
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
              <div class="space-y-3 px-2">
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
          <Tabs.TabPane key="followups" tab="跟进记录">
            <div class="text-gray-400 text-center py-16 text-sm">跟进记录模块开发中</div>
          </Tabs.TabPane>
          <Tabs.TabPane key="logs" tab="操作日志">
            <div class="text-gray-400 text-center py-16 text-sm">操作日志模块开发中</div>
          </Tabs.TabPane>
        </Tabs>
      </Card>
      <ContactEditDrawer />
    </Skeleton>
  </div>
</template>