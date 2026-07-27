<script lang="ts" setup>
import { computed, ref } from 'vue';
import { useVbenDrawer, z } from '@vben/common-ui';
import { useVbenForm } from '#/adapter/form';
import { message, Tabs, Upload, Button, Input, Modal, Skeleton, Empty, Tag } from 'ant-design-vue';
import type { UploadFile } from 'ant-design-vue';
import { LucideMaximize2, LucideMinimize2 } from '@vben/icons';
import { siteApi, templateApi } from '#/api';
import type { SiteVO } from '#/api/core/website/site';
import { uploadFileApi } from '#/api/core/attachment/file';

const data = ref();
const isCreate = computed(() => data.value?.create);
const getTitle = computed(() => (isCreate.value ? '新增网站' : '网站设置'));
const activeTab = ref('basic');

// --- 图片上传公共方法 ---
async function handleImageUpload(file: File, field: string, setValue: (val: string) => void, setList: (list: UploadFile[]) => void) {
  try {
    const res: any = await uploadFileApi(file, 'website');
    const url = res?.data?.url || res?.url;
    if (url) {
      setValue(url);
      setList([{ uid: '-1', name: field, status: 'done', url }]);
      message.success('上传成功');
    }
    return false;
  } catch {
    message.error('上传失败');
    return false;
  }
}

// Logo 上传
const logoList = ref<UploadFile[]>([]);
const logoUrl = ref('');
async function handleLogoUpload(file: File) {
  return handleImageUpload(file, 'logo',
    (v) => { logoUrl.value = v; },
    (l) => { logoList.value = l; }
  );
}
function handleLogoRemove() {
  logoList.value = [];
  logoUrl.value = '';
}

// 水印图片上传
const watermarkImageList = ref<UploadFile[]>([]);
const watermarkImageUrl = ref('');
async function handleWatermarkImageUpload(file: File) {
  return handleImageUpload(file, 'watermark',
    (v) => { watermarkImageUrl.value = v; },
    (l) => { watermarkImageList.value = l; }
  );
}
function handleWatermarkImageRemove() {
  watermarkImageList.value = [];
  watermarkImageUrl.value = '';
}

// 微信二维码上传
const wechatQrcodeList = ref<UploadFile[]>([]);
const wechatQrcodeUrl = ref('');
async function handleWechatQrcodeUpload(file: File) {
  return handleImageUpload(file, 'wechat',
    (v) => { wechatQrcodeUrl.value = v; },
    (l) => { wechatQrcodeList.value = l; }
  );
}
function handleWechatQrcodeRemove() {
  wechatQrcodeList.value = [];
  wechatQrcodeUrl.value = '';
}

// 分享图片上传
const shareImageList = ref<UploadFile[]>([]);
const shareImageUrl = ref('');
async function handleShareImageUpload(file: File) {
  return handleImageUpload(file, 'share',
    (v) => { shareImageUrl.value = v; },
    (l) => { shareImageList.value = l; }
  );
}
function handleShareImageRemove() {
  shareImageList.value = [];
  shareImageUrl.value = '';
}

// 模板选择弹窗
const templateModalVisible = ref(false);
const templateList = ref<any[]>([]);
const templateLoading = ref(false);
const templateSearch = ref('');
const selectedTemplateName = ref('');
const currentTemplateDetail = ref<any>(null);
const previewModalVisible = ref(false);
const previewUrl = ref('');

async function openTemplateModal() {
  templateModalVisible.value = true;
  templateLoading.value = true;
  try {
    const res: any = await templateApi.list({ page: 1, pageSize: 100, status: 1, keywords: templateSearch.value });
    templateList.value = res?.items || [];
  } catch {
    templateList.value = [];
  } finally {
    templateLoading.value = false;
  }
}

function selectTemplate(tpl: any) {
  baseFormApi.setValues({ templateId: tpl.id });
  selectedTemplateName.value = tpl.name;
  currentTemplateDetail.value = tpl;
  templateModalVisible.value = false;
}

async function loadTemplateDetail(templateId?: number) {
  if (!templateId) {
    currentTemplateDetail.value = null;
    return;
  }
  try {
    const res: any = await templateApi.detail(templateId);
    currentTemplateDetail.value = res?.data || res;
  } catch {
    currentTemplateDetail.value = null;
  }
}

function previewCurrentTemplate() {
  if (currentTemplateDetail.value?.previewUrl) {
    previewUrl.value = currentTemplateDetail.value.previewUrl;
    previewModalVisible.value = true;
  }
}

