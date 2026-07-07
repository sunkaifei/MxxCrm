<script lang="ts" setup>
import { computed, nextTick, ref } from 'vue';
import dayjs from 'dayjs';
import { useVbenDrawer } from '#/adapter/drawer';
import { $t } from '#/locales';
import { useVbenForm, z } from '#/adapter/form';
import { Divider, message } from 'ant-design-vue';
import { createCustomerApi, updateCustomerApi, getCountriesApi, getCustomerInfoApi, checkCustomerNameApi } from '#/api';
import TagSelector from '../components/TagSelector.vue';
import { useDebounceFn } from '@vueuse/core';

const data = ref<Record<string, any>>();
const tagSelectorRef = ref<InstanceType<typeof TagSelector>>();
const isFullscreen = ref(false);

// 持久跟踪公司名称的查重错误（防止 Zod 校验通过后被清除）
const companyNameError = ref<string | undefined>(undefined);

// 错误闪烁动画触发
const isFlashing = ref(false);
let flashTimer: ReturnType<typeof setTimeout> | null = null;
function triggerErrorFlash() {
  if (flashTimer) clearTimeout(flashTimer);
  isFlashing.value = false;
  nextTick(() => {
    isFlashing.value = true;
    flashTimer = setTimeout(() => {
      isFlashing.value = false;
    }, 900);
  });
}

// 滚动到指定字段并聚焦（重试3次，适配 Drawer 异步渲染）
function scrollToField(fieldName: string) {
  let retries = 0;
  const doScroll = () => {
    const ref = baseFormApi.getFieldComponentRef(fieldName);
    if (!ref) {
      if (retries < 3) { retries++; setTimeout(doScroll, retries * 80); }
      return;
    }
    const el = (ref as any)?.$el instanceof HTMLElement
      ? (ref as any).$el
      : (ref as any) instanceof HTMLElement
        ? ref
        : null;
    if (el) {
      el.scrollIntoView?.({ behavior: 'smooth', block: 'center' });
      el.focus?.();
    } else {
      (ref as any)?.focus?.();
    }
  };
  doScroll();
}

// 防抖的公司名称查重（提交时使用）
const checkNameDebounced = useDebounceFn(async (val: string, excludeId?: number): Promise<boolean> => {
  try {
    const res = await checkCustomerNameApi({ companyName: val, excludeId });
    return !!(res as any)?.exists;
  } catch {
    return false;
  }
}, 400);

// 公司名称输入时实时查重（防抖）—— 重复则设红框+闪烁，不重复则清除错误
const checkNameOnInput = useDebounceFn(async (val: string, excludeId?: number) => {
  try {
    const res = await checkCustomerNameApi({ companyName: val, excludeId });
    if (!!(res as any)?.exists) {
      // 仅在从"无错误→有错误"时触发闪烁，避免每次按键都闪
      const wasError = !!companyNameError.value;
      companyNameError.value = '该公司名称已存在';
      baseFormApi.form.setFieldError('companyName', '该公司名称已存在');
      if (!wasError) {
        triggerErrorFlash();
      }
    } else {
      companyNameError.value = undefined;
      baseFormApi.form.setFieldError('companyName', undefined);
    }
  } catch {
    // 忽略错误
  }
}, 500);

const getTitle = computed(() =>
  data.value?.create
    ? $t('ui.modal.create', { moduleName: $t('page.crm.customer.title') })
    : $t('ui.modal.update', { moduleName: $t('page.crm.customer.title') }),
);

const isCreate = computed(() => data.value?.create);

const drawerClass = computed(() => [
  'customer-drawer',
  { 'customer-drawer--fullscreen': isFullscreen.value },
]);

function toggleFullscreen() {
  isFullscreen.value = !isFullscreen.value;
}

