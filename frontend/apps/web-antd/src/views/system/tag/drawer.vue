<script lang="ts" setup>
import { computed, ref } from 'vue';
import { message } from 'ant-design-vue';
import { useVbenDrawer, z } from '@vben/common-ui';
import { $t } from '#/locales';
import { useVbenForm } from '#/adapter/form';
import { createTagApi, updateTagApi } from '#/api';
import { getTagGroupListApi } from '#/api/core/system/tag_group';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() =>
  isCreate.value
    ? $t('ui.modal.create', { moduleName: $t('page.system.tag.module') })
    : $t('ui.modal.update', { moduleName: $t('page.system.tag.module') }),
);

const groupOptions = ref<any[]>([]);

async function loadGroups() {
  const res = await getTagGroupListApi();
  const list = Array.isArray(res) ? res : (res?.data || []);
  groupOptions.value = list.map((item: any) => ({
    label: item.groupName,
    value: item.id,
  }));
}

loadGroups();

const [BaseForm, baseFormApi] = useVbenForm({
  showDefaultActions: false,
  commonConfig: {
    componentProps: {
      class: 'w-full',
    },
  },
  schema: [
    {
      component: 'Select',
      fieldName: 'groupId',
      label: $t('page.system.tag.group'),
      componentProps: {
        options: groupOptions,
        placeholder: $t('ui.placeholder.select'),
      },
      rules: z.string().min(1, { message: $t('ui.formRules.required') }),
    },
    {
      component: 'Input',
      fieldName: 'tagName',
      label: $t('page.system.tag.name'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
      },
      rules: z.string().min(1, { message: $t('ui.formRules.required') }),
    },
    {
      component: 'Input',
      fieldName: 'tagColor',
      label: $t('page.system.tag.color'),
      defaultValue: '#1890ff',
      componentProps: {
        placeholder: '#1890ff',
      },
    },
    {
      component: 'Input',
      fieldName: 'description',
      label: $t('ui.form.description'),
      componentProps: {
        type: 'textarea',
        rows: 3,
        placeholder: $t('ui.placeholder.input'),
      },
    },
    {
      component: 'Switch',
      fieldName: 'isGlobal',
      label: $t('page.system.tag.global'),
      defaultValue: true,
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
        ? createTagApi(values)
        : updateTagApi({ id: data.value.row.id, ...values }));

      message.success(
        data.value?.create
          ? $t('ui.notification.create_success')
          : $t('ui.notification.update_success'),
      );
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
      baseFormApi.setValues(data.value?.row || {});
      loadGroups();
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