// --- 基础表单（Tab切换间共享数据） ---
const [BaseForm, baseFormApi] = useVbenForm({
  showDefaultActions: false,
  commonConfig: {
    componentProps: { class: 'w-full' },
  },
  schema: [
    // --- 基本设置 ---
    { fieldName: 'siteName', label: '网站名称', component: 'Input',
      componentProps: { placeholder: '请输入网站名称', allowClear: true },
      rules: z.string().min(1, { message: '请输入网站名称' }) },
    { fieldName: 'logo', label: '网站Logo', component: 'Input', componentProps: { style: 'display: none' } },
    { fieldName: 'domain', label: '二级域名', component: 'Input',
      componentProps: { placeholder: '如：demo', allowClear: true } },
    { fieldName: 'bindDomain', label: '绑定域名', component: 'Input',
      componentProps: { placeholder: '如：www.example.com', allowClear: true } },
    { fieldName: 'siteType', label: '网站类型', component: 'RadioGroup', defaultValue: 1,
      componentProps: { options: [
        { label: '企业官网', value: 1 },
        { label: '商城', value: 2 },
        { label: '其他', value: 3 },
      ] } },
    { fieldName: 'client', label: '客户端', component: 'RadioGroup', defaultValue: 1,
      componentProps: { options: [
        { label: 'PC', value: 1 },
        { label: 'WAP', value: 2 },
        { label: 'CMS', value: 3 },
      ] } },
    { fieldName: 'showBanner', label: '首页Banner', component: 'Switch', defaultValue: 1 },
    { fieldName: 'status', label: '网站状态', component: 'RadioGroup', defaultValue: 1,
      componentProps: { options: [
        { label: '正常', value: 1 },
        { label: '关闭', value: 0 },
      ] } },
    { fieldName: 'closeReason', label: '关闭原因', component: 'Input',
      componentProps: { type: 'textarea', rows: 2, placeholder: '网站关闭时显示的原因', allowClear: true } },
    { fieldName: 'sort', label: '排序', component: 'InputNumber', defaultValue: 0,
      componentProps: { min: 0, style: 'width: 100%' } },
    { fieldName: 'isDefault', label: '默认站点', component: 'Switch', defaultValue: 0 },

    // --- SEO ---
    { fieldName: 'keywords', label: 'SEO关键词', component: 'Input',
      componentProps: { placeholder: '多个关键词用逗号分隔', allowClear: true } },
    { fieldName: 'description', label: 'SEO描述', component: 'Input',
      componentProps: { type: 'textarea', rows: 3, placeholder: '网站描述，用于搜索引擎收录', allowClear: true } },

    // --- 模板设置 ---
    { fieldName: 'templateId', label: '模板选择', component: 'Input', componentProps: { style: 'display: none' } },

    // --- 上传设置 ---
    { fieldName: 'watermarkEnable', label: '图片水印', component: 'Switch', defaultValue: 0 },
    { fieldName: 'watermarkType', label: '水印类型', component: 'RadioGroup', defaultValue: 1,
      componentProps: { options: [
        { label: '文字水印', value: 1 },
        { label: '图片水印', value: 2 },
      ] } },
    { fieldName: 'watermarkText', label: '水印文字', component: 'Input',
      componentProps: { placeholder: '文字水印内容', allowClear: true } },
    { fieldName: 'watermarkPosition', label: '水印位置', component: 'Select', defaultValue: 9,
      componentProps: { options: [
        { label: '左上', value: 1 }, { label: '上中', value: 2 }, { label: '右上', value: 3 },
        { label: '左中', value: 4 }, { label: '居中', value: 5 }, { label: '右中', value: 6 },
        { label: '左下', value: 7 }, { label: '下中', value: 8 }, { label: '右下', value: 9 },
      ] } },
    { fieldName: 'watermarkOpacity', label: '透明度(%)', component: 'InputNumber', defaultValue: 50,
      componentProps: { min: 0, max: 100, style: 'width: 100%' } },
    { fieldName: 'uploadAllowedTypes', label: '允许上传类型', component: 'Input',
      componentProps: { placeholder: 'jpg,png,gif,pdf,doc 等，逗号分隔', allowClear: true } },
    { fieldName: 'uploadMaxSize', label: '单文件最大(MB)', component: 'InputNumber', defaultValue: 10,
      componentProps: { min: 1, max: 500, style: 'width: 100%' } },

    // --- 公司信息 ---
    { fieldName: 'companyName', label: '公司名称', component: 'Input',
      componentProps: { placeholder: '请输入公司全称', allowClear: true } },
    { fieldName: 'companyPhone', label: '联系电话', component: 'Input',
      componentProps: { placeholder: '请输入联系电话', allowClear: true } },
    { fieldName: 'companyEmail', label: '联系邮箱', component: 'Input',
      componentProps: { placeholder: '请输入联系邮箱', allowClear: true } },
    { fieldName: 'companyAddress', label: '公司地址', component: 'Input',
      componentProps: { placeholder: '请输入公司地址', allowClear: true } },
    { fieldName: 'workDays', label: '工作日', component: 'Input',
      componentProps: { placeholder: '如：周一至周五', allowClear: true } },
    { fieldName: 'workTimeStart', label: '上班时间', component: 'TimePicker',
      componentProps: { format: 'HH:mm', allowClear: true, style: 'width: 100%' } },
    { fieldName: 'workTimeEnd', label: '下班时间', component: 'TimePicker',
      componentProps: { format: 'HH:mm', allowClear: true, style: 'width: 100%' } },
    { fieldName: 'qq', label: '客服QQ', component: 'Input',
      componentProps: { placeholder: '请输入QQ号', allowClear: true } },
    { fieldName: 'wechat', label: '微信号', component: 'Input',
      componentProps: { placeholder: '请输入微信号', allowClear: true } },
    { fieldName: 'wechatQrcode', label: '微信二维码', component: 'Input', componentProps: { style: 'display: none' } },
    { fieldName: 'icp', label: '备案号', component: 'Input',
      componentProps: { placeholder: '如：京ICP备12345678号', allowClear: true } },
    { fieldName: 'copyright', label: '版权信息', component: 'Input',
      componentProps: { placeholder: '如：Copyright © 2024 xxx', allowClear: true } },

    // --- 分享设置 ---
    { fieldName: 'shareTitle', label: '分享标题', component: 'Input',
      componentProps: { placeholder: '微信分享时显示的标题', allowClear: true } },
    { fieldName: 'shareDesc', label: '分享描述', component: 'Input',
      componentProps: { type: 'textarea', rows: 2, placeholder: '微信分享时显示的描述', allowClear: true } },
    { fieldName: 'shareImage', label: '分享图片', component: 'Input', componentProps: { style: 'display: none' } },

    // --- 代码设置 ---
    { fieldName: 'statisticsCode', label: '统计代码', component: 'Input',
      componentProps: { type: 'textarea', rows: 4, placeholder: '百度统计/Google Analytics 等代码', allowClear: true } },
    { fieldName: 'customCss', label: '自定义CSS', component: 'Input',
      componentProps: { type: 'textarea', rows: 4, placeholder: '自定义 CSS 样式代码', allowClear: true } },
    { fieldName: 'customJs', label: '自定义JS', component: 'Input',
      componentProps: { type: 'textarea', rows: 4, placeholder: '自定义 JavaScript 代码', allowClear: true } },

    { fieldName: 'remark', label: '备注', component: 'Input',
      componentProps: { type: 'textarea', rows: 2, placeholder: '内部备注，不对外展示', allowClear: true } },
  ],
});

