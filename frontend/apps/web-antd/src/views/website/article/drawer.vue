<script lang="ts" setup>
import type { UploadFile } from 'ant-design-vue';

import {
  computed,
  defineAsyncComponent,
  nextTick,
  onMounted,
  reactive,
  ref,
} from 'vue';

import { useVbenDrawer } from '@vben/common-ui';
import { LucideUpload } from '@vben/icons';

import {
  Col,
  Collapse,
  CollapsePanel,
  DatePicker,
  Form,
  FormItem,
  Input,
  message,
  Row,
  Select,
  Switch,
  Textarea,
  TreeSelect,
  Upload,
} from 'ant-design-vue';

import { useAccessStore } from '@vben/stores';

import { articleApi, categoryApi, getCategoryListApi } from '#/api';
import { articleTagApi } from '#/api/core/website/article-tag';
import { useAssetDomain } from '#/composables/use-asset-domain';
import { toPublicFileUrl } from '#/utils/asset-url';

const AInput = Input;
const ATextarea = Textarea;
const ASelect = Select;
const ATreeSelect = TreeSelect;
const ADatePicker = DatePicker;
const ASwitch = Switch;
const AForm = Form;
const AFormItem = FormItem;
const ARow = Row;
const ACol = Col;
const AUpload = Upload;
const ACollapse = Collapse;
const ACollapsePanel = CollapsePanel;

const data = ref<any>();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() => (isCreate.value ? '新增文章' : '编辑文章'));

/** 文章栏目树（/api/system/category/tree 原始数据） */
const rawCategoryTree = ref<any[]>([]);
/** 产品模块分类树（/api/system/product/category/list，表 mxx_product_category） */
const productCategoryTree = ref<any[]>([]);
const tagOptions = ref<any[]>([]);

/**
 * 文章状态：与后端 status 对齐。
 * 0=未审核（原「待审核」，保持历史数据语义）/ 1=已审核（原「已通过」）/ 2=已拒绝（原「已驳回」）/ 3=草稿
 */
const statusOptions = [
  { label: '未审核', value: 0 },
  { label: '已审核', value: 1 },
  { label: '已拒绝', value: 2 },
  { label: '草稿', value: 3 },
];

const formRef = ref();
/** 正文区切换：富文本编辑器 / HTML 源码 */
const contentSourceMode = ref(false);
/** 富文本编辑器延迟挂载：抽屉打开后再初始化，确保回显内容正确 */
const editorReady = ref(false);
/** SEO 折叠面板展开项，默认折叠 */
const seoActive = ref<string[]>([]);

const RichTextEditor = defineAsyncComponent(
  () => import('#/components/RichTextEditor/index.vue'),
);

/** 表单模型：字段与后端 ArticleSaveDTO 对齐 */
const formState = reactive<Record<string, any>>({
  categoryId: undefined,
  title: '',
  shortTitle: '',
  author: '',
  shortUrl: '',
  labelIds: [],
  titleImage: '',
  description: '',
  content: '',
  istop: 0,
  isrecommend: 0,
  status: 0,
  publishTime: undefined,
  seoTitle: '',
  seoKeywords: '',
  seoDescription: '',
});

const rules: Record<string, any> = {
  categoryId: [
    {
      required: true,
      message: '请选择所属分类',
      trigger: 'change',
      type: 'number',
    },
  ],
  title: [{ required: true, message: '请输入文章标题', trigger: 'blur' }],
};

// --- 主图上传（附件URL统一方案 v1.1：公开图 is_public=1） ---
const accessStore = useAccessStore();
const { assetDomain } = useAssetDomain();
const fileList = ref<UploadFile[]>([]);
const uploading = ref(false);
/** 本次上传的附件 id（仅用于上传后立即回显走 /api/open/file/{id}，不落库） */
const uploadedCoverId = ref('');

