<script lang="ts" setup>
import { computed, ref } from 'vue';

import { useVbenDrawer, z } from '@vben/common-ui';

import { message, Radio, Select, Tree } from 'ant-design-vue';

import { useVbenForm } from '#/adapter/form';
import {
  createRoleApi,
  getMenuOptionsApi,
  getMenusRouterApi,
  getMenuTreeApi,
  updateRoleApi,
  updateRoleAuthApi,
} from '#/api';
import { $t } from '#/locales';
import { statusList } from '#/store';

import { matchPerm, roleTemplateOptions, roleTemplates } from './role-templates';

const data = ref();
const getTitle = computed(() =>
  data.value?.create
    ? $t('ui.modal.create', { moduleName: $t('page.system.role.module') })
    : $t('ui.modal.update', { moduleName: $t('page.system.role.module') }),
);

// ===== 默认首页（方案 5.3-M4）：候选来自当前用户可见菜单路由，值为路由 path =====
const homePathOptions = ref<Array<{ label: string; value: string }>>([]);
let homePathOptionsLoaded = false;
async function loadHomePathOptions() {
  if (homePathOptionsLoaded) return;
  homePathOptionsLoaded = true;
  try {
    const res: any = await getMenusRouterApi({});
    const list = Array.isArray(res) ? res : res?.data || [];
    const options: Array<{ label: string; value: string }> = [];
    const seen = new Set<string>();
    const walk = (nodes: any[], prefix: string) => {
      for (const n of nodes || []) {
        const title = n?.meta?.title || n?.name || '';
        const path = String(n?.path || '');
        const label = prefix ? `${prefix} / ${title}` : title;
        if (path.startsWith('/') && !seen.has(path)) {
          seen.add(path);
          options.push({ label: `${label}（${path}）`, value: path });
        }
        if (Array.isArray(n?.children) && n.children.length > 0) {
          walk(n.children, label);
        }
      }
    };
    walk(list, '');
    homePathOptions.value = options;
  } catch {
    homePathOptions.value = [];
    homePathOptionsLoaded = false;
  }
}
// 超级管理员角色首页由系统固定，后端忽略该字段（前端同步禁用）
const isSuperAdminRole = computed(() => String(data.value?.row?.id ?? '') === '1');

