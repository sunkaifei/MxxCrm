<script lang="ts" setup>
import { ref } from 'vue';
import { Page } from '@vben/common-ui';
import {
  LucideLoader2,
  LucideBuilding2,
  LucideLayers,
  LucideFileText,
  LucideTrash2,
  LucideCheckCircle,
  LucideMaximize2,
} from '@vben/icons';

import {
  Button,
  Card,
  message,
  Modal,
  Statistic,
  Row,
  Col,
} from 'ant-design-vue';

import { staticGenerateApi } from '#/api/core/website/static-generate';

// --- State ---
const generating = ref<string | null>(null);
const lastResult = ref<{ action: string; message: string } | null>(null);

// --- Methods ---
async function handleGenerateAll() {
  generating.value = 'all';
  lastResult.value = null;
  try {
    const result: any = await staticGenerateApi.generateAll();
    const data = result?.data || result;
    lastResult.value = {
      action: '全站静态化',
      message: `栏目 ${data?.categories || 0} 个，文章 ${data?.articles || 0} 个`,
    };
    message.success('全站静态化完成');
  } catch (e: any) {
    message.error(e?.message || '静态化失败');
  } finally {
    generating.value = null;
  }
}

async function handleGenerateIndex() {
  generating.value = 'index';
  lastResult.value = null;
  try {
    await staticGenerateApi.generateIndex();
    lastResult.value = { action: '首页静态化', message: '首页静态化完成' };
    message.success('首页静态化完成');
  } catch (e: any) {
    message.error(e?.message || '静态化失败');
  } finally {
    generating.value = null;
  }
}

async function handleGenerateCategories() {
  generating.value = 'categories';
  lastResult.value = null;
  try {
    const result: any = await staticGenerateApi.generateCategories();
    const data = result?.data || result;
    lastResult.value = {
      action: '栏目页静态化',
      message: `已生成 ${data?.count || 0} 个栏目页`,
    };
    message.success('栏目页静态化完成');
  } catch (e: any) {
    message.error(e?.message || '静态化失败');
  } finally {
    generating.value = null;
  }
}

async function handleGenerateArticles() {
  generating.value = 'articles';
  lastResult.value = null;
  try {
    const result: any = await staticGenerateApi.generateArticles();
    const data = result?.data || result;
    lastResult.value = {
      action: '文章页静态化',
      message: `已生成 ${data?.count || 0} 篇文章页`,
    };
    message.success('文章页静态化完成');
  } catch (e: any) {
    message.error(e?.message || '静态化失败');
  } finally {
    generating.value = null;
  }
}

function handleClearOutput() {
  Modal.confirm({
    title: '确认清空',
    content: '确定要清空静态化输出目录吗？所有已生成的静态文件将被删除，需要重新生成。',
    okText: '确认清空',
    okType: 'danger',
    cancelText: '取消',
    onOk: async () => {
      try {
        await staticGenerateApi.clearOutput();
        lastResult.value = { action: '清空输出', message: '静态化目录已清空' };
        message.success('静态化目录已清空');
      } catch (e: any) {
        message.error(e?.message || '清空失败');
      }
    },
  });
}
</script>

