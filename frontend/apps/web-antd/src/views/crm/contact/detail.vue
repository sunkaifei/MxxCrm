<script lang="ts" setup>
import type { ContactEditLogVO } from '#/api/core/crm/contact-edit-log';

import { computed, ref, watch } from 'vue';

import {
  LucideBuilding2,
  LucideCalendar,
  LucideFilePenLine,
  LucideMail,
  LucideMessageCircle,
  LucidePhone,
  LucideSmartphone,
  LucideUnlink,
} from '@vben/icons';
import { formatDateTime } from '@vben/utils';

import {
  Avatar,
  Button,
  Card,
  Col,
  Descriptions,
  Empty,
  message,
  Popconfirm,
  Row,
  Skeleton,
  Spin,
  Tabs,
  Tag,
  Timeline,
} from 'ant-design-vue';

import { getContactInfoApi, unbindContactApi } from '#/api';
import { getContactEditLogApi } from '#/api/core/crm/contact-edit-log';
import { $t } from '#/locales';

const props = defineProps<{ id?: number | string }>();
const emit = defineEmits<{
  (e: 'edit', contact: any): void;
  (e: 'viewCustomer', customerId: number): void;
  (e: 'unbind'): void;
}>();

const loading = ref(false);
const contact = ref<any>({});

// 当前激活的选项卡
const activeTab = ref('basic');

// 修改记录
const editLogs = ref<ContactEditLogVO[]>([]);
const editLogLoading = ref(false);

const roleTypeMap: Record<number, string> = {
  0: '决策人',
  1: '影响者',
  2: '使用者',
  3: '其他',
};
const roleColorMap: Record<number, string> = {
  0: 'red',
  1: 'blue',
  2: 'green',
  3: 'default',
};
const genderMap: Record<number, string> = {
  0: '男',
  1: '女',
  2: '未知',
};

const initials = computed(() =>
  (contact.value.name || '?').slice(0, 1).toUpperCase(),
);

const contactChannels = computed(() => {
  const channels: Array<{ icon: any; label: string; value: string }> = [];
  if (contact.value.email)
    channels.push({
      icon: LucideMail,
      label: '邮箱',
      value: contact.value.email,
    });
  if (contact.value.mobile)
    channels.push({
      icon: LucideSmartphone,
      label: '手机',
      value: contact.value.mobile,
    });
  if (contact.value.phone)
    channels.push({
      icon: LucidePhone,
      label: '座机',
      value: contact.value.phone,
    });
  if (contact.value.whatsapp)
    channels.push({
      icon: LucideMessageCircle,
      label: 'WhatsApp',
      value: contact.value.whatsapp,
    });
  return channels;
});

const loadData = async () => {
  if (!props.id) return;
  loading.value = true;
  try {
    const result = await getContactInfoApi(Number(props.id));
    contact.value = result || {};
  } finally {
    loading.value = false;
  }
};

// 加载修改记录
async function loadEditLogs() {
  if (!props.id) return;
  editLogLoading.value = true;
  try {
    const result: any = await getContactEditLogApi({
      contactId: Number(props.id),
      page: 1,
      pageSize: 50,
    });
    editLogs.value = (result as any)?.items || [];
  } catch {
    editLogs.value = [];
  } finally {
    editLogLoading.value = false;
  }
}

// 选项卡切换时按需加载修改记录
function handleTabChange(tab: number | string) {
  if (tab === 'logs' && editLogs.value.length === 0) {
    loadEditLogs();
  }
}

const handleEdit = () => emit('edit', contact.value);

const handleUnbind = async () => {
  if (!contact.value.currentCompany) return;
  try {
    await unbindContactApi({ contactId: Number(props.id) });
    message.success('解绑成功');
    emit('unbind');
    loadData();
  } catch {
    /* ignore */
  }
};

const handleViewCustomer = (customerId: number) =>
  emit('viewCustomer', customerId);

watch(
  () => props.id,
  () => {
    if (props.id) {
      editLogs.value = [];
      activeTab.value = 'basic';
      loadData();
    }
  },
  { immediate: true },
);
</script>

