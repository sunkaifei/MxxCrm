<script lang="ts" setup>
import { computed, onMounted, ref } from 'vue';
import { useVbenDrawer } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import { message } from 'ant-design-vue';
import {
  addMediaCategoryApi,
  getMediaCategoryAllApi,
  updateMediaCategoryApi,
} from '#/api';

const data = ref<{ create?: boolean; row?: any; parentId?: number }>({});
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() => (isCreate.value ? '新增分类' : '编辑分类'));

const categoryTreeData = ref<any[]>([]);

const buildTree = (list: any[]): any[] =>
  list.map((item) => ({
    title: item.categoryName,
    value: item.id,
    key: item.id,
    children: item.children?.length ? buildTree(item.children) : undefined,
  }));

async function loadCategoryTree() {
  try {
    const result: any = await getMediaCategoryAllApi();
    const list = Array.isArray(result) ? result : result?.data || [];
    categoryTreeData.value = buildTree(list);
  } catch {
    categoryTreeData.value = [];
  }
}

onMounted(() => {
  loadCategoryTree();
});

const [BaseForm, baseFormApi] = useVbenForm({
  showDefaultActions: false,
  schema: [
    {
      component: 'Input',
      fieldName: 'categoryName',
      label: '分类名称',
      rules: 'required',
      componentProps: {
        placeholder: '请输入分类名称',
        allowClear: true,
      },
    },
    {
      component: 'TreeSelect',
      fieldName: 'parentId',
      label: '上级分类',
      componentProps: {
        treeData: categoryTreeData,
        placeholder: '请选择上级分类（留空为顶级）',
        allowClear: true,
        treeDefaultExpandAll: true,
      },
    },
    {
      component: 'InputNumber',
      fieldName: 'sort',
      label: '排序',
      defaultValue: 0,
      componentProps: {
        min: 0,
        style: 'width: 100%',
      },
    },
  ],
});

const [Drawer, drawerApi] = useVbenDrawer({
  class: 'w-[480px] max-w-[100vw]',
  onCancel() {
    drawerApi.close();
  },
  async onConfirm() {
    const validate = await baseFormApi.validate();
    if (!validate.valid) return;

    setLoading(true);
    try {
      const values = await baseFormApi.getValues();
      if (isCreate.value) {
        await addMediaCategoryApi(values);
        message.success('分类创建成功');
      } else {
        await updateMediaCategoryApi(data.value.row.id, values);
        message.success('分类更新成功');
      }
      drawerApi.setData({ needRefresh: true });
      drawerApi.close();
    } finally {
      setLoading(false);
    }
  },
  async onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      await loadCategoryTree();
      if (!isCreate.value && data.value?.row) {
        baseFormApi.setValues(data.value.row);
      } else {
        baseFormApi.setValues({
          parentId: data.value?.parentId ?? undefined,
          sort: 0,
        });
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