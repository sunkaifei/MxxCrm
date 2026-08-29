import type { FieldSchemaItem } from '#/api';

import { computed, markRaw, ref } from 'vue';

import { useUserStore } from '@vben/stores';

import { getAdminOptionsApi } from '#/api/core/system/user';
import { getFieldSchemaApi } from '#/api/core/system/field';
import { getFileListApi } from '#/api/core/attachment/file';
import { useSuperAdminGuard } from '#/composables/use-super-admin-guard';

import DynamicFieldControl from './DynamicFieldControl.vue';

export type { FieldSchemaItem };

/** 选项展示结构 */
export interface OptionItem {
  label: string;
  value: string;
}

/** 筛选操作符文案（与后端 build_filter_expr 支持集对齐） */
const CF_OP_LABELS: Record<string, string> = {
  eq: '等于',
  ne: '不等于',
  like: '包含',
  gt: '大于',
  gte: '大于等于',
  lt: '小于',
  lte: '小于等于',
  contains: '包含',
};

/** 筛选字段条目（P1-2）：filterable=1 的字段 + 操作符/控件元信息 */
export interface FilterFieldMeta {
  /** 原始 schema 条目（供 buildCfParam 判型） */
  item: FieldSchemaItem;
  fieldKey: string;
  fieldLabel: string;
  fieldType: number;
  /** 可选操作符（空数组 = 固定操作符） */
  operators: OptionItem[];
  /** 固定操作符：数组型 contains、布尔 eq */
  fixedOp?: string;
  /** 下拉选项数据源（6/7 选择、9 附件、10 成员；其余为空） */
  choices: OptionItem[];
  /** 是否多选控件（7/9/10） */
  multiple: boolean;
}

/** 表单布局跨度：多行文本独占整行，其余半行 */
export function fieldLayoutSpan(item: FieldSchemaItem): number {
  return Number(item.fieldType) === 2 ? 24 : 12;
}

/** 成员字段（10）用户选项缓存：一次拉全量，列表/表单共用，防逐行查询 N+1 */
let userOptionsCache: OptionItem[] = [];
let userMapCache: Map<string, string> = new Map();
let userOptionsLoaded = false;

export async function loadUserOptions(): Promise<OptionItem[]> {
  if (userOptionsLoaded) return userOptionsCache;
  try {
    const res: any = await getAdminOptionsApi({ bizOnly: true });
    const list = Array.isArray(res) ? res : (res?.list ?? []);
    userOptionsCache = list.map((u: any) => ({
      label: String(u.label ?? u.realName ?? u.username ?? u.id ?? ''),
      value: String(u.value ?? u.id ?? ''),
    }));
    userMapCache = new Map(userOptionsCache.map((u) => [u.value, u.label]));
  } catch {
    /* 加载失败按空选项处理，不影响主流程 */
  } finally {
    userOptionsLoaded = true;
  }
  return userOptionsCache;
}

export function getUserOptions(): OptionItem[] {
  return userOptionsCache;
}

export function getUserNameMap(): Map<string, string> {
  return userMapCache;
}

/** 附件字段（9）选项缓存：拉取已有文件做「选择已有附件」下拉 */
let fileOptionsCache: OptionItem[] = [];
let fileMapCache: Map<string, string> = new Map();
let fileOptionsLoaded = false;

export async function loadFileOptions(): Promise<OptionItem[]> {
  if (fileOptionsLoaded) return fileOptionsCache;
  try {
    const res: any = await getFileListApi({ page: 1, pageSize: 200 });
    const list = Array.isArray(res) ? res : (res?.items ?? res?.list ?? []);
    fileOptionsCache = list.map((f: any) => ({
      label: String(f.fileName ?? f.name ?? f.id ?? ''),
      value: String(f.id),
    }));
    fileMapCache = new Map(fileOptionsCache.map((f) => [f.value, f.label]));
  } catch {
    /* ignore */
  } finally {
    fileOptionsLoaded = true;
  }
  return fileOptionsCache;
}

export function getFileOptions(): OptionItem[] {
  return fileOptionsCache;
}

export function getFileNameMap(): Map<string, string> {
  return fileMapCache;
}

/**
 * 模块字段 schema 适配器
 * - loadSchema：拉取启用中字段（后端已按角色过滤，缓存 60s + 主动失效）
 * - toGridColumns：生成 vxe-grid 动态列（listVisible=1；数组型字段跳过排序）
 * - formatFieldValue：值按类型格式化（选项 value→label 反查、失效兜底、成员名/附件名映射）
 * - buildSubmitPayload：表单值 → customFields 提交结构（空值统一 null 清空语义）
 */