<template>
  <Page auto-content-height>
    <div class="static-generate-page">
      <!-- 说明 -->
      <Card class="guide-card" :bordered="false">
        <div class="guide-content">
          <h3>静态化生成</h3>
          <p>
            将动态页面渲染为静态 HTML 文件，Nginx 可直接读取静态文件返回，显著降低数据库压力。
            静态文件输出到 <code>static_output/</code> 目录。
          </p>
        </div>
      </Card>

      <!-- 操作按钮组 -->
      <Row :gutter="[16, 16]" class="action-row">
        <Col :span="6">
          <Card class="action-card" hoverable @click="handleGenerateAll">
            <div class="action-inner">
              <component :is="generating === 'all' ? LucideLoader2 : LucideMaximize2"
                class="action-icon all-icon"
                :class="{ spinning: generating === 'all' }" />
              <div class="action-info">
                <Statistic title="全站静态化" :value="''" :formatter="() => ''" />
                <span class="action-desc">生成所有页面（首页 + 栏目 + 文章）</span>
              </div>
            </div>
          </Card>
        </Col>
        <Col :span="6">
          <Card class="action-card" hoverable @click="handleGenerateIndex">
            <div class="action-inner">
              <component :is="generating === 'index' ? LucideLoader2 : LucideBuilding2"
                class="action-icon index-icon"
                :class="{ spinning: generating === 'index' }" />
              <div class="action-info">
                <Statistic title="首页" :value="''" :formatter="() => ''" />
                <span class="action-desc">仅生成首页 static_output/index.html</span>
              </div>
            </div>
          </Card>
        </Col>
        <Col :span="6">
          <Card class="action-card" hoverable @click="handleGenerateCategories">
            <div class="action-inner">
              <component :is="generating === 'categories' ? LucideLoader2 : LucideLayers"
                class="action-icon cat-icon"
                :class="{ spinning: generating === 'categories' }" />
              <div class="action-info">
                <Statistic title="栏目页" :value="''" :formatter="() => ''" />
                <span class="action-desc">生成所有栏目列表页</span>
              </div>
            </div>
          </Card>
        </Col>
        <Col :span="6">
          <Card class="action-card" hoverable @click="handleGenerateArticles">
            <div class="action-inner">
              <component :is="generating === 'articles' ? LucideLoader2 : LucideFileText"
                class="action-icon art-icon"
                :class="{ spinning: generating === 'articles' }" />
              <div class="action-info">
                <Statistic title="文章页" :value="''" :formatter="() => ''" />
                <span class="action-desc">生成所有文章详情页</span>
              </div>
            </div>
          </Card>
        </Col>
      </Row>

      <!-- 危险操作 -->
      <Card class="danger-card" :bordered="false">
        <div class="danger-inner">
          <div class="danger-info">
            <h4>危险操作</h4>
            <p>清空后所有已生成的静态文件将被删除，访问网站将回退到动态渲染模式。</p>
          </div>
          <Button danger type="primary" @click="handleClearOutput">
            <template #icon><component :is="LucideTrash2" /></template>
            清空静态化目录
          </Button>
        </div>
      </Card>

      <!-- 最近结果 -->
      <Card v-if="lastResult" class="result-card" :bordered="false">
        <div class="result-inner">
          <component :is="LucideCheckCircle" class="result-icon" />
          <div class="result-content">
            <strong>{{ lastResult.action }}</strong>：{{ lastResult.message }}
          </div>
        </div>
      </Card>
    </div>
  </Page>
</template>

<style scoped>
.static-generate-page {
  padding: 24px;
  max-width: 1200px;
  margin: 0 auto;
}

.guide-card {
  margin-bottom: 24px;
  border-radius: 8px;
}

.guide-content h3 {
  margin: 0 0 8px;
  font-size: 18px;
  font-weight: 600;
}

.guide-content p {
  margin: 0;
  color: #595959;
  font-size: 14px;
  line-height: 1.6;
}

.guide-content code {
  background: #f5f5f5;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 13px;
}

.action-row {
  margin-bottom: 24px;
}

.action-card {
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.action-card:hover {
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
}

.action-inner {
  display: flex;
  align-items: center;
  gap: 16px;
}

.action-icon {
  font-size: 36px;
  flex-shrink: 0;
}

.action-icon.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.all-icon { color: #1677ff; }
.index-icon { color: #52c41a; }
.cat-icon { color: #faad14; }
.art-icon { color: #722ed1; }

.action-info {
  flex: 1;
}

.action-desc {
  font-size: 12px;
  color: #8c8c8c;
  display: block;
  margin-top: 4px;
}

.danger-card {
  margin-bottom: 24px;
  border-radius: 8px;
  border: 1px solid #ffccc7;
  background: #fff2f0;
}

.danger-inner {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.danger-info h4 {
  margin: 0 0 4px;
  font-size: 14px;
  font-weight: 600;
  color: #cf1322;
}

.danger-info p {
  margin: 0;
  font-size: 13px;
  color: #8c8c8c;
}

.result-card {
  border-radius: 8px;
  border: 1px solid #b7eb8f;
  background: #f6ffed;
}

.result-inner {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 14px;
}

.result-icon {
  font-size: 20px;
  color: #52c41a;
}
</style>