/** 封面预览地址（上传缩略图优先，回退到表单里的 URL 统一解析） */
const coverPreview = computed(() => {
  const first = fileList.value[0];
  if (first?.url) return first.url as string;
  return toPublicFileUrl(formState.titleImage, assetDomain.value);
});

function syncFileList(url: string) {
  fileList.value = url
    ? [{ uid: '-1', name: 'cover', status: 'done', url }]
    : [];
}

/**
 * 上传主图到附件系统，返回 { id, url }（id 仅用于本次回显，不落库）。
 * 说明：/api/system/attachment/upload 返回 JSON（JsonResp{code,msg,data}），
 * 此处沿用原生 fetch 直读 JSON（与 uploadCategoryImageApi 同款处理）。
 */
async function uploadCover(file: File): Promise<{ id: string; url: string }> {
  const formData = new FormData();
  formData.append('file', file);
  formData.append('entity_type', 'common');
  formData.append('is_public', '1');

  const resp = await fetch('/api/system/attachment/upload', {
    body: formData,
    headers: {
      Authorization: accessStore.accessToken
        ? `Bearer ${accessStore.accessToken}`
        : '',
    },
    method: 'POST',
  });

  if (!resp.ok) {
    throw new Error(`上传失败：${resp.status} ${resp.statusText}`);
  }

  const json: any = await resp.json();
  if (json?.code !== 200) {
    throw new Error(json?.msg || '上传失败');
  }
  const data = json?.data;
  // data 兼容「字符串 URL」与「对象 {id,url}」两种形态
  if (typeof data === 'string') return { id: '', url: data };
  return {
    id: data?.id ? String(data.id) : '',
    url: data?.url || data?.fileUrl || '',
  };
}

async function handleUpload(file: File) {
  uploading.value = true;
  try {
    const { id, url } = await uploadCover(file);
    if (!url) {
      message.error('上传返回异常：未获取到图片地址');
      return;
    }
    // 数据库永远存相对路径（附件URL统一方案 §8.4），展示层才解析回显地址
    formState.titleImage = url;
    uploadedCoverId.value = id;
    syncFileList(toPublicFileUrl({ id, url }, assetDomain.value));
    message.success('上传成功');
  } catch (error: any) {
    message.error(error?.message || '上传失败，请检查网络或服务器配置');
  } finally {
    uploading.value = false;
  }
}

/** 封面上传前置钩子：拦截默认行为，走自定义上传（before-upload 返回 false） */
function beforeCoverUpload(file: File) {
  handleUpload(file);
  return false;
}

function handleRemove() {
  fileList.value = [];
  formState.titleImage = '';
  uploadedCoverId.value = '';
}

function resetForm() {
  formState.categoryId = undefined;
  formState.title = '';
  formState.shortTitle = '';
  formState.author = '';
  formState.shortUrl = '';
  formState.labelIds = [];
  formState.titleImage = '';
  formState.description = '';
  formState.content = '';
  formState.istop = 0;
  formState.isrecommend = 0;
  formState.status = 0;
  formState.publishTime = undefined;
  formState.seoTitle = '';
  formState.seoKeywords = '';
  formState.seoDescription = '';
  fileList.value = [];
  uploadedCoverId.value = '';
  contentSourceMode.value = false;
  formRef.value?.clearValidate?.();
}

