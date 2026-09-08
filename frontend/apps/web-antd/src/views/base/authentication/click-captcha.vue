<script lang="ts" setup>
/**
 * 点选方案: 文字点选验证码浮层组件
 *
 * 交互（对标极验/AJ-Captcha）：
 * 1. 打开时自动拉取合成图（背景 + 3 个汉字，坐标服务端留存）
 * 2. 用户按提示条顺序在图上点击，落点渲染序号标记
 * 3. 确认 → 提交显示尺寸坐标 → 后端等比换算校验 → 成功签发一次性 captchaTicket
 * 4. 失败自动换新图并清空落点
 */
import { nextTick, onMounted, ref } from 'vue';


import { getClickCaptchaApi, verifyClickCaptchaApi } from '#/api';

const emit = defineEmits<{
  verified: [ticket: string];
  cancelled: [];
}>();

const loading = ref(false);
const verifying = ref(false);
const imageBase64 = ref('');
const hintChars = ref<string[]>([]);
const captchaKey = ref('');
const imageWidth = ref(320);
const imageHeight = ref(160);
const clicks = ref<Array<{ x: number; y: number }>>([]);
const imgEl = ref<HTMLImageElement | null>(null);
// 错误内嵌提示条（替代 message 弹窗，点击横条重试）
const errorTip = ref('');

async function refresh() {
  loading.value = true;
  errorTip.value = '';
  try {
    const res: any = await getClickCaptchaApi();
    imageBase64.value = res?.imageBase64 ?? '';
    hintChars.value = res?.hintChars ?? [];
    captchaKey.value = res?.captchaKey ?? '';
    imageWidth.value = res?.imageWidth ?? 320;
    imageHeight.value = res?.imageHeight ?? 160;
    clicks.value = [];
  } catch (e: any) {
    errorTip.value = e?.msg || e?.message || '验证码加载失败，请点击重试';
  } finally {
    loading.value = false;
  }
}

function onImageClick(e: MouseEvent) {
  const el = imgEl.value;
  if (!el || clicks.value.length >= hintChars.value.length) return;
  const rect = el.getBoundingClientRect();
  // 记录"显示尺寸"坐标（后端按原图尺寸等比换算）
  clicks.value.push({
    x: Math.round(e.clientX - rect.left),
    y: Math.round(e.clientY - rect.top),
  });
  // 点选升级: 点满即自动请求验证（省去确认按钮）
  if (clicks.value.length === hintChars.value.length) {
    nextTick(() => submit());
  }
}

async function submit() {
  if (clicks.value.length !== hintChars.value.length) {
    errorTip.value = `请按顺序点击 ${hintChars.value.length} 个文字`;
    return;
  }
  verifying.value = true;
  try {
    const res: any = await verifyClickCaptchaApi({
      captchaKey: captchaKey.value,
      clicks: clicks.value,
      displayWidth: imgEl.value?.clientWidth ?? imageWidth.value,
      displayHeight: imgEl.value?.clientHeight ?? imageHeight.value,
    });
    errorTip.value = '';
    emit('verified', res.captchaTicket);
  } catch (e: any) {
    // 失败不弹窗：横条显示原因，点击横条换图重试
    errorTip.value = e?.msg || e?.message || '验证未通过，请点击重试';
    await refresh();
  } finally {
    verifying.value = false;
  }
}

onMounted(refresh);

// 模板辅助：落点标记的百分比定位
function dotStyle(i: number) {
  const c = clicks.value[i];
  if (!c || !imageWidth.value) return {};
  return {
    left: `${(c.x / (imgEl.value?.clientWidth || imageWidth.value)) * 100}%`,
    top: `${(c.y / (imgEl.value?.clientHeight || imageHeight.value)) * 100}%`,
  };
}
</script>

<template>
  <div class="click-captcha">
    <div class="hint-bar">
      <span class="hint-text">
        请按顺序点击：
        <b v-for="(h, i) in hintChars" :key="i" class="hint-char">{{ h }}</b>
      </span>
      <button class="refresh" title="换一张" @click="refresh">刷新</button>
    </div>
    <div
      v-if="errorTip"
      class="error-bar"
      @click="refresh"
    >
      ✕ {{ errorTip }}，点击重试
    </div>
    <div class="img-wrap">
      <img
        v-if="imageBase64"
        ref="imgEl"
        :src="`data:image/png;base64,${imageBase64}`"
        alt="captcha"
        class="captcha-img"
        @click="onImageClick"
      />
      <span
        v-for="(_, i) in clicks"
        :key="i"
        :style="dotStyle(i)"
        class="click-dot"
      >{{ i + 1 }}</span>
    </div>
  </div>
</template>

<style scoped>
.error-bar {
  background: #fdf0ef;
  border: 1px solid #fbc4c4;
  border-radius: 4px;
  color: #f56c6c;
  cursor: pointer;
  font-size: 13px;
  margin-bottom: 8px;
  padding: 8px 10px;
  text-align: center;
}
.error-bar:hover {
  background: #fde2e0;
}
.actions {
  display: flex;
  gap: 10px;
  margin-top: 10px;
}
.btn {
  background: transparent;
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
  cursor: pointer;
  flex: 1;
  height: 36px;
}
.btn.primary {
  background: hsl(var(--primary));
  border: none;
  color: #fff;
}
.btn:disabled {
  opacity: 0.6;
}
.captcha-img {
  border-radius: 6px;
  cursor: crosshair;
  display: block;
  width: 100%;
}
.click-captcha {
  width: 100%;
}
.click-dot {
  align-items: center;
  background: hsl(var(--primary) / 85%);
  border: 2px solid #fff;
  border-radius: 50%;
  color: #fff;
  display: flex;
  font-size: 12px;
  font-weight: 700;
  height: 22px;
  justify-content: center;
  pointer-events: none;
  position: absolute;
  transform: translate(-50%, -50%);
  width: 22px;
}
.hint-bar {
  align-items: center;
  background: hsl(var(--border) / 40%);
  border-radius: 6px;
  display: flex;
  font-size: 13px;
  justify-content: space-between;
  margin-bottom: 8px;
  padding: 6px 10px;
}
.hint-char {
  color: hsl(var(--primary));
  font-size: 16px;
  margin: 0 2px;
}
.img-wrap {
  position: relative;
}
.refresh {
  background: transparent;
  border: none;
  color: hsl(var(--primary));
  cursor: pointer;
  font-size: 12px;
}
</style>
