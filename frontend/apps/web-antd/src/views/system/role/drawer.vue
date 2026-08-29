<script lang="ts" setup>
import { computed, ref } from 'vue';

import { useVbenDrawer, z } from '@vben/common-ui';

import { message } from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import { createRoleApi, getMenusRouterApi, updateRoleApi } from '#/api';
import { $t } from '#/locales';
import { statusList } from '#/store';

const data = ref();
const getTitle = computed(() =>
  data.value?.create
    ? $t('ui.modal.create', { moduleName: $t('page.system.role.module') })
    : $t('ui.modal.update', { moduleName: $t('page.system.role.module') }),
);

// ===== 默认首页（方案 5.3-M4）：候选来自当前用户可见菜单路由，值为路由 path =====
const homePathOptions = ref<Array<{ label: string; value: string }>>([]);
let homePathOptionsLoaded = false;
async function loadHomePathOptions() {
  if (homePathOptionsLoaded) return;
  homePathOptionsLoaded = true;
  try {
    const res: any = await getMenusRouterApi({});
    const list = Array.isArray(res) ? res : res?.data || [];
    const options: Array<{ label: string; value: string }> = [];
    const seen = new Set<string>();
    const walk = (nodes: any[], prefix: string) => {
      for (const n of nodes || []) {
        const title = n?.meta?.title || n?.name || '';
        const path = String(n?.path || '');
        const label = prefix ? `${prefix} / ${title}` : title;
        if (path.startsWith('/') && !seen.has(path)) {
          seen.add(path);
          options.push({ label: `${label}（${path}）`, value: path });
        }
        if (Array.isArray(n?.children) && n.children.length > 0) {
          walk(n.children, label);
        }
      }
    };
    walk(list, '');
    homePathOptions.value = options;
  } catch {
    homePathOptions.value = [];
    homePathOptionsLoaded = false;
  }
}
// 超级管理员角色首页由系统固定，后端忽略该字段（前端同步禁用）
const isSuperAdminRole = computed(() => String(data.value?.row?.id ?? '') === '1');

const [BaseForm, baseFormApi] = useVbenForm({
  showDefaultActions: false,
  commonConfig: {
    componentProps: {
      class: 'w-full',
    },
  },
  schema: [
    {
      component: 'Input',
      fieldName: 'roleName',
      label: $t('page.system.role.name'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
      rules: z.string().min(1, { message: $t('ui.formRules.required') }),
    },
    {
      component: 'Input',
      fieldName: 'roleKey',
      label: $t('page.system.role.code'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
      rules: 'required',
    },
    {
      component: 'InputNumber',
      fieldName: 'sort',
      label: $t('ui.table.sortId'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'homePath',
      label: $t('page.system.role.homePath'),
      componentProps: () => ({
        placeholder: $t('page.system.role.homePathPlaceholder'),
        allowClear: true,
        showSearch: true,
        optionFilterProp: 'label',
        options: homePathOptions.value,
        disabled: isSuperAdminRole.value,
      }),
    },
    {
      component: 'Input',
      fieldName: 'remark',
      label: $t('ui.table.remark'),
      componentProps: {
        type: 'textarea',
        autosize: true,
        rows: 5,
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'RadioGroup',
      fieldName: 'status',
      defaultValue: 1,
      label: $t('ui.table.status'),
      rules: 'selectRequired',
      componentProps: {
        optionType: 'button',
        class: 'flex flex-wrap',
        options: statusList,
      },
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
      return;
    }

    setLoading(true);

    const values = await baseFormApi.getValues();

    try {
      await (data.value?.create
        ? createRoleApi(values)
        : updateRoleApi(data.value.row.id, values));

      message.success(
        data.value?.create
          ? $t('ui.notification.create_success')
          : $t('ui.notification.update_success'),
      );
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
      baseFormApi.setValues(data.value?.row);
      loadHomePathOptions();
      setLoading(false);
    }
  },
});

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}
</script>

<template>
  <Drawer :title="getTitle">
    <BaseForm />
  </Drawer>
</template>
