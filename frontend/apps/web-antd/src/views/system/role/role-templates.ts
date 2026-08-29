// P3-3 角色模板向导：前端常量，不落库。
// 模板与菜单 ID 解耦：按权限码（perm）前缀粗匹配，防止菜单 ID 变动导致模板失效。
// 匹配到的菜单会连同其全部祖先链一并预填勾选（后端校验菜单链路完整性：父级未勾选不保存）。

export interface RoleTemplate {
  key: string;
  label: string;
  desc: string;
  dataScope: number;
  permPatterns: string[];
}

export const roleTemplates: RoleTemplate[] = [
  {
    key: 'general_manager',
    label: '总经理',
    desc: '全模块全菜单（含系统管理），可查看系统中所有业务数据',
    dataScope: 1,
    permPatterns: ['*'],
  },
  {
    key: 'sales_director',
    label: '销售总监',
    desc: 'CRM/销售全链路 + 统计报表 + 员工管理，查看本部门及以下数据',
    dataScope: 4,
    permPatterns: [
      'crm:*:*',
      'sale:*:*',
      'statistics:*:*',
      'dashboard:*',
      'report:*',
      'system:employee:*',
    ],
  },
  {
    key: 'sales_manager',
    label: '销售经理',
    desc: 'CRM 主链路 + 业绩查看，查看本部门数据',
    dataScope: 3,
    permPatterns: ['crm:*:*', 'sale:*:*', 'statistics:performance:*'],
  },
  {
    key: 'salesperson',
    label: '业务员',
    desc: '客户/线索/商机/联系人等基础录入跟进 + 个人工作台，仅看本人数据',
    dataScope: 5,
    permPatterns: ['crm:*:*', 'dashboard:workspace'],
  },
];

export const roleTemplateOptions = roleTemplates.map((t) => ({
  label: t.label,
  value: t.key,
}));

/**
 * 权限码粗匹配：
 * - '*' / '*:*:*' 匹配全部
 * - 以 '*' 结尾的通配模式（如 'crm:*:*'、'dashboard:*'）按前缀匹配
 * - 不含 '*' 的模式（如 'dashboard:workspace'）精确匹配
 */
export function matchPerm(
  perm: null | string | undefined,
  patterns: string[],
): boolean {
  if (!perm) return false;
  return patterns.some((p) => {
    if (p === '*' || p === '*:*:*') return true;
    if (p.endsWith('*')) {
      const idx = p.indexOf('*');
      return perm.startsWith(idx > 0 ? p.slice(0, idx) : '');
    }
    return perm === p;
  });
}