export function useFieldSchema(module: string) {
  const loading = ref(false);
  const items = ref<FieldSchemaItem[]>([]);

  // 当前用户角色与超管判定：toFormSchema 计算 editableRoles 软约束（硬约束由后端越权 400 拦截兜底）
  const userStore = useUserStore();
  const { isSuperAdmin } = useSuperAdminGuard();
  const userRoles = computed(
    () => ((userStore.userInfo as any)?.roles ?? []) as string[],
  );

  async function loadSchema() {
    loading.value = true;
    try {
      const res: any = await getFieldSchemaApi(module);
      const list = Array.isArray(res?.list) ? res.list : [];
      items.value = list;
      // 成员/附件字段存在时预拉选项，保证格式化映射就绪
      if (list.some((i: any) => Number(i.fieldType) === 10)) {
        await loadUserOptions();
      }
      if (list.some((i: any) => Number(i.fieldType) === 9)) {
        await loadFileOptions();
      }
    } catch {
      /* schema 拉取失败时动态字段不渲染，标准字段不受影响 */
    } finally {
      loading.value = false;
    }
  }

  /** 启用中的选项列表（停用选项不出现在表单/筛选） */
  function activeChoices(item: FieldSchemaItem): OptionItem[] {
    return (
      item.options?.choices
        ?.filter((c) => Number(c.active) === 1)
        .map((c) => ({ label: c.label, value: String(c.value) })) ?? []
    );
  }

  /** 失效选项兜底（7.4 规则 3）：value 无启用 label 时显示「原值（已失效）」，不空白、不报错 */
  function choiceLabel(item: FieldSchemaItem, value: any): string {
    const v = String(value);
    const choices = item.options?.choices ?? [];
    const hit = choices.find((c) => String(c.value) === v);
    if (hit && Number(hit.active) === 1) return hit.label;
    return `${hit?.label ?? v}（已失效）`;
  }

  /** 值格式化（列表 formatter / 详情展示共用） */
  function formatFieldValue(item: FieldSchemaItem, value: any): string {
    if (value === undefined || value === null || value === '') return '';
    const type = Number(item.fieldType);
    switch (type) {
      case 3: {
        const precision = item.options?.precision;
        const num = Number(value);
        // precision=0 合法（固定 0 位小数），不能用 falsy 判断
        return Number.isFinite(num)
          ? precision !== undefined && precision !== null
            ? num.toFixed(precision)
            : String(num)
          : String(value);
      }
      case 6: {
        return choiceLabel(item, value);
      }
      case 7: {
        return (Array.isArray(value) ? value : [value])
          .map((v) => choiceLabel(item, v))
          .join('、');
      }
      case 8: {
        return value === true || value === 'true' ? '是' : '否';
      }
      case 9: {
        const map = getFileNameMap();
        return (Array.isArray(value) ? value : [value])
          .map((v) => map.get(String(v)) ?? String(v))
          .join('、');
      }
      case 10: {
        const map = getUserNameMap();
        return (Array.isArray(value) ? value : [value])
          .map((v) => map.get(String(v)) ?? String(v))
          .join('、');
      }
      case 11: {
        const num = Number(value);
        if (!Number.isFinite(num)) return String(value);
        const precision = item.options?.precision ?? 2;
        return num.toLocaleString('zh-CN', {
          maximumFractionDigits: precision,
          minimumFractionDigits: precision,
        });
      }
      default: {
        return String(value);
      }
    }
  }

  /** vxe-grid 动态列：listVisible=1 按 listSort 排序；数组型（7/9/10）无序语义跳过 sortable */
  function toGridColumns() {
    return items.value
      .filter((i) => Number(i.listVisible) === 1)
      .sort((a, b) => (a.listSort ?? 0) - (b.listSort ?? 0))
      .map((item) => ({
        title: item.fieldLabel,
        field: item.fieldKey,
        minWidth: 120,
        sortable: ![7, 9, 10].includes(Number(item.fieldType)),
        formatter: ({ row }: any) =>
          formatFieldValue(item, row?.customFields?.[item.fieldKey]),
      }));
  }

  /** 表单值归一化：空值统一 null（后端 null=清空该键；未提交键由后端合并存量） */
  function normalizeSubmitValue(
    item: FieldSchemaItem,
    value: any,
  ): any {
    const type = Number(item.fieldType);
    if (type === 7 || type === 9 || type === 10) {
      return Array.isArray(value) && value.length > 0 ? value : null;
    }
    if (type === 8) {
      // 布尔空值（未设置/清除）同样按清空语义提交 null（后端 null=清空该键）；仅显式 true/'true' 视为真
      if (value === undefined || value === null || value === '') return null;
      return value === true || value === 'true';
    }
    if (value === undefined || value === null || value === '') return null;
    if (type === 3) return Number(value);
    if (type === 11) return String(value);
    return value;
  }

  /** 表单值 → customFields 提交结构（仅含当前启用字段；停用键存量由后端合并保留） */
  function buildSubmitPayload(values: Record<string, any>): Record<string, any> {
    const payload: Record<string, any> = {};
    for (const item of items.value) {
      payload[item.fieldKey] = normalizeSubmitValue(
        item,
        values?.[item.fieldKey],
      );
    }
    return payload;
  }

  /** 编辑角色软约束判定（与后端 field_editable 三态口径一致）：超管不受限；editableRoles 非空数组=角色交集；
   * 空数组=明确限制仅超管（兼容历史数据）；null=未配置 → 跟随 visible_roles（schema 已按可见过滤，可见即等价可编辑）；
   * 硬约束由后端越权 400 拦截兜底 */
  function isRoleEditable(item: FieldSchemaItem): boolean {
    if (isSuperAdmin.value) return true;
    const editableRoles = item.editableRoles;
    if (Array.isArray(editableRoles)) {
      if (editableRoles.length === 0) return false;
      return userRoles.value.some((r) => editableRoles.includes(r));
    }
    return true;
  }

  /**
   * useVbenForm 动态条目：挂 DynamicFieldControl 渲染（baseModelPropName=value 天然匹配）；多行文本独占整行
   * weakRequired（编辑模式传入）：必填字段保留星号提示但移除 required 规则，允许保存——
   * 对齐 7.4 规则 5 存量弱约束（P0-21），避免「为改一个字段被新增必填项卡死」；新建强约束挂 required 硬拦截
   */
  function toFormSchema(opts?: { weakRequired?: boolean }): any[] {
    const weak = opts?.weakRequired === true;
    return items.value.map((item) => {
      const isRequired = Number(item.required) === 1;
      return {
        component: markRaw(DynamicFieldControl),
        fieldName: item.fieldKey,
        label: item.fieldLabel,
        defaultValue: item.options?.defaultValue,
        // required 仅控制星号提示（vben shouldRequired），始终保留；校验阻断走 rules：新建挂 required 硬拦截，编辑弱约束放行
        required: isRequired,
        rules: isRequired && !weak ? 'required' : undefined,
        componentProps: { item, disabled: !isRoleEditable(item) },
        formItemClass: Number(item.fieldType) === 2 ? 'col-span-2' : '',
      };
    });
  }

  /** 筛选字段清单（P1-2/P1-3）：filterable=1；操作符与选项在调用时机取值（loadSchema 完成后调用） */
  function getFilterFields(): FilterFieldMeta[] {
    return items.value
      .filter((i) => Number(i.filterable) === 1)
      .map((item) => {
        const type = Number(item.fieldType);
        const opItems = (list: string[]) =>
          list.map((v) => ({ label: CF_OP_LABELS[v] ?? v, value: v }));
        const isArrayType = type === 7 || type === 9 || type === 10;
        return {
          item,
          fieldKey: item.fieldKey,
          fieldLabel: item.fieldLabel,
          fieldType: type,
          // 数字/金额/日期 走区间比较，文本/单选 默认模糊；数组/布尔固定操作符不展示
          operators: isArrayType || type === 8
            ? []
            : opItems(
                [3, 11, 4, 5].includes(type)
                  ? ['eq', 'gt', 'gte', 'lt', 'lte']
                  : ['like', 'eq', 'ne'],
              ),
          fixedOp: isArrayType ? 'contains' : type === 8 ? 'eq' : undefined,
          choices:
            type === 6 || type === 7
              ? activeChoices(item)
              : type === 9
                ? getFileOptions()
                : type === 10
                  ? getUserOptions()
                  : [],
          multiple: isArrayType,
        };
      });
  }

  /** 筛选值 → cfVal query 参数：与后端 deserialize_cf_val/翻译器类型对齐（数组传 JSON 字面量、布尔传 true/false、其余字符串） */
  function buildCfParam(item: FieldSchemaItem, value: any): string | undefined {
    if (value === undefined || value === null || value === '') return undefined;
    const type = Number(item.fieldType);
    if ([7, 9, 10].includes(type)) {
      const arr = (Array.isArray(value) ? value : [value])
        .filter((v) => v !== undefined && v !== null && v !== '')
        .map((v) => String(v));
      return arr.length > 0 ? JSON.stringify(arr) : undefined;
    }
    if (type === 8) {
      return value === true || value === 'true' ? 'true' : 'false';
    }
    return String(value);
  }

  const formItems = computed(() => items.value);

  return {
    loading,
    items,
    formItems,
    loadSchema,
    activeChoices,
    choiceLabel,
    formatFieldValue,
    toGridColumns,
    toFormSchema,
    isRoleEditable,
    getFilterFields,
    buildCfParam,
    normalizeSubmitValue,
    buildSubmitPayload,
  };
}
