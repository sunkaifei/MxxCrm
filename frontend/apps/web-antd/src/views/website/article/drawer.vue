<script lang="ts" setup>
import { computed, ref, onMounted } from 'vue';
import { useVbenDrawer, z } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import { articleApi, categoryApi } from '#/api';
import { message } from 'ant-design-vue';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() => (isCreate.value ? '新增文章' : '编辑文章'));

const categoryTree = ref<any[]>([]);

const [BaseForm, baseFormApi] = useVbenForm({
  showDefaultActions: false,
  commonConfig: {
    componentProps: {
      class: 'w-full',
    },
  },
  schema: [
    {
      component: 'TreeSelect',
      fieldName: 'categoryId',
      label: '所属分类',
      componentProps: {
        treeData: categoryTree,
        placeholder: '请选择分类',
        treeDefaultExpandAll: true,
      },
      rules: z.number({ required_error: '请选择分类' }),
    },
    {
      component: 'Input',
      fieldName: 'title',
      label: '文章标题',
      componentProps: {
        placeholder: '请输入文章标题',
        allowClear: true,
      },
      rules: z.string().min(1, { message: '请输入文章标题' }),
    },
    {
      component: 'Input',
      fieldName: 'shortTitle',
      label: '短标题',
      componentProps: {
        placeholder: '请输入短标题',
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'titleImage',
      label: '主图地址',
      componentProps: {
        placeholder: '请输入主图URL',
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'author',
      label: '作者',
      componentProps: {
        placeholder: '请输入作者',
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'shortUrl',
      label: '短链接',
      componentProps: {
        placeholder: '请输入短链接',
        allowClear: true,
      },
    },
    {
      component: 'Textarea',
      fieldName: 'description',
      label: '摘要',
      componentProps: {
        placeholder: '请输入文章摘要',
        allowClear: true,
        rows: 3,
      },
    },
    {
      component: 'Textarea',
      fieldName: 'content',
      label: '文章内容',
      componentProps: {
        placeholder: '请输入HTML内容',
        allowClear: true,
        rows: 6,
      },
    },
    {
      component: 'Switch',
      fieldName: 'istop',
      label: '是否置顶',
      defaultValue: 0,
      componentProps: {
        checkedValue: 1,
        unCheckedValue: 0,
      },
    },
    {
      component: 'Switch',
      fieldName: 'isrecommend',
      label: '是否推荐',
      defaultValue: 0,
      componentProps: {
        checkedValue: 1,
        unCheckedValue: 0,
      },
    },
    {
      component: 'Select',
      fieldName: 'status',
      label: '状态',
      defaultValue: 0,
      componentProps: {
        options: [
          { label: '待审核', value: 0 },
          { label: '已通过', value: 1 },
          { label: '已驳回', value: 2 },
        ],
        placeholder: '请选择状态',
      },
    },
    {
      component: 'DatePicker',
      fieldName: 'publishTime',
      label: '定时发布',
      componentProps: {
        placeholder: '到达该时间后自动发布',
        showTime: true,
        format: 'YYYY-MM-DD HH:mm:ss',
        allowClear: true,
        style: 'width: 100%',
      },
    },
    {
      component: 'Input',
      fieldName: 'seoTitle',
      label: 'SEO标题',
      componentProps: {
        placeholder: '留空则使用文章标题',
        allowClear: true,
      },
    },
    {
      component: 'Input',
      fieldName: 'seoKeywords',
      label: 'SEO关键词',
      componentProps: {
        placeholder: '多个关键词用英文逗号分隔',
        allowClear: true,
      },
    },
    {
      component: 'Textarea',
      fieldName: 'seoDescription',
      label: 'SEO描述',
      componentProps: {
        placeholder: '用于搜索引擎收录，建议 80-200 字',
        allowClear: true,
        rows: 3,
      },
    },
  ],
});

const [Drawer, drawerApi] = useVbenDrawer({
  width: '80%',
  drawerStyle: {
    maxWidth: '100vw',
  },
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
      if (isCreate.value) {
        await articleApi.save(values);
        message.success('新增成功');
      } else {
        await articleApi.update(data.value.row.id, values);
        message.success('修改成功');
      }
      drawerApi.setData({ needRefresh: true });
      drawerApi.close();
    } finally {
      setLoading(false);
    }
  },

  onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      baseFormApi.setValues(data.value?.row || {});
      setLoading(false);
    }
  },
});

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}

async function loadCategoryTree() {
  const result = await categoryApi.tree();
  const mapTree = (nodes: any[]): any[] =>
    nodes.map((node) => ({
      title: node.name,
      value: node.id,
      key: node.id,
      children: node.children ? mapTree(node.children) : undefined,
    }));
  categoryTree.value = mapTree(result);
}

onMounted(() => {
  loadCategoryTree();
});
</script>

<template>
  <Drawer :title="getTitle">
    <BaseForm />
  </Drawer>
</template>

<style>
@media (max-width: 767px) {
  .vben-drawer .ant-drawer-content-wrapper {
    width: 100% !important;
    max-width: 100vw !important;
  }
}
</style>