// --- 缩放状态 ---
const zoomLevel = ref(1); // 0=60vw, 1=75vw(默认), 2=90vw
const zoomLevels = ['drawer-width-60', 'drawer-width-75', 'drawer-width-90'];

function zoomIn() {
  if (zoomLevel.value < 2) {
    zoomLevel.value++;
    drawerApi.setState({ class: zoomLevels[zoomLevel.value] });
  }
}
function zoomOut() {
  if (zoomLevel.value > 0) {
    zoomLevel.value--;
    drawerApi.setState({ class: zoomLevels[zoomLevel.value] });
  }
}

// --- Drawer ---
const [Drawer, drawerApi] = useVbenDrawer({
  class: 'drawer-width-75',
  drawerStyle: { maxWidth: '100vw', minHeight: '70vh' },
  bodyStyle: { paddingTop: '0', paddingBottom: '64px' },
  onCancel() {
    drawerApi.close();
  },
  async onConfirm() {
    const validate = await baseFormApi.validate();
    if (!validate.valid) {
      // 切到第一个有错误的tab
      activeTab.value = 'basic';
      return;
    }
    setLoading(true);
    const values = await baseFormApi.getValues();
    // 合并上传的图片URL
    values.logo = logoUrl.value;
    values.watermarkImage = watermarkImageUrl.value;
    values.wechatQrcode = wechatQrcodeUrl.value;
    values.shareImage = shareImageUrl.value;
    // 时间转字符串
    if (values.workTimeStart && typeof values.workTimeStart !== 'string') {
      values.workTimeStart = values.workTimeStart?.format('HH:mm');
    }
    if (values.workTimeEnd && typeof values.workTimeEnd !== 'string') {
      values.workTimeEnd = values.workTimeEnd?.format('HH:mm');
    }
    try {
      if (isCreate.value) {
        await siteApi.add(values);
        message.success('新增成功');
      } else {
        await siteApi.update(data.value.row.id, values);
        message.success('保存成功');
      }
      drawerApi.setData({ needRefresh: true });
      drawerApi.close();
    } finally {
      setLoading(false);
    }
  },
  onOpenChange(isOpen: boolean) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      const row = data.value?.row || {};
      // 设置表单值
      baseFormApi.setValues(row);
      // 选中模板名
      selectedTemplateName.value = row.templateName || '';
      // 加载模板详情
      loadTemplateDetail(row.templateId ? Number(row.templateId) : undefined);
      // 图片上传列表
      logoUrl.value = row.logo || '';
      logoList.value = row.logo ? [{ uid: '-1', name: 'logo', status: 'done' as const, url: row.logo }] : [];
      watermarkImageUrl.value = row.watermarkImage || '';
      watermarkImageList.value = row.watermarkImage
        ? [{ uid: '-1', name: 'watermark', status: 'done' as const, url: row.watermarkImage }] : [];
      wechatQrcodeUrl.value = row.wechatQrcode || '';
      wechatQrcodeList.value = row.wechatQrcode
        ? [{ uid: '-1', name: 'wechat', status: 'done' as const, url: row.wechatQrcode }] : [];
      shareImageUrl.value = row.shareImage || '';
      shareImageList.value = row.shareImage
        ? [{ uid: '-1', name: 'share', status: 'done' as const, url: row.shareImage }] : [];
      setLoading(false);
      activeTab.value = 'basic';
    }
  },
});

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}

