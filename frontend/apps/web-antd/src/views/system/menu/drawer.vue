<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer } from '@vben/common-ui';
import { $t } from '#/locales';
import { useVbenForm } from '#/adapter/form';
import { message } from 'ant-design-vue';
import {
  createMenuApi,
  updateMenuApi,
  getMenuTreeApi,
} from '#/api';
import { MenuType, statusList } from '#/store';

const isMenu = (type: string) => type === MenuType.MENU;
const isButton = (type: string) => type === MenuType.BUTTON;

const menuTypeList = computed(() => [
  { value: MenuType.FOLDER, label: $t('enum.menuType.folder') },
  { value: MenuType.MENU, label: $t('enum.menuType.menu') },
  { value: MenuType.BUTTON, label: $t('enum.menuType.button') },
]);

const data = ref();

const getTitle = computed(() =>
  data.value?.create
    ? $t('ui.modal.create', { moduleName: $t('page.system.menu.module') })
    : $t('ui.modal.update', { moduleName: $t('page.system.menu.module') }),
);

const [BaseForm, baseFormApi] = useVbenForm({
  showDefaultActions: false,
  commonConfig: {
    componentProps: {
      class: 'w-full',
    },
  },
  schema: [
    {
      component: 'RadioGroup',
      fieldName: 'type',
      label: $t('page.system.menu.type'),
      componentProps: {
        optionType: 'button',
        class: 'flex flex-wrap',
        options: menuTypeList,
      },
      defaultValue: MenuType.FOLDER,
      rules: 'selectRequired',
    },
    {
      component: 'Input',
      fieldName: 'meta.name',
      label: $t('page.system.menu.name'),
      rules: 'required',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'routeName',
      label: $t('page.system.menu.routeName'),
      rules: 'required',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'ApiTreeSelect',
      fieldName: 'parentId',
      label: $t('page.system.menu.parentId'),
      componentProps: {
        placeholder: $t('ui.placeholder.select'),
        api: async () => {
          const result = await getMenuTreeApi({});
          const items = result.items ?? (Array.isArray(result) ? result : []);
          return items;
        },
        childrenField: 'children',
        labelField: 'name',
        valueField: 'id',
        afterFetch: (fetchData: any) => {
          const translateAndFilter = (nodes: any[]): any[] =>
            nodes
              .filter((n) => n.type !== 'BUTTON')
              .map((n) => ({
                ...n,
                name: n.meta?.name ? $t(n.meta.name) : n.name,
                children: n.children ? translateAndFilter(n.children) : n.children,
              }));
          const rootNode = {
            id: 0,
            name: $t('page.system.menu.mainDirectory') || '主目录',
            parentId: -1,
            children: translateAndFilter(Array.isArray(fetchData) ? fetchData : []),
          };
          return [rootNode];
        },
      },
    },
    {
      component: 'IconPicker',
      fieldName: 'meta.icon',
      label: $t('page.system.menu.icon'),
      componentProps: {
        prefix: 'lucide',
      },
      dependencies: {
        show: (values: any) => !isButton(values.type),
        triggerFields: ['type'],
      },
    },
    {
      component: 'Input',
      fieldName: 'perm',
      label: $t('page.system.menu.perm'),
      rules: 'required',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'path',
      label: $t('page.system.menu.path'),
      rules: 'required',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
      dependencies: {
        show: (values: any) => !isButton(values.type),
        triggerFields: ['type'],
      },
    },
    {
      component: 'Input',
      fieldName: 'component',
      label: $t('page.system.menu.component'),
      defaultValue: 'BasicLayout',
      rules: 'required',
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
      dependencies: {
        show: (values: any) => isMenu(values.type),
        triggerFields: ['type'],
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'meta.sort',
      label: $t('ui.table.sortId'),
      defaultValue: 50,
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
      dependencies: {
        show: (values: any) => !isButton(values.type),
        triggerFields: ['type'],
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
    {
      component: 'Switch',
      fieldName: 'meta.affixTab',
      label: $t('page.system.menu.affixTab'),
      componentProps: {
        activeValue: 1,
        inactiveValue: 0,
        class: 'w-auto',
      },
      dependencies: {
        show: (values: any) => !isButton(values.type),
        triggerFields: ['type'],
      },
    },
    {
      component: 'Switch',
      fieldName: 'meta.hideChildrenInMenu',
      label: $t('page.system.menu.hideChildrenInMenu'),
      componentProps: {
        activeValue: 1,
        inactiveValue: 0,
        class: 'w-auto',
      },
      dependencies: {
        show: (values: any) => !isButton(values.type),
        triggerFields: ['type'],
      },
    },
    {
      component: 'Switch',
      fieldName: 'meta.hideInBreadcrumb',
      label: $t('page.system.menu.hideInBreadcrumb'),
      componentProps: {
        activeValue: 1,
        inactiveValue: 0,
        class: 'w-auto',
      },
      dependencies: {
        show: (values: any) => !isButton(values.type),
        triggerFields: ['type'],
      },
    },
    {
      component: 'Switch',
      fieldName: 'meta.hideInMenu',
      label: $t('page.system.menu.hideInMenu'),
      componentProps: {
        activeValue: 1,
        inactiveValue: 0,
        class: 'w-auto',
      },
      dependencies: {
        show: (values: any) => !isButton(values.type),
        triggerFields: ['type'],
      },
    },
    {
      component: 'Switch',
      fieldName: 'meta.hideInTab',
      label: $t('page.system.menu.hideInTab'),
      componentProps: {
        activeValue: 1,
        inactiveValue: 0,
        class: 'w-auto',
      },
      dependencies: {
        show: (values: any) => !isButton(values.type),
        triggerFields: ['type'],
      },
    },
    {
      component: 'Switch',
      fieldName: 'meta.keepAlive',
      label: $t('page.system.menu.keepAlive'),
      componentProps: {
        activeValue: 1,
        inactiveValue: 0,
        class: 'w-auto',
      },
      dependencies: {
        show: (values: any) => isMenu(values.type),
        triggerFields: ['type'],
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

    if (values.parentId === null || values.parentId === undefined) {
      values.parentId = '0';
    }
    if (values.parentId === 0) {
      values.parentId = '0';
    }

    try {
      await (data.value?.create
        ? createMenuApi(values)
        : updateMenuApi(data.value.row.id, values));

      message.success(
        data.value?.create
          ? $t('ui.notification.create_success')
          : $t('ui.notification.update_success'),
      );
      const onRefresh = data.value?.onRefresh;
      if (onRefresh) onRefresh();
    } finally {
      drawerApi.close();
      setLoading(false);
    }
  },

  onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();

      const row = data.value?.row ? { ...data.value.row } : {};

      if (row.name && !row.meta?.name) {
        row.meta = row.meta || {};
        row.meta.name = row.name;
      }

      if (row.meta?.title && !row.meta?.name) {
        row.meta.name = row.meta.title;
      }

      baseFormApi.setValues(row);

      if (data.value?.parentId !== undefined) {
        baseFormApi.setFieldValue('parentId', data.value.parentId);
      }

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