const [Drawer, drawerApi] = useVbenDrawer({
  class: 'w-[75%]! max-w-[75%]!',
  onCancel() {
    drawerApi.close();
  },

  async onConfirm() {
    try {
      await formRef.value.validate();
    } catch {
      return;
    }

    setLoading(true);

    const values: Record<string, any> = { ...formState };
    // labelIds 转数字数组
    if (Array.isArray(values.labelIds)) {
      values.labelIds = values.labelIds.map(Number);
    }

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
    if (!isOpen) return;
    data.value = drawerApi.getData<Record<string, any>>();
    resetForm();
    // 先卸载编辑器，避免残留上一次的 HTML 内容
    editorReady.value = false;

    const row = data.value?.row || {};
    if (!isCreate.value && row) {
      formState.categoryId = row.categoryId;
      formState.title = row.title ?? '';
      formState.shortTitle = row.shortTitle ?? '';
      formState.author = row.author ?? '';
      formState.shortUrl = row.shortUrl ?? '';
      formState.titleImage = row.titleImage ?? '';
      formState.description = row.description ?? '';
      formState.content = row.content ?? '';
      formState.istop = row.istop ?? 0;
      formState.isrecommend = row.isrecommend ?? 0;
      formState.status = row.status ?? 0;
      formState.publishTime = row.publishTime || undefined;
      formState.seoTitle = row.seoTitle ?? '';
      formState.seoKeywords = row.seoKeywords ?? '';
      formState.seoDescription = row.seoDescription ?? '';
      uploadedCoverId.value = '';
      // 编辑回显统一走 toPublicFileUrl：存量 /upload/common/... 相对路径自动走 /by-path 反查
      syncFileList(toPublicFileUrl(formState.titleImage, assetDomain.value));

      // 编辑模式：加载已有标签 ID
      if (row.id) {
        articleApi
          .getLabels(row.id)
          .then((res: any) => {
            formState.labelIds = Array.isArray(res) ? res.map(Number) : [];
          })
          .catch(() => {
            formState.labelIds = [];
          });
      }
    }

    // 表单值就绪后再挂载富文本编辑器，保证回显内容正确
    nextTick(() => {
      editorReady.value = true;
    });
    setLoading(false);
  },
});

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}

/**
 * 映射文章栏目树（后端 CategoryTreeVO）。
 * 注意：名称字段是 categoryName（不是 name），id 经 u64_to_string 是字符串，
 * 而文章的 categoryId 是数字 → 必须 Number 归一，否则名称空白 + 编辑不回显。
 */
function mapCatNodes(nodes: any[]): any[] {
  return (nodes || []).map((n) => ({
    title: n.categoryName ?? n.name ?? '未命名分类',
    value: Number(n.id),
    key: Number(n.id),
    contentType: Number(n.contentType ?? n.content_type ?? 1),
    children:
      Array.isArray(n.children) && n.children.length > 0
        ? mapCatNodes(n.children)
        : undefined,
  }));
}

/** 平铺列表按 parentId 构树（产品模块分类接口返回平铺数据） */
function buildFlatTree(list: any[], parentId = 0): any[] {
  return (list || [])
    .filter((n) => Number(n.parentId ?? n.parent_id ?? 0) === parentId)
    .map((n) => {
      const id = Number(n.id);
      const kids = buildFlatTree(list, id);
      return {
        title: n.name ?? n.categoryName ?? '未命名分类',
        value: id,
        key: id,
        children: kids.length > 0 ? kids : undefined,
      };
    });
}

/** 顶层分组节点：仅作标题，不可选中 */
function groupNode(title: string, key: string, children: any[]): any {
  return { title, value: key, key, selectable: false, children };
}

/**
 * 所属分类下拉树：按来源分组，覆盖各内容模型的分类。
 * 文章栏目(contentType=1) / 产品栏目(2) / 外链栏目(3) / 产品模块分类(mxx_product_category)
 */
const categoryTree = computed<any[]>(() => {
  const groups: any[] = [];
  const pick = (type: number) =>
    rawCategoryTree.value.filter((n) => n.contentType === type);

  const article = pick(1);
  const product = pick(2);
  const link = pick(3);

  if (article.length > 0)
    groups.push(groupNode('文章栏目', 'g-article', article));
  if (product.length > 0)
    groups.push(groupNode('产品栏目', 'g-product', product));
  if (link.length > 0) groups.push(groupNode('外链栏目', 'g-link', link));
  if (productCategoryTree.value.length > 0)
    groups.push(
      groupNode('产品分类（产品模块）', 'g-pcat', productCategoryTree.value),
    );

  return groups;
});

