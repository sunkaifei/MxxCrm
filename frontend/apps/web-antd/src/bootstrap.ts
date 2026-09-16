import { createApp, watchEffect } from 'vue';

import { registerAccessDirective } from '@vben/access';
import { registerLoadingDirective } from '@vben/common-ui/es/loading';
import { preferences } from '@vben/preferences';
import { initStores } from '@vben/stores';
import '@vben/styles';
import '@vben/styles/antd';

import { useTitle } from '@vueuse/core';
import { message } from 'ant-design-vue';

import { $t, setupI18n } from '#/locales';

import { initComponentAdapter } from './adapter/component';
import { initSetupVbenForm } from './adapter/form';
import App from './app.vue';
import { registerLucideIcons } from './icons';
import { router } from './router';

async function bootstrap(namespace: string) {
  // 全局注册 ant-design-vue message 实例，供 window.$message 调用
  window.$message = message;

  // 图标集改为**并行**预加载：先启动下载，再做其余初始化，最后统一等待。
  //
  // 原实现在此处 `await registerLucideIcons()`，而它在 createApp 之前，
  // 会把关键路径拉成串行瀑布：index → bootstrap → 图标chunk → 登录页。
  // 现在图标加载与下面的组件适配器/表单初始化**同时进行**，省掉一整个 RTT。
  // 仍会在 app.mount 之前 await 完成，因此不会出现图标闪烁。
  const iconsReady = registerLucideIcons();

  // 初始化组件适配器
  await initComponentAdapter();

  // 初始化表单组件
  await initSetupVbenForm();

  // // 设置弹窗的默认配置
  // setDefaultModalProps({
  //   fullscreenButton: false,
  // });
  // // 设置抽屉的默认配置
  // setDefaultDrawerProps({
  //   zIndex: 1020,
  // });

  const app = createApp(App);

  // 注册v-loading指令
  registerLoadingDirective(app, {
    loading: 'loading', // 在这里可以自定义指令名称，也可以明确提供false表示不注册这个指令
    spinning: 'spinning',
  });

  // 国际化 i18n 配置
  await setupI18n(app);

  // 配置 pinia-tore
  await initStores(app, { namespace });

  // A-2.4: 多标签页登出同步——其他标签登出后，本标签立即失效并弹出重新登录
  {
    const { useAccessStore } = await import('@vben/stores');
    window.addEventListener('storage', (event) => {
      if (event.key === 'mxx_force_logout_at' && event.newValue) {
        const accessStore = useAccessStore();
        accessStore.setAccessToken(null);
        accessStore.setRefreshToken(null);
        if (preferences.app.loginExpiredMode === 'modal') {
          accessStore.setLoginExpired(true);
        }
      }
    });
  }

  // 安装权限指令
  registerAccessDirective(app);

  // 初始化 tippy
  const { initTippy } = await import('@vben/common-ui/es/tippy');
  initTippy(app);

  // 配置路由及路由守卫
  app.use(router);

  // 配置Motion插件
  const { MotionPlugin } = await import('@vben/plugins/motion');
  app.use(MotionPlugin);

  // 动态更新标题
  watchEffect(() => {
    if (preferences.app.dynamicTitle) {
      const routeTitle = router.currentRoute.value.meta?.title;
      const pageTitle =
        (routeTitle ? `${$t(routeTitle)} - ` : '') + preferences.app.name;
      useTitle(pageTitle);
    }
  });

  // 确保在挂载前图标集就绪 —— 与上面的初始化并行完成，避免阻塞的同時不闪烁
  await iconsReady;

  app.mount('#app');
}

export { bootstrap };
