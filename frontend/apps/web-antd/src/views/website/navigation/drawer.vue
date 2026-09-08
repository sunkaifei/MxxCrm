<script lang="ts" setup>
import { computed, onMounted, reactive, ref } from 'vue';

import { useVbenDrawer } from '@vben/common-ui';

import {
  Alert,
  AutoComplete,
  Checkbox,
  Divider,
  Form,
  Input,
  InputNumber,
  message,
  Radio,
  Select,
  Tag,
  TreeSelect,
} from 'ant-design-vue';

import { navigationApi } from '#/api';
import { articleApi } from '#/api/core/website/article';
import { categoryApi } from '#/api/core/website/category';
import { getProductLibraryListApi } from '#/api/core/website/product';
import { getPageListApi } from '#/api/core/website/website-content';

defineOptions({ name: 'WebsiteNavigationDrawer' });

const data = ref<any>();
const isCreate = computed(() => !!data.value?.create);
const getTitle = computed(() => (isCreate.value ? '新增导航' : '修改导航'));

const formRef = ref();
const loading = ref(false);

// ── 选项定义 ──
const navTypeOptions = [
  { label: '顶部导航 (header)', value: 'header' },
  { label: '底部导航 (footer)', value: 'footer' },
  { label: '侧边栏 (sidebar)', value: 'sidebar' },
  { label: '移动端 (mobile)', value: 'mobile' },
  { label: '顶栏 (topbar)', value: 'topbar' },
];

// 数据类型（来源/模型）：custom / 内容模型 / 单页 / 外链
const dataTypeOptions = [
  { label: '自定义导航（外链）', value: 'custom' },
  { label: '文章分类', value: 'article_class' },
  { label: '产品分类', value: 'product_class' },
  { label: '自定义页面', value: 'customview' },
  { label: '文章', value: 'article' },
  { label: '产品', value: 'product' },
  { label: '分组标题（下拉父项）', value: 'link_group' },
];

const targetOptions = [
  { label: '当前窗口 (_self)', value: '_self' },
  { label: '新窗口 (_blank)', value: '_blank' },
];

const relOptions = [
  { label: '无', value: '' },
  { label: 'nofollow', value: 'nofollow' },
  { label: 'noopener', value: 'noopener' },
  { label: 'nofollow noopener', value: 'nofollow noopener' },
];

// 常用图标（FontAwesome / Bootstrap-Icons 类名，前台 <i class="..."> 渲染）
const iconOptions = [
  { value: 'bi bi-house' },
  { value: 'bi bi-shop' },
  { value: 'bi bi-info-circle' },
  { value: 'bi bi-newspaper' },
  { value: 'bi bi-headset' },
  { value: 'bi bi-telephone' },
  { value: 'bi bi-geo-alt' },
  { value: 'bi bi-cart' },
  { value: 'bi bi-person' },
  { value: 'bi bi-envelope' },
  { value: 'bi bi-star' },
  { value: 'bi bi-search' },
];

// ── 表单状态 ──
const form = reactive<any>({
  id: undefined,
  name: '',
  navType: 'header',
  dataType: 'custom',
  value: undefined,
  webUrl: '',
  parentId: undefined as number | undefined,
  icon: '',
  target: '_self',
  rel: '',
  sort: 0,
  isShow: 1,
  visibleGuest: 1,
});
const devices = ref<string[]>([]);

const isReference = computed(
  () => form.dataType !== 'custom' && form.dataType !== 'link_group',
);

// ── 数据源 ──
const rawCategoryTree = ref<any[]>([]);
const pageOptions = ref<any[]>([]);
const articleOptions = ref<any[]>([]);
const productOptions = ref<any[]>([]);
const allNavs = ref<any[]>([]);
let sourcesLoaded = false;

// 后端 /category/tree 返回 CategoryTreeVO：字段为 categoryName / contentType（camelCase），
// 并非 name。此处统一映射成 TreeSelect 需要的 { title, value, key, children }。
function mapCatTree(nodes: any[]): any[] {
  return (nodes || []).map((n) => ({
    title: n.categoryName ?? n.category_name ?? n.name ?? '(未命名分类)',
    value: n.id,
    key: n.id,
    contentType: n.contentType,
    children: n.children ? mapCatTree(n.children) : undefined,
  }));
}