// Tab 配置
const tabItems = [
  { key: 'basic', label: '基本设置' },
  { key: 'seo', label: 'SEO设置' },
  { key: 'template', label: '模板设置' },
  { key: 'upload', label: '上传设置' },
  { key: 'company', label: '公司信息' },
  { key: 'share', label: '分享设置' },
  { key: 'code', label: '代码设置' },
];
</script>

<template>
  <Drawer :title="getTitle">
    <template #extra>
      <div class="drawer-zoom-actions">
        <Button
          type="text"
          size="small"
          :disabled="zoomLevel === 0"
          @click="zoomOut"
          title="缩小"
        >
          <LucideMinimize2 class="size-4" />
        </Button>
        <Button
          type="text"
          size="small"
          :disabled="zoomLevel === 2"
          @click="zoomIn"
          title="放大"
        >
          <LucideMaximize2 class="size-4" />
        </Button>
      </div>
    </template>
    <div class="site-config-wrap">
      <!-- 顶部信息 -->
      <div v-if="!isCreate" class="site-config-header">
        <div class="site-header-left">
          <img v-if="logoUrl" :src="logoUrl" class="site-logo-preview" />
          <div class="site-header-info">
            <h2 class="site-header-name">{{ (data?.row as SiteVO)?.siteName || '网站设置' }}</h2>
            <div class="site-header-meta">
              <span class="meta-item">域名: {{ (data?.row as SiteVO)?.bindDomain || '—' }}</span>
              <span class="meta-item">
                <a-tag :color="(data?.row as SiteVO)?.status === 1 ? 'success' : 'default'">
                  {{ (data?.row as SiteVO)?.status === 1 ? '正常' : '关闭' }}
                </a-tag>
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Tab 导航 -->
      <div class="site-config-tabs">
        <Tabs v-model:activeKey="activeTab" size="large" :items="tabItems" />
      </div>

      <!-- Tab 内容 -->
      <div class="site-config-content">
        <!-- 基本设置 -->
        <div v-show="activeTab === 'basic'" class="config-tab-panel">
          <div class="config-section">
            <h3 class="section-title">基础信息</h3>
            <div class="form-two-col">
              <div class="form-item">
                <label class="form-label">网站名称 <span class="required">*</span></label>
                <div class="form-control"><BaseForm.Item name="siteName" /></div>
              </div>
              <div class="form-item">
                <label class="form-label">网站类型</label>
                <div class="form-control"><BaseForm.Item name="siteType" /></div>
              </div>
              <div class="form-item">
                <label class="form-label">客户端</label>
                <div class="form-control"><BaseForm.Item name="client" /></div>
              </div>
              <div class="form-item">
                <label class="form-label">排序</label>
                <div class="form-control"><BaseForm.Item name="sort" /></div>
              </div>
            </div>
          </div>

          <div class="config-section">
            <h3 class="section-title">域名设置</h3>
            <div class="form-two-col">
              <div class="form-item">
                <label class="form-label">二级域名</label>
                <div class="form-control"><BaseForm.Item name="domain" /></div>
              </div>
              <div class="form-item">
                <label class="form-label">绑定域名</label>
                <div class="form-control"><BaseForm.Item name="bindDomain" /></div>
              </div>
            </div>
          </div>

          <div class="config-section">
            <h3 class="section-title">网站Logo</h3>
            <div class="upload-section">
              <Upload
                :file-list="logoList"
                :before-upload="(file: File) => { handleLogoUpload(file); return false; }"
                :remove="handleLogoRemove"
                list-type="picture-card"
                accept="image/*"
              >
                <div v-if="logoList.length < 1" class="upload-plus">
                  <div class="upload-icon">+</div>
                  <div class="upload-text">上传Logo</div>
                </div>
              </Upload>
              <p class="upload-tip">建议尺寸 200x60，支持 JPG/PNG/GIF</p>
            </div>
          </div>

          <div class="config-section">
            <h3 class="section-title">状态设置</h3>
            <div class="form-two-col">
              <div class="form-item">
                <label class="form-label">网站状态</label>
                <div class="form-control"><BaseForm.Item name="status" /></div>
              </div>
              <div class="form-item">
                <label class="form-label">首页Banner</label>
                <div class="form-control"><BaseForm.Item name="showBanner" /></div>
              </div>
              <div class="form-item">
                <label class="form-label">设为默认</label>
                <div class="form-control"><BaseForm.Item name="isDefault" /></div>
              </div>
            </div>
            <div class="form-item form-item-full" style="margin-top: 12px">
              <label class="form-label">关闭原因</label>
              <div class="form-control"><BaseForm.Item name="closeReason" /></div>
            </div>
          </div>

          <div class="config-section">
            <h3 class="section-title">备注信息</h3>
            <div class="form-item form-item-full">
              <label class="form-label">内部备注</label>
              <div class="form-control"><BaseForm.Item name="remark" /></div>
            </div>
          </div>
        </div>

        <!-- SEO设置 -->
        <div v-show="activeTab === 'seo'" class="config-tab-panel">
          <div class="config-section">
            <h3 class="section-title">SEO 优化</h3>
            <p class="section-desc">设置网站的 SEO 信息，有助于搜索引擎收录和排名。</p>
            <div class="form-item form-item-full">
              <label class="form-label">SEO关键词</label>
              <div class="form-control"><BaseForm.Item name="keywords" /></div>
              <p class="field-tip">多个关键词之间用英文逗号分隔，建议 5-10 个</p>
            </div>
            <div class="form-item form-item-full" style="margin-top: 12px">
              <label class="form-label">SEO描述</label>
              <div class="form-control"><BaseForm.Item name="description" /></div>
              <p class="field-tip">网站简短描述，建议 50-200 字，将显示在搜索结果中</p>
            </div>
          </div>
        </div>

        <!-- 模板设置 -->
        <div v-show="activeTab === 'template'" class="config-tab-panel">
          <div class="config-section">
            <h3 class="section-title">当前模板</h3>
            <p class="section-desc">当前网站正在使用的模板，点击可更换。</p>

            <div class="current-template-card">
              <div class="tpl-preview">
                <img
                  v-if="currentTemplateDetail?.previewPic"
                  :src="currentTemplateDetail.previewPic"
                  :alt="selectedTemplateName"
                />
                <div v-else class="tpl-preview-placeholder">
                  <span>暂无预览图</span>
                </div>
              </div>
              <div class="tpl-info">
                <h4 class="tpl-name">{{ selectedTemplateName || '尚未选择模板' }}</h4>
                <div v-if="currentTemplateDetail" class="tpl-meta">
                  <div class="meta-row">
                    <span class="meta-label">模板文件夹：</span>
                    <span class="meta-value">{{ currentTemplateDetail.templateFolder || '—' }}</span>
                  </div>
                  <div class="meta-row">
                    <span class="meta-label">支持终端：</span>
                    <div class="meta-tags">
                      <Tag v-if="currentTemplateDetail.terminalPc === 1" color="blue">PC端</Tag>
                      <Tag v-if="currentTemplateDetail.terminalMobile === 1" color="green">手机端</Tag>
                      <Tag v-if="currentTemplateDetail.terminalIpad === 1" color="orange">平板</Tag>
                      <Tag v-if="currentTemplateDetail.terminalDisplay === 1" color="purple">展示机</Tag>
                    </div>
                  </div>
                  <div v-if="currentTemplateDetail.remark" class="meta-row">
                    <span class="meta-label">简介：</span>
                    <span class="meta-value">{{ currentTemplateDetail.remark }}</span>
                  </div>
                </div>
                <div class="tpl-actions">
                  <Button type="primary" @click="openTemplateModal">
                    更换模板
                  </Button>
                  <Button
                    v-if="currentTemplateDetail?.previewUrl"
                    @click="previewCurrentTemplate"
                  >
                    预览演示
                  </Button>
                </div>
              </div>
            </div>
          </div>

          <div class="config-section">
            <h3 class="section-title">模板说明</h3>
            <p class="section-desc">关于模板切换的注意事项。</p>
            <div class="template-tips">
              <div class="tip-item">
                <span class="tip-icon">💡</span>
                <span class="tip-text">更换模板后，网站前台将立即使用新模板展示</span>
              </div>
              <div class="tip-item">
                <span class="tip-icon">📱</span>
                <span class="tip-text">请根据模板支持的终端类型，确保您的内容适配</span>
              </div>
              <div class="tip-item">
                <span class="tip-icon">🔧</span>
                <span class="tip-text">模板内容数据在「模板数据管理」中维护</span>
              </div>
            </div>
          </div>
        </div>

        <!-- 上传设置 -->
        <div v-show="activeTab === 'upload'" class="config-tab-panel">
          <div class="config-section">
            <h3 class="section-title">图片水印</h3>
            <p class="section-desc">上传的图片自动添加水印，可选择文字或图片水印。</p>
            <div class="form-two-col">
              <div class="form-item">
                <label class="form-label">开启水印</label>
                <div class="form-control"><BaseForm.Item name="watermarkEnable" /></div>
              </div>
              <div class="form-item">
                <label class="form-label">水印类型</label>
                <div class="form-control"><BaseForm.Item name="watermarkType" /></div>
              </div>
              <div class="form-item">
                <label class="form-label">水印位置</label>
                <div class="form-control"><BaseForm.Item name="watermarkPosition" /></div>
              </div>
              <div class="form-item">
                <label class="form-label">透明度(%)</label>
                <div class="form-control"><BaseForm.Item name="watermarkOpacity" /></div>
              </div>
            </div>

            <div class="form-item form-item-full" style="margin-top: 12px">
              <label class="form-label">水印文字</label>
              <div class="form-control"><BaseForm.Item name="watermarkText" /></div>
              <p class="field-tip">文字水印时必填</p>
            </div>

            <div class="form-item form-item-full" style="margin-top: 12px">
              <label class="form-label">水印图片</label>
              <div class="upload-section">
                <Upload
                  :file-list="watermarkImageList"
                  :before-upload="(file: File) => { handleWatermarkImageUpload(file); return false; }"
                  :remove="handleWatermarkImageRemove"
                  list-type="picture-card"
                  accept="image/png"
                >
                  <div v-if="watermarkImageList.length < 1" class="upload-plus">
                    <div class="upload-icon">+</div>
                    <div class="upload-text">上传水印</div>
                  </div>
                </Upload>
                <p class="upload-tip">图片水印时使用，建议使用透明 PNG 格式</p>
              </div>
            </div>
          </div>

          <div class="config-section">
            <h3 class="section-title">上传限制</h3>
            <div class="form-two-col">
              <div class="form-item">
                <label class="form-label">单文件最大(MB)</label>
                <div class="form-control"><BaseForm.Item name="uploadMaxSize" /></div>
              </div>
            </div>
            <div class="form-item form-item-full" style="margin-top: 12px">
              <label class="form-label">允许上传文件类型</label>
              <div class="form-control"><BaseForm.Item name="uploadAllowedTypes" /></div>
              <p class="field-tip">多个扩展名用英文逗号分隔，如：jpg,png,gif,pdf,doc,xls,zip</p>
            </div>
          </div>
        </div>

        <!-- 公司信息 -->
        <div v-show="activeTab === 'company'" class="config-tab-panel">
          <div class="config-section">
            <h3 class="section-title">基础信息</h3>
            <div class="form-two-col">
              <div class="form-item">
                <label class="form-label">公司名称</label>
                <div class="form-control"><BaseForm.Item name="companyName" /></div>
              </div>
              <div class="form-item">
                <label class="form-label">联系电话</label>
                <div class="form-control"><BaseForm.Item name="companyPhone" /></div>
              </div>
              <div class="form-item">
                <label class="form-label">联系邮箱</label>
                <div class="form-control"><BaseForm.Item name="companyEmail" /></div>
              </div>
            </div>
            <div class="form-item form-item-full" style="margin-top: 12px">
              <label class="form-label">公司地址</label>
              <div class="form-control"><BaseForm.Item name="companyAddress" /></div>
            </div>
          </div>

          <div class="config-section">
            <h3 class="section-title">工作时间</h3>
            <div class="form-two-col">
              <div class="form-item">
                <label class="form-label">工作日</label>
                <div class="form-control"><BaseForm.Item name="workDays" /></div>
              </div>
              <div class="form-item" style="grid-column: span 2">
                <div class="time-range-row">
                  <div class="time-field">
                    <label class="form-label">上班时间</label>
                    <div class="form-control"><BaseForm.Item name="workTimeStart" /></div>
                  </div>
                  <div class="time-separator">—</div>
                  <div class="time-field">
                    <label class="form-label">下班时间</label>
                    <div class="form-control"><BaseForm.Item name="workTimeEnd" /></div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div class="config-section">
            <h3 class="section-title">联系方式</h3>
            <div class="form-two-col">
              <div class="form-item">
                <label class="form-label">客服QQ</label>
                <div class="form-control"><BaseForm.Item name="qq" /></div>
              </div>
              <div class="form-item">
                <label class="form-label">微信号</label>
                <div class="form-control"><BaseForm.Item name="wechat" /></div>
              </div>
            </div>
            <div class="form-item form-item-full" style="margin-top: 12px">
              <label class="form-label">微信二维码</label>
              <div class="upload-section">
                <Upload
                  :file-list="wechatQrcodeList"
                  :before-upload="(file: File) => { handleWechatQrcodeUpload(file); return false; }"
                  :remove="handleWechatQrcodeRemove"
                  list-type="picture-card"
                  accept="image/*"
                >
                  <div v-if="wechatQrcodeList.length < 1" class="upload-plus">
                    <div class="upload-icon">+</div>
                    <div class="upload-text">上传二维码</div>
                  </div>
                </Upload>
                <p class="upload-tip">微信客服二维码，用于前台展示</p>
              </div>
            </div>
          </div>

          <div class="config-section">
            <h3 class="section-title">备案与版权</h3>
            <div class="form-two-col">
              <div class="form-item">
                <label class="form-label">备案号</label>
                <div class="form-control"><BaseForm.Item name="icp" /></div>
              </div>
            </div>
            <div class="form-item form-item-full" style="margin-top: 12px">
              <label class="form-label">版权信息</label>
              <div class="form-control"><BaseForm.Item name="copyright" /></div>
            </div>
          </div>
        </div>

        <!-- 分享设置 -->
        <div v-show="activeTab === 'share'" class="config-tab-panel">
          <div class="config-section">
            <h3 class="section-title">微信分享设置</h3>
            <p class="section-desc">设置网站分享到微信/朋友圈时显示的标题、描述和缩略图。</p>
            <div class="form-item form-item-full">
              <label class="form-label">分享标题</label>
              <div class="form-control"><BaseForm.Item name="shareTitle" /></div>
            </div>
            <div class="form-item form-item-full" style="margin-top: 12px">
              <label class="form-label">分享描述</label>
              <div class="form-control"><BaseForm.Item name="shareDesc" /></div>
            </div>
            <div class="form-item form-item-full" style="margin-top: 12px">
              <label class="form-label">分享图片</label>
              <div class="upload-section">
                <Upload
                  :file-list="shareImageList"
                  :before-upload="(file: File) => { handleShareImageUpload(file); return false; }"
                  :remove="handleShareImageRemove"
                  list-type="picture-card"
                  accept="image/*"
                >
                  <div v-if="shareImageList.length < 1" class="upload-plus">
                    <div class="upload-icon">+</div>
                    <div class="upload-text">上传图片</div>
                  </div>
                </Upload>
                <p class="upload-tip">建议尺寸 500x400，支持 JPG/PNG</p>
              </div>
            </div>
          </div>
        </div>

        <!-- 代码设置 -->
        <div v-show="activeTab === 'code'" class="config-tab-panel">
          <div class="config-section">
            <h3 class="section-title">统计代码</h3>
            <div class="form-item form-item-full">
              <label class="form-label">统计代码</label>
              <div class="form-control"><BaseForm.Item name="statisticsCode" /></div>
              <p class="field-tip">将被插入到网站 &lt;/body&gt; 之前</p>
            </div>
          </div>

          <div class="config-section">
            <h3 class="section-title">自定义代码</h3>
            <p class="section-desc">高级用户可添加自定义 CSS 和 JavaScript 来定制网站样式。</p>
            <div class="form-item form-item-full">
              <label class="form-label">自定义CSS</label>
              <div class="form-control code-editor"><BaseForm.Item name="customCss" /></div>
            </div>
            <div class="form-item form-item-full" style="margin-top: 12px">
              <label class="form-label">自定义JS</label>
              <div class="form-control code-editor"><BaseForm.Item name="customJs" /></div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 模板预览弹窗 -->
    <Modal
      v-model:open="previewModalVisible"
      title="模板演示预览"
      width="90%"
      :footer="null"
      destroy-on-close
      class="template-preview-modal"
    >
      <div class="preview-iframe-wrap">
        <iframe
          v-if="previewUrl"
          :src="previewUrl"
          class="preview-iframe"
          frameborder="0"
        />
      </div>
    </Modal>

    <!-- 模板选择弹窗 -->
    <Modal
      v-model:open="templateModalVisible"
      title="选择模板"
      width="80%"
      :footer="null"
      destroy-on-close
      class="template-select-modal"
    >
      <div class="template-modal-search">
        <Input
          v-model:value="templateSearch"
          placeholder="搜索模板名称..."
          allow-clear
          style="width: 240px"
          @press-enter="openTemplateModal"
        />
      </div>
      <div class="template-modal-grid">
        <div v-if="templateLoading" class="template-grid-loading">
          <a-skeleton active :paragraph="{ rows: 3 }" />
        </div>
        <template v-else>
          <div
            v-for="tpl in templateList"
            :key="tpl.id"
            class="tpl-card"
            :class="{ active: String(data?.row?.templateId) === String(tpl.id) }"
            @click="selectTemplate(tpl)"
          >
            <div class="tpl-card-img">
              <img
                :src="tpl.previewPic || 'https://via.placeholder.com/400x300/f0f0f0/bfbfbf?text=No+Preview'"
                :alt="tpl.name"
              />
            </div>
            <div class="tpl-card-name">{{ tpl.name }}</div>
          </div>
          <div v-if="templateList.length === 0" class="template-grid-empty">
            <Empty description="暂无可用模板" />
          </div>
        </template>
      </div>
    </Modal>
  </Drawer>
