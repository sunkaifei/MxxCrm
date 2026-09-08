<script lang="ts" setup>
import type { VbenFormSchema } from '@vben/common-ui';
import type { Recordable } from '@vben/types';

import { computed, markRaw, onMounted, ref, shallowRef } from 'vue';
import { useRouter } from 'vue-router';

import { AuthenticationRegister, z } from '@vben/common-ui';
import { $t } from '@vben/locales';

import { getCaptchaConfigApi, getRegisterStatusApi } from '#/api';
import { checkUsernameApi, registerApi } from '#/api/core/auth';
import { useAuthStore } from '#/store';
import AgreementCheckbox from './agreement-checkbox.vue';
import ImageCaptcha, { captchaKeyRef } from './image-captcha.vue';
import RegisterClickCaptcha from './register-click-captcha.vue';

defineOptions({ name: 'Register' });

const authStore = useAuthStore();
const router = useRouter();

const usernameChecking = ref(false);
// A-2.10: 协议默认不勾选，未勾选时提交按钮禁用
const agreePolicy = ref(false);

// 验证码设置: 注册页形态（image/click/none），由系统设置-验证设置控制
const captchaMode = ref<'click' | 'disabled' | 'image'>('image');

// 注册策略: 用户名规则（由系统设置-注册策略下发，注册页据此动态校验）
const usernameRules = ref({
  minLength: 3,
  maxLength: 20,
  charset: 'alnum_underscore',
  letterStart: false,
  notPureNumber: true,
  bannedKeywords: '',
});

// A-2.11: 注册页精简后不再收集部门，部门分配改由审批/管理员维护

// A-2.12: 点选验证码组件实例（通过 onApiReady 回调注册），提交时未验证则自动弹出气泡窗
const captchaApiRef = shallowRef<{ open: () => void } | null>(null);

// 注册开关：关闭时重定向回登录页
onMounted(async () => {
  try {
    const data = await getRegisterStatusApi();
    if (!data?.registerEnabled) {
      window.$message?.warning('注册已关闭，请联系管理员');
      router.replace('/auth/login');
    }
    usernameRules.value = {
      minLength: data?.usernameMinLength ?? 3,
      maxLength: data?.usernameMaxLength ?? 20,
      charset: data?.usernameCharset ?? 'alnum_underscore',
      letterStart: data?.usernameLetterStart ?? false,
      notPureNumber: data?.usernameNotPureNumber ?? true,
      bannedKeywords: data?.usernameBannedKeywords ?? '',
    };
  } catch {
    router.replace('/auth/login');
  }
  try {
    const cfg: any = await getCaptchaConfigApi();
    if (cfg?.enabled && cfg?.register === 'click') {
      captchaMode.value = 'click';
    } else if (cfg?.enabled && cfg?.register === 'none') {
      captchaMode.value = 'disabled';
    } else {
      captchaMode.value = 'image';
    }
  } catch {
    captchaMode.value = 'image';
  }
});

// 注册策略: 根据配置动态构建用户名校验规则（长度/字符集/字母开头/纯数字/敏感词）
function buildUsernameSchema() {
  const r = usernameRules.value;
  const charsetRegex: Record<string, RegExp> = {
    alnum_underscore: /^[a-zA-Z0-9_]+$/,
    alnum: /^[a-zA-Z0-9]+$/,
    alpha: /^[a-zA-Z]+$/,
    cjk_alnum_underscore: /^[a-zA-Z0-9_\u4e00-\u9fa5]+$/,
  };
  let schema = z
    .string()
    .min(r.minLength, { message: `用户名至少需要${r.minLength}个字符` })
    .max(r.maxLength, { message: `用户名不能超过${r.maxLength}个字符` })
    .regex(charsetRegex[r.charset] ?? /^[a-zA-Z0-9_]+$/, {
      message: '用户名包含不允许的字符',
    });
  if (r.letterStart) {
    schema = schema.regex(/^[a-zA-Z]/, { message: '用户名必须以字母开头' });
  }
  if (r.notPureNumber) {
    schema = schema.regex(/\D/, { message: '用户名不能为纯数字' });
  }
  if (r.bannedKeywords) {
    const kws = r.bannedKeywords
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean)
      .map((s) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'));
    if (kws.length > 0) {
      schema = schema.regex(new RegExp(`^(?!${kws.join('|')})`, 'i'), {
        message: '用户名包含系统保留关键字，不允许注册',
      });
    }
  }
  return schema;
}