const [BaseForm, baseFormApi] = useVbenForm({
  showDefaultActions: false,
  wrapperClass: 'grid-cols-2',
  compact: true,
  scrollToFirstError: true,
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
      renderComponentContent: () => ({ default: () => '基本信息' }),
      formItemClass: 'col-span-2',
    },
    {
      component: 'Input',
      fieldName: 'companyName',
      label: '公司名称',
      rules: z.string().min(1, '请输入公司名称'),
      componentProps: {
        placeholder: '请输入公司名称',
        allowClear: true,
        onChange: (e: any) => {
          const val = e?.target?.value ?? e;
          if (!val || !val.trim()) {
            companyNameError.value = undefined;
            baseFormApi.form.setFieldError('companyName', undefined);
            return;
          }
          // Zod 校验通过后会清除错误，需在 nextTick 中恢复已知的查重错误
          nextTick(() => {
            if (companyNameError.value) {
              baseFormApi.form.setFieldError('companyName', companyNameError.value);
            }
          });
          const excludeId = data.value?.create
            ? undefined
            : Number(data.value?.row?.id) || undefined;
          checkNameOnInput(val.trim(), excludeId);
        },
        onBlur: () => {
          // 光标离开时 vee-validate 会重新校验并清除手动设置的错误，需恢复
          nextTick(() => {
            if (companyNameError.value) {
              baseFormApi.form.setFieldError('companyName', companyNameError.value);
            }
          });
        },
      },
    },
    {
      component: 'Input',
      fieldName: 'customerNo',
      label: '客户编号',
      componentProps: {
        placeholder: '保存时自动生成',
        disabled: true,
        allowClear: false,
      },
    },
    {
      component: 'Input',
      fieldName: 'shortName',
      label: '简称',
      componentProps: { placeholder: '请输入公司简称', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'level',
      label: '客户等级',
      componentProps: {
        placeholder: '请选择客户等级',
        allowClear: true,
        options: [
          { label: '无级别', value: 1 },
          { label: '重点客户', value: 2 },
          { label: '优质客户', value: 3 },
          { label: '普通客户', value: 4 },
          { label: '其他', value: 5 },
        ],
      },
    },
    {
      component: 'Select',
      fieldName: 'source',
      label: '客户来源',
      componentProps: {
        placeholder: '请选择来源',
        allowClear: true,
        options: [
          { label: '官网', value: 1 },
          { label: '展会', value: 2 },
          { label: '社交媒体', value: 3 },
          { label: '客户转介', value: 4 },
          { label: '陌生拜访', value: 5 },
          { label: '海关数据', value: 6 },
          { label: '邮件营销', value: 7 },
          { label: '阿里国际站', value: 8 },
          { label: 'Amazon', value: 9 },
          { label: 'TikTok', value: 10 },
          { label: '微信', value: 11 },
          { label: '其他', value: 12 },
        ],
      },
    },
    {
      component: 'Select',
      fieldName: 'industry',
      label: '行业',
      componentProps: {
        placeholder: '请选择行业',
        allowClear: true,
        options: [
          { label: '零售', value: 1 },
          { label: '批发', value: 2 },
          { label: '制造', value: 3 },
          { label: '贸易代理', value: 4 },
          { label: '电商', value: 5 },
          { label: '微商', value: 6 },
          { label: '社交电商', value: 7 },
          { label: '其他', value: 8 },
        ],
      },
    },
    // 地区信息
    {
      component: 'Divider',
      fieldName: '_div2',
      hideLabel: true,
      componentProps: { orientation: 'left', plain: true },
      renderComponentContent: () => ({ default: () => '地区信息' }),
      formItemClass: 'col-span-2',
    },
    {
      component: 'ApiSelect',
      fieldName: 'country',
      label: '国家',
      componentProps: {
        placeholder: '请选择国家',
        allowClear: true,
        showSearch: true,
        filterOption: (input: string, option: any) => {
          return (
            option.label?.toLowerCase().includes(input.toLowerCase()) ||
            option.value?.toLowerCase().includes(input.toLowerCase())
          );
        },
        api: async () => {
          const result = await getCountriesApi();
          const items = Array.isArray(result) ? result : [];
          return items.map((item: any) => ({
            label: item.name,
            value: item.name,
            labelEn: item.nameEn,
          }));
        },
      },
    },
    {
      component: 'Input',
      fieldName: 'region',
      label: '省/州',
      componentProps: { placeholder: '省/州', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'address',
      label: '详细地址',
      formItemClass: 'col-span-2',
      componentProps: { placeholder: '详细地址', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'website',
      label: '网站',
      componentProps: { placeholder: 'https://', allowClear: true },
    },
    // 财务信息
    {
      component: 'Divider',
      fieldName: '_div3',
      hideLabel: true,
      componentProps: { orientation: 'left', plain: true },
      renderComponentContent: () => ({ default: () => '财务信息' }),
      formItemClass: 'col-span-2',
    },
    {
      component: 'Select',
      fieldName: 'currency',
      label: '币种',
      defaultValue: 1,
      componentProps: {
        options: [
          { label: 'CNY (人民币)', value: 1 },
          { label: 'USD (美元)', value: 2 },
          { label: 'EUR (欧元)', value: 3 },
          { label: 'GBP (英镑)', value: 4 },
          { label: 'JPY (日元)', value: 5 },
          { label: 'HKD (港币)', value: 6 },
          { label: 'AUD (澳元)', value: 7 },
        ],
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'creditLimit',
      label: '信用额度',
      defaultValue: 0,
      rules: 'positiveInteger',
      componentProps: { placeholder: '信用额度', min: 0, precision: 0, step: 1, class: 'w-full', addonAfter: '元' },
    },
    {
      component: 'InputNumber',
      fieldName: 'creditDays',
      label: '账期(天)',
      defaultValue: 15,
      rules: 'positiveInteger',
      componentProps: { placeholder: '账期', min: 0, precision: 0, step: 1, class: 'w-full', addonAfter: '天' },
    },
    // 其他信息
    {
      component: 'Divider',
      fieldName: '_div4',
      hideLabel: true,
      componentProps: { orientation: 'left', plain: true },
      renderComponentContent: () => ({ default: () => '其他信息' }),
      formItemClass: 'col-span-2',
    },
    {
      component: 'DatePicker',
      fieldName: 'cooperatedAt',
      label: '合作起始日期',
      componentProps: { placeholder: '选择日期', class: 'w-full', allowClear: true },
    },
    {
      component: 'Textarea',
      fieldName: 'description',
      label: '备注',
      formItemClass: 'col-span-2',
      componentProps: { placeholder: '备注信息', rows: 3, allowClear: true },
    },
  ],
});

const [Drawer, drawerApi] = useVbenDrawer({
  onCancel() {
    drawerApi.close();
  },

  async onConfirm() {
    const validate = await baseFormApi.validate();
    if (!validate.valid) {
      // 找到第一个出错的字段，闪烁红框 + 滚动并聚焦
      const firstErrorField = Object.keys(validate.errors || {})[0];
      if (firstErrorField) {
        triggerErrorFlash();
        await nextTick();
        scrollToField(firstErrorField);
      }
      return;
    }

    // 公司名称查重
    const companyName = (await baseFormApi.getValues()).companyName;
    if (companyName && companyName.trim()) {
      const excludeId = data.value?.create ? undefined : Number(data.value?.row?.id) || undefined;
      const exists = await checkNameDebounced(companyName.trim(), excludeId);
      if (exists) {
        companyNameError.value = '该公司名称已存在';
        baseFormApi.form.setFieldError('companyName', '该公司名称已存在');
        triggerErrorFlash();
        await nextTick();
        scrollToField('companyName');
        return;
      }
    }

    drawerApi.setState({ loading: true });
    const values = await baseFormApi.getValues();
    // 合作起始日期：dayjs 对象转字符串，空值默认当前日期
    if (values.cooperatedAt) {
      values.cooperatedAt = dayjs.isDayjs(values.cooperatedAt)
        ? values.cooperatedAt.format('YYYY-MM-DD')
        : values.cooperatedAt;
    } else {
      values.cooperatedAt = dayjs().format('YYYY-MM-DD');
    }
    try {
      const result = await (data.value?.create
        ? createCustomerApi(values)
        : updateCustomerApi({ ...values, id: data.value.row.id }));

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
      drawerApi.close();
    } catch {
      // 错误由全局拦截器处理，保留抽屉打开以便用户修改后重试
    } finally {
      drawerApi.setState({ loading: false });
    }
  },

  onOpenChange(isOpen) {
    if (isOpen) {
      isFullscreen.value = false;
      companyNameError.value = undefined;
      data.value = drawerApi.getData<Record<string, any>>();
      const row = data.value?.row ? { ...data.value.row } : {};
      baseFormApi.resetForm();
      // 先用列表行数据设置基本表单值，level 确保为数字
      if (Object.keys(row).length > 0) {
        baseFormApi.setValues({
          ...row,
          level: row.level != null ? Number(row.level) : undefined,
          industry: row.industry != null ? Number(row.industry) : undefined,
          source: row.source != null ? Number(row.source) : undefined,
          cooperatedAt: row.cooperatedAt ? dayjs(row.cooperatedAt) : undefined,
        });
      }
      // 编辑模式下调用详情 API 回填完整数据
      if (!data.value?.create && row?.id) {
        getCustomerInfoApi(row.id)
          .then((detail: any) => {
            const d = detail?.data || detail || {};
            baseFormApi.setValues({
              companyName: d.companyName || d.name || '',
              shortName: d.shortName,
              customerNo: d.customerNo,
              level: d.level != null ? Number(d.level) : undefined,
              industry: d.industry != null ? Number(d.industry) : undefined,
              source: d.source != null ? Number(d.source) : undefined,
              country: d.country,
              region: d.region,
              address: d.address,
              website: d.website,
              currency: d.currency != null ? Number(d.currency) : undefined,
              creditLimit: d.creditLimit,
              creditDays: d.creditDays,
              cooperatedAt: d.cooperatedAt ? dayjs(d.cooperatedAt) : undefined,
              description: d.description,
            });
          })
          .catch(() => {});
      }
    }
  },
});

function closeDrawer() {
  drawerApi.close();
}
</script>

<template>
  <Drawer
    :title="getTitle"
    :class="drawerClass"
    :destroy-on-close="true"
    :z-index="2000"
  >
    <template #extra>
      <button
        type="button"
        class="customer-drawer__fs-btn"
        @click="toggleFullscreen"
      >
        <svg v-if="!isFullscreen" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M8 3H5a2 2 0 0 0-2 2v3" />
          <path d="M21 8V5a2 2 0 0 0-2-2h-3" />
          <path d="M3 16v3a2 2 0 0 0 2 2h3" />
          <path d="M16 21h3a2 2 0 0 0 2-2v-3" />
        </svg>
        <svg v-else viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M8 3v3a2 2 0 0 1-2 2H3" />
          <path d="M21 8h-3a2 2 0 0 1-2-2V3" />
          <path d="M3 16h3a2 2 0 0 1 2 2v3" />
          <path d="M16 21v-3a2 2 0 0 1 2-2h3" />
        </svg>
      </button>
    </template>
    <div :class="['customer-form-wrapper', { 'error-flash': isFlashing }]">
      <BaseForm />
    </div>
    <Divider />
    <div class="mb-2 text-sm font-medium text-gray-700">标签</div>
    <TagSelector
      ref="tagSelectorRef"
      entity-type="customer"
      :entity-id="isCreate ? null : Number(data?.row?.id) || null"
    />
  </Drawer>
</template>

<style>
.customer-drawer {
  width: 75vw !important;
}
.customer-drawer--fullscreen {
  width: 100vw !important;
}
.customer-drawer__fs-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  margin-right: 8px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: rgba(0, 0, 0, 0.45);
  cursor: pointer;
  transition: all 0.2s;
}
.customer-drawer__fs-btn:hover {
  color: #1890ff;
  background-color: rgba(0, 0, 0, 0.06);
}

/* ===== 表单校验错误状态增强 ===== */

/* 错误状态：红色边框 + 浅红背景 */
/* 覆盖所有 Ant Design 表单控件的实际边框元素 */
.customer-form-wrapper .form-valid-error .ant-input,
.customer-form-wrapper .form-valid-error .ant-input-affix-wrapper,
.customer-form-wrapper .form-valid-error .ant-input-number,
.customer-form-wrapper .form-valid-error .ant-input-number-input,
.customer-form-wrapper .form-valid-error .ant-input-number-group-wrapper,
.customer-form-wrapper .form-valid-error .ant-select .ant-select-selector,
.customer-form-wrapper .form-valid-error .ant-picker,
.customer-form-wrapper .form-valid-error textarea {
  border-color: #ff4d4f !important;
  background-color: #fff2f0 !important;
}

/* 错误状态 hover/focus 保持红色 */
.customer-form-wrapper .form-valid-error .ant-input:hover,
.customer-form-wrapper .form-valid-error .ant-input:focus,
.customer-form-wrapper .form-valid-error .ant-input-affix-wrapper:hover,
.customer-form-wrapper .form-valid-error .ant-input-affix-wrapper-focused,
.customer-form-wrapper .form-valid-error .ant-input-number:hover,
.customer-form-wrapper .form-valid-error .ant-input-number-focused,
.customer-form-wrapper .form-valid-error .ant-input-number-group-wrapper:hover,
.customer-form-wrapper .form-valid-error .ant-select:hover .ant-select-selector,
.customer-form-wrapper .form-valid-error .ant-select-focused .ant-select-selector,
.customer-form-wrapper .form-valid-error .ant-picker:hover,
.customer-form-wrapper .form-valid-error .ant-picker-focused {
  border-color: #ff4d4f !important;
  box-shadow: 0 0 0 2px rgba(255, 77, 79, 0.2) !important;
}

/* compact 模式下显示错误提示文字（覆盖 tooltip-only 行为） */
.customer-form-wrapper .form-valid-error [role='alert'],
.customer-form-wrapper .form-valid-error .form-message {
  display: block !important;
  position: static !important;
  color: #ff4d4f;
  font-size: 12px;
  line-height: 1.5;
  margin-top: 2px;
  animation: none !important;
}


/* ===== 红色闪烁动画（校验失败时触发） ===== */
@keyframes error-flash-anim {
  0%, 100% {
    border-color: #ff4d4f;
    box-shadow: none;
  }
  50% {
    border-color: #ff4d4f;
    box-shadow: 0 0 0 3px rgba(255, 77, 79, 0.35);
  }
}

.error-flash .form-valid-error .ant-input,
.error-flash .form-valid-error .ant-input-affix-wrapper,
.error-flash .form-valid-error .ant-input-number,
.error-flash .form-valid-error .ant-input-number-group-wrapper,
.error-flash .form-valid-error .ant-select .ant-select-selector,
.error-flash .form-valid-error .ant-picker,
.error-flash .form-valid-error textarea {
  animation: error-flash-anim 0.3s ease-in-out 3;
}
</style>