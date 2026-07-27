<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer } from '@vben/common-ui';
import { $t } from '#/locales';
import { useVbenForm } from '#/adapter/form';
import { message } from 'ant-design-vue';
import { createContactApi, updateContactApi, getCustomerListApi, getContactInfoApi, checkContactDuplicateApi } from '#/api';

const data = ref();

const currentCompanyName = ref<string>('');
const currentCustomerId = ref<number | null>(null);

const getTitle = computed(() =>
  data.value?.create
    ? $t('ui.modal.create', { moduleName: $t('page.crm.contact.title') })
    : $t('ui.modal.update', { moduleName: $t('page.crm.contact.title') }),
);

// 格式校验规则
const validateMobile = (_rule: any, value: string) => {
  if (!value || !value.trim()) {
    return Promise.reject('请输入手机号');
  }
  // 支持11位手机号或带国际区号格式
  if (/^1[3-9]\d{9}$/.test(value) || /^\+\d{1,4}\s?\d{6,14}$/.test(value)) {
    return Promise.resolve();
  }
  return Promise.reject('请输入正确的手机号格式');
};

const validatePhone = (_rule: any, value: string) => {
  if (!value) return Promise.resolve();
  // 座机格式：区号-号码，如 010-12345678
  if (/^\d{3,4}-?\d{7,8}$/.test(value)) {
    return Promise.resolve();
  }
  return Promise.reject('请输入正确的座机格式，如 010-12345678');
};

const validateEmail = (_rule: any, value: string) => {
  if (!value) return Promise.resolve();
  if (/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value)) {
    return Promise.resolve();
  }
  return Promise.reject('请输入正确的邮箱格式');
};

const validateQq = (_rule: any, value: string) => {
  if (!value) return Promise.resolve();
  if (/^[1-9]\d{4,11}$/.test(value)) {
    return Promise.resolve();
  }
  return Promise.reject('QQ号应为5-12位数字');
};

const validateWhatsapp = (_rule: any, value: string) => {
  if (!value) return Promise.resolve();
  // 支持字母、数字，或两者混合；也支持常见格式字符：+ - _ . 空格
  if (/^[a-zA-Z0-9+\-_.\s]{3,50}$/.test(value)) {
    return Promise.resolve();
  }
  return Promise.reject('WhatsApp 长度应为 3-50 位，支持字母、数字及 + - _ . 空格');
};

// 实时查重校验
const checkDuplicate = async (field: string, valuePromise: Promise<string>) => {
  const value = await valuePromise;
  if (!value || !value.trim()) return Promise.resolve();
  // 列表 VO 的 id 序列化为字符串，后端期望 i64，需转 Number
  const editId = data.value?.create ? undefined : Number(data.value?.row?.id);
  try {
    const results: any = await checkContactDuplicateApi({
      id: editId,
      [field]: value.trim(),
    });
    const item = (results as any[])?.find((r: any) => r.field === field);
    if (item?.duplicated) {
      return Promise.reject(`该${fieldLabelMap[field]}已被「${item.contactName || '其他联系人'}」使用`);
    }
    return Promise.resolve();
  } catch {
    return Promise.resolve();
  }
};