async function loadCategoryTree() {
  try {
    const result: any = await categoryApi.tree();
    rawCategoryTree.value = mapCatNodes(Array.isArray(result) ? result : []);
  } catch {
    rawCategoryTree.value = [];
  }
}

/** 加载产品模块分类（独立表 mxx_product_category，id 与文章分类不冲突） */
async function loadProductCategories() {
  try {
    const res: any = await getCategoryListApi({ page: 1, pageSize: 200 });
    const list = Array.isArray(res)
      ? res
      : (res?.items ?? res?.list ?? res?.rows ?? res?.data ?? []);
    productCategoryTree.value = buildFlatTree(Array.isArray(list) ? list : []);
  } catch {
    productCategoryTree.value = [];
  }
}

async function loadTags() {
  try {
    const result: any = await articleTagApi.all();
    tagOptions.value = (Array.isArray(result) ? result : []).map(
      (tag: any) => ({
        label: tag.name,
        value: tag.id,
      }),
    );
  } catch {
    tagOptions.value = [];
  }
}

onMounted(() => {
  loadCategoryTree();
  loadProductCategories();
  loadTags();
});
</script>

<template>
  <Drawer :title="getTitle">
    <AForm
      ref="formRef"
      :model="formState"
      :rules="rules"
      layout="vertical"
      class="article-form"
    >
      <div class="article-layout">
        <!-- 主栏：标题 + 正文 -->
        <div class="article-main">
          <!-- 文档式大标题 -->
          <AFormItem name="title" class="title-field">
            <AInput
              v-model:value="formState.title"
              class="article-title-input"
              placeholder="请输入文章标题"
              :maxlength="60"
            />
            <template #extra>
              <span class="title-count">{{ formState.title.length }}/60</span>
            </template>
          </AFormItem>

          <!-- 正文内容 -->
          <div class="content-block">
            <div class="content-head">
              <span class="content-label">正文内容</span>
              <a
                class="content-toggle"
                @click="contentSourceMode = !contentSourceMode"
              >
                {{ contentSourceMode ? '返回富文本' : '编辑 HTML 源码' }}
              </a>
            </div>

            <!-- 富文本编辑器（项目内置 wangEditor 封装） -->
            <div v-if="!contentSourceMode" class="editor-wrapper">
              <RichTextEditor
                v-if="editorReady"
                v-model="formState.content"
                placeholder="请输入文章正文..."
                :height="420"
              />
              <div v-else class="editor-loading">
                <span class="editor-loading-spinner"></span>
                编辑器加载中…
              </div>
            </div>

            <!-- HTML 源码模式 -->
            <div v-else class="source-mode">
              <ATextarea
                v-model:value="formState.content"
                placeholder="可直接粘贴或修改 HTML 源码，切回富文本即生效"
                :rows="16"
                class="content-editor"
              />
              <div class="source-tip">源码模式适用于批量粘贴带样式的外部内容</div>
            </div>
          </div>

          <!-- SEO 高级设置（默认折叠，放正文下方平衡右侧高度） -->
          <ACollapse
            v-model:active-key="seoActive"
            ghost
            class="seo-collapse"
          >
            <ACollapsePanel key="seo">
              <template #header>
                <span class="seo-header">SEO 优化设置</span>
              </template>
              <AFormItem label="SEO 标题">
                <AInput
                  v-model:value="formState.seoTitle"
                  placeholder="留空则使用文章标题"
                  allow-clear
                  :maxlength="60"
                  show-count
                />
              </AFormItem>
              <AFormItem label="SEO 关键词">
                <AInput
                  v-model:value="formState.seoKeywords"
                  placeholder="多个关键词用英文逗号分隔"
                  allow-clear
                />
              </AFormItem>
              <AFormItem label="SEO 描述">
                <ATextarea
                  v-model:value="formState.seoDescription"
                  placeholder="用于搜索引擎收录，建议 80-200 字"
                  allow-clear
                  :rows="3"
                  :maxlength="200"
                  show-count
                />
              </AFormItem>
            </ACollapsePanel>
          </ACollapse>
        </div>

        <!-- 侧栏：属性设置 -->
        <div class="article-side">
          <!-- 发布设置 -->
          <section class="side-card">
            <h3 class="side-card-title">发布设置</h3>
            <AFormItem label="审核状态" name="status">
              <ASelect
                v-model:value="formState.status"
                :options="statusOptions"
                placeholder="请选择状态"
                class="w-full"
              />
            </AFormItem>
            <AFormItem label="定时发布">
              <ADatePicker
                v-model:value="formState.publishTime"
                placeholder="留空则按状态立即发布"
                show-time
                value-format="YYYY-MM-DD HH:mm:ss"
                allow-clear
                class="w-full"
              />
            </AFormItem>
            <div class="switch-row">
              <span class="switch-label">置顶展示</span>
              <ASwitch
                v-model:checked="formState.istop"
                :checked-value="1"
                :un-checked-value="0"
              />
            </div>
            <div class="switch-row">
              <span class="switch-label">首页推荐</span>
              <ASwitch
                v-model:checked="formState.isrecommend"
                :checked-value="1"
                :un-checked-value="0"
              />
            </div>
          </section>

          <!-- 归类 -->
          <section class="side-card">
            <h3 class="side-card-title">归类</h3>
            <AFormItem label="所属分类" name="categoryId">
              <ATreeSelect
                v-model:value="formState.categoryId"
                :tree-data="categoryTree"
                placeholder="请选择分类"
                tree-default-expand-all
                allow-clear
                show-search
                tree-node-filter-prop="title"
                class="w-full"
              />
            </AFormItem>
            <AFormItem label="文章标签">
              <ASelect
                v-model:value="formState.labelIds"
                mode="multiple"
                :options="tagOptions"
                placeholder="选择标签，可多选"
                allow-clear
                :max-tag-count="4"
                class="w-full"
              />
            </AFormItem>
            <ARow :gutter="12">
              <ACol :span="12">
                <AFormItem label="短标题">
                  <AInput
                    v-model:value="formState.shortTitle"
                    placeholder="移动端展示"
                    allow-clear
                    :maxlength="30"
                  />
                </AFormItem>
              </ACol>
              <ACol :span="12">
                <AFormItem label="作者">
                  <AInput
                    v-model:value="formState.author"
                    placeholder="请输入作者"
                    allow-clear
                  />
                </AFormItem>
              </ACol>
            </ARow>
          </section>

          <!-- 封面与摘要 -->
          <section class="side-card">
            <h3 class="side-card-title">封面与摘要</h3>
            <AFormItem label="主图封面">
              <div class="cover-uploader">
                <div class="cover-drop" :class="{ 'is-empty': !coverPreview }">
                  <AUpload
                    v-model:file-list="fileList"
                    :before-upload="beforeCoverUpload"
                    list-type="picture-card"
                    accept="image/*"
                    :disabled="uploading"
                    :show-upload-list="false"
                    class="cover-upload"
                  >
                    <div
                      v-if="!coverPreview"
                      class="upload-placeholder"
                      :class="{ uploading }"
                    >
                      <LucideUpload class="upload-icon" />
                      <div class="upload-text">
                        {{ uploading ? '上传中…' : '点击上传主图' }}
                      </div>
                      <div class="upload-hint">建议 800×450（16:9）</div>
                    </div>
                  </AUpload>
                  <div v-if="coverPreview" class="cover-preview">
                    <img :src="coverPreview" alt="封面预览" />
                    <div class="cover-mask">
                      <span class="cover-mask-btn" @click="handleRemove">
                        移除图片
                      </span>
                    </div>
                  </div>
                </div>
                <div class="cover-url">
                  <AInput
                    v-model:value="formState.titleImage"
                    placeholder="或粘贴图片地址 URL"
                    allow-clear
                    @blur="
                      syncFileList(
                        toPublicFileUrl(formState.titleImage, assetDomain),
                      )
                    "
                  />
                </div>
              </div>
            </AFormItem>
            <AFormItem label="文章摘要">
              <ATextarea
                v-model:value="formState.description"
                placeholder="列表页与搜索引擎展示，建议 80-200 字"
                allow-clear
                :rows="3"
                :maxlength="200"
                show-count
              />
            </AFormItem>
          </section>

          <!-- 展示选项 -->
          <section class="side-card">
            <h3 class="side-card-title">展示选项</h3>
            <AFormItem label="访问路径（短链接）">
              <AInput
                v-model:value="formState.shortUrl"
                placeholder="如 about-us"
                allow-clear
              />
            </AFormItem>
          </section>
        </div>
      </div>
    </AForm>
  </Drawer>
