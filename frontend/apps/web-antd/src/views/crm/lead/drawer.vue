<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer } from '@vben/common-ui';
import { $t } from '#/locales';
import { useVbenForm } from '#/adapter/form';
import { Divider, message } from 'ant-design-vue';
import { LucideMaximize2, LucideMinimize2 } from '@vben/icons';
import { createLeadApi, updateLeadApi } from '#/api';
import TagSelector from '../components/TagSelector.vue';

const data = ref();
const tagSelectorRef = ref<InstanceType<typeof TagSelector>>();
const isFullscreen = ref(false);

const drawerClass = computed(() => isFullscreen.value ? 'w-screen' : 'w-[60vw]');

const getTitle = computed(() =>
  data.value?.create
    ? $t('ui.modal.create', { moduleName: $t('page.crm.lead.title') })
    : $t('ui.modal.update', { moduleName: $t('page.crm.lead.title') }),
);

const isCreate = computed(() => data.value?.create);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

const [BaseForm, baseFormApi] = useVbenForm({
  showDefaultActions: false,
  wrapperClass: 'grid-cols-2',
  compact: true,
  commonConfig: {
    componentProps: { class: 'w-full' },
  },
  schema: [
    // 基本信息
    {
      component: 'Divider',
      fieldName: '_div1',
      hideLabel: true,
      componentProps: { orientation: 'left', plain: true },
      renderComponentContent: () => ({ default: () => $t('page.crm.lead.sections.basic') }),
      formItemClass: 'col-span-2',
    },
    {
      component: 'Input',
      fieldName: 'companyName',
      label: $t('page.crm.lead.fields.companyName'),
      rules: 'required',
      formItemClass: 'col-span-2',
      componentProps: { placeholder: $t('ui.placeholder.input'), allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'contactName',
      label: $t('page.crm.lead.fields.contactName'),
      rules: 'required',
      componentProps: { placeholder: $t('ui.placeholder.input'), allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'title',
      label: $t('page.crm.lead.fields.title'),
      componentProps: { placeholder: $t('ui.placeholder.input'), allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'industry',
      label: $t('page.crm.lead.fields.industry'),
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: [
          { label: $t('enum.lead_industry.retail'), value: 'retail' },
          { label: $t('enum.lead_industry.wholesale'), value: 'wholesale' },
          { label: $t('enum.lead_industry.manufacturer'), value: 'manufacturer' },
          { label: $t('enum.lead_industry.trade_agent'), value: 'trade_agent' },
          { label: $t('enum.lead_industry.ecommerce'), value: 'ecommerce' },
          { label: $t('enum.lead_industry.wechat_business'), value: 'wechat_business' },
          { label: $t('enum.lead_industry.social'), value: 'social' },
          { label: $t('enum.lead_industry.other'), value: 'other' },
        ],
      },
    },
    {
      component: 'Select',
      fieldName: 'level',
      label: $t('page.crm.lead.fields.level'),
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: [
          { label: $t('enum.lead_level.hot'), value: 'hot' },
          { label: $t('enum.lead_level.warm'), value: 'warm' },
          { label: $t('enum.lead_level.cold'), value: 'cold' },
          { label: $t('enum.lead_level.a'), value: 'a' },
          { label: $t('enum.lead_level.b'), value: 'b' },
          { label: $t('enum.lead_level.c'), value: 'c' },
        ],
      },
    },
    // 联系方式
    {
      component: 'Divider',
      fieldName: '_div2',
      hideLabel: true,
      componentProps: { orientation: 'left', plain: true },
      renderComponentContent: () => ({ default: () => $t('page.crm.lead.sections.contact') }),
      formItemClass: 'col-span-2',
    },
    {
      component: 'Input',
      fieldName: 'email',
      label: $t('page.crm.lead.fields.email'),
      componentProps: { placeholder: $t('ui.placeholder.input'), allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'phone',
      label: $t('page.crm.lead.fields.phone'),
      componentProps: { placeholder: $t('ui.placeholder.input'), allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'mobile',
      label: $t('page.crm.lead.fields.mobile'),
      componentProps: { placeholder: $t('ui.placeholder.input'), allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'website',
      label: $t('page.crm.lead.fields.website'),
      componentProps: { placeholder: 'https://', allowClear: true },
    },
    // 业务信息
    {
      component: 'Divider',
      fieldName: '_div3',
      hideLabel: true,
      componentProps: { orientation: 'left', plain: true },
      renderComponentContent: () => ({ default: () => $t('page.crm.lead.sections.business') }),
      formItemClass: 'col-span-2',
    },
    {
      component: 'Select',
      fieldName: 'source',
      label: $t('page.crm.lead.fields.source'),
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: [
          { label: $t('enum.lead_source.website'), value: 'website' },
          { label: $t('enum.lead_source.exhibition'), value: 'exhibition' },
          { label: $t('enum.lead_source.social'), value: 'social' },
          { label: $t('enum.lead_source.referral'), value: 'referral' },
          { label: $t('enum.lead_source.cold_call'), value: 'cold_call' },
          { label: $t('enum.lead_source.customs'), value: 'customs' },
          { label: $t('enum.lead_source.email'), value: 'email' },
          { label: $t('enum.lead_source.alibaba'), value: 'alibaba' },
          { label: $t('enum.lead_source.amazon'), value: 'amazon' },
          { label: $t('enum.lead_source.tiktok'), value: 'tiktok' },
          { label: $t('enum.lead_source.wechat'), value: 'wechat' },
          { label: $t('enum.lead_source.other'), value: 'other' },
        ],
      },
    },
    {
      component: 'Input',
      fieldName: 'sourceDetail',
      label: $t('page.crm.lead.fields.sourceDetail'),
      componentProps: { placeholder: $t('ui.placeholder.input'), allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'country',
      label: $t('page.crm.lead.fields.country'),
      componentProps: { placeholder: $t('ui.placeholder.input'), allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'region',
      label: $t('page.crm.lead.fields.region'),
      componentProps: { placeholder: $t('ui.placeholder.input'), allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'address',
      label: $t('page.crm.lead.fields.address'),
      formItemClass: 'col-span-2',
      componentProps: { placeholder: $t('ui.placeholder.input'), allowClear: true },
    },
    {
      component: 'InputNumber',
      fieldName: 'budget',
      label: $t('page.crm.lead.fields.budget'),
      componentProps: { placeholder: $t('ui.placeholder.input'), min: 0, class: 'w-full' },
    },
    {
      component: 'Select',
      fieldName: 'currency',
      label: $t('page.crm.lead.fields.currency'),
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        allowClear: true,
        options: [
          { label: 'CNY', value: 'CNY' },
          { label: 'USD', value: 'USD' },
          { label: 'EUR', value: 'EUR' },
          { label: 'GBP', value: 'GBP' },
          { label: 'JPY', value: 'JPY' },
          { label: 'KRW', value: 'KRW' },
          { label: 'HKD', value: 'HKD' },
          { label: 'AUD', value: 'AUD' },
        ],
      },
    },
    {
      component: 'DatePicker',
      fieldName: 'nextFollowAt',
      label: $t('page.crm.lead.fields.nextFollowAt'),
      componentProps: { placeholder: $t('ui.placeholder.select'), class: 'w-full', allowClear: true },
    },
    // 其他信息
    {
      component: 'Divider',
      fieldName: '_div4',
      hideLabel: true,
      componentProps: { orientation: 'left', plain: true },
      renderComponentContent: () => ({ default: () => $t('page.crm.lead.sections.other') }),
      formItemClass: 'col-span-2',
    },
    {
      component: 'Input',
      fieldName: 'assignee',
      label: $t('page.crm.lead.fields.assignee'),
      componentProps: { placeholder: $t('ui.placeholder.input'), allowClear: true },
    },
    {
      component: 'Textarea',
      fieldName: 'description',
      label: $t('page.crm.lead.fields.description'),
      formItemClass: 'col-span-2',
      componentProps: { placeholder: $t('ui.placeholder.input'), rows: 3, allowClear: true },
    },
  ],
});

const [Drawer, drawerApi] = useVbenDrawer({
  onCancel() { drawerApi.close(); },
  async onConfirm() {
    const validate = await baseFormApi.validate();
    if (!validate.valid) return;
    setLoading(true);
    const values = await baseFormApi.getValues();
    try {
      // 创建模式：默认状态为 unchecked
      const payload = data.value?.create
        ? { ...values, status: 'unchecked' }
        : { ...values, id: data.value.row.id };

      const result = await (data.value?.create
        ? createLeadApi(payload)
        : updateLeadApi(payload));

      if (data.value?.create && tagSelectorRef.value) {
        const newId = result?.id || result?.data?.id;
        if (newId) {
          await tagSelectorRef.value.saveToEntity(newId);
        }
      }

      message.success(data.value?.create
        ? $t('ui.notification.create_success')
        : $t('ui.notification.update_success'));
      drawerApi.setData({ needRefresh: true });
    } finally {
      drawerApi.close();
      setLoading(false);
    }
  },
  onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      const row = data.value?.row ? { ...data.value.row } : {};
      baseFormApi.setValues(row);
      setLoading(false);
    }
  },
});

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}
</script>

<template>
  <Drawer :title="getTitle" :class="drawerClass">
    <template #extra>
      <button
        class="mr-2 cursor-pointer rounded p-1 opacity-80 transition-opacity hover:opacity-100 hover:bg-black/10"
        @click="toggleFullscreen"
      >
        <LucideMinimize2 v-if="isFullscreen" class="size-4" />
        <LucideMaximize2 v-else class="size-4" />
      </button>
    </template>
    <BaseForm />
    <Divider />
    <div class="mb-2 text-sm font-medium text-gray-700">{{ $t('page.crm.lead.fields.tags') }}</div>
    <TagSelector
      ref="tagSelectorRef"
      entity-type="lead"
      :entity-id="isCreate ? null : data?.row?.id"
    />
  </Drawer>
</template>