const fieldLabelMap: Record<string, string> = {
  mobile: '手机号',
  phone: '座机',
  wechat: '微信号',
  qq: 'QQ号',
  email: '邮箱',
};

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
      renderComponentContent: () => ({ default: () => '基本信息' }),
      formItemClass: 'col-span-2',
    },
    {
      component: 'Input',
      fieldName: 'name',
      label: '姓名',
      rules: 'required',
      componentProps: { placeholder: '请输入姓名', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'gender',
      label: '性别',
      defaultValue: 2,
      componentProps: {
        placeholder: '选择性别',
        allowClear: true,
        options: [
          { label: '男', value: 0 },
          { label: '女', value: 1 },
          { label: '未知', value: 2 },
        ],
      },
    },
    {
      component: 'Input',
      fieldName: 'title',
      label: '职位',
      componentProps: { placeholder: '如 采购经理', allowClear: true },
    },
    {
      component: 'Select',
      fieldName: 'roleType',
      label: '角色',
      componentProps: {
        placeholder: '选择角色',
        allowClear: true,
        options: [
          { label: '决策人', value: 0 },
          { label: '影响者', value: 1 },
          { label: '使用者', value: 2 },
          { label: '其他', value: 3 },
        ],
      },
    },
    // 所属公司（独立一行）
    {
      component: 'ApiSelect',
      fieldName: 'customerId',
      label: '所属公司',
      formItemClass: 'col-span-2',
      componentProps: {
        placeholder: '搜索并选择客户',
        allowClear: true,
        showSearch: true,
        filterOption: false,
        remote: true,
        params: { companyName: '' },
        api: async (params: any) => {
          const res: any = await getCustomerListApi({
            page: 1,
            pageSize: 20,
            ...(params?.companyName ? { companyName: params.companyName } : {}),
          });
          const items = res?.items || [];
          if (currentCompanyName.value && currentCustomerId.value) {
            const exists = items.some((item: any) => String(item.id) === String(currentCustomerId.value));
            if (!exists) {
              items.unshift({ id: String(currentCustomerId.value), companyName: currentCompanyName.value });
            }
          }
          return items;
        },
        labelField: 'companyName',
        valueField: 'id',
        onSearch(keyword: string) {
          baseFormApi.updateSchema([
            {
              fieldName: 'customerId',
              componentProps: {
                params: { companyName: keyword },
              },
            },
          ]);
        },
        immediate: true,
      },
    },
    // 联系方式
    {
      component: 'Divider',
      fieldName: '_div2',
      hideLabel: true,
      componentProps: { orientation: 'left', plain: true },
      renderComponentContent: () => ({ default: () => '联系方式' }),
      formItemClass: 'col-span-2',
    },
    {
      component: 'Input',
      fieldName: 'email',
      label: '邮箱',
      rules: [
        { validator: validateEmail, trigger: 'blur' },
        { validator: () => checkDuplicate('email', getEmailValue()), trigger: 'blur' },
      ],
      componentProps: { placeholder: 'email@example.com', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'mobile',
      label: '手机号',
      required: true,
      rules: [
        { validator: validateMobile, trigger: 'blur' },
        { validator: () => checkDuplicate('mobile', getMobileValue()), trigger: 'blur' },
      ],
      componentProps: { placeholder: '手机号', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'phone',
      label: '座机',
      rules: [
        { validator: validatePhone, trigger: 'blur' },
        { validator: () => checkDuplicate('phone', getPhoneValue()), trigger: 'blur' },
      ],
      componentProps: { placeholder: '座机号码', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'whatsapp',
      label: 'WhatsApp',
      rules: [
        { validator: validateWhatsapp, trigger: 'blur' },
      ],
      componentProps: { placeholder: 'WhatsApp 号码或账号', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'wechat',
      label: '微信',
      rules: [
        { validator: () => checkDuplicate('wechat', getWechatValue()), trigger: 'blur' },
      ],
      componentProps: { placeholder: '微信号', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'qq',
      label: 'QQ号',
      rules: [
        { validator: validateQq, trigger: 'blur' },
        { validator: () => checkDuplicate('qq', getQqValue()), trigger: 'blur' },
      ],
      componentProps: { placeholder: 'QQ号', allowClear: true },
    },
    // 其他信息
    {
      component: 'Divider',
      fieldName: '_div3',
      hideLabel: true,
      componentProps: { orientation: 'left', plain: true },
      renderComponentContent: () => ({ default: () => '其他信息' }),
      formItemClass: 'col-span-2',
    },
    {
      component: 'DatePicker',
      fieldName: 'birthday',
      label: '生日',
      componentProps: {
        placeholder: '选择日期',
        class: 'w-full',
        allowClear: true,
        valueFormat: 'YYYY-MM-DD',
      },
    },
    {
      component: 'Textarea',
      fieldName: 'notes',
      label: '备注',
      formItemClass: 'col-span-2',
      componentProps: { placeholder: '备注信息', rows: 3, allowClear: true },
    },
  ],
});

// 获取表单值的辅助函数
function getEmailValue() {
  return baseFormApi.getValues().then((v: any) => v?.email || '');
}
function getMobileValue() {
  return baseFormApi.getValues().then((v: any) => v?.mobile || '');
}
function getPhoneValue() {
  return baseFormApi.getValues().then((v: any) => v?.phone || '');
}
function getWechatValue() {
  return baseFormApi.getValues().then((v: any) => v?.wechat || '');
}
function getQqValue() {
  return baseFormApi.getValues().then((v: any) => v?.qq || '');
}

const [Drawer, drawerApi] = useVbenDrawer({
  onCancel() { drawerApi.close(); },
  async onConfirm() {
    const validate = await baseFormApi.validate();
    if (!validate.valid) return;
    setLoading(true);
    try {
      const values = await baseFormApi.getValues();
      const { customerId, _div1, _div2, _div3, ...rawFields } = values;

      // 清理空值：空字符串/null/undefined 不提交，后端按 None 处理
      const contactFields: Record<string, any> = {};
      for (const [key, val] of Object.entries(rawFields)) {
        if (val !== '' && val !== null && val !== undefined) {
          contactFields[key] = val;
        }
      }

      const isCreate = data.value?.create;
      const payload: Record<string, any> = isCreate
        ? contactFields
        : { ...contactFields, id: Number(data.value.row.id) };
      if (customerId) {
        payload.customerId = Number(customerId);
      }

      const result = isCreate
        ? await createContactApi(payload)
        : await updateContactApi(payload);

      message.success(isCreate
        ? $t('ui.notification.create_success')
        : $t('ui.notification.update_success'));
      drawerApi.setData({ needRefresh: true });
      drawerApi.close();
    } catch {
      // 错误由全局拦截器处理，保留抽屉打开以便用户修改后重试
    } finally {
      setLoading(false);
    }
  },
  async onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      const row = data.value?.row ? { ...data.value.row } : {};
      setLoading(false);

      const isCreate = data.value?.create;

      // 编辑模式下所属企业禁止修改
      baseFormApi.updateSchema([
        {
          fieldName: 'customerId',
          componentProps: { disabled: !isCreate },
        },
      ]);

      // 新建模式先重置表单，清除上次编辑残留的数据
      if (isCreate) {
        await baseFormApi.resetForm();
      }

      // 先设置当前公司信息，确保 ApiSelect 的选项列表包含当前选中项
      if (row.customerId) {
        currentCompanyName.value = row.companyName || '';
        currentCustomerId.value = row.customerId;
        row.customerId = String(row.customerId);
      }

      // 编辑模式：加载详情，把详情数据合并到 row 中
      if (!isCreate && row?.id) {
        try {
          const detail: any = await getContactInfoApi(Number(row.id));
          const d = detail?.data || detail || {};
          if (d) {
            row.name = d.name ?? row.name;
            row.title = d.title ?? row.title;
            row.email = d.email ?? row.email;
            row.phone = d.phone ?? row.phone;
            row.mobile = d.mobile ?? row.mobile;
            row.whatsapp = d.whatsapp ?? row.whatsapp;
            row.wechat = d.wechat ?? row.wechat;
            row.qq = d.qq ?? row.qq;
            row.gender = d.gender;
            row.birthday = d.birthday;
            row.notes = d.notes;
            if (d.currentCompany) {
              row.roleType = d.currentCompany.roleType ?? row.roleType;
              if (d.currentCompany.customerId) {
                row.customerId = String(d.currentCompany.customerId);
                currentCompanyName.value = d.currentCompany.companyName || '';
                currentCustomerId.value = d.currentCompany.customerId;
              }
            }
          }
        } catch {
          // 详情加载失败忽略，用列表行数据回显
        }
      }

      // 一次性 setValues，参考 crm/contract/drawer.vue 的模式
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
  <Drawer :title="getTitle" :width="580">
    <BaseForm />
  </Drawer>
</template>