</template>

<style scoped>
.article-form {
  padding-bottom: 8px;
}

/* ===== 双栏布局：主栏（标题+正文）+ 侧栏（属性面板） ===== */
.article-layout {
  display: flex;
  align-items: flex-start;
  gap: 16px;
}

.article-main {
  flex: 1;
  min-width: 0;
}

.article-side {
  display: flex;
  flex: 0 0 300px;
  flex-direction: column;
  gap: 12px;
}

@media (max-width: 1100px) {
  .article-layout {
    flex-direction: column;
  }

  .article-side {
    flex-basis: auto;
    width: 100%;
  }
}

/* ===== 文档式大标题 ===== */
.title-field {
  margin-bottom: 20px;
}

.article-title-input :deep(.ant-input) {
  padding: 10px 14px;
  font-size: 20px;
  font-weight: 600;
  line-height: 1.4;
  color: hsl(var(--foreground));
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 10px;
  transition:
    border-color 0.25s ease,
    box-shadow 0.25s ease,
    background 0.25s ease;
}

.article-title-input :deep(.ant-input:hover) {
  border-color: hsl(var(--primary) / 45%);
}

.article-title-input :deep(.ant-input:focus),
.article-title-input :deep(.ant-input-focused) {
  border-color: hsl(var(--primary));
  box-shadow: 0 0 0 3px hsl(var(--primary) / 14%);
}

