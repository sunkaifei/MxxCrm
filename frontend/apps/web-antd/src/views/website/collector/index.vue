<script lang="ts" setup>
import { ref } from 'vue';
import { Page } from '@vben/common-ui';
import {
  LucideLoader2,
  LucideGlobe,
} from '@vben/icons';

import {
  Button,
  Card,
  message,
  Alert,
} from 'ant-design-vue';

import { collectorApi } from '#/api/core/website/collector';

// --- State ---
const running = ref(false);
const lastResult = ref<{ success: boolean; message: string } | null>(null);

// --- Methods ---
async function handleRun() {
  running.value = true;
  lastResult.value = null;
  try {
    const result: any = await collectorApi.run();
    const data = result?.data || result;
    const count = data?.collected ?? 0;
    lastResult.value = {
      success: true,
      message: `采集完成，本次共采集 ${count} 篇文章`,
    };
    message.success(`采集完成，共 ${count} 篇`);
  } catch (e: any) {
    lastResult.value = {
      success: false,
      message: e?.message || '采集执行失败',
    };
    message.error(e?.message || '采集执行失败');
  } finally {
    running.value = false;
  }
}
</script>

<template>
  <Page auto-content-height>
    <div class="collector-page">
      <!-- 说明 -->
      <Card class="guide-card" :bordered="false">
        <div class="guide-content">
          <div class="guide-header">
            <component :is="LucideGlobe" class="guide-icon" />
            <div>
              <h3>内容采集器</h3>
              <p>
                根据采集规则配置，定时从外部源（RSS/Atom feed）抓取内容并自动发布到文章系统。
                采集规则需在数据库中配置 <code>mxx_website_collect_rule</code> 表。
              </p>
            </div>
          </div>
        </div>
      </Card>

      <!-- 操作区 -->
      <Card class="action-card" :bordered="false">
        <div class="action-area">
          <div class="action-left">
            <h4>执行采集</h4>
            <p>点击下方按钮手动执行所有已启用的采集规则。</p>
          </div>
          <Button
            type="primary"
            size="large"
            :loading="running"
            :disabled="running"
            @click="handleRun"
          >
            <template #icon>
              <component :is="running ? LucideLoader2 : LucideLoader2"
                :class="{ spinning: running }" />
            </template>
            {{ running ? '采集中...' : '开始采集' }}
          </Button>
        </div>
      </Card>

      <!-- 结果提示 -->
      <Alert
        v-if="lastResult"
        :type="lastResult.success ? 'success' : 'error'"
        :message="lastResult.message"
        show-icon
        closable
        class="result-alert"
      />

      <!-- 说明提示 -->
      <Card class="info-card" :bordered="false">
        <div class="info-content">
          <h4>使用说明</h4>
          <ul>
            <li>采集规则需在数据库 <code>mxx_website_collect_rule</code> 表中配置</li>
            <li>支持 RSS 2.0 / Atom feed 格式采集</li>
            <li>系统定时任务会自动调用采集器，也可手动触发</li>
            <li>采集的文章会按规则配置自动归入指定栏目</li>
            <li>重复文章检测：按原文链接（original_link）判重</li>
          </ul>
        </div>
      </Card>
    </div>
  </Page>
</template>

<style scoped>
.collector-page {
  padding: 24px;
  max-width: 1000px;
  margin: 0 auto;
}

.guide-card {
  margin-bottom: 24px;
  border-radius: 8px;
}

.guide-header {
  display: flex;
  align-items: flex-start;
  gap: 16px;
}

.guide-icon {
  font-size: 40px;
  color: #1677ff;
  flex-shrink: 0;
  margin-top: 4px;
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

.action-card {
  margin-bottom: 24px;
  border-radius: 8px;
}

.action-area {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.action-left h4 {
  margin: 0 0 4px;
  font-size: 15px;
  font-weight: 600;
}

.action-left p {
  margin: 0;
  font-size: 13px;
  color: #8c8c8c;
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.result-alert {
  margin-bottom: 24px;
}

.info-card {
  border-radius: 8px;
  background: #fafafa;
}

.info-content h4 {
  margin: 0 0 12px;
  font-size: 15px;
  font-weight: 600;
}

.info-content ul {
  margin: 0;
  padding-left: 20px;
  font-size: 14px;
  color: #595959;
  line-height: 2;
}

.info-content code {
  background: #f5f5f5;
  padding: 1px 6px;
  border-radius: 3px;
  font-size: 13px;
}
</style>