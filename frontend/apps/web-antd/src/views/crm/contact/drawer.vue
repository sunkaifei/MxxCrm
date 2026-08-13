<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer } from '@vben/common-ui';
import { $t } from '#/locales';
import { useVbenForm } from '#/adapter/form';
import { message } from 'ant-design-vue';
import { createContactApi, updateContactApi, getCustomerListApi, getContactInfoApi, checkContactDuplicateApi, getCountriesApi } from '#/api';
import { requestClient } from '#/api/request';

const data = ref();

const currentCompanyName = ref<string>('');
const currentCustomerId = ref<number | null>(null);

const getTitle = computed(() =>
  data.value?.create
    ? $t('ui.modal.create', { moduleName: $t('page.crm.contact.title') })
    : $t('ui.modal.update', { moduleName: $t('page.crm.contact.title') }),
);

// 编辑模式抽屉宽度 75%（Vben Drawer 无 width prop，通过 class 覆盖默认 w-130）；新建保持默认宽度
const drawerClass = computed(() =>
  data.value && !data.value.create ? 'w-[75%]' : '',
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

// ==================== 国家 / 省市 / 详细地址（与客户表单一致） ====================

// 中国省市区三级联动数据（Cascader 格式）
const chinaAreaOptions = ref<any[]>([]);

// 是否中国：country 字段存的是国家名称
function isChinaCountry(c?: null | string) {
  const v = (c || '').trim();
  return v === '中国' || v === 'China' || v === 'CN';
}

// 加载中国省市区三级联动数据（/api/system/region/treelist）
async function loadChinaArea() {
  if (chinaAreaOptions.value.length > 0) return; // 已加载则跳过
  try {
    const result: any = await requestClient.get('/api/system/region/treelist');
    const tree = Array.isArray(result) ? result : (result as any)?.data ?? [];
    // RegionTreeVO { id, parentId, title, regionName, sort, status, children }
    // → Cascader { value, label, children }
    chinaAreaOptions.value = convertRegionTreeToCascader(tree);
  } catch { /* ignore */ }
}

// 将 region 树转为 Cascader 选项（value/label 用 regionName，便于回填"省/市/区"文本）
function convertRegionTreeToCascader(nodes: any[]): any[] {
  if (!Array.isArray(nodes)) return [];
  return nodes
    .filter((n) => n && (n.regionName || n.title))
    .map((n) => {
      const label = n.regionName || n.title || '';
      const node: any = { value: label, label };
      if (Array.isArray(n.children) && n.children.length > 0) {
        node.children = convertRegionTreeToCascader(n.children);
      }
      return node;
    });
}

// 国家选择变化：清空两种省市字段，避免与新国家不匹配
// （字段显隐由 schema 的 dependencies.if 保证，此处仅清残留值）
async function handleCountryChange(value: any) {
  // filterFields=false：默认会过滤 undefined，导致旧值清不掉
  await baseFormApi.setValues(
    { region: undefined, regionChina: undefined },
    false,
  );
  if (isChinaCountry(value)) {
    await loadChinaArea();
  }
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
      // 角色是"在客户公司的角色"，存在联系人与客户的关系表中；
      // 未选择所属公司时无法保存，隐藏该字段避免提交被静默丢弃
      dependencies: {
        triggerFields: ['customerId'],
        if: (values) => !!values.customerId,
      },
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
    {
      component: 'ApiSelect',
      fieldName: 'country',
      label: '国家',
      componentProps: {
        placeholder: '请选择国家',
        allowClear: true,
        showSearch: true,
        filterOption: (input: string, option: any) =>
          option.label?.toLowerCase().includes(input.toLowerCase()),
        api: async () => {
          const result: any = await getCountriesApi();
          const items = Array.isArray(result) ? result : [];
          return items.map((item: any) => ({ label: item.name, value: item.name }));
        },
        labelField: 'label',
        valueField: 'value',
        immediate: true,
        onChange: (value: any) => handleCountryChange(value),
      },
    },
    // 省/州：非中国时显示（自由输入，如"纽约"）
    {
      component: 'Input',
      fieldName: 'region',
      label: '省/市',
      dependencies: {
        triggerFields: ['country'],
        if: (values) => !!values.country && !isChinaCountry(values.country),
      },
      componentProps: { placeholder: '省/州，如 纽约', allowClear: true },
    },
    // 省/市/区：中国时显示 Cascader 三级联动（与客户表单一致）
    {
      component: 'Cascader',
      fieldName: 'regionChina',
      label: '省/市',
      dependencies: {
        triggerFields: ['country'],
        if: (values) => isChinaCountry(values.country),
        componentProps: async (values: any) => {
          if (!isChinaCountry(values?.country)) return {};
          await loadChinaArea();
          return {
            options: chinaAreaOptions.value,
            placeholder: '请选择省/市/区',
            changeOnSelect: true,
            allowClear: true,
          };
        },
      },
    },
    {
      component: 'Input',
      fieldName: 'address',
      label: '详细地址',
      formItemClass: 'col-span-2',
      componentProps: { placeholder: '详细地址', allowClear: true },
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
      const { customerId, _div1, _div2, _div3, regionChina, ...rawFields } =
        values;

      // 省市合并：中国取 Cascader 数组转"省/市/区"文本，其他国家直接取输入文本
      if (isChinaCountry(rawFields.country)) {
        rawFields.region = Array.isArray(regionChina) && regionChina.length > 0
          ? regionChina.join('/')
          : undefined;
      }

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

      // 每次打开都重置表单，清除上次编辑/新建残留的数据（setValues 是合并写入，不重置会串数据）
      await baseFormApi.resetForm();

      // 清空上次的公司回显信息，避免上一个联系人的公司串进当前选项列表
      currentCompanyName.value = '';
      currentCustomerId.value = null;

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
            row.country = d.country ?? row.country;
            row.region = d.region ?? row.region;
            row.address = d.address ?? row.address;
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

      // 省市回显：中国时把"省/市/区"文本拆为 Cascader 数组路径（写入 regionChina 字段）
      if (isChinaCountry(row.country)) {
        if (typeof row.region === 'string' && row.region) {
          row.regionChina = row.region.split(/\s*\/\s*/).filter(Boolean);
        }
        row.region = undefined;
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
  <Drawer :title="getTitle" :class="drawerClass">
    <BaseForm />
  </Drawer>
</template>