</template>

<style scoped>
/* ========== 布局 ========== */
.site-config-wrap {
  height: 100%;
  display: flex;
  flex-direction: column;
}

/* ========== 顶部信息 ========== */
.site-config-header {
  padding: 20px 24px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 8px;
  margin: 16px 24px 0;
  color: #fff;
}
.site-header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}
.site-logo-preview {
  width: 48px;
  height: 48px;
  object-fit: contain;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 8px;
  padding: 4px;
}
.site-header-name {
  margin: 0 0 4px;
  font-size: 20px;
  font-weight: 600;
}
.site-header-meta {
  display: flex;
  align-items: center;
  gap: 16px;
  font-size: 13px;
  opacity: 0.9;
}

/* ========== Tab 导航 ========== */
.site-config-tabs {
  padding: 0 24px;
  border-bottom: 1px solid #f0f0f0;
  margin-top: 8px;
}

/* ========== 内容区 ========== */
.site-config-content {
  flex: 1;
  padding: 20px 24px 0;
  overflow-y: auto;
}
.config-tab-panel {
  max-width: 900px;
}
.config-section {
  background: #fff;
  border: 1px solid #f0f0f0;
  border-radius: 8px;
  padding: 20px 24px;
  margin-bottom: 16px;
}
.section-title {
  margin: 0 0 4px;
  font-size: 16px;
  font-weight: 600;
  color: rgba(0, 0, 0, 0.88);
}
.section-desc {
  margin: 0 0 16px;
  font-size: 13px;
  color: rgba(0, 0, 0, 0.45);
}