// 关联分类树：按数据类型过滤（文章分类=content_type 1，产品分类=content_type 2）
const categoryTree = computed(() => {
  const ct =
    form.dataType === 'product_class'
      ? 2
      : form.dataType === 'article_class'
        ? 1
        : null;
  if (ct === null) return rawCategoryTree.value;
  const keep = (nodes: any[]): any[] =>
    nodes
      .filter((n) => n.contentType === null || n.contentType === undefined || n.contentType === ct)
      .map((n) => ({ ...n, children: n.children ? keep(n.children) : undefined }));
  return keep(rawCategoryTree.value);
});

/** 收集某节点的全部后代 id */
function collectDescendants(items: any[], rootId: number): Set<number> {
  const result = new Set<number>();
  if (!rootId) return result;
  let frontier = [rootId];
  // 最多 10 层，防脏数据成环
  for (let depth = 0; depth < 10 && frontier.length > 0; depth++) {
    const next: number[] = [];
    for (const it of items) {
      if (frontier.includes(it.parentId)) {
        if (!result.has(it.id)) {
          result.add(it.id);
          next.push(it.id);
        }
      }
    }
    frontier = next;
  }
  return result;
}

/** 扁平导航 → TreeSelect 树（仅同一 navType，排除自身及后代） */
const parentTree = computed(() => {
  const selfId = isCreate.value ? 0 : (data.value?.row?.id ?? 0);
  const navType = form.navType || 'header';
  const descendants = collectDescendants(allNavs.value, selfId);
  const items = allNavs.value.filter(
    (n) =>
      (n.navType || 'header') === navType &&
      n.id !== selfId &&
      !descendants.has(n.id),
  );
  const byId = new Map<number, any>();
  items.forEach((n) => byId.set(n.id, { title: n.name, value: n.id, key: n.id, children: [] }));
  const roots: any[] = [];
  items.forEach((n) => {
    const node = byId.get(n.id);
    const pid = n.parentId || 0;
    if (pid && byId.has(pid)) byId.get(pid)!.children.push(node);
    else roots.push(node);
  });
  const prune = (nodes: any[]) =>
    nodes.forEach((nd) => {
      if (nd.children.length === 0) delete nd.children;
      else prune(nd.children);
    });
  prune(roots);
  return roots;
});

async function loadSources() {
  if (sourcesLoaded) return;
  sourcesLoaded = true;
  const [cats, pages, arts, prods, navs] = await Promise.allSettled([
    categoryApi.tree(),
    getPageListApi({ page: 1, pageSize: 200 }),
    articleApi.list({ page: 1, pageSize: 200, status: 1 }),
    getProductLibraryListApi({ page: 1, pageSize: 200 }),
    navigationApi.list({ page: 1, pageSize: 200 }),
  ]);
  if (cats.status === 'fulfilled') rawCategoryTree.value = mapCatTree(cats.value as any[]);
  if (pages.status === 'fulfilled') {
    pageOptions.value = ((pages.value as any)?.items || []).map((p: any) => ({
      label: `${p.pageName || p.pageCode} (${p.pageCode})`,
      value: p.id,
    }));
  }
  if (arts.status === 'fulfilled') {
    articleOptions.value = ((arts.value as any)?.items || []).map((a: any) => ({
      label: a.title,
      value: Number(a.id),
    }));
  }
  if (prods.status === 'fulfilled') {
    productOptions.value = ((prods.value as any)?.items || []).map((p: any) => ({
      label: p.name || p.productName,
      value: Number(p.id),
    }));
  }
  if (navs.status === 'fulfilled') {
    allNavs.value = (navs.value as any)?.items || [];
  }
}

// dataType 变化：清空关联对象值
function onDataTypeChange() {
  form.value = undefined;
  if (form.dataType === 'custom') {
    // custom 时保留/可编辑外链
  } else {
    form.webUrl = '';
  }
}