const [BaseForm, baseFormApi] = useVbenForm({
  showDefaultActions: false,
  commonConfig: {
    componentProps: {
      class: 'w-full',
    },
  },
  schema: [
    {
      component: 'Input',
      fieldName: 'roleName',
      label: $t('page.system.role.name'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
      rules: z.string().min(1, { message: $t('ui.formRules.required') }),
    },
    {
      component: 'Input',
      fieldName: 'roleKey',
      label: $t('page.system.role.code'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
      rules: 'required',
    },
    {
      component: 'InputNumber',
      fieldName: 'sort',
      label: $t('ui.table.sortId'),
      componentProps: {
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'Select',
      fieldName: 'homePath',
      label: $t('page.system.role.homePath'),
      componentProps: () => ({
        placeholder: $t('page.system.role.homePathPlaceholder'),
        allowClear: true,
        showSearch: true,
        optionFilterProp: 'label',
        options: homePathOptions.value,
        disabled: isSuperAdminRole.value,
      }),
    },
    {
      component: 'Input',
      fieldName: 'remark',
      label: $t('ui.table.remark'),
      componentProps: {
        type: 'textarea',
        autosize: true,
        rows: 5,
        placeholder: $t('ui.placeholder.input'),
        allowClear: true,
      },
    },
    {
      component: 'RadioGroup',
      fieldName: 'status',
      defaultValue: 1,
      label: $t('ui.table.status'),
      rules: 'selectRequired',
      componentProps: {
        optionType: 'button',
        class: 'flex flex-wrap',
        options: statusList,
      },
    },
  ],
});

// ===== P3-3 角色模板向导（仅新建模式）：选择模板后预填菜单勾选与数据范围，管理员微调后保存 =====
const templateKey = ref<string>();
const activeTemplate = computed(() =>
  roleTemplates.find((t) => t.key === templateKey.value),
);
const templateTreeData = ref<any[]>([]);
const templateCheckedKeys = ref<string[]>([]);
const templateExpandedKeys = ref<string[]>([]);
const templateMenusLoading = ref(false);
let templateMenusLoaded = false;
// 切换模板的竞态防护：异步加载期间再次切换时，旧回调不得写入
let templateLoadSeq = 0;

// 五档数据范围文案与 set-auth.vue 的选项一一对应
const dataScopeValue = ref<number>(5);
const dataScopeOptions = [
  {
    value: 1,
    label: '全部数据',
    desc: '可查看系统中所有业务数据，不受部门和个人限制',
  },
  { value: 2, label: '自定义数据', desc: '可查看指定部门及以下的业务数据' },
  {
    value: 3,
    label: '本部门数据',
    desc: '只能查看所在部门所有成员负责的业务数据',
  },
  {
    value: 4,
    label: '本部门及以下',
    desc: '可查看本部门及下属部门所有成员的业务数据',
  },
  { value: 5, label: '仅本人数据', desc: '只能查看自己负责的业务数据' },
];
const activeScopeDesc = computed(
  () =>
    dataScopeOptions.find((o) => o.value === dataScopeValue.value)?.desc ?? '',
);

const fieldNames = {
  children: 'children',
  title: 'name',
  key: 'id',
};

// 菜单名 i18n（与 set-auth.vue 口径一致：目录级 key 带 .title fallback）
const translateName = (key?: null | string): string => {
  if (!key) return '';
  // 已是翻译后的中文直接返回，避免对非 i18n key 二次翻译触发 intlify 警告
  if (!/^[a-zA-Z][\w-]*(\.[a-zA-Z][\w-]*)+$/.test(key)) return key;
  const direct = $t(key);
  if (direct !== key && !direct.startsWith('[object ')) return direct;
  const withTitle = $t(`${key}.title`);
  return withTitle === `${key}.title` ? key : withTitle;
};

const translateMenu = (items: any[]): any[] => {
  return items.map((item) => {
    if (item.name) item.name = translateName(item.name);
    if (item.meta?.name) item.meta.name = translateName(item.meta.name);
    if (item.children?.length) {
      item.children = translateMenu(item.children);
    }
    return item;
  });
};

const collectAllIds = (nodes: any[]): string[] => {
  const keys: string[] = [];
  const walk = (list: any[]) => {
    list.forEach((node) => {
      if (node.id !== undefined && node.id !== null) {
        keys.push(String(node.id));
      }
      if (node.children?.length) {
        walk(node.children);
      }
    });
  };
  walk(nodes);
  return keys;
};

// 菜单 ID -> 权限码 索引（来自菜单管理列表 /menu/list；/menu/options 不含 perm）
const permIdMap = new Map<string, string>();
// 权限码索引不可用（如无菜单列表权限）时无法按模板预填，前端明确提示
const templatePrefillUnavailable = computed(
  () => templateMenusLoaded && permIdMap.size === 0,
);

async function loadTemplateMenus() {
  // 授权树：当前用户可授权的菜单（后端按权限过滤，避免越权授权）
  const optionsList = await getMenuOptionsApi();
  const tree = Array.isArray(optionsList) ? optionsList : [];
  // 权限码索引：全量菜单列表（含 perm 字段）
  try {
    const fullList: any = await getMenuTreeApi({});
    const nodes = Array.isArray(fullList) ? fullList : fullList?.data || [];
    const walkFull = (list: any[]) => {
      (list || []).forEach((n: any) => {
        if (n?.id !== undefined && n?.id !== null) {
          permIdMap.set(String(n.id), String(n.perm || ''));
        }
        if (n?.children?.length) walkFull(n.children);
      });
    };
    walkFull(nodes);
  } catch {
    // 无菜单列表权限时索引为空，模板预填为空集，管理员保存后可手动配置
  }
  return tree;
}

async function applyTemplate() {
  const template = activeTemplate.value;
  templateCheckedKeys.value = [];
  if (!template) {
    templateTreeData.value = [];
    templateExpandedKeys.value = [];
    return;
  }
  // 数据范围随模板预填，可自由调整
  dataScopeValue.value = template.dataScope;

  const seq = ++templateLoadSeq;
  templateMenusLoading.value = true;
  try {
    if (!templateMenusLoaded) {
      const tree = await loadTemplateMenus();
      if (seq !== templateLoadSeq) return;
      templateTreeData.value = translateMenu(tree || []);
      templateMenusLoaded = true;
    }
    if (seq !== templateLoadSeq) return;

    // 勾选 = 权限码匹配的节点 + 其全部祖先链（后端校验菜单链路完整性：父级未勾选不保存）
    const checked = new Set<string>();
    const walk = (nodes: any[], ancestors: string[]) => {
      (nodes || []).forEach((n: any) => {
        const id = String(n.id);
        const chain = [...ancestors, id];
        if (matchPerm(permIdMap.get(id), template.permPatterns)) {
          chain.forEach((x) => checked.add(x));
        }
        if (n.children?.length) walk(n.children, chain);
      });
    };
    walk(templateTreeData.value, []);
    templateCheckedKeys.value = [...checked];
    templateExpandedKeys.value = collectAllIds(templateTreeData.value);
  } catch {
    message.warning('模板菜单加载失败，请稍后重试或保存后手动配置权限');
  } finally {
    if (seq === templateLoadSeq) {
      templateMenusLoading.value = false;
    }
  }
}

const [Drawer, drawerApi] = useVbenDrawer({
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
      // P3-3：选择模板时随创建一并提交数据范围（save 接口原生支持 dataScope）
      const payload: Record<string, any> = { ...values };
      if (data.value?.create && activeTemplate.value) {
        payload.dataScope = dataScopeValue.value;
      }

      const newRoleId = data.value?.create
        ? await createRoleApi(payload)
        : await updateRoleApi(data.value.row.id, values);

      // P3-3：模板菜单预填——创建成功后按勾选集写入菜单授权（失败不阻断角色创建）
      if (
        data.value?.create &&
        activeTemplate.value &&
        templateCheckedKeys.value.length > 0
      ) {
        try {
          await updateRoleAuthApi(Number(newRoleId), {
            authId: templateCheckedKeys.value,
          });
        } catch {
          message.warning('角色已创建，但模板菜单预填失败，请在角色列表"权限"中手动配置');
        }
      }

      message.success(
        data.value?.create
          ? $t('ui.notification.create_success')
          : $t('ui.notification.update_success'),
      );
      drawerApi.close();
    } catch {
      // 错误由全局拦截器处理，保留抽屉打开以便用户修改后重试
    } finally {
      setLoading(false);
    }
  },

  onOpenChange(isOpen) {
    if (isOpen) {
      data.value = drawerApi.getData<Record<string, any>>();
      baseFormApi.setValues(data.value?.row);
      loadHomePathOptions();
      // P3-3：重置模板向导状态，避免残留上一次的勾选与数据范围
      templateKey.value = undefined;
      templateCheckedKeys.value = [];
      templateExpandedKeys.value = [];
      dataScopeValue.value = 5;
      setLoading(false);
    }
  },
});

function setLoading(loading: boolean) {
  drawerApi.setState({ loading });
}
</script>

<template>
  <Drawer :title="getTitle">
    <!-- P3-3 角色模板向导：仅新建模式展示，选择后预填菜单勾选与数据范围，可自由增删 -->
    <div
      v-if="data?.create"
      class="mb-4 rounded-lg border border-gray-100 p-3"
    >
      <div class="mb-2 flex items-center justify-between gap-2">
        <span class="text-sm font-medium">从模板开始</span>
        <span class="text-xs text-gray-400">预填授权与数据范围，保存前可自由调整</span>
      </div>
      <Select
        v-model:value="templateKey"
        :options="roleTemplateOptions"
        :loading="templateMenusLoading"
        placeholder="不使用模板，从零配置"
        allow-clear
        class="w-full"
        @change="applyTemplate"
      />
      <template v-if="activeTemplate">
        <div class="mt-2 text-xs text-gray-500">{{ activeTemplate.desc }}</div>

        <div class="mt-3">
          <div class="mb-1 text-xs font-medium text-gray-600">数据范围</div>
          <Radio.Group v-model:value="dataScopeValue" class="flex flex-wrap">
            <Radio
              v-for="opt in dataScopeOptions"
              :key="opt.value"
              :value="opt.value"
            >
              <span class="text-xs">{{ opt.label }}</span>
            </Radio>
          </Radio.Group>
          <div class="mt-1 text-xs text-gray-400">{{ activeScopeDesc }}</div>
          <div v-if="dataScopeValue === 2" class="mt-1 text-xs text-amber-500">
            选择"自定义数据"后，请保存角色并在其"权限"抽屉中勾选可见部门
          </div>
        </div>

        <div class="mt-3">
          <div class="mb-1 flex items-center justify-between">
            <span class="text-xs font-medium text-gray-600">菜单授权预填</span>
            <span class="text-xs text-gray-400">
              共勾选 {{ templateCheckedKeys.length }} 项
            </span>
          </div>
          <div
            v-if="templatePrefillUnavailable"
            class="mb-1 text-xs text-amber-500"
          >
            无菜单列表权限，无法按模板预填，请保存后在"权限"中手动配置
          </div>
          <div
            class="max-h-[240px] overflow-y-auto rounded-lg border border-gray-100 p-3"
          >
            <Tree
              v-model:expanded-keys="templateExpandedKeys"
              v-model:checked-keys="templateCheckedKeys"
              :tree-data="templateTreeData"
              checkable
              :check-strictly="true"
              :field-names="fieldNames"
              class="w-full"
            >
              <template #title="{ data: item }">
                <span class="text-sm">{{ item.name }}</span>
              </template>
            </Tree>
            <div
              v-if="templateTreeData.length === 0"
              class="py-8 text-center text-gray-400"
            >
              {{ templateMenusLoading ? '菜单加载中…' : '暂无菜单数据' }}
            </div>
          </div>
        </div>
      </template>
    </div>

    <BaseForm />
  </Drawer>
</template>
