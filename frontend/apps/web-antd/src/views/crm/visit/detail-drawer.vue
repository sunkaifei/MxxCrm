<script lang="ts" setup>
import { computed, ref } from 'vue';

import { formatDateTime } from '@vben/utils';

import {
  Descriptions,
  DescriptionsItem,
  Empty,
  Image as AImage,
  Spin,
  Tag,
} from 'ant-design-vue';

import { useVbenDrawer } from '#/adapter/drawer';
import { getVisitInfoApi } from '#/api';

const loading = ref(false);
const detail = ref<any>(null);

// 解析签到照片字段（后端 visit_photos 为 JSON 数组）
const photos = computed<string[]>(() => {
  const raw = detail.value?.visitPhotos ?? detail.value?.visit_photos;
  if (!raw) return [];
  if (Array.isArray(raw)) return raw.filter((u: any) => !!u);
  if (typeof raw === 'string') {
    try {
      const parsed = JSON.parse(raw);
      return Array.isArray(parsed) ? parsed.filter((u: any) => !!u) : [raw];
    } catch {
      return raw ? [raw] : [];
    }
  }
  return [];
});

// 拜访时长：签退时间 - 签到时间
const duration = computed(() => {
  const start = detail.value?.checkInTime ?? detail.value?.check_in_time;
  const end = detail.value?.checkOutTime ?? detail.value?.check_out_time;
  if (!start || !end) return '-';
  const startMs = new Date(start).getTime();
  const endMs = new Date(end).getTime();
  if (Number.isNaN(startMs) || Number.isNaN(endMs) || endMs <= startMs)
    return '-';
  const diffMs = endMs - startMs;
  const minutes = Math.floor(diffMs / 60000);
  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  if (hours > 0) return `${hours}小时${mins}分钟`;
  return `${mins}分钟`;
});

// 距客户距离格式化
const distance = computed(() => {
  const dist =
    detail.value?.visitDistance ?? detail.value?.visit_distance;
  if (dist == null || Number.isNaN(Number(dist))) return '-';
  const d = Number(dist);
  if (d < 0) return '-';
  if (d < 1000) return `${d.toFixed(0)}米`;
  return `${(d / 1000).toFixed(2)}公里`;
});

async function loadDetail(id: number) {
  loading.value = true;
  try {
    const res: any = await getVisitInfoApi(id);
    detail.value = res;
  } catch {
    detail.value = null;
  } finally {
    loading.value = false;
  }
}

const [Drawer, drawerApi] = useVbenDrawer({
  onOpenChange(isOpen) {
    if (isOpen) {
      const data = drawerApi.getData() as { id?: number };
      detail.value = null;
      if (data?.id) {
        loadDetail(Number(data.id));
      }
    }
  },
});
</script>

<template>
  <Drawer
    title="拜访详情"
    :destroy-on-close="true"
    :footer="false"
    :width="'min(720px, 92vw)'"
  >
    <Spin :spinning="loading">
      <Empty v-if="!detail && !loading" description="暂无数据" />
      <div v-else-if="detail" class="visit-detail-wrap">
        <!-- 客户信息 -->
        <Descriptions
          title="客户信息"
          :column="2"
          bordered
          size="small"
          class="visit-desc"
        >
          <DescriptionsItem label="客户名称">
            {{ detail.customerName || detail.leadName || '-' }}
          </DescriptionsItem>
          <DescriptionsItem label="客户负责人">
            {{ detail.assigneeName || '-' }}
          </DescriptionsItem>
          <DescriptionsItem label="拜访人">
            {{ detail.createdByName || '-' }}
          </DescriptionsItem>
          <DescriptionsItem label="跟进时间">
            {{ detail.followTime ? formatDateTime(detail.followTime) : '-' }}
          </DescriptionsItem>
        </Descriptions>

        <!-- 签到信息 -->
        <Descriptions
          title="签到信息"
          :column="2"
          bordered
          size="small"
          class="visit-desc"
        >
          <DescriptionsItem label="签到时间" :span="2">
            {{ detail.checkInTime ? formatDateTime(detail.checkInTime) : '-' }}
          </DescriptionsItem>
          <DescriptionsItem label="签到地址" :span="2">
            {{ detail.visitAddress || '-' }}
          </DescriptionsItem>
          <DescriptionsItem label="经纬度">
            {{ detail.visitLongitude ?? '-' }}, {{ detail.visitLatitude ?? '-' }}
          </DescriptionsItem>
          <DescriptionsItem label="定位精度">
            {{
              detail.visitAccuracy != null
                ? `${Number(detail.visitAccuracy).toFixed(0)}米`
                : '-'
            }}
          </DescriptionsItem>
          <DescriptionsItem label="距客户距离" :span="2">
            {{ distance }}
          </DescriptionsItem>
        </Descriptions>

        <!-- 签到照片 -->
        <div class="visit-section">
          <div class="visit-section-title">签到照片</div>
          <div v-if="photos.length > 0" class="visit-photo-grid">
            <AImage
              v-for="(url, idx) in photos"
              :key="idx"
              :src="url"
              :width="100"
              :height="100"
              class="visit-photo-item"
            />
          </div>
          <Empty v-else description="无签到照片" :image="Empty.PRESENTED_IMAGE_SIMPLE" />
        </div>

        <!-- 拜访内容 -->
        <Descriptions
          title="拜访内容"
          :column="1"
          bordered
          size="small"
          class="visit-desc"
        >
          <DescriptionsItem label="拜访内容">
            <div class="visit-content-text">
              {{ detail.content || '-' }}
            </div>
          </DescriptionsItem>
          <DescriptionsItem label="拜访结果">
            {{ detail.result || '-' }}
          </DescriptionsItem>
          <DescriptionsItem label="下次跟进">
            {{ detail.nextFollowDate || '-' }}
          </DescriptionsItem>
        </Descriptions>

        <!-- 签退信息 -->
        <Descriptions
          title="签退信息"
          :column="2"
          bordered
          size="small"
          class="visit-desc"
        >
          <DescriptionsItem label="签退时间" :span="2">
            <Tag v-if="detail.checkOutTime" color="green">已签退</Tag>
            <Tag v-else color="default">未签退</Tag>
            <span class="ml-2">
              {{
                detail.checkOutTime ? formatDateTime(detail.checkOutTime) : '-'
              }}
            </span>
          </DescriptionsItem>
          <DescriptionsItem label="拜访时长" :span="2">
            <span class="visit-duration">{{ duration }}</span>
          </DescriptionsItem>
        </Descriptions>
      </div>
    </Spin>
  </Drawer>
</template>

<style scoped>
.visit-detail-wrap {
  padding: 4px 0 16px;
}
.visit-desc {
  margin-bottom: 20px;
}
.visit-desc :deep(.ant-descriptions-title) {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 12px;
  color: hsl(var(--foreground));
}
.visit-section {
  margin-bottom: 20px;
}
.visit-section-title {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 12px;
  color: hsl(var(--foreground));
}
.visit-photo-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}
.visit-photo-item {
  border-radius: 6px;
  border: 1px solid hsl(var(--border));
  object-fit: cover;
}
.visit-content-text {
  white-space: pre-wrap;
  word-break: break-all;
  line-height: 1.6;
  min-height: 40px;
}
.visit-duration {
  font-weight: 600;
  color: #1677ff;
}
</style>