// ── 预览 ──
const previewUrl = computed(() => {
  if (form.dataType === 'custom') return form.webUrl || '#';
  if (!form.value) return '（未选择关联对象）';
  switch (form.dataType) {
    case 'article_class':
    case 'product_class': {
      return '提交后由后端解析为 /category/{短链} 或 /product?category_id={id}';
    }
    case 'customview': {
      return '提交后由后端解析为 /page/{页面编码}';
    }
    case 'article': {
      return '提交后由后端解析为 /article/{短链}';
    }
    case 'product': {
      return '提交后由后端解析为 /product/{id}';
    }
    default: {
      return '#';
    }
  }
});

const [Drawer, drawerApi] = useVbenDrawer({
  class: 'w-[60%] max-w-[100vw]',
  onCancel() {
    drawerApi.close();
  },

  async onConfirm() {
    try {
      await formRef.value?.validate();
    } catch {
      return;
    }
    loading.value = true;
    drawerApi.setState({ loading: true });
    try {
      const payload: any = {
        id: form.id,
        name: form.name,
        navType: form.navType,
        dataType: form.dataType,
        value: form.value ?? undefined,
        parentId: form.parentId ?? 0,
        icon: form.icon || undefined,
        target: form.target || '_self',
        isNewWindowOpen: form.target === '_blank' ? 1 : 0,
        rel: form.rel || undefined,
        sort: form.sort ?? 0,
        isShow: form.isShow,
        visibleGuest: form.visibleGuest,
        visibleDevices: devices.value.length > 0 ? devices.value.join(',') : undefined,
        // 仅 custom 允许手填外链；引用式由后端解析
        webUrl: form.dataType === 'custom' ? form.webUrl : undefined,
      };
      // 分组标题无链接
      if (form.dataType === 'link_group') {
        payload.webUrl = undefined;
        payload.value = undefined;
      }
      if (isCreate.value) {
        await navigationApi.add(payload);
        message.success('新增成功');
      } else {
        await navigationApi.update(data.value.row.id, payload);
        message.success('修改成功');
      }
      drawerApi.setData({ needRefresh: true });
      drawerApi.close();
    } finally {
      loading.value = false;
      drawerApi.setState({ loading: false });
    }
  },

  async onOpenChange(isOpen) {
    if (!isOpen) return;
    data.value = drawerApi.getData<Record<string, any>>();
    const row = data.value?.row || {};
    Object.assign(form, {
      id: isCreate.value ? undefined : row.id,
      name: row.name || '',
      navType: row.navType || 'header',
      dataType: row.dataType || 'custom',
      value: row.value ?? undefined,
      webUrl: row.webUrl || '',
      parentId: row.parentId && row.parentId > 0 ? row.parentId : undefined,
      icon: row.icon || '',
      target: row.target || (row.isNewWindowOpen === 1 ? '_blank' : '_self'),
      rel: row.rel || '',
      sort: row.sort ?? 0,
      isShow: row.isShow ?? 1,
      visibleGuest: row.visibleGuest ?? 1,
    });
    devices.value = (row.visibleDevices || '')
      .split(',')
      .map((s: string) => s.trim())
      .filter((s: string) => s.length > 0);
    loading.value = false;
    drawerApi.setState({ loading: false });
    loadSources();
  },
});

onMounted(() => {
  // 预热（抽屉未打开也无妨）
});
</script>

