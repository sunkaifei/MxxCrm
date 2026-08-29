import { requestClient } from '#/api/request';

/** 选项结构（单选/多选 choices；active：1 启用 0 停用） */
export interface FieldChoice {
  active: number;
  label: string;
  value: string;
}

/** 类型配置（choices：选项列表；precision：小数位；defaultValue：新建表单预填默认值） */
export interface FieldOptions {
  choices?: FieldChoice[];
  precision?: number;
  defaultValue?: any;
}

/** Schema 下发项（运行侧；后端已按当前登录角色过滤 visible_roles）
 * editableRoles/visibleRoles：null=未配置（editableRoles 未配置时跟随 visibleRoles）；数组=角色限制（空数组=仅超管，兼容历史数据） */
export interface FieldSchemaItem {
  editableRoles: string[] | null;
  fieldKey: string;
  fieldLabel: string;
  fieldType: number;
  filterable: number;
  listSort: number;
  listVisible: number;
  options: FieldOptions | null;
  required: number;
  visibleRoles: string[] | null;
}

/** 字段定义列表项（管理侧） */
export interface FieldDefItem extends FieldSchemaItem {
  createTime: string;
  id: string;
  indexed: number;
  module: string;
  remark: string;
  sort: number;
  status: number;
  storageType: number;
  updateTime: string;
}

/** 字段类型：1文本 2多行文本 3数字 4日期 5日期时间 6单选 7多选 8布尔 9附件 10成员 11金额 */
export const FIELD_TYPE_OPTIONS = [
  { label: '文本', value: 1 },
  { label: '多行文本', value: 2 },
  { label: '数字', value: 3 },
  { label: '日期', value: 4 },
  { label: '日期时间', value: 5 },
  { label: '单选', value: 6 },
  { label: '多选', value: 7 },
  { label: '布尔', value: 8 },
  { label: '附件', value: 9 },
  { label: '成员', value: 10 },
  { label: '金额', value: 11 },
];

/** 字段类型 label 映射（列表展示用） */
export const FIELD_TYPE_LABELS: Record<number, string> =
  Object.fromEntries(FIELD_TYPE_OPTIONS.map((o) => [o.value, o.label]));

/** 数组型字段（多选/附件/成员）：无排序语义，列表列跳过 sortable */
export const ARRAY_FIELD_TYPES = [7, 9, 10];

/**
 * 获取字段定义列表
 */
export const getFieldListApi = async (params?: Record<string, any>) => {
  return requestClient.get('/api/system/field/list', { params });
};

/**
 * 获取已接入自定义字段的模块注册表
 */
export const getModuleListApi = async () => {
  return requestClient.get('/api/system/field/modules');
};

/**
 * 获取模块 schema（运行侧下发，面向登录用户）
 */
export const getFieldSchemaApi = async (module: string) => {
  return requestClient.get('/api/system/field/schema', { params: { module } });
};

/**
 * 新增字段定义
 */
export const saveFieldApi = async (data: Record<string, any>) => {
  return requestClient.post('/api/system/field/save', data);
};

/**
 * 修改字段定义（module/fieldKey/fieldType 创建后锁定；choices 旧 value 锁定）
 */
export const updateFieldApi = async (
  id: number | string,
  data: Record<string, any>,
) => {
  return requestClient.put(`/api/system/field/update/${id}`, {
    id: String(id),
    ...data,
  });
};

/**
 * 启用/停用字段定义
 */
export const setFieldStatusApi = async (data: {
  id: number | string;
  status: number;
}) => {
  return requestClient.post('/api/system/field/status', data);
};

/**
 * 删除字段定义（软删，唯一索引含 deleted 条件支持同名重建）
 */
export const deleteFieldApi = async (id: number | string) => {
  return requestClient.delete(`/api/system/field/${id}`);
};

/**
 * 一键加速：为模块内 filterable 字段批量创建表达式索引（CONCURRENTLY 无长阻塞）
 */
export const accelerateFieldApi = async (module: string) => {
  return requestClient.post('/api/system/field/accelerate', { module });
};

/** 标准敏感字段权限清单项（管理侧；P2-1） */
export interface FieldPermItem {
  editableRoles: any[] | null;
  fieldKey: string;
  fieldLabel: string;
  id: number | string;
  module: string;
  updateBy: string | null;
  updateTime: string | null;
  visibleRoles: any[] | null;
}

/**
 * 获取标准敏感字段权限清单（module 空=全部模块）
 */
export const getFieldPermListApi = async (module?: string) => {
  return requestClient.get('/api/system/field-perm/list', {
    params: { module },
  });
};

/**
 * 保存标准敏感字段角色配置（逐行保存；visibleRoles/editableRoles 传 null=未配置）
 */
export const saveFieldPermApi = async (data: {
  editableRoles: any[] | null;
  fieldKey: string;
  module: string;
  visibleRoles: any[] | null;
}) => {
  return requestClient.post('/api/system/field-perm/save', data);
};