.article-title-input :deep(.ant-input::placeholder) {
  font-weight: 400;
  color: hsl(var(--muted-foreground) / 70%);
}

.title-count {
  float: right;
  margin-top: 2px;
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

/* ===== 正文区 ===== */
.content-block {
  overflow: hidden;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 10px;
  transition: border-color 0.25s ease;
}

.content-block:focus-within {
  border-color: hsl(var(--primary) / 45%);
}

.content-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid hsl(var(--border));
}

.content-label {
  font-size: 13px;
  font-weight: 600;
  color: hsl(var(--foreground));
}

.content-toggle {
  font-size: 12px;
  color: hsl(var(--primary));
  cursor: pointer;
}

.content-toggle:hover {
  text-decoration: underline;
}

.editor-wrapper {
  min-height: 420px;
}

.editor-wrapper :deep(.rich-editor-container) {
  border: none !important;
  border-radius: 0 !important;
}

.editor-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  height: 420px;
  font-size: 13px;
  color: hsl(var(--muted-foreground));
  background: hsl(var(--muted) / 30%);
}

.editor-loading-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid hsl(var(--border));
  border-top-color: hsl(var(--primary));
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.source-mode {
  padding: 12px;
}

.content-editor {
  font-family: SFMono-Regular, Consolas, 'Liberation Mono', Menlo, monospace;
  font-size: 12px;
  line-height: 1.7;
  resize: vertical;
}

