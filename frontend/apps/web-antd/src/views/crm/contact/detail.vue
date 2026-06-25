<script lang="ts" setup>
import { ref, computed, watch } from 'vue';
import { Card, Descriptions, Tag, Button, Timeline, Popconfirm, Avatar, Row, Col, Skeleton } from 'ant-design-vue';
import { LucideFilePenLine, LucideUnlink, LucideMail, LucidePhone, LucideSmartphone, LucideBuilding2, LucideCalendar, LucideMessageCircle } from '@vben/icons';
import { getContactInfoApi, unbindContactApi } from '#/api';
import { message } from 'ant-design-vue';

const props = defineProps<{ id?: number | string }>();
const emit = defineEmits<{
  (e: 'edit', contact: any): void;
  (e: 'view-customer', customerId: number): void;
}>();

const loading = ref(false);
const contact = ref<any>({});

const roleTypeMap: Record<number, string> = {
  0: '决策人', 1: '影响者', 2: '使用者', 3: '其他',
};
const roleColorMap: Record<number, string> = {
  0: 'red', 1: 'blue', 2: 'green', 3: 'default',
};

const initials = computed(() => (contact.value.name || '?').slice(0, 1).toUpperCase());

const contactChannels = computed(() => {
  const channels: Array<{ icon: any; label: string; value: string }> = [];
  if (contact.value.email) channels.push({ icon: LucideMail, label: '邮箱', value: contact.value.email });
  if (contact.value.mobile) channels.push({ icon: LucideSmartphone, label: '手机', value: contact.value.mobile });
  if (contact.value.phone) channels.push({ icon: LucidePhone, label: '座机', value: contact.value.phone });
  if (contact.value.whatsapp) channels.push({ icon: LucideMessageCircle, label: 'WhatsApp', value: contact.value.whatsapp });
  return channels;
});

const loadData = async () => {
  if (!props.id) return;
  loading.value = true;
  try {
    const result = await getContactInfoApi(Number(props.id));
    contact.value = result || {};
  } finally { loading.value = false; }
};

const handleEdit = () => emit('edit', contact.value);

const handleUnbind = async () => {
  if (!contact.value.currentCompany) return;
  try {
    await unbindContactApi({ contactId: Number(props.id) });
    message.success('解绑成功');
    loadData();
  } catch { /* ignore */ }
};

const handleViewCustomer = (customerId: number) => emit('view-customer', customerId);

watch(() => props.id, () => { if (props.id) loadData(); }, { immediate: true });
</script>