/* ========== 表单 ========== */
.form-two-col {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px 24px;
}
.form-item-full {
  width: 100%;
}
.form-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.form-label {
  font-size: 14px;
  font-weight: 500;
  color: rgba(0, 0, 0, 0.88);
}
.form-label .required {
  color: #ff4d4f;
  margin-left: 2px;
}
.form-control {
  width: 100%;
}
.field-tip {
  margin: 4px 0 0;
  font-size: 12px;
  color: rgba(0, 0, 0, 0.45);
}

/* 时间范围 */
.time-range-row {
  display: flex;
  align-items: flex-end;
  gap: 12px;
}
.time-field {
  flex: 1;
}
.time-separator {
  padding-bottom: 4px;
  color: #999;
}

/* 代码编辑 */
.code-editor :deep(.ant-input) {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.6;
}

/* ========== 上传 ========== */
.upload-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.upload-plus {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #8c8c8c;
}
.upload-icon {
  font-size: 22px;
  line-height: 1;
  margin-bottom: 2px;
}
.upload-text {
  font-size: 12px;
}
.upload-tip {
  margin: 4px 0 0;
  font-size: 12px;
  color: rgba(0, 0, 0, 0.45);
}

/* ========== 当前模板卡片 ========== */
.current-template-card {
  display: flex;
  gap: 20px;
  padding: 20px;
  background: #fafafa;
  border: 1px solid #e8e8e8;
  border-radius: 8px;
}
.tpl-preview {
  width: 240px;
  flex-shrink: 0;
  aspect-ratio: 4 / 3;
  border-radius: 6px;
  overflow: hidden;
  background: #fff;
  border: 1px solid #e8e8e8;
}
.tpl-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.tpl-preview-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #bfbfbf;
  font-size: 14px;
}
.tpl-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
}
.tpl-name {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: rgba(0, 0, 0, 0.88);
}
.tpl-meta {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.meta-row {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  font-size: 13px;
}
.meta-label {
  color: rgba(0, 0, 0, 0.45);
  flex-shrink: 0;
}
.meta-value {
  color: rgba(0, 0, 0, 0.75);
  word-break: break-all;
}
.meta-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.tpl-actions {
  display: flex;
  gap: 10px;
  margin-top: auto;
}

/* 模板说明 */
.template-tips {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.tip-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  background: #f6f8ff;
  border-radius: 6px;
  font-size: 13px;
  color: rgba(0, 0, 0, 0.75);
}
.tip-icon {
  font-size: 16px;
}

/* 预览iframe */
.preview-iframe-wrap {
  min-height: 600px;
}
.preview-iframe {
  width: 100%;
  height: 75vh;
  border: none;
  background: #fff;
}

/* 模板弹窗 */
.template-modal-search {
  margin-bottom: 16px;
  display: flex;
  justify-content: flex-end;
}
.template-modal-grid {
  min-height: 300px;
}
.tpl-card {
  display: inline-block;
  width: calc(25% - 12px);
  margin: 0 12px 12px 0;
  border: 2px solid transparent;
  border-radius: 8px;
  overflow: hidden;
  cursor: pointer;
  transition: all 0.2s ease;
  vertical-align: top;
  background: #fff;
}
.tpl-card:hover {
  border-color: #91caff;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}
.tpl-card.active {
  border-color: #1677ff;
  box-shadow: 0 0 0 2px rgba(22, 119, 255, 0.1);
}
.tpl-card-img {
  aspect-ratio: 4 / 3;
  overflow: hidden;
  background: #f5f5f5;
}
.tpl-card-img img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.tpl-card-name {
  padding: 8px 10px;
  font-size: 13px;
  font-weight: 500;
  text-align: center;
  border-top: 1px solid #f0f0f0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.template-grid-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 200px;
}

/* ========== 响应式 ========== */
@media (max-width: 1200px) {
  .form-two-col {
    grid-template-columns: 1fr;
  }
  .tpl-card {
    width: calc(33.333% - 12px);
  }
}
@media (max-width: 768px) {
  .tpl-card {
    width: calc(50% - 12px);
  }
}
</style>

<style>
/* Drawer 宽度缩放（非 scoped，覆盖 VbenDrawer 默认 w-130） */
.drawer-width-60 {
  width: 60vw !important;
  max-width: 100vw !important;
}
.drawer-width-75 {
  width: 75vw !important;
  max-width: 100vw !important;
}
.drawer-width-90 {
  width: 90vw !important;
  max-width: 100vw !important;
}
</style>