<template>
  <div class="p-4">
    <Skeleton :loading="loading" active>
      <!-- 头部信息卡片 -->
      <Card
        :body-style="{ padding: '20px 24px' }"
        :style="{ marginBottom: '15px' }"
      >
        <div class="flex items-start justify-between">
          <div class="flex items-start gap-4">
            <Avatar
              :size="56"
              :style="{ backgroundColor: '#1677ff', fontSize: '22px' }"
            >
              {{ initials }}
            </Avatar>
            <div>
              <div class="flex items-center gap-3 mb-1">
                <h2 class="text-xl font-bold m-0">{{ contact.name }}</h2>
                <Tag v-if="contact.title" color="blue" size="small">
                  {{ contact.title }}
                </Tag>
                <Tag
                  v-if="contact.roleType"
                  :color="roleColorMap[contact.roleType] || 'default'"
                  size="small"
                >
                  {{ roleTypeMap[contact.roleType] || contact.roleType }}
                </Tag>
              </div>
              <div
                class="flex items-center gap-4 text-sm text-gray-500 flex-wrap"
              >
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
          <Card
            v-if="contact.currentCompany"
            size="small"
            :body-style="{ padding: '16px 20px' }"
            :style="{ marginBottom: '15px' }"
          >
            <template #title>
              <span class="text-blue-600 font-medium">当前任职</span>
            </template>
            <template #extra>
              <Popconfirm title="确认解绑该联系人？" @confirm="handleUnbind">
                <Button size="small" danger>
                  <template #icon><LucideUnlink /></template
                  >{{ $t('page.crm.contact.button.unbind') }}
                </Button>
              </Popconfirm>
            </template>
            <div class="flex items-center justify-between">
              <div>
                <div class="flex items-center gap-2">
                  <span
                    class="font-bold text-base cursor-pointer text-blue-600 hover:underline"
                    @click="
                      handleViewCustomer(contact.currentCompany.customerId)
                    "
                  >
                    {{ contact.currentCompany.companyName }}
                  </span>
                  <Tag v-if="contact.currentCompany.shortName" size="small">
                    {{ contact.currentCompany.shortName }}
                  </Tag>
                </div>
                <div
                  class="text-gray-500 mt-1 flex items-center gap-2 flex-wrap"
                >
                  <span>{{ contact.currentCompany.title || '-' }}</span>
                  <Tag
                    :color="
                      roleColorMap[contact.currentCompany.roleType] || 'default'
                    "
                    size="small"
                  >
                    {{
                      roleTypeMap[contact.currentCompany.roleType] ||
                      contact.currentCompany.roleType
                    }}
                  </Tag>
                  <Tag
                    v-if="contact.currentCompany.isPrimary"
                    color="gold"
                    size="small"
                  >
                    首要联系人
                  </Tag>
                  <Tag
                    v-if="contact.currentCompany.isBilling"
                    color="purple"
                    size="small"
                  >
                    账单
                  </Tag>
                  <Tag
                    v-if="contact.currentCompany.isShipping"
                    color="cyan"
                    size="small"
                  >
                    收货
                  </Tag>
                </div>
                <div class="text-gray-400 text-sm mt-1">
                  {{ contact.currentCompany.boundAt }} 至今
                </div>
              </div>
              <Button
                size="small"
                @click="handleViewCustomer(contact.currentCompany.customerId)"
              >
                <template #icon><LucideBuilding2 /></template>查看公司
              </Button>
            </div>
          </Card>

          <!-- 基本信息 + 修改记录（选项卡形式） -->
          <Card
            size="small"
            :style="{ marginBottom: '15px' }"
            :body-style="{ padding: '0' }"
          >
            <Tabs
              v-model:active-key="activeTab"
              :tab-bar-style="{ paddingLeft: '16px' }"
              @change="handleTabChange"
            >
              <Tabs.TabPane key="basic" tab="基本信息">
                <div style="padding: 16px 20px">
                  <Descriptions :column="2" bordered size="small">
                    <Descriptions.Item label="姓名">
                      {{ contact.name }}
                    </Descriptions.Item>
                    <Descriptions.Item label="性别">
                      {{ genderMap[contact.gender] ?? '-' }}
                    </Descriptions.Item>
                    <Descriptions.Item label="生日">
                      {{ contact.birthday || '-' }}
                    </Descriptions.Item>
                    <Descriptions.Item label="QQ号">
                      {{ contact.qq || '-' }}
                    </Descriptions.Item>
                    <Descriptions.Item label="邮箱">
                      {{ contact.email || '-' }}
                    </Descriptions.Item>
                    <Descriptions.Item label="手机">
                      {{ contact.mobile || '-' }}
                    </Descriptions.Item>
                    <Descriptions.Item label="座机">
                      {{ contact.phone || '-' }}
                    </Descriptions.Item>
                    <Descriptions.Item label="WhatsApp">
                      {{ contact.whatsapp || '-' }}
                    </Descriptions.Item>
                    <Descriptions.Item label="微信">
                      {{ contact.wechat || '-' }}
                    </Descriptions.Item>
                    <Descriptions.Item label="备注" :span="2">
                      {{ contact.notes || '-' }}
                    </Descriptions.Item>
                  </Descriptions>
                </div>
              </Tabs.TabPane>
              <Tabs.TabPane key="logs" tab="更新记录">
                <div style="min-height: 200px; padding: 16px 20px">
                  <Spin :spinning="editLogLoading">
                    <Timeline v-if="editLogs.length > 0">
                      <Timeline.Item
                        v-for="log in editLogs"
                        :key="log.id"
                        color="blue"
                      >
                        <div class="flex items-start justify-between mb-1">
                          <div class="flex items-center gap-2">
                            <Avatar
                              size="small"
                              :style="{ backgroundColor: '#1677ff' }"
                            >
                              {{ log.editorName?.charAt(0) || '?' }}
                            </Avatar>
                            <span class="font-medium text-sm">{{
                              log.editorName || '未知'
                            }}</span>
                          </div>
                          <span class="text-xs text-gray-400">{{
                            log.editTime ? formatDateTime(log.editTime) : '-'
                          }}</span>
                        </div>
                        <div class="mt-1 space-y-1">
                          <div
                            v-for="(item, idx) in log.content"
                            :key="idx"
                            class="text-xs flex items-center gap-1 py-1 px-2 rounded bg-gray-50 flex-wrap"
                          >
                            <Tag
                              color="blue"
                              size="small"
                              class="!mr-0"
                              style="font-size: 11px"
                            >
                              {{ item.fieldLabel }}
                            </Tag>
                            <template
                              v-if="
                                item.old !== null &&
                                item.old !== undefined &&
                                item.new !== null &&
                                item.new !== undefined
                              "
                            >
                              <span class="text-gray-400 line-through">{{
                                item.old
                              }}</span>
                              <span class="text-gray-400">→</span>
                              <span class="text-green-600 font-medium">{{
                                item.new
                              }}</span>
                            </template>
                            <template
                              v-else-if="
                                item.new === null || item.new === undefined
                              "
                            >
                              <span class="text-red-500"
                                >删除：{{ item.old }}</span
                              >
                            </template>
                            <template v-else>
                              <span class="text-green-600 font-medium">{{
                                item.new
                              }}</span>
                            </template>
                          </div>
                        </div>
                      </Timeline.Item>
                    </Timeline>
                    <Empty
                      v-else-if="!editLogLoading"
                      description="暂无修改记录"
                    />
                  </Spin>
                </div>
              </Tabs.TabPane>
            </Tabs>
          </Card>
        </Col>

        <!-- 右侧：联系方式 + 职业生涯 -->
        <Col :span="8">
          <!-- 联系方式快速卡片 -->
          <Card
            v-if="contactChannels.length > 0"
            size="small"
            :style="{ marginBottom: '15px' }"
          >
            <template #title>联系方式</template>
            <div class="flex flex-col gap-2">
              <div
                v-for="ch in contactChannels"
                :key="ch.label"
                class="flex items-center gap-2 text-sm"
              >
                <component :is="ch.icon" :size="14" class="text-gray-400" />
                <span class="text-gray-500">{{ ch.label }}:</span>
                <span class="text-gray-700">{{ ch.value }}</span>
              </div>
            </div>
          </Card>

          <!-- 职业生涯履历 -->
          <Card size="small" :style="{ marginBottom: '15px' }">
            <template #title>职业生涯履历</template>
            <Timeline
              v-if="contact.careerHistory && contact.careerHistory.length > 0"
            >
              <Timeline.Item
                v-for="item in contact.careerHistory"
                :key="item.id"
                :color="item.isCurrent ? 'green' : 'gray'"
              >
                <div class="mb-1">
                  <span
                    class="font-bold cursor-pointer text-blue-600 hover:underline text-sm"
                    @click="handleViewCustomer(item.customerId)"
                  >
                    {{ item.companyName }}
                  </span>
                  <Tag v-if="item.shortName" size="small" class="ml-1">
                    {{ item.shortName }}
                  </Tag>
                </div>
                <div class="text-xs text-gray-500">
                  {{ item.title }}
                  <Tag
                    :color="roleColorMap[item.roleType] || 'default'"
                    size="small"
                    class="ml-1"
                  >
                    {{ roleTypeMap[item.roleType] || item.roleType }}
                  </Tag>
                  <Tag
                    v-if="item.isPrimary"
                    color="gold"
                    size="small"
                    class="ml-1"
                  >
                    首要
                  </Tag>
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