<template>
  <div class="p-4">
    <Skeleton :loading="loading" active>
      <!-- 头部信息卡片 -->
      <Card class="mb-4" :body-style="{ padding: '20px 24px' }">
        <div class="flex items-start justify-between">
          <div class="flex items-start gap-4">
            <Avatar :size="56" :style="{ backgroundColor: '#1677ff', fontSize: '22px' }">{{ initials }}</Avatar>
            <div>
              <div class="flex items-center gap-3 mb-1">
                <h2 class="text-xl font-bold m-0">{{ contact.name }}</h2>
                <Tag v-if="contact.title" color="blue" size="small">{{ contact.title }}</Tag>
                <Tag v-if="contact.roleType" :color="roleColorMap[contact.roleType] || 'default'" size="small">
                  {{ roleTypeMap[contact.roleType] || contact.roleType }}
                </Tag>
              </div>
              <div class="flex items-center gap-4 text-sm text-gray-500 flex-wrap">
                <span v-if="contact.email" class="flex items-center gap-1">
                  <LucideMail :size="14" />{{ contact.email }}
                </span>
                <span v-if="contact.mobile" class="flex items-center gap-1">
                  <LucideSmartphone :size="14" />{{ contact.mobile }}
                </span>
                <span v-if="contact.phone" class="flex items-center gap-1">
                  <LucidePhone :size="14" />{{ contact.phone }}
                </span>
                <span v-if="contact.birthday" class="flex items-center gap-1">
                  <LucideCalendar :size="14" />{{ contact.birthday }}
                </span>
              </div>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <Button type="primary" @click="handleEdit">
              <template #icon><LucideFilePenLine /></template>编辑
            </Button>
          </div>
        </div>
      </Card>

      <Row :gutter="16">
        <!-- 左侧：当前任职 + 基本信息 -->
        <Col :span="16">
          <!-- 当前任职 -->
          <Card v-if="contact.currentCompany" size="small" class="mb-4" :body-style="{ padding: '16px 20px' }">
            <template #title>
              <span class="text-blue-600 font-medium">当前任职</span>
            </template>
            <template #extra>
              <Popconfirm title="确认解绑该联系人？" @confirm="handleUnbind">
                <Button size="small" danger>
                  <template #icon><LucideUnlink /></template>解绑/离职
                </Button>
              </Popconfirm>
            </template>
            <div class="flex items-center justify-between">
              <div>
                <div class="flex items-center gap-2">
                  <span class="font-bold text-base cursor-pointer text-blue-600 hover:underline" @click="handleViewCustomer(contact.currentCompany.customerId)">
                    {{ contact.currentCompany.companyName }}
                  </span>
                  <Tag v-if="contact.currentCompany.shortName" size="small">{{ contact.currentCompany.shortName }}</Tag>
                </div>
                <div class="text-gray-500 mt-1 flex items-center gap-2 flex-wrap">
                  <span>{{ contact.currentCompany.title || '-' }}</span>
                  <Tag :color="roleColorMap[contact.currentCompany.roleType] || 'default'" size="small">
                    {{ roleTypeMap[contact.currentCompany.roleType] || contact.currentCompany.roleType }}
                  </Tag>
                  <Tag v-if="contact.currentCompany.isPrimary" color="gold" size="small">首要联系人</Tag>
                  <Tag v-if="contact.currentCompany.isBilling" color="purple" size="small">账单</Tag>
                  <Tag v-if="contact.currentCompany.isShipping" color="cyan" size="small">收货</Tag>
                </div>
                <div class="text-gray-400 text-sm mt-1">
                  {{ contact.currentCompany.boundAt }} 至今
                </div>
              </div>
              <Button size="small" @click="handleViewCustomer(contact.currentCompany.customerId)">
                <template #icon><LucideBuilding2 /></template>查看公司
              </Button>
            </div>
          </Card>

          <!-- 基本信息 -->
          <Card size="small" class="mb-4">
            <template #title>基本信息</template>
            <Descriptions :column="2" bordered size="small">
              <Descriptions.Item label="姓名">{{ contact.name }}</Descriptions.Item>
              <Descriptions.Item label="生日">{{ contact.birthday || '-' }}</Descriptions.Item>
              <Descriptions.Item label="邮箱">{{ contact.email || '-' }}</Descriptions.Item>
              <Descriptions.Item label="手机">{{ contact.mobile || '-' }}</Descriptions.Item>
              <Descriptions.Item label="座机">{{ contact.phone || '-' }}</Descriptions.Item>
              <Descriptions.Item label="WhatsApp">{{ contact.whatsapp || '-' }}</Descriptions.Item>
              <Descriptions.Item label="微信">{{ contact.wechat || '-' }}</Descriptions.Item>
              <Descriptions.Item label="备注" :span="2">{{ contact.notes || '-' }}</Descriptions.Item>
            </Descriptions>
          </Card>
        </Col>

        <!-- 右侧：联系方式 + 职业生涯 -->
        <Col :span="8">
          <!-- 联系方式快速卡片 -->
          <Card v-if="contactChannels.length > 0" size="small" class="mb-4">
            <template #title>联系方式</template>
            <div class="flex flex-col gap-2">
              <div v-for="ch in contactChannels" :key="ch.label" class="flex items-center gap-2 text-sm">
                <component :is="ch.icon" :size="14" class="text-gray-400" />
                <span class="text-gray-500">{{ ch.label }}:</span>
                <span class="text-gray-700">{{ ch.value }}</span>
              </div>
            </div>
          </Card>

          <!-- 职业生涯履历 -->
          <Card size="small">
            <template #title>职业生涯履历</template>
            <Timeline v-if="contact.careerHistory && contact.careerHistory.length > 0">
              <Timeline.Item v-for="item in contact.careerHistory" :key="item.id" :color="item.isCurrent ? 'green' : 'gray'">
                <div class="mb-1">
                  <span class="font-bold cursor-pointer text-blue-600 hover:underline text-sm" @click="handleViewCustomer(item.customerId)">
                    {{ item.companyName }}
                  </span>
                  <Tag v-if="item.shortName" size="small" class="ml-1">{{ item.shortName }}</Tag>
                </div>
                <div class="text-xs text-gray-500">
                  {{ item.title }}
                  <Tag :color="roleColorMap[item.roleType] || 'default'" size="small" class="ml-1">
                    {{ roleTypeMap[item.roleType] || item.roleType }}
                  </Tag>
                  <Tag v-if="item.isPrimary" color="gold" size="small" class="ml-1">首要</Tag>
                </div>
                <div class="text-xs text-gray-400 mt-0.5">
                  {{ item.boundAt }} ~ {{ item.unboundAt || '至今' }}
                </div>
                <div v-if="item.notes" class="text-xs text-gray-400 mt-0.5">
                  {{ item.notes }}
                </div>
              </Timeline.Item>
            </Timeline>
            <div v-else class="text-gray-400 text-center py-8 text-sm">
              暂无职业生涯记录
            </div>
          </Card>
        </Col>
      </Row>
    </Skeleton>
  </div>
</template>