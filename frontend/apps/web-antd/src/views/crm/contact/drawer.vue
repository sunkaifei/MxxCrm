<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer } from '@vben/common-ui';
import { $t } from '#/locales';
import { useVbenForm } from '#/adapter/form';
import { message } from 'ant-design-vue';
import { createContactApi, updateContactApi, getCustomerListApi, getContactInfoApi } from '#/api';

const data = ref();

// 当前编辑的联系人所属公司信息（用于 ApiSelect 回显，绕过表单异步读取问题）
const currentCompanyName = ref<string>('');
const currentCustomerId = ref<number | null>(null);

const getTitle = computed(() =>
  data.value?.create
    ? $t('ui.modal.create', { moduleName: $t('page.crm.contact.title') })
    : $t('ui.modal.update', { moduleName: $t('page.crm.contact.title') }),
);

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
          // 如果当前公司不在列表中，手动加入（用于编辑回显）
          // 注意：API 序列化 id 为 string，需保持一致
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
        // 远程搜索：输入关键词时更新params触发重新请求
        onSearch(keyword: string) {
          baseFormApi.updateSchema('customerId', {
            componentProps: {
              params: { companyName: keyword },
            },
          });
        },
        // 初次打开时加载最近客户
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
      componentProps: { placeholder: 'email@example.com', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'mobile',
      label: '手机号',
      componentProps: { placeholder: '手机号', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'phone',
      label: '座机',
      componentProps: { placeholder: '座机号码', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'whatsapp',
      label: 'WhatsApp',
      componentProps: { placeholder: 'WhatsApp 号码', allowClear: true },
    },
    {
      component: 'Input',
      fieldName: 'wechat',
      label: '微信',
      componentProps: { placeholder: '微信号', allowClear: true },
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

const [Drawer, drawerApi] = useVbenDrawer({
  onCancel() { drawerApi.close(); },
  async onConfirm() {
    const validate = await baseFormApi.validate();
    if (!validate.valid) return;
    setLoading(true);
    try {
      const values = await baseFormApi.getValues();
      const { customerId, ...contactFields } = values;

      const isCreate = data.value?.create;
      const payload = isCreate
        ? contactFields
        : { ...contactFields, id: data.value.row.id };
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
  onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      const row = data.value?.row ? { ...data.value.row } : {};
      setLoading(false);

      // 重置表单，清除上一次打开的数据
      baseFormApi.resetForm();

      // 从列表行数据中直接获取公司信息，用于 ApiSelect 回显
      // API 中 id 序列化为 string，统一使用 string 避免类型不匹配
      currentCompanyName.value = row.companyName || '';
      currentCustomerId.value = row.customerId || null;
      // 只设置 row 中存在的字段，避免 undefined 覆盖 resetForm 的默认值
      const definedValues = Object.fromEntries(
        Object.entries(row).filter(([_, v]) => v !== undefined && v !== null),
      );
      baseFormApi.setValues({
        ...definedValues,
        customerId: row.customerId != null ? String(row.customerId) : undefined,
      });

      if (!data.value?.create && row?.id) {
        getContactInfoApi(row.id)
          .then((detail: any) => {
            const d = detail?.data || detail || {};
            // 从详情 API 获取完整数据，回填列表 API 未提供的字段
            baseFormApi.setValues({
              whatsapp: d.whatsapp,
              wechat: d.wechat,
              gender: d.gender,
              birthday: d.birthday,
              notes: d.notes,
              customerId: d.currentCompany?.customerId ? String(d.currentCompany.customerId) : undefined,
            });
            if (d.currentCompany?.customerId) {
              currentCompanyName.value = d.currentCompany.companyName || '';
            }
          })
          .catch(() => {});
      }
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