.source-tip {
  margin-top: 8px;
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

/* ===== 侧栏卡片 ===== */
.side-card {
  padding: 14px 16px;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 10px;
  transition:
    border-color 0.25s ease,
    box-shadow 0.25s ease;
}

.side-card:hover {
  border-color: hsl(var(--border) / 90%);
  box-shadow: 0 2px 8px hsl(var(--foreground) / 4%);
}

.side-card-title {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0 0 12px;
  font-size: 13px;
  font-weight: 600;
  color: hsl(var(--foreground));
}

.side-card-title::before {
  width: 3px;
  height: 13px;
  content: '';
  background: hsl(var(--primary));
  border-radius: 2px;
}

.side-card :deep(.ant-form-item) {
  margin-bottom: 12px;
}

.side-card :deep(.ant-form-item:last-child) {
  margin-bottom: 0;
}

.switch-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
  margin-bottom: 4px;
  border-bottom: 1px dashed hsl(var(--border) / 60%);
}

.switch-row:last-child {
  border-bottom: none;
}

.switch-label {
  font-size: 13px;
  color: hsl(var(--foreground) / 85%);
}

/* ===== 封面上传 ===== */
.cover-uploader {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.cover-drop {
  position: relative;
  overflow: hidden;
  border: 1px dashed hsl(var(--border));
  border-radius: 10px;
  transition: border-color 0.25s ease;
}

.cover-drop.is-empty:hover {
  border-color: hsl(var(--primary) / 55%);
}

.cover-upload {
  display: block;
}

.cover-upload :deep(.ant-upload) {
  width: 100%;
  height: 140px;
  margin: 0;
  padding: 0;
  border: none;
  background: hsl(var(--muted) / 35%);
  border-radius: 0;
}

.cover-upload :deep(.ant-upload:hover) {
  border-color: transparent;
}

.upload-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  width: 100%;
  height: 100%;
  transition: background 0.25s ease;
}

.upload-placeholder.uploading {
  cursor: wait;
}

.upload-icon {
  width: 28px;
  height: 28px;
  color: hsl(var(--muted-foreground));
  transition: color 0.25s ease;
}

.upload-placeholder:hover .upload-icon {
  color: hsl(var(--primary));
}

.upload-placeholder.uploading .upload-icon {
  animation: pulse 1s ease-in-out infinite;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }

  50% {
    opacity: 0.35;
  }
}

.upload-text {
  margin-top: 4px;
  font-size: 13px;
  font-weight: 500;
  color: hsl(var(--foreground) / 80%);
}

.upload-hint {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
}

.cover-preview {
  position: absolute;
  inset: 0;
  animation: coverIn 0.3s ease both;
}

@keyframes coverIn {
  from {
    opacity: 0;
    transform: scale(0.98);
  }

  to {
    opacity: 1;
    transform: scale(1);
  }
}

.cover-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.cover-mask {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: hsl(var(--foreground) / 55%);
  opacity: 0;
  transition: opacity 0.25s ease;
}

.cover-drop:hover .cover-mask {
  opacity: 1;
}

.cover-mask-btn {
  padding: 6px 16px;
  font-size: 13px;
  color: #fff;
  cursor: pointer;
  background: hsl(var(--foreground) / 70%);
  border-radius: 6px;
  transition: background 0.2s ease;
}

.cover-mask-btn:hover {
  background: hsl(var(--destructive));
}

.cover-url :deep(.ant-input) {
  font-size: 12px;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
}

/* ===== SEO 折叠 ===== */
.seo-collapse {
  overflow: hidden;
  margin-top: 16px;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 10px;
}

.seo-header {
  font-size: 13px;
  font-weight: 600;
  color: hsl(var(--foreground));
}

:deep(.ant-collapse-header) {
  padding-right: 16px !important;
  padding-left: 16px !important;
}

:deep(.ant-collapse-content-box) {
  padding: 0 16px 14px !important;
}

/* 暗色适配（跟随 antd 主题，这里只需保证对比度） */
@media (max-width: 767px) {
  :deep(.ant-drawer-content-wrapper) {
    width: 100% !important;
    max-width: 100vw !important;
  }
}
</style>