<template>
  <Drawer :title="getTitle">
    <Form ref="formRef" :model="form" :label-col="{ span: 5 }" :wrapper-col="{ span: 18 }">
      <Form.Item
        label="导航名称"
        name="name"
        :rules="[{ required: true, message: '请输入导航名称' }]"
      >
        <Input v-model:value="form.name" placeholder="请输入导航名称" allow-clear />
      </Form.Item>

      <Form.Item label="导航位置" name="navType">
        <Select v-model:value="form.navType" :options="navTypeOptions" placeholder="请选择导航位置" />
      </Form.Item>

      <Form.Item label="数据类型" name="dataType">
        <Select
          v-model:value="form.dataType"
          :options="dataTypeOptions"
          placeholder="请选择数据类型/来源"
          @change="onDataTypeChange"
        />
      </Form.Item>

      <!-- 关联对象：随 dataType 联动 -->
      <Form.Item v-if="['article_class', 'product_class'].includes(form.dataType)" label="关联分类">
        <TreeSelect
          v-model:value="form.value"
          :tree-data="categoryTree"
          :dropdown-style="{ maxHeight: '320px', overflow: 'auto' }"
          tree-default-expand-all
          placeholder="请选择分类"
          allow-clear
          style="width: 100%"
        />
      </Form.Item>
      <Form.Item v-else-if="form.dataType === 'customview'" label="关联页面">
        <Select
          v-model:value="form.value"
          :options="pageOptions"
          placeholder="请选择自定义页面"
          show-search
          option-filter-prop="label"
          allow-clear
          style="width: 100%"
        />
      </Form.Item>
      <Form.Item v-else-if="form.dataType === 'article'" label="关联文章">
        <Select
          v-model:value="form.value"
          :options="articleOptions"
          placeholder="请选择文章"
          show-search
          option-filter-prop="label"
          allow-clear
          style="width: 100%"
        />
      </Form.Item>
      <Form.Item v-else-if="form.dataType === 'product'" label="关联产品">
        <Select
          v-model:value="form.value"
          :options="productOptions"
          placeholder="请选择产品"
          show-search
          option-filter-prop="label"
          allow-clear
          style="width: 100%"
        />
      </Form.Item>

      <Form.Item label="链接地址">
        <Input
          v-model:value="form.webUrl"
          :disabled="isReference || form.dataType === 'link_group'"
          :placeholder="isReference ? '引用式由系统自动解析，无需手填' : '请输入链接地址（含 http://）'"
          allow-clear
        />
        <div v-if="isReference" class="text-xs text-gray-400 mt-1">{{ previewUrl }}</div>
      </Form.Item>

      <Form.Item label="上级导航" name="parentId">
        <TreeSelect
          v-model:value="form.parentId"
          :tree-data="parentTree"
          :dropdown-style="{ maxHeight: '320px', overflow: 'auto' }"
          placeholder="不选则为顶级导航（仅同一位置内可选）"
          allow-clear
          style="width: 100%"
        />
      </Form.Item>

      <Form.Item label="图标" name="icon">
        <AutoComplete
          v-model:value="form.icon"
          :options="iconOptions"
          placeholder="输入图标类名（如 bi bi-house）或从下拉选择"
          allow-clear
          style="width: 100%"
        />
      </Form.Item>

      <Form.Item label="打开方式" name="target">
        <Select v-model:value="form.target" :options="targetOptions" />
      </Form.Item>

      <Form.Item label="rel 属性" name="rel">
        <Select
          v-model:value="form.rel"
          :options="relOptions"
          placeholder="SEO 链接关系（可留空）"
          allow-clear
        />
      </Form.Item>

      <Form.Item label="可见性" name="visibleGuest">
        <Radio.Group v-model:value="form.visibleGuest">
          <Radio :value="1">所有访客可见</Radio>
          <Radio :value="0">仅登录用户可见</Radio>
        </Radio.Group>
      </Form.Item>

      <Form.Item label="可见设备">
        <Checkbox.Group v-model:value="devices">
          <Checkbox value="pc">PC</Checkbox>
          <Checkbox value="mobile">移动端</Checkbox>
        </Checkbox.Group>
        <span class="text-xs text-gray-400 ml-2">都不勾选＝全部设备可见</span>
      </Form.Item>

      <Form.Item label="排序" name="sort">
        <InputNumber v-model:value="form.sort" :min="0" style="width: 100%" />
      </Form.Item>

      <Form.Item label="是否显示" name="isShow">
        <Radio.Group v-model:value="form.isShow">
          <Radio :value="1">显示</Radio>
          <Radio :value="0">隐藏</Radio>
        </Radio.Group>
      </Form.Item>

      <Divider orientation="left" plain>导航项预览</Divider>
      <Alert type="info" show-icon :banner="false">
        <template #message>
          <span>
            <Tag color="blue">{{ navTypeOptions.find((o) => o.value === form.navType)?.label || form.navType }}</Tag>
            <span v-if="form.icon" class="mr-1"><i :class="form.icon"></i></span>
            <span class="font-medium">{{ form.name || '（未命名）' }}</span>
            <span class="text-gray-400 ml-2">→ {{ previewUrl }}</span>
          </span>
        </template>
      </Alert>
    </Form>
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