const formSchema = computed((): VbenFormSchema[] => {
  return [
    {
      component: 'VbenInput',
      componentProps: {
        placeholder: $t('authentication.usernameTip'),
        allowClear: true,
      },
      fieldName: 'username',
      label: $t('authentication.username'),
      rules: buildUsernameSchema(),
    },
    {
      component: 'VbenInput',
      componentProps: {
        placeholder: '请输入手机号',
        allowClear: true,
      },
      fieldName: 'mobile',
      label: '手机号',
      rules: z
        .string()
        .min(1, { message: '手机号不能为空' })
        .regex(/^1[3-9]\d{9}$/, { message: '请输入正确的手机号' }),
    },
    {
      component: 'VbenInput',
      componentProps: {
        placeholder: '请输入邮箱（选填）',
        allowClear: true,
      },
      fieldName: 'email',
      label: '邮箱',
    },
    {
      component: 'VbenInputPassword',
      componentProps: {
        passwordStrength: true,
        placeholder: $t('authentication.password'),
      },
      fieldName: 'password',
      label: $t('authentication.password'),
      renderComponentContent() {
        return {
          strengthText: () => $t('authentication.passwordStrength'),
        };
      },
      rules: z
        .string()
        .min(6, { message: '密码至少需要6个字符' })
        .max(32, { message: '密码不能超过32个字符' }),
    },
    {
      component: 'VbenInputPassword',
      componentProps: {
        placeholder: $t('authentication.confirmPassword'),
      },
      fieldName: 'confirmPassword',
      label: $t('authentication.confirmPassword'),
    },
    // A-2.11: 姓名/部门/岗位/期望薪资已从注册页移除，移入用户中心维护
    ...(captchaMode.value === 'image'
      ? [
          {
            // 图形方案: 复用登录页验证码组件（输入框 + 图片，点击刷新）
            component: markRaw(ImageCaptcha),
            componentProps: {
              placeholder: '请输入验证码',
            },
            fieldName: 'captchaCode',
            label: '验证码',
            rules: z.string().min(1, { message: '请输入验证码' }),
          },
        ]
      : captchaMode.value === 'click'
        ? [
            {
              // 点选方案: 整行"点击完成验证"触发气泡浮窗（锚定本字段附近），验证通过回填 ticket
              component: markRaw(RegisterClickCaptcha),
              componentProps: {
                onApiReady: (api: { open: () => void } | null) => {
                  captchaApiRef.value = api;
                },
              },
              fieldName: 'captchaTicket',
              label: '验证码',
            },
          ]
        : []),
    // A-2.10: 协议默认不勾选；onChange 同步 agreePolicy，父级据此禁用/启用提交按钮
    {
      component: markRaw(AgreementCheckbox),
      componentProps: {
        onChange: (val: boolean) => {
          agreePolicy.value = !!val;
        },
      },
      defaultValue: false,
      fieldName: 'agreePolicy',
    },
  ];
});

async function handleSubmit(value: Recordable<any>) {
  if (!value.agreePolicy) {
    return;
  }

  if (value.password !== value.confirmPassword) {
    return;
  }

  const username = value.username;
  if (username && username.length >= usernameRules.value.minLength) {
    usernameChecking.value = true;
    try {
      const result = await checkUsernameApi(username);
      if (result.exists) {
        return;
      }
    } finally {
      usernameChecking.value = false;
    }
  }

  const payload = { ...value };

  if (captchaMode.value === 'image') {
    payload.captchaCode = payload.captchaCode ?? '';
    payload.captchaKey = captchaKeyRef.value;
  } else if (captchaMode.value === 'click') {
    if (!value.captchaTicket) {
      // A-2.12: 未完成验证时自动弹出气泡验证窗，让用户在浮窗内完成验证
      captchaApiRef.value?.open();
      return;
    }
    payload.captchaTicket = value.captchaTicket;
  }

  await registerApi(payload);

  // 注册即启用（status=1），不自动登录，提示后跳转登录页
  window.$message?.success('注册成功，请登录');
  router.replace('/auth/login');
}
</script>

<template>
  <div class="register-wrap">
    <AuthenticationRegister
      :form-schema="formSchema"
      :loading="authStore.loginLoading || usernameChecking"
      :submit-button-disabled="!agreePolicy"
      @submit="handleSubmit"
    />
  </div>
</template>

<style scoped>
.register-wrap {
  width: 100%;
}
